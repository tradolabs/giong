use crate::application::exceptions::base::Exception;
use crate::application::helpers::nodes::stored_nodes;
use crate::application::helpers::secrets::validate_vaults;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateFlowVaultsInput {
    pub flow_id: Uuid,
    pub inherited_vault_ids: Vec<Uuid>,
}
#[derive(Injectable)]
pub struct UpdateFlowVaultsUseCase {
    database: Arc<Database>,
}
impl UseCase for UpdateFlowVaultsUseCase {
    type Input = UpdateFlowVaultsInput;
    type Output = FlowEntity;
    async fn execute(&self, input: Self::Input) -> Result<FlowEntity, Exception> {
        let mut flow = self
            .database
            .flows
            .find_one(Condition::new().eq("id", input.flow_id))
            .await?
            .ok_or(Exception::NotFound("modules.flow.not_found"))?;
        validate_vaults(&self.database, flow.vault_id, &input.inherited_vault_ids).await?;
        let updated = self
            .database
            .flows
            .update_one(
                Condition::new().eq("id", flow.id),
                Document::new()
                    .set("inherited_vault_ids", serde_json::to_value(input.inherited_vault_ids)?),
            )
            .await?;
        if !updated {
            return Err(Exception::NotFound("modules.flow.not_found"));
        }
        flow = self
            .database
            .flows
            .find_one(Condition::new().eq("id", flow.id))
            .await?
            .ok_or(Exception::NotFound("modules.flow.not_found"))?;
        flow.nodes = stored_nodes(&self.database, flow.id).await?;
        Ok(flow)
    }
}
