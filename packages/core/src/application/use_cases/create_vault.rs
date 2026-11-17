use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::vault_entity::{VaultEntity, VaultScope};
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateVaultInput {
    pub name: String,
}

#[derive(Injectable)]
pub struct CreateVaultUseCase {
    database: Arc<Database>,
}

impl UseCase for CreateVaultUseCase {
    type Input = CreateVaultInput;
    type Output = VaultEntity;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let name = input.name.as_str();
        if self.database.vaults.exists(Condition::new().eq("name", name)).await? {
            return Err(ResourceException::AlreadyExists(Resource::Vault).into());
        }
        let vault = self
            .database
            .vaults
            .create_one(Document::new().set("name", name).set("scope", VaultScope::Global))
            .await?;
        Ok(vault)
    }
}
