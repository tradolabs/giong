use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::Database;
use giong_database::entities::repository_entity::RepositoryEntity;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;

#[derive(Injectable)]
pub struct GetRepositoriesUseCase {
    database: Arc<Database>,
}

impl UseCase for GetRepositoriesUseCase {
    type Input = ();
    type Output = Vec<RepositoryEntity>;
    async fn execute(&self, (): ()) -> Result<Self::Output, Exception> {
        let mut repositories = self.database.repositories.find(Condition::new()).await?;
        repositories.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(repositories)
    }
}
