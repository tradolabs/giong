use super::use_case::UseCase;
use crate::{
    application::exceptions::base::Exception,
    application::helpers::crypto::decrypt_token,
    application::helpers::filesystem::{remove_dir, root_dir},
    config::Config,
};
use anyhow::{Context, Result};
use chrono::Utc;
use giong_database::{
    Condition, Database, Document, JobEntity, JobStatus, RepositoryEntity, StackEntity, Uuid,
};
use giong_di::Injectable;
use giong_storage::storage::Storage;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct RunJobInput {
    pub job_id: Uuid,
}

#[derive(Injectable)]
pub struct RunJobUseCase {
    config: Arc<Config>,
    database: Arc<Database>,
    storage: Arc<Storage>,
}

impl UseCase for RunJobUseCase {
    type Input = RunJobInput;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        let db = &self.database;

        let job = db
            .jobs
            .find_one(Condition::new().eq("id", input.job_id))
            .await?
            .ok_or_else(|| Exception::unexpected("Queued job was not found"))?;

        let updated = db
            .jobs
            .update_one(
                Condition::new().eq("id", input.job_id).eq("status", JobStatus::Pending),
                Document::new()
                    .set("status", JobStatus::Running)
                    .set("started_at", Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            )
            .await?;

        if !updated {
            return Ok(());
        }

        eprintln!("Job {} running (stack: {}, commit: {})", job.id, job.stack_id, job.commit_sha);

        let stack = db
            .stacks
            .find_one(Condition::new().eq("id", job.stack_id))
            .await?
            .ok_or_else(|| Exception::unexpected("Queued job stack was not found"))?;

        let repo = db
            .repositories
            .find_one(Condition::new().eq("url", stack.repo_url.as_str()))
            .await?
            .ok_or_else(|| Exception::unexpected("Queued job repository was not found"))?;

        let (status, code) = self.run_job(repo, stack, job).await;

        db.jobs
            .update_one(
                Condition::new().eq("id", input.job_id),
                Document::new()
                    .set("status", status)
                    .set("exit_code", code)
                    .set("finished_at", Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            )
            .await?;

        eprintln!("Job {} finished: {} (exit_code: {code:?})", input.job_id, status.as_str());

        Ok(())
    }
}

impl RunJobUseCase {
    async fn run_job(
        &self,
        repo: RepositoryEntity,
        stack: StackEntity,
        job: JobEntity,
    ) -> (JobStatus, Option<i32>) {
        let job_id = job.id;
        let root = root_dir();
        let checkout_path = root.join("runs").join(job_id.to_string());
        let log_path = root.join("logs").join(job_id.to_string());
        let encryption_key = self.config.encryption_key.clone();

        let result = tokio::task::spawn_blocking({
            let (checkout_path, log_path) = (checkout_path.clone(), log_path.clone());
            move || -> Result<Option<i32>> {
                let mut logger = File::create(&log_path)?;
                let result =
                    Self::checkout(&repo, &stack, &job, &checkout_path, encryption_key.as_deref())
                        .and_then(|dir| {
                            let image = format!("giong-image:{job_id}");
                            giong_runner::run_docker(
                                &image,
                                &stack.id.to_string(),
                                &dir,
                                &mut logger,
                            )
                            .map_err(Into::into)
                        });
                if let Err(err) = &result {
                    let _ = writeln!(logger, "{err:#}");
                }
                result
            }
        })
        .await
        .unwrap_or_else(|err| Err(err.into()));

        let (status, code) = match result {
            Ok(Some(0)) => (JobStatus::Succeeded, Some(0)),
            Ok(code) => (JobStatus::Failed, code),
            Err(err) => {
                eprintln!("Job {job_id}: {err:#}");
                (JobStatus::Failed, None)
            }
        };

        let _ = self
            .storage
            .upload(&log_path, &format!("logs/{job_id}.log"), "text/plain; charset=utf-8")
            .await;

        remove_dir(&checkout_path);
        (status, code)
    }

    fn checkout(
        repo: &RepositoryEntity,
        stack: &StackEntity,
        job: &JobEntity,
        checkout_path: &Path,
        key: Option<&str>,
    ) -> Result<PathBuf> {
        let token = repo
            .credentials
            .as_deref()
            .map(|val| {
                let key = key.context("Encryption key is not configured")?;
                decrypt_token(val, key, repo.url.as_bytes())
            })
            .transpose()?;

        giong_git::checkout(&repo.url, checkout_path, &job.commit_sha, token.as_deref())?;

        let stack_path = checkout_path.join(&stack.folder_path);
        let working_dir = fs::canonicalize(&stack_path)
            .with_context(|| format!("Could not resolve stack folder {}", stack_path.display()))?;

        Ok(working_dir)
    }
}
