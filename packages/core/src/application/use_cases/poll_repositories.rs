use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::helpers::crypto::decrypt_token;
use crate::application::ports::queue::{events::RunJobEvent, queue_manager::QueueManager};
use crate::config::Config;
use anyhow::Context;
use giong_database::{Condition, Document, JobStatus, RepositoryEntity, StackEntity};
use giong_database::{Database, Uuid};
use giong_di::Injectable;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Injectable)]
pub struct PollRepositoriesUseCase {
    queue_manager: Arc<dyn QueueManager>,
    config: Arc<Config>,
    database: Arc<Database>,
}

impl UseCase for PollRepositoriesUseCase {
    type Input = Option<Vec<String>>;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let repository_urls = input;
        let repositories = self
            .database
            .repositories
            .find(match repository_urls {
                Some(repository_urls) => Condition::new().is_in("url", repository_urls),
                None => Condition::new(),
            })
            .await?;

        let repository_map: HashMap<_, _> = repositories
            .into_iter()
            .map(|repository| (repository.url.clone(), repository))
            .collect();

        let stacks = self
            .database
            .stacks
            .find(Condition::new().is_in(
                "repo_url",
                repository_map.keys().map(|k| k.to_string()).collect::<Vec<_>>(),
            ))
            .await?;

        for stack in stacks {
            if let Some(repository) = repository_map.get(&stack.repo_url) {
                let stack_name = stack.name.clone();
                match self.poll_stack(repository, stack).await {
                    Ok(Some(job_id)) => {
                        self.queue_manager.enqueue(RunJobEvent { job_id }.into()).await.ok();
                    }
                    Ok(None) => {}
                    Err(error) => eprintln!("{stack_name}: {error:#}"),
                }
            }
        }

        Ok(())
    }
}

impl PollRepositoriesUseCase {
    async fn has_active_job(&self, stack_id: Uuid) -> Result<bool, giong_database::error::Error> {
        self.database
            .jobs
            .exists(
                Condition::new()
                    .eq("stack_id", stack_id)
                    .is_in("status", [JobStatus::Pending, JobStatus::Running]),
            )
            .await
    }

    async fn poll_stack(
        &self,
        repository: &RepositoryEntity,
        stack: StackEntity,
    ) -> Result<Option<Uuid>, Exception> {
        if self.has_active_job(stack.id).await? {
            return Ok(None);
        }

        let commit_sha = match tokio::task::spawn_blocking({
            let encryption_key = self.config.encryption_key.clone();
            let repository = repository.clone();
            let branch = stack.branch.clone();
            move || -> Result<String, Exception> {
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
                Ok(giong_git::get_last_commit(&repository.url, &branch, token.as_deref())?)
            }
        })
        .await
        .unwrap_or_else(|error| Err(Exception::unexpected(error)))
        {
            Ok(sha) => sha,
            Err(error) => {
                eprintln!("{}: {error:#}", stack.name);
                return Ok(None);
            }
        };

        if stack.last_commit.as_deref() == Some(&commit_sha) {
            return Ok(None);
        }

        let job = self
            .database
            .transaction(async || {
                if self.has_active_job(stack.id).await? {
                    return Ok(None);
                }
                let current_stack =
                    self.database.stacks.find_one(Condition::new().eq("id", stack.id)).await?;
                let Some(current_stack) = current_stack else {
                    return Ok(None);
                };
                if current_stack.last_commit.as_deref() == Some(&commit_sha) {
                    return Ok(None);
                }
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
                Ok(Some(job))
            })
            .await?;

        Ok(job.map(|job| job.id))
    }
}
