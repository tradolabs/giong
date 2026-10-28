use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::ports::queue::{events::RunJobEvent, queue_manager::QueueManager};
use anyhow::Context;
use giong_database::Database;
use giong_database::entities::job_entity::JobEntity;
use giong_database::enums::job_status::JobStatus;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct CreateJobUseCase {
    database: Arc<Database>,
    queue_manager: Arc<dyn QueueManager>,
}

impl UseCase for CreateJobUseCase {
    type Input = Uuid;
    type Output = JobEntity;

    async fn execute(&self, stack_id: Uuid) -> Result<Self::Output, Exception> {
        let is_running = self
            .database
            .jobs
            .exists(
                Condition::new()
                    .eq("stack_id", stack_id)
                    .is_in("status", [JobStatus::Pending, JobStatus::Running]),
            )
            .await?;

        if is_running {
            return Err(Exception::Conflict("modules.job.active_job_exists"));
        }

        let Some(stack) =
            self.database.stacks.find_one(Condition::new().eq("id", stack_id)).await?
        else {
            return Err(Exception::NotFound("modules.stack.not_found"));
        };

        let job = self
            .database
            .jobs
            .create_one(
                Document::new()
                    .set("stack_id", stack_id)
                    .set(
                        "commit_sha",
                        stack.last_commit.as_deref().context("Stack has no commit yet")?,
                    )
                    .set("status", JobStatus::Pending),
            )
            .await?;

        self.queue_manager.enqueue(RunJobEvent { job_id: job.id }.into()).await?;
        Ok(job)
    }
}
