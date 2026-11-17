use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::flow::FlowService;
use crate::application::services::node::NodeService;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::node_entity::NodeEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use giong_node::types::node::Trigger;
use std::sync::Arc;
use uuid::Uuid;
pub struct CreateNodeInput {
    pub flow_id: Uuid,
    pub name: String,
    pub kind: String,
    pub properties: serde_json::Value,
    pub triggers: Vec<Trigger>,
}
#[derive(Injectable)]
pub struct CreateNodeUseCase {
    database: Arc<Database>,
    node_service: Arc<NodeService>,
    flow_service: Arc<FlowService>,
}
impl UseCase for CreateNodeUseCase {
    type Input = CreateNodeInput;
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
                None,
                &input.name,
                &input.kind,
                &input.properties,
                &input.triggers,
            )
            .await?;
        let changes = Document::new()
            .set("name", input.name)
            .set("kind", input.kind)
            .set("properties", input.properties)
            .set("triggers", serde_json::to_value(&input.triggers)?);
        let node = self
            .database
            .nodes
            .create_one(changes.set("id", Uuid::now_v7()).set("flow_id", flow.id))
            .await?;
        self.node_service.sync_interval_trigger(flow.id, node.id, &input.triggers).await?;
        self.flow_service.invalidate_graph(flow.id).await;
        Ok(node)
    }
}
