use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::ports::queue::{events::RunJobEvent, queue_manager::QueueManager};
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
    async fn execute(&self, stack_id: Uuid) -> Result<JobEntity, Exception> {
        self.submit(stack_id).await?.ok_or(Exception::Conflict("modules.job.active_job_exists"))
    }
}
impl CreateJobUseCase {
    pub async fn submit(&self, stack_id: Uuid) -> Result<Option<JobEntity>, Exception> {
        let active = Condition::new()
            .eq("stack_id", stack_id)
            .is_in("status", [JobStatus::Pending, JobStatus::Running]);
        if self.database.jobs.exists(active.clone()).await? {
            return Ok(None);
        }
        let stack = self
            .database
            .stacks
            .find_one(Condition::new().eq("id", stack_id))
            .await?
            .ok_or(Exception::NotFound("modules.stack.not_found"))?;
        let job = self
            .database
            .transaction(async || {
                if self.database.jobs.exists(active).await? {
                    return Ok(None);
                }
                let current =
                    self.database.stacks.find_one(Condition::new().eq("id", stack_id)).await?;
                if current.as_ref().map(|stack| &stack.configurations)
                    != Some(&stack.configurations)
                {
                    return Ok(None);
                }
                let job = self
                    .database
                    .jobs
                    .create_one(
                        Document::new().set("stack_id", stack_id).set("status", JobStatus::Pending),
                    )
                    .await?;
                Ok(Some(job))
            })
            .await?;
        if let Some(job) = &job {
            self.queue_manager.enqueue(RunJobEvent { job_id: job.id }.into()).await?;
        }
        Ok(job)
    }
}
