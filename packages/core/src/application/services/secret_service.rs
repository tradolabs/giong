use crate::application::helpers::crypto::{decrypt_token, encrypt_token};
use crate::config::Config;
use anyhow::{Context, Result};
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct SecretService {
    config: Arc<Config>,
    database: Arc<Database>,
}

impl SecretService {
    pub fn encrypt_secret(&self, stack_id: Uuid, key: &str, value: &str) -> Result<String> {
        let context = format!("stack:{stack_id}:{key}");
        encrypt_token(value, self.encryption_key()?, context.as_bytes())
    }

    pub fn decrypt_secret(&self, stack_id: Uuid, key: &str, value: &str) -> Result<String> {
        let context = format!("stack:{stack_id}:{key}");
        decrypt_token(value, self.encryption_key()?, context.as_bytes())
    }

    pub async fn get_credentials(&self, stack_id: Uuid) -> Result<Vec<(String, String)>> {
        self.database
            .secrets
            .find(Condition::new().eq("stack_id", stack_id))
            .await?
            .into_iter()
            .map(|secret| {
                let value = self
                    .decrypt_secret(stack_id, &secret.key, &secret.value)
                    .with_context(|| format!("Could not decrypt credential {}", secret.key))?;
                Ok((secret.key, value))
            })
            .collect()
    }

    fn encryption_key(&self) -> Result<&str> {
        self.config.encryption_key.as_deref().context("Encryption key is not configured")
    }
}
