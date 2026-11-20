use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::node::NodeService;
use crate::application::types::node_execution::NodeExecution;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct NodeDetails {
    pub output: Option<serde_json::Value>,
    pub executions: Vec<NodeExecution>,
}

#[derive(Injectable)]
pub struct GetNodeDetailsUseCase {
    database: Arc<Database>,
    node_service: Arc<NodeService>,
}

impl UseCase for GetNodeDetailsUseCase {
    type Input = (Uuid, Uuid);
    type Output = NodeDetails;

    async fn execute(&self, (flow_id, node_id): Self::Input) -> Result<Self::Output, Exception> {
        if !self
            .database
            .nodes
            .exists(Condition::new().eq("id", node_id).eq("flow_id", flow_id))
            .await?
        {
            return Err(ResourceException::NotFound(Resource::Node).into());
        }
        let output = self.node_service.get_output(node_id)?;
        let executions = self.node_service.get_executions(node_id)?;
        Ok(NodeDetails { output, executions })
    }
}
