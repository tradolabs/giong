use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::{Condition, Database, JobEntity, Uuid};
use giong_di::Injectable;
use std::sync::Arc;

pub struct GetJobsInput {
    pub stack_id: Uuid,
}

#[derive(Injectable)]
pub struct GetJobsUseCase {
    database: Arc<Database>,
}

impl UseCase for GetJobsUseCase {
    type Input = GetJobsInput;
    type Output = Vec<JobEntity>;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let mut jobs =
            self.database.jobs.find(Condition::new().eq("stack_id", input.stack_id)).await?;
        jobs.reverse();
        Ok(jobs)
    }
}
