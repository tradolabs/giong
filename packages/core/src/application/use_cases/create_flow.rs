use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::flow::FlowService;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::entities::vault_entity::VaultScope;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct CreateFlowUseCase {
    database: Arc<Database>,
}

pub struct CreateFlowInput {
    pub name: String,
    pub inherited_vault_ids: Vec<Uuid>,
}

impl CreateFlowUseCase {
    async fn validate_inherited_vaults(&self, vault_ids: &[Uuid]) -> Result<(), Exception> {
        let mut seen = HashSet::with_capacity(vault_ids.len());
        for &vault_id in vault_ids {
            if !seen.insert(vault_id) {
                return Err(Exception::BadRequest("Duplicate vault".into()));
            }
        }
        if !vault_ids.is_empty() {
            let found = self
                .database
                .vaults
                .find(Condition::new().is_in("id", vault_ids.iter().copied()))
                .await?;
            if found.len() != vault_ids.len() {
                return Err(ResourceException::NotFound(Resource::Vault).into());
            }
        }
        Ok(())
    }
}

impl UseCase for CreateFlowUseCase {
    type Input = CreateFlowInput;
    type Output = FlowEntity;

    async fn execute(&self, input: Self::Input) -> Result<FlowEntity, Exception> {
        let flow_name = input.name;

        if self.database.flows.exists(Condition::new().eq("name", flow_name.clone())).await? {
            return Err(ResourceException::AlreadyExists(Resource::Flow).into());
        }
        self.validate_inherited_vaults(&input.inherited_vault_ids).await?;

        let inherited_vault_ids = serde_json::to_value(input.inherited_vault_ids)?;
        let flow = self
            .database
            .transaction(async || {
                let vault_name = FlowService::unique_vault_name(&self.database, &flow_name).await?;
                let vault = self
                    .database
                    .vaults
                    .create_one(
                        Document::new().set("name", vault_name).set("scope", VaultScope::Flow),
                    )
                    .await?;
                self.database
                    .flows
                    .create_one(
                        Document::new()
                            .set("name", flow_name.clone())
                            .set("vault_id", vault.id)
                            .set("inherited_vault_ids", inherited_vault_ids),
                    )
                    .await
            })
            .await?;

        Ok(flow)
    }
}
