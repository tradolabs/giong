use crate::application::exceptions::base::Exception;
use crate::application::helpers::secrets::get_vault;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::vault_entity::VaultEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateVaultInput {
    pub name: String,
}
pub struct UpdateVaultInput {
    pub id: Uuid,
    pub name: String,
}
#[derive(Injectable)]
pub struct GetVaultsUseCase {
    database: Arc<Database>,
}
impl UseCase for GetVaultsUseCase {
    type Input = ();
    type Output = Vec<VaultEntity>;
    async fn execute(&self, _: ()) -> Result<Self::Output, Exception> {
        Ok(self
            .database
            .transaction(async || {
                let owned: std::collections::HashSet<_> = self
                    .database
                    .flows
                    .find(Condition::new())
                    .await?
                    .into_iter()
                    .filter_map(|flow| flow.vault_id)
                    .collect();
                let vaults = self.database.vaults.find(Condition::new()).await?;
                Ok(vaults.into_iter().filter(|vault| !owned.contains(&vault.id)).collect())
            })
            .await?)
    }
}
#[derive(Injectable)]
pub struct CreateVaultUseCase {
    database: Arc<Database>,
}
impl UseCase for CreateVaultUseCase {
    type Input = CreateVaultInput;
    type Output = VaultEntity;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let name = input.name.trim();
        if name.is_empty() || name.chars().any(char::is_control) {
            return Err(Exception::BadRequest(
                "Vault name is required and cannot contain control characters".into(),
            ));
        }
        if self.database.vaults.exists(Condition::new().eq("name", name)).await? {
            return Err(Exception::Conflict("Vault already exists"));
        }
        Ok(self.database.vaults.create_one(Document::new().set("name", name)).await?)
    }
}
#[derive(Injectable)]
pub struct UpdateVaultUseCase {
    database: Arc<Database>,
}
impl UseCase for UpdateVaultUseCase {
    type Input = UpdateVaultInput;
    type Output = VaultEntity;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        get_vault(&self.database, input.id).await?;
        let name = input.name.trim();
        if name.is_empty() || name.chars().any(char::is_control) {
            return Err(Exception::BadRequest(
                "Vault name is required and cannot contain control characters".into(),
            ));
        }
        if let Some(existing) =
            self.database.vaults.find_one(Condition::new().eq("name", name)).await?
        {
            if existing.id != input.id {
                return Err(Exception::Conflict("Vault already exists"));
            }
        }
        self.database
            .vaults
            .update_one(Condition::new().eq("id", input.id), Document::new().set("name", name))
            .await?;
        get_vault(&self.database, input.id).await
    }
}
#[derive(Injectable)]
pub struct DeleteVaultUseCase {
    database: Arc<Database>,
}
impl UseCase for DeleteVaultUseCase {
    type Input = Uuid;
    type Output = ();
    async fn execute(&self, id: Uuid) -> Result<(), Exception> {
        get_vault(&self.database, id).await?;
        for flow in self.database.flows.find(Condition::new()).await? {
            let inherited: Vec<Uuid> = serde_json::from_value(flow.inherited_vault_ids)?;
            if flow.vault_id == Some(id) || inherited.contains(&id) {
                return Err(Exception::Conflict("Vault is used by a flow"));
            }
        }
        self.database
            .transaction(async || {
                for secret in
                    self.database.secrets.find(Condition::new().eq("vault_id", id)).await?
                {
                    self.database.secrets.delete_one(Condition::new().eq("id", secret.id)).await?;
                }
                self.database.vaults.delete_one(Condition::new().eq("id", id)).await?;
                Ok(())
            })
            .await?;
        Ok(())
    }
}
