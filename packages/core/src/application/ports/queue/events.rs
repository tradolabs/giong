use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum QueueEvent {
    ExecuteNode(ExecuteNodeEvent),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExecuteNodeEvent {
    pub node_id: Uuid,
}

impl From<ExecuteNodeEvent> for QueueEvent {
    fn from(event: ExecuteNodeEvent) -> Self {
        Self::ExecuteNode(event)
    }
}
