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

pub struct CreateSecretInput {
    pub vault_id: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(Injectable)]
pub struct CreateSecretUseCase {
    database: Arc<Database>,
    config: Arc<Config>,
}

impl UseCase for CreateSecretUseCase {
    type Input = CreateSecretInput;
    type Output = ();
    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        let CreateSecretInput { vault_id, key, value } = input;

        let is_key_existed = self
            .database
            .secrets
            .exists(Condition::new().eq("vault_id", vault_id).eq("key", key.as_str()))
            .await?;
        if is_key_existed {
            return Err(ResourceException::AlreadyExists(Resource::Secret).into());
        }

        let value = encrypt_secret(self.config.encryption_key.as_deref(), vault_id, &key, &value)?;
        self.database
            .secrets
            .create_one(
                Document::new().set("vault_id", vault_id).set("key", key).set("value", value),
            )
            .await?;

        Ok(())
    }
}
