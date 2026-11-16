use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Pending,
    #[serde(alias = "running")]
    Executing,
    Succeeded,
    Failed,
    Interrupted,
}
impl ExecutionStatus {
    pub fn active(self) -> bool {
        matches!(self, Self::Pending | Self::Executing)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeExecution {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub node_id: Uuid,
    pub status: ExecutionStatus,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub exit_code: Option<i32>,
    pub created_at: String,
}
