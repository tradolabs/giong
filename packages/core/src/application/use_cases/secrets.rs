use crate::application::exceptions::base::Exception;
use crate::application::helpers::secrets::{decrypt, encrypt, get_vault, validate_vaults};
use crate::application::use_cases::use_case::UseCase;
use crate::config::Config;
use giong_database::Database;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct SecretInfo {
    pub key: String,
}
pub struct PutSecretInput {
    pub vault_id: Uuid,
    pub key: String,
    pub value: String,
}
pub struct SecretScope {
    pub vault_id: Option<Uuid>,
    pub inherited_vault_ids: Vec<Uuid>,
}
pub struct GetSecretValueInput {
    pub scope: SecretScope,
    pub key: String,
}
#[derive(Injectable)]
pub struct GetSecretsUseCase {
    database: Arc<Database>,
}
impl UseCase for GetSecretsUseCase {
    type Input = Uuid;
    type Output = Vec<SecretInfo>;
    async fn execute(&self, id: Uuid) -> Result<Self::Output, Exception> {
        get_vault(&self.database, id).await?;
        Ok(self
            .database
            .secrets
            .find(Condition::new().eq("vault_id", id))
            .await?
            .into_iter()
            .map(|secret| SecretInfo { key: secret.key })
            .collect())
    }
}
#[derive(Injectable)]
pub struct PutSecretUseCase {
    database: Arc<Database>,
    config: Arc<Config>,
}
impl UseCase for PutSecretUseCase {
    type Input = PutSecretInput;
    type Output = ();
    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        get_vault(&self.database, input.vault_id).await?;
        if input.key.is_empty() {
            return Err(Exception::BadRequest("Secret key is required".into()));
        }
        let value = encrypt(&self.config, input.vault_id, &input.key, &input.value)?;
        let condition =
            Condition::new().eq("vault_id", input.vault_id).eq("key", input.key.as_str());
        if !self
            .database
            .secrets
            .update_one(condition, Document::new().set("value", value.clone()))
            .await?
        {
            self.database
                .secrets
                .create_one(
                    Document::new()
                        .set("vault_id", input.vault_id)
                        .set("key", input.key)
                        .set("value", value),
                )
                .await?;
        }
        Ok(())
    }
}
#[derive(Injectable)]
pub struct DeleteSecretUseCase {
    database: Arc<Database>,
}
impl UseCase for DeleteSecretUseCase {
    type Input = (Uuid, String);
    type Output = ();
    async fn execute(&self, (id, key): Self::Input) -> Result<(), Exception> {
        get_vault(&self.database, id).await?;
        self.database
            .secrets
            .delete_one(Condition::new().eq("vault_id", id).eq("key", key))
            .await?;
        Ok(())
    }
}
#[derive(Injectable)]
pub struct GetSecretValueUseCase {
    database: Arc<Database>,
    config: Arc<Config>,
}
impl UseCase for GetSecretValueUseCase {
    type Input = GetSecretValueInput;
    type Output = String;
    async fn execute(&self, input: Self::Input) -> Result<String, Exception> {
        validate_vaults(&self.database, input.scope.vault_id, &input.scope.inherited_vault_ids)
            .await?;
        let vaults = input
            .scope
            .vault_id
            .into_iter()
            .chain(input.scope.inherited_vault_ids.into_iter().rev());
        for id in vaults {
            if let Some(secret) = self
                .database
                .secrets
                .find_one(Condition::new().eq("vault_id", id).eq("key", input.key.as_str()))
                .await?
            {
                return Ok(decrypt(&self.config, id, &input.key, &secret.value)?);
            }
        }
        Err(Exception::NotFound("modules.secret.not_found"))
    }
}
