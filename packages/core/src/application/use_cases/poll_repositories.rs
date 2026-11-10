use super::create_job::CreateJobUseCase;
use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;

#[derive(Injectable)]
pub struct PollRepositoriesUseCase {
    database: Arc<Database>,
    create_job: Arc<CreateJobUseCase>,
}
impl UseCase for PollRepositoriesUseCase {
    type Input = Option<Vec<String>>;
    type Output = ();
    async fn execute(&self, urls: Self::Input) -> Result<(), Exception> {
        for stack in self.database.stacks.find(Condition::new()).await? {
            if let Some(urls) = &urls {
                let sources = giong_runner::types::configuration_sources(&stack.configurations)
                    .unwrap_or_default();
                if !sources.iter().any(|source| urls.contains(&source.repo_url)) {
                    continue;
                }
            }
            if let Err(error) = self.create_job.submit(stack.id).await {
                eprintln!("Could not poll stack {}: {error:#}", stack.id);
            }
        }
        Ok(())
    }
}
