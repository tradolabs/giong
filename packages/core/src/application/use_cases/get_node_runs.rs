use crate::application::exceptions::base::Exception;
use crate::application::types::node_run::NodeRun;
use crate::application::use_cases::use_case::UseCase;
use crate::infrastructure::adapters::node_runs::NodeRuns;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;
#[derive(Injectable)]
pub struct GetNodeRunsUseCase {
    runs: Arc<NodeRuns>,
}
impl UseCase for GetNodeRunsUseCase {
    type Input = Uuid;
    type Output = Vec<NodeRun>;
    async fn execute(&self, flow_id: Uuid) -> Result<Vec<NodeRun>, Exception> {
        Ok(self.runs.list()?.into_iter().filter(|run| run.flow_id == flow_id).collect())
    }
}
