use super::use_case::UseCase;
use crate::application::services::secret_service::SecretService;
use crate::{
    application::exceptions::base::Exception,
    application::helpers::crypto::decrypt_token,
    application::helpers::filesystem::{remove_dir, root_dir},
    config::Config,
};
use anyhow::{Context, Result};
use chrono::Utc;
use giong_database::Database;
use giong_database::entities::{job_entity::JobEntity, stack_entity::StackEntity};
use giong_database::enums::job_status::JobStatus;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use giong_storage::storage::Storage;
use std::{
    fs::{self, File},
    io::Write,
    sync::Arc,
};
use uuid::Uuid;

pub struct RunJobInput {
    pub job_id: Uuid,
}

#[derive(Injectable)]
pub struct RunJobUseCase {
    secret_service: Arc<SecretService>,
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

        eprintln!("Job {} running (stack: {})", job.id, job.stack_id);

        let stack = db
            .stacks
            .find_one(Condition::new().eq("id", job.stack_id))
            .await?
            .ok_or_else(|| Exception::unexpected("Queued job stack was not found"))?;

        let (status, code) = self.run_job(stack, job).await;

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
    async fn run_job(&self, stack: StackEntity, job: JobEntity) -> (JobStatus, Option<i32>) {
        let job_id = job.id;
        let root = root_dir();
        let checkout_path = root.join("runs").join(job_id.to_string());
        let log_path = root.join("logs").join(job_id.to_string());
        let encryption_key = self.config.encryption_key.clone();
        let secrets = self.secret_service.get_run_secrets(stack.id).await;
        let database = self.database.clone();
        let runtime = tokio::runtime::Handle::current();

        let result = tokio::task::spawn_blocking({
            let (checkout_path, log_path) = (checkout_path.clone(), log_path.clone());
            move || -> Result<Option<i32>> {
                let mut logger = File::create(&log_path)?;
                let result = (|| -> Result<Option<i32>> {
                    let sources =
                        giong_runner::types::configuration_sources(&stack.configurations)?;
                    fs::create_dir_all(&checkout_path)?;
                    for (position, source) in sources.iter().enumerate() {
                        let repository =
                            runtime
                                .block_on(database.repositories.find_one(
                                    Condition::new().eq("url", source.repo_url.as_str()),
                                ))?
                                .context("Source repository no longer exists")?;
                        let token = repository
                            .credentials
                            .as_deref()
                            .map(|value| {
                                let key = encryption_key
                                    .as_deref()
                                    .context("Encryption key is not configured")?;
                                decrypt_token(value, key, repository.url.as_bytes())
                            })
                            .transpose()?;
                        let commit_sha = giong_git::resolve_revision(
                            &repository.url,
                            &source.revision,
                            token.as_deref(),
                        )?;
                        let directory = checkout_path.join(format!("source-{position}"));
                        writeln!(logger, "Checking out {} at {}", repository.url, commit_sha)?;
                        giong_git::checkout(
                            &repository.url,
                            &directory,
                            &commit_sha,
                            token.as_deref(),
                        )?;
                        fs::remove_dir_all(directory.join(".git"))?;
                    }
                    let secrets = secrets?;
                    Ok(giong_runner::run(
                        &job_id.to_string(),
                        &stack.id.to_string(),
                        &stack.name,
                        &stack.configurations,
                        &secrets,
                        &checkout_path,
                        &mut logger,
                    )?)
                })();
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
}
