use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::vault_entity::VaultEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateVaultInput {
    pub id: Uuid,
    pub name: String,
}

#[derive(Injectable)]
pub struct UpdateVaultUseCase {
    database: Arc<Database>,
}

impl UseCase for UpdateVaultUseCase {
    type Input = UpdateVaultInput;
    type Output = VaultEntity;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let name = input.name.trim();
        let is_name_used = self
            .database
            .vaults
            .exists(Condition::new().eq("name", name).ne("id", input.id))
            .await?;

        if is_name_used {
            return Err(ResourceException::AlreadyExists(Resource::Vault).into());
        }

        self.database
            .vaults
            .update_one(Condition::new().eq("id", input.id), Document::new().set("name", name))
            .await?;
        self.database
            .vaults
            .find_one(Condition::new().eq("id", input.id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Vault))
            .map_err(Exception::from)
    }
}
