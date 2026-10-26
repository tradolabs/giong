use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum QueueEvent {
    RunJob(RunJobEvent),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunJobEvent {
    pub job_id: Uuid,
}

impl From<RunJobEvent> for QueueEvent {
    fn from(event: RunJobEvent) -> Self {
        Self::RunJob(event)
    }
}
