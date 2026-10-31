use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetSecretsInput {
    pub stack_id: Uuid,
}

#[derive(Injectable)]
pub struct GetSecretsUseCase {
    database: Arc<Database>,
}

impl UseCase for GetSecretsUseCase {
    type Input = GetSecretsInput;
    type Output = Vec<String>;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        if !self.database.stacks.exists(Condition::new().eq("id", input.stack_id)).await? {
            return Err(Exception::NotFound("modules.stack.not_found"));
        }
        let mut keys: Vec<_> = self
            .database
            .secrets
            .find(Condition::new().eq("stack_id", input.stack_id))
            .await?
            .into_iter()
            .map(|secret| secret.key)
            .collect();
        keys.sort();
        Ok(keys)
    }
}
