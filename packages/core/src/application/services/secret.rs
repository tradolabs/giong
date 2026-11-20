use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::config::Config;
use crate::utils::crypto::decrypt_secret;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct SecretService {
    database: Arc<Database>,
    config: Arc<Config>,
}

impl SecretService {
    pub(crate) async fn get_secret_value(
        &self,
        vault_ids: &[Uuid],
        requested_vault: Option<&str>,
        key: &str,
    ) -> Result<String, Exception> {
        let name = requested_vault.ok_or_else(|| {
            Exception::BadRequest("Secret reference requires a vault name".into())
        })?;
        let id = self
            .database
            .vaults
            .find_one(Condition::new().eq("name", name).is_in("id", vault_ids.iter().copied()))
            .await?
            .ok_or_else(|| {
                Exception::BadRequest("Secret vault name is not attached to this flow".into())
            })?
            .id;
        let secret = self
            .database
            .secrets
            .find_one(Condition::new().eq("vault_id", id).eq("key", key))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Secret))?;
        Ok(decrypt_secret(self.config.encryption_key.as_deref(), id, key, &secret.value)?)
    }
}
