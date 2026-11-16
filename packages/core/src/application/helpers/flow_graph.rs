use crate::application::exceptions::base::Exception;
use giong_database::entities::node_entity::NodeEntity;
use giong_node::types::node::Trigger;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Default)]
pub struct FlowGraphCache {
    pub(crate) graphs: RwLock<HashMap<Uuid, Arc<FlowGraph>>>,
}

pub struct FlowGraph {
    subscribers: HashMap<(Uuid, String), Vec<Uuid>>,
}

impl FlowGraph {
    pub(crate) fn from_nodes(nodes: &[NodeEntity]) -> Result<Self, Exception> {
        let name_to_id: HashMap<&str, Uuid> =
            nodes.iter().map(|node| (node.name.as_str(), node.id)).collect();
        let mut subscribers: HashMap<(Uuid, String), Vec<Uuid>> = HashMap::new();
        for node in nodes {
            let triggers: Vec<Trigger> = serde_json::from_value(node.triggers.clone())?;
            for trigger in triggers {
                if let Trigger::Event { node: source_name, event } = trigger
                    && let Some(&source_id) = name_to_id.get(source_name.as_str())
                {
                    subscribers.entry((source_id, event)).or_default().push(node.id);
                }
            }
        }
        Ok(Self { subscribers })
    }

    pub(crate) fn targets(&self, source_node_id: Uuid, event: &str) -> Vec<Uuid> {
        self.subscribers
            .get(&(source_node_id, event.to_owned()))
            .cloned()
            .unwrap_or_default()
    }
}
