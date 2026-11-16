use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::flow_service::FlowService;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct UpdateFlowUseCase {
    database: Arc<Database>,
    flow_service: Arc<FlowService>,
}
pub struct UpdateFlowInput {
    pub id: Uuid,
    pub name: String,
    pub inherited_vault_ids: Vec<Uuid>,
}

impl UseCase for UpdateFlowUseCase {
    type Input = UpdateFlowInput;
    type Output = FlowEntity;
    async fn execute(&self, input: Self::Input) -> Result<FlowEntity, Exception> {
        let name = input.name;
        let inherited_vault_ids = input.inherited_vault_ids;
        let name = name.trim().to_owned();

        let is_name_existed = self
            .database
            .flows
            .exists(Condition::new().eq("name", name.as_str()).ne("id", input.id))
            .await?;
        if is_name_existed {
            return Err(ResourceException::AlreadyExists(Resource::Flow).into());
        }

        self.database
            .flows
            .update_one(
                Condition::new().eq("id", input.id),
                Document::new()
                    .set("name", name)
                    .set("inherited_vault_ids", serde_json::to_value(inherited_vault_ids)?),
            )
            .await?;

        self.flow_service.get_flow_with_nodes(input.id).await
    }
}
