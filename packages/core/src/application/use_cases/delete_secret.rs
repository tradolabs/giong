use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteSecretInput {
    pub stack_id: Uuid,
    pub key: String,
}

#[derive(Injectable)]
pub struct DeleteSecretUseCase {
    database: Arc<Database>,
}

impl UseCase for DeleteSecretUseCase {
    type Input = DeleteSecretInput;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        if !self.database.stacks.exists(Condition::new().eq("id", input.stack_id)).await? {
            return Err(Exception::NotFound("modules.stack.not_found"));
        }
        self.database
            .secrets
            .delete_one(Condition::new().eq("stack_id", input.stack_id).eq("key", input.key))
            .await?;
        Ok(())
    }
}
