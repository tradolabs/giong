use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::helpers::flow_graph::{FlowGraph, FlowGraphCache};
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::entities::node_entity::NodeEntity;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct FlowService {
    database: Arc<Database>,
    cache: Arc<FlowGraphCache>,
}

impl FlowService {
    pub(crate) async fn get_flow_with_nodes(&self, flow_id: Uuid) -> Result<FlowEntity, Exception> {
        let mut flow = self
            .database
            .flows
            .find_one(Condition::new().eq("id", flow_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Flow))?;
        self.populate_nodes(std::slice::from_mut(&mut flow)).await?;
        Ok(flow)
    }

    pub(crate) async fn populate_nodes(&self, flows: &mut [FlowEntity]) -> Result<(), Exception> {
        if flows.is_empty() {
            return Ok(());
        }
        let flow_ids = flows.iter().map(|flow| flow.id).collect::<Vec<_>>();
        let nodes = self.database.nodes.find(Condition::new().is_in("flow_id", flow_ids)).await?;
        let mut by_flow: HashMap<Uuid, Vec<NodeEntity>> = HashMap::new();
        for node in nodes {
            by_flow.entry(node.flow_id).or_default().push(node);
        }
        for flow in flows {
            let mut flow_nodes = by_flow.remove(&flow.id).unwrap_or_default();
            flow_nodes.sort_by_key(|node| node.id);
            flow.nodes = flow_nodes;
        }
        Ok(())
    }

    pub(crate) async fn get_event_targets(
        &self,
        flow_id: Uuid,
        source_node_id: Uuid,
    ) -> Result<Vec<Uuid>, Exception> {
        if let Some(graph) = self.cache.graphs.read().await.get(&flow_id).cloned() {
            return Ok(graph.targets(source_node_id));
        }

        let nodes = self.database.nodes.find(Condition::new().eq("flow_id", flow_id)).await?;
        let graph = Arc::new(FlowGraph::from_nodes(&nodes)?);
        let targets = graph.targets(source_node_id);
        self.cache.graphs.write().await.insert(flow_id, graph);
        Ok(targets)
    }

    pub(crate) async fn invalidate_graph(&self, flow_id: Uuid) {
        self.cache.graphs.write().await.remove(&flow_id);
    }
}
