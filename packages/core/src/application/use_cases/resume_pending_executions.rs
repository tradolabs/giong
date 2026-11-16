use crate::application::exceptions::base::Exception;
use crate::application::ports::queue::{events::ExecuteNodeEvent, queue_manager::QueueManager};
use crate::application::services::node_service::NodeService;
use crate::application::types::node_execution::ExecutionStatus;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_kv::kv::Kv;
use std::sync::Arc;

#[derive(Injectable)]
pub struct ResumePendingExecutionsUseCase {
    database: Arc<Database>,
    kv: Arc<Kv>,
    queue: Arc<dyn QueueManager>,
    node_service: Arc<NodeService>,
}

impl UseCase for ResumePendingExecutionsUseCase {
    type Input = ();
    type Output = ();

    async fn execute(&self, _: ()) -> Result<(), Exception> {
        for node in self.database.nodes.find(Condition::new()).await? {
            for mut execution in self.node_service.get_executions(node.id)? {
                if execution.status == ExecutionStatus::Executing {
                    execution.status = ExecutionStatus::Interrupted;
                    execution.exit_code = None;
                    execution.finished_at =
                        Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
                    self.kv
                        .namespace(&format!("nodes/{}", execution.node_id))
                        .put(
                            format!("executions/{}", execution.id).as_bytes(),
                            &serde_json::to_vec(&execution)?,
                        )?;
                } else if execution.status == ExecutionStatus::Pending {
                    self.queue
                        .enqueue(ExecuteNodeEvent { node_id: execution.node_id }.into())
                        .await?;
                }
            }
        }
        Ok(())
    }
}
