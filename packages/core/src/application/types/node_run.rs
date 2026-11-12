use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Interrupted,
}
impl RunStatus {
    pub fn active(self) -> bool {
        matches!(self, Self::Pending | Self::Running)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeRun {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub node_id: Uuid,
    pub status: RunStatus,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub exit_code: Option<i32>,
    pub created_at: String,
    #[serde(default)]
    pub log_timestamp: Option<String>,
}
