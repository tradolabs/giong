use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::node::NodeService;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::node_entity::NodeEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use giong_node::host::Trigger;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateNodeInput {
    pub flow_id: Uuid,
    pub node_id: Uuid,
    pub name: String,
    pub kind: String,
    pub properties: serde_json::Value,
    pub triggers: Vec<Trigger>,
}

#[derive(Injectable)]
pub struct UpdateNodeUseCase {
    database: Arc<Database>,
    node_service: Arc<NodeService>,
}

impl UseCase for UpdateNodeUseCase {
    type Input = UpdateNodeInput;
    type Output = NodeEntity;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let flow = self
            .database
            .flows
            .find_one(Condition::new().eq("id", input.flow_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Flow))?;

        self.node_service
            .validate_flow_nodes(
                flow.id,
                Some(input.node_id),
                &input.name,
                &input.kind,
                &input.properties,
                &input.triggers,
            )
            .await?;

        self.database
            .nodes
            .update_one(
                Condition::new().eq("id", input.node_id).eq("flow_id", flow.id),
                Document::new()
                    .set("name", input.name)
                    .set("kind", input.kind)
                    .set("properties", input.properties)
                    .set("triggers", serde_json::to_value(&input.triggers)?),
            )
            .await?;

        self.node_service.sync_node(flow.id, input.node_id, &input.triggers).await
    }
}
