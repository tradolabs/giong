use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::error::Error as DatabaseError;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct DeleteVaultUseCase {
    database: Arc<Database>,
}

impl UseCase for DeleteVaultUseCase {
    type Input = Uuid;
    type Output = ();
    async fn execute(&self, id: Uuid) -> Result<(), Exception> {
        if self.database.flows.exists(Condition::new().eq("vault_id", id)).await? {
            return Err(ResourceException::InUse(Resource::Vault).into());
        }
        self.database
            .transaction(async || {
                for flow in self.database.flows.find(Condition::new()).await? {
                    let mut inherited: Vec<Uuid> = serde_json::from_value(flow.inherited_vault_ids)
                        .map_err(|error| DatabaseError::Unexpected(error.to_string()))?;
                    if !inherited.contains(&id) {
                        continue;
                    }
                    inherited.retain(|vault_id| *vault_id != id);
                    let inherited = serde_json::to_value(inherited)
                        .map_err(|error| DatabaseError::Unexpected(error.to_string()))?;
                    self.database
                        .flows
                        .update_one(
                            Condition::new().eq("id", flow.id),
                            Document::new().set("inherited_vault_ids", inherited),
                        )
                        .await?;
                }
                self.database.vaults.delete_one(Condition::new().eq("id", id)).await?;
                Ok(())
            })
            .await?;
        Ok(())
    }
}
