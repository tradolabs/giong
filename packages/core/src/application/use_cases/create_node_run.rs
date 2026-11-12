use crate::application::exceptions::base::Exception;
use crate::application::ports::queue::{events::RunNodeEvent, queue_manager::QueueManager};
use crate::application::types::node_run::NodeRun;
use crate::application::use_cases::use_case::UseCase;
use crate::infrastructure::adapters::{node_runs::NodeRuns, node_storage::NodeStorage};
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;
#[derive(Injectable)]
pub struct CreateNodeRunUseCase {
    database: Arc<Database>,
    runs: Arc<NodeRuns>,
    storage: Arc<NodeStorage>,
    queue: Arc<dyn QueueManager>,
}
pub struct CreateNodeRunInput {
    pub flow_id: Uuid,
    pub node_id: Uuid,
}
impl UseCase for CreateNodeRunUseCase {
    type Input = CreateNodeRunInput;
    type Output = NodeRun;
    async fn execute(&self, input: Self::Input) -> Result<NodeRun, Exception> {
        self.submit(input).await?.ok_or(Exception::Conflict("Node already has an active run"))
    }
}
impl CreateNodeRunUseCase {
    pub async fn submit(&self, input: CreateNodeRunInput) -> Result<Option<NodeRun>, Exception> {
        let lock = self.storage.flow_lock(&input.flow_id.to_string())?;
        let _guard = lock.lock().await;
        if !self
            .database
            .nodes
            .exists(Condition::new().eq("id", input.node_id).eq("flow_id", input.flow_id))
            .await?
        {
            return Err(Exception::NotFound("Node not found in flow"));
        }
        if self.runs.list()?.iter().any(|run| run.node_id == input.node_id && run.status.active()) {
            return Ok(None);
        }
        let run = self.runs.create(input.flow_id, input.node_id)?;
        self.queue.enqueue(RunNodeEvent { run_id: run.id }.into()).await?;
        Ok(Some(run))
    }
}
