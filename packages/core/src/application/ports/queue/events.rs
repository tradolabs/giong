use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum QueueEvent {
    RunNode(RunNodeEvent),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunNodeEvent {
    pub run_id: Uuid,
}

impl From<RunNodeEvent> for QueueEvent {
    fn from(event: RunNodeEvent) -> Self {
        Self::RunNode(event)
    }
}
