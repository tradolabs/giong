use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::secret_service::SecretService;
use giong_database::Database;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateSecretInput {
    pub stack_id: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(Injectable)]
pub struct CreateSecretUseCase {
    secret_service: Arc<SecretService>,
    database: Arc<Database>,
}

impl UseCase for CreateSecretUseCase {
    type Input = CreateSecretInput;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        if !self.database.stacks.exists(Condition::new().eq("id", input.stack_id)).await? {
            return Err(Exception::NotFound("modules.stack.not_found"));
        }
        let condition =
            Condition::new().eq("stack_id", input.stack_id).eq("key", input.key.as_str());
        if self.database.secrets.exists(condition).await? {
            return Err(Exception::Conflict("modules.secret.already_exists"));
        }
        let value = self.secret_service.encrypt_secret(input.stack_id, &input.key, &input.value)?;
        self.database
            .secrets
            .create_one(
                Document::new()
                    .set("stack_id", input.stack_id)
                    .set("key", input.key)
                    .set("value", value),
            )
            .await?;
        Ok(())
    }
}
