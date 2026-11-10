use crate::application::exceptions::base::Exception;
use crate::application::helpers::crypto::{decrypt_token, encrypt_token};
use crate::config::Config;
use anyhow::{Context, Result};
use giong_database::Database;
use giong_database::entities::{secret_set_entity::SecretSetEntity, stack_entity::StackEntity};
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use giong_runner::types::Secret;
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub enum SecretScope {
    Stack(Uuid),
    Set(Uuid),
}

impl SecretScope {
    fn build_encryption_context(self, key: &str) -> String {
        match self {
            Self::Stack(id) => format!("stack:{id}:{key}"),
            // Preserve the encryption context independently of table names.
            Self::Set(id) => format!("variable-set:{id}:{key}"),
        }
    }
}

#[derive(Serialize)]
pub struct SecretInfo {
    pub key: String,
    pub source: Option<String>,
}

#[derive(Serialize)]
pub struct SecretSetInfo {
    pub id: Uuid,
    pub name: String,
    pub r#type: String,
}

struct ResolvedSecret {
    scope: SecretScope,
    value: String,
    source: Option<String>,
}

#[derive(Injectable)]
pub struct SecretService {
    config: Arc<Config>,
    database: Arc<Database>,
}

impl SecretService {
    pub async fn list_secrets(&self, scope: SecretScope) -> Result<Vec<SecretInfo>, Exception> {
        Ok(self
            .resolve_secrets(scope)
            .await?
            .into_iter()
            .map(|(key, secret)| SecretInfo { key, source: secret.source })
            .collect())
    }

    pub async fn get_run_secrets(&self, stack_id: Uuid) -> Result<Vec<Secret>, Exception> {
        let mut secrets = Vec::new();
        for (key, secret) in self.resolve_secrets(SecretScope::Stack(stack_id)).await? {
            let value = decrypt_token(
                &secret.value,
                self.get_encryption_key()?,
                secret.scope.build_encryption_context(&key).as_bytes(),
            )
            .with_context(|| format!("Could not decrypt secret {key}"))?;
            secrets.push(Secret { key, value });
        }
        Ok(secrets)
    }

    pub async fn create_secret(
        &self,
        scope: SecretScope,
        key: String,
        value: String,
    ) -> Result<(), Exception> {
        let value = encrypt_token(
            &value,
            self.get_encryption_key()?,
            scope.build_encryption_context(&key).as_bytes(),
        )?;
        let set_id = match scope {
            SecretScope::Stack(id) => self.get_or_create_local_set(id).await?,
            SecretScope::Set(id) => self.get_shared_set(id).await?.id,
        };
        let condition = Condition::new().eq("secret_set_id", set_id).eq("key", key.as_str());
        if self.database.secrets.exists(condition).await? {
            return Err(Exception::Conflict("modules.secret.already_exists"));
        }
        self.database
            .secrets
            .create_one(
                Document::new().set("secret_set_id", set_id).set("key", key).set("value", value),
            )
            .await?;
        Ok(())
    }

    pub async fn update_secret(
        &self,
        scope: SecretScope,
        key: String,
        value: String,
    ) -> Result<(), Exception> {
        let set_id = self.get_set_id(scope).await?;
        let condition = Condition::new().eq("secret_set_id", set_id).eq("key", key.as_str());
        let value = encrypt_token(
            &value,
            self.get_encryption_key()?,
            scope.build_encryption_context(&key).as_bytes(),
        )?;
        if !self.database.secrets.update_one(condition, Document::new().set("value", value)).await?
        {
            return Err(Exception::NotFound("modules.secret.not_found"));
        }
        Ok(())
    }

    pub async fn delete_secret(&self, scope: SecretScope, key: String) -> Result<(), Exception> {
        let set_id = self.get_set_id(scope).await?;
        self.database
            .secrets
            .delete_one(Condition::new().eq("secret_set_id", set_id).eq("key", key))
            .await?;
        Ok(())
    }

    pub async fn list_sets(&self) -> Result<Vec<SecretSetInfo>, Exception> {
        let mut sets: Vec<_> = self
            .database
            .secret_sets
            .find(Condition::new().eq("type", "shared"))
            .await?
            .into_iter()
            .map(|set| SecretSetInfo {
                id: set.id,
                name: set.name.unwrap_or_default(),
                r#type: set.r#type,
            })
            .collect();
        sets.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(sets)
    }

    pub async fn create_set(&self, name: String) -> Result<SecretSetInfo, Exception> {
        let name = name.trim();
        if name.is_empty() {
            return Err(Exception::BadRequest("Secret set name is required".into()));
        }
        if self.database.secret_sets.exists(Condition::new().eq("name", name)).await? {
            return Err(Exception::Conflict("Secret set already exists"));
        }
        let set = self
            .database
            .secret_sets
            .create_one(Document::new().set("name", name).set("type", "shared"))
            .await?;
        Ok(SecretSetInfo { id: set.id, name: name.to_owned(), r#type: set.r#type })
    }

    pub async fn delete_set(&self, id: Uuid) -> Result<(), Exception> {
        self.get_shared_set(id).await?;
        for stack in self.database.stacks.find(Condition::new()).await? {
            let inherited_ids: Vec<Uuid> = serde_json::from_value(stack.inherited_secret_set_ids)
                .map_err(Exception::unexpected)?;
            if inherited_ids.contains(&id) {
                return Err(Exception::Conflict("Secret set is inherited by a stack"));
            }
        }
        self.database.secret_sets.delete_one(Condition::new().eq("id", id)).await?;
        Ok(())
    }

    pub async fn update_inherited_sets(
        &self,
        stack_id: Uuid,
        set_ids: Vec<Uuid>,
    ) -> Result<(), Exception> {
        self.get_stack(stack_id).await?;
        let mut seen = HashSet::new();
        for id in &set_ids {
            if !seen.insert(id) {
                return Err(Exception::BadRequest("Duplicate secret set".into()));
            }
            self.get_shared_set(*id).await?;
        }
        self.database
            .stacks
            .update_one(
                Condition::new().eq("id", stack_id),
                Document::new().set(
                    "inherited_secret_set_ids",
                    serde_json::to_value(set_ids).map_err(Exception::unexpected)?,
                ),
            )
            .await?;
        Ok(())
    }

    async fn resolve_secrets(
        &self,
        scope: SecretScope,
    ) -> Result<BTreeMap<String, ResolvedSecret>, Exception> {
        let mut secrets = BTreeMap::new();
        let local_set_id = match scope {
            SecretScope::Set(id) => Some(self.get_shared_set(id).await?.id),
            SecretScope::Stack(id) => {
                let stack = self.get_stack(id).await?;
                let inherited_ids: Vec<Uuid> =
                    serde_json::from_value(stack.inherited_secret_set_ids)
                        .map_err(Exception::unexpected)?;
                // Later shared sets override earlier ones.
                for id in inherited_ids {
                    let set = self.get_shared_set(id).await?;
                    secrets.extend(
                        self.read_set_secrets(Some(id), SecretScope::Set(id), set.name).await?,
                    );
                }
                stack.secret_set_id
            }
        };
        // Stack-local values take precedence over inherited values.
        secrets.extend(self.read_set_secrets(local_set_id, scope, None).await?);
        Ok(secrets)
    }

    async fn read_set_secrets(
        &self,
        set_id: Option<Uuid>,
        scope: SecretScope,
        source: Option<String>,
    ) -> Result<BTreeMap<String, ResolvedSecret>, Exception> {
        let mut secrets = BTreeMap::new();
        let Some(set_id) = set_id else {
            return Ok(secrets);
        };
        for secret in
            self.database.secrets.find(Condition::new().eq("secret_set_id", set_id)).await?
        {
            secrets.insert(
                secret.key,
                ResolvedSecret { scope, value: secret.value, source: source.clone() },
            );
        }
        Ok(secrets)
    }

    async fn get_set_id(&self, scope: SecretScope) -> Result<Option<Uuid>, Exception> {
        match scope {
            SecretScope::Stack(id) => Ok(self.get_stack(id).await?.secret_set_id),
            SecretScope::Set(id) => Ok(Some(self.get_shared_set(id).await?.id)),
        }
    }

    async fn get_or_create_local_set(&self, stack_id: Uuid) -> Result<Uuid, Exception> {
        if let Some(id) = self.get_stack(stack_id).await?.secret_set_id {
            return Ok(id);
        }
        let set =
            self.database.secret_sets.create_one(Document::new().set("type", "local")).await?;
        let linked = self
            .database
            .stacks
            .update_one(
                Condition::new().eq("id", stack_id).eq("secret_set_id", None::<Uuid>),
                Document::new().set("secret_set_id", set.id),
            )
            .await?;
        if linked {
            return Ok(set.id);
        }
        // Another request may have linked a set first. Discard the unused set.
        self.database.secret_sets.delete_one(Condition::new().eq("id", set.id)).await?;
        self.get_stack(stack_id)
            .await?
            .secret_set_id
            .ok_or(Exception::NotFound("Secret set not found"))
    }

    async fn get_stack(&self, id: Uuid) -> Result<StackEntity, Exception> {
        self.database
            .stacks
            .find_one(Condition::new().eq("id", id))
            .await?
            .ok_or(Exception::NotFound("modules.stack.not_found"))
    }

    async fn get_shared_set(&self, id: Uuid) -> Result<SecretSetEntity, Exception> {
        self.database
            .secret_sets
            .find_one(Condition::new().eq("id", id).eq("type", "shared"))
            .await?
            .ok_or(Exception::NotFound("Secret set not found"))
    }

    fn get_encryption_key(&self) -> Result<&str> {
        self.config.encryption_key.as_deref().context("Encryption key is not configured")
    }
}
