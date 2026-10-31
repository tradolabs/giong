use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::secret_service::SecretService;
use giong_database::Database;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateSecretInput {
    pub stack_id: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(Injectable)]
pub struct UpdateSecretUseCase {
    secret_service: Arc<SecretService>,
    database: Arc<Database>,
}

impl UseCase for UpdateSecretUseCase {
    type Input = UpdateSecretInput;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        let value = self.secret_service.encrypt_secret(input.stack_id, &input.key, &input.value)?;
        let updated = self
            .database
            .secrets
            .update_one(
                Condition::new().eq("stack_id", input.stack_id).eq("key", input.key),
                Document::new().set("value", value),
            )
            .await?;
        if !updated {
            return Err(Exception::NotFound("modules.secret.not_found"));
        }
        Ok(())
    }
}
