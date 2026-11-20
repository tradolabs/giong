use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
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
    pub vault_ids: Vec<Uuid>,
}

impl CreateFlowUseCase {
    async fn validate_vaults(&self, vault_ids: &[Uuid]) -> Result<(), Exception> {
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
        self.validate_vaults(&input.vault_ids).await?;

        let vault_ids = serde_json::to_value(input.vault_ids)?;
        let flow = self
            .database
            .flows
            .create_one(Document::new().set("name", flow_name).set("vault_ids", vault_ids))
            .await?;

        Ok(flow)
    }
}
