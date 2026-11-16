use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::config::Config;
use crate::utils::crypto::decrypt_secret;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::collections::HashSet;
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
        vault_id: Option<Uuid>,
        inherited_vault_ids: &[Uuid],
        key: &str,
    ) -> Result<String, Exception> {
        let mut seen = HashSet::new();
        for id in inherited_vault_ids.iter().copied().chain(vault_id) {
            if !seen.insert(id) {
                return Err(Exception::BadRequest("Duplicate vault".into()));
            }
            if !self.database.vaults.exists(Condition::new().eq("id", id)).await? {
                return Err(ResourceException::NotFound(Resource::Vault).into());
            }
        }

        let vaults = vault_id.into_iter().chain(inherited_vault_ids.iter().copied().rev());
        for id in vaults {
            if let Some(secret) = self
                .database
                .secrets
                .find_one(Condition::new().eq("vault_id", id).eq("key", key))
                .await?
            {
                return Ok(decrypt_secret(
                    self.config.encryption_key.as_deref(),
                    id,
                    key,
                    &secret.value,
                )?);
            }
        }

        Err(ResourceException::NotFound(Resource::Secret).into())
    }
}
