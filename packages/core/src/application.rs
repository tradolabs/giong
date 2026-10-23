use crate::{Job, credentials::Credentials, mutex_pool::MutexPool, utils::filesystem::remove_dir};
use anyhow::Result;
use chrono::Utc;
use futures_util::{StreamExt, stream};
use giong_database::{
    Condition, Database, Document, JobStatus, RepositoryEntity, StackEntity, Uuid,
};
use giong_storage::Storage;
use std::{collections::HashMap, fs, path::PathBuf};
use tokio::sync::{
    Mutex,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
};

pub struct Application {
    root_folder: PathBuf,
    database: Database,
    storage: Option<Storage>,
    queue: UnboundedSender<Job>,
    queue_receiver: Mutex<UnboundedReceiver<Job>>,
    poll_locks: MutexPool<Uuid>,
}

impl Application {
    pub async fn new(data_dir: Option<PathBuf>) -> Result<Self> {
        let root_folder = data_dir
            .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".giong"));
        fs::create_dir_all(&root_folder)?;
        let root_folder = fs::canonicalize(root_folder)?;
        fs::create_dir_all(root_folder.join("logs"))?;
        fs::create_dir_all(root_folder.join("runs"))?;

        let database_url = std::env::var("GIONG_DATABASE_URL").unwrap_or_else(|_| {
            format!(
                "sqlite://{}?mode=rwc",
                root_folder.join("giong.sqlite3").display()
            )
        });
        let database = Database::new(&database_url).await?;
        let storage = std::env::var("GIONG_STORAGE_URL")
            .ok()
            .filter(|url| !url.is_empty())
            .map(|url| Storage::new(&url))
            .transpose()?;

        let (queue, queue_receiver) = mpsc::unbounded_channel();
        Ok(Self {
            root_folder,
            database,
            storage,
            queue,
            queue_receiver: Mutex::new(queue_receiver),
            poll_locks: MutexPool::new(),
        })
    }

    pub async fn add_repo(&self, name: &str, url: &str, token: Option<&str>) -> Result<()> {
        let encrypted_token = token
            .map(|token| Credentials::new(url.as_bytes()).encrypt(token))
            .transpose()?;
        self.database
            .repositories
            .create_one(
                Document::new()
                    .set("name", name)
                    .set("url", url)
                    .set("credentials", encrypted_token),
            )
            .await
            .map(|_| ())
    }

    pub async fn add_stack(
        &self,
        name: &str,
        repo_url: &str,
        branch: &str,
        folder_path: &str,
    ) -> Result<()> {
        self.database
            .stacks
            .create_one(
                Document::new()
                    .set("name", name)
                    .set("repo_url", repo_url)
                    .set("branch", branch)
                    .set("folder_path", folder_path),
            )
            .await
            .map(|_| ())
    }

    pub async fn consume_jobs(&self) -> Result<()> {
        stream::unfold(self.queue_receiver.lock().await, async |mut receiver| {
            receiver.recv().await.map(|job| (job, receiver))
        })
        .for_each_concurrent(None, async |job| {
            let job_id = job.job.id;
            if let Err(error) = self.run_job(job).await {
                eprintln!("Job {job_id}: {error:#}");
            }
        })
        .await;
        Ok(())
    }

    async fn run_job(&self, job: Job) -> Result<()> {
        let job_id = job.job.id;
        self.database
            .jobs
            .update_one(
                Condition::new()
                    .eq("id", job_id)
                    .eq("status", JobStatus::Pending),
                Document::new().set("status", JobStatus::Running).set(
                    "started_at",
                    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                ),
            )
            .await?;

        let checkout_path = self.root_folder.join("runs").join(job_id.to_string());
        let log_path = self.root_folder.join("logs").join(job_id.to_string());
        let (status, code) = job.run().await;

        if let Some(storage) = &self.storage {
            let _ = storage
                .upload(
                    &log_path,
                    &format!("logs/{job_id}.log"),
                    "text/plain; charset=utf-8",
                )
                .await;
        }

        self.database
            .jobs
            .update_one(
                Condition::new().eq("id", job_id),
                Document::new()
                    .set("status", status)
                    .set("exit_code", code)
                    .set(
                        "finished_at",
                        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    ),
            )
            .await?;

        remove_dir(&checkout_path);
        Ok(())
    }

    pub async fn poll_repositories(&self, repository_urls: Option<Vec<String>>) -> Result<()> {
        let repository_urls = repository_urls.unwrap_or_default();
        let repositories = self
            .database
            .repositories
            .find(Condition::new().is_in("url", repository_urls.clone()))
            .await?;
        let stacks = self
            .database
            .stacks
            .find(Condition::new().is_in("repo_url", repository_urls.clone()))
            .await?;

        let repository_map: HashMap<_, _> = repositories
            .into_iter()
            .map(|repository| (repository.url.clone(), repository))
            .collect();

        for stack in stacks {
            let Some(repository) = repository_map.get(&stack.repo_url) else {
                eprintln!("{}: Stack repository is missing", stack.name);
                continue;
            };
            self.poll_stack(repository, stack).await?;
        }

        Ok(())
    }

    async fn poll_stack(&self, repository: &RepositoryEntity, stack: StackEntity) -> Result<()> {
        let _poll = self.poll_locks.lock(stack.id).await;

        let commit_sha = match tokio::task::spawn_blocking({
            let repository = repository.clone();
            let branch = stack.branch.clone();
            move || {
                let token = repository
                    .credentials
                    .as_deref()
                    .map(|value| Credentials::new(repository.url.as_bytes()).decrypt(value))
                    .transpose()?;
                giong_git::get_last_commit(&repository.url, &branch, token.as_deref())
            }
        })
        .await
        .unwrap_or_else(|error| Err(error.into()))
        {
            Ok(sha) => sha,
            Err(error) => {
                eprintln!("{}: {error:#}", stack.name);
                return Ok(());
            }
        };

        if stack.last_commit.as_deref() == Some(&commit_sha) {
            return Ok(());
        }

        let job = self
            .database
            .transaction(async || {
                let job = self
                    .database
                    .jobs
                    .create_one(
                        Document::new()
                            .set("stack_id", stack.id)
                            .set("commit_sha", commit_sha.as_str())
                            .set("status", JobStatus::Pending),
                    )
                    .await?;
                self.database
                    .stacks
                    .update_one(
                        Condition::new().eq("id", stack.id),
                        Document::new().set("last_commit", commit_sha.as_str()),
                    )
                    .await?;
                Ok(job)
            })
            .await?;

        self.queue.send(Job::new(
            self.root_folder.clone(),
            repository.clone(),
            stack,
            job,
        ))?;

        Ok(())
    }
}
