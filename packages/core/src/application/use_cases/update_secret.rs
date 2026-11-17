use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::use_cases::use_case::UseCase;
use crate::config::Config;
use crate::utils::crypto::encrypt_secret;
use giong_database::Database;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateSecretInput {
    pub vault_id: Uuid,
    pub secret_id: Uuid,
    pub value: String,
}

#[derive(Injectable)]
pub struct UpdateSecretUseCase {
    database: Arc<Database>,
    config: Arc<Config>,
}

impl UseCase for UpdateSecretUseCase {
    type Input = UpdateSecretInput;
    type Output = ();
    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        let UpdateSecretInput { vault_id, secret_id, value } = input;
        let secret = self
            .database
            .secrets
            .find_one(Condition::new().eq("vault_id", vault_id).eq("id", secret_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Secret))?;

        let encrypted_value =
            encrypt_secret(self.config.encryption_key.as_deref(), vault_id, &secret.key, &value)?;

        self.database
            .secrets
            .update_one(
                Condition::new().eq("vault_id", vault_id).eq("id", secret_id),
                Document::new().set("value", encrypted_value),
            )
            .await?;

        Ok(())
    }
}
