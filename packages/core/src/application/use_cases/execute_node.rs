use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::node::NodeService;
use crate::application::services::node_host::NodeHostService;
use crate::application::types::node_execution::{ExecutionStatus, NodeExecution};
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_node::host::NodeScope;
use giong_node::runtime::{ExecutionContext, Plugin};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

pub struct ExecuteNodeInput {
    pub node_id: Uuid,
}

#[derive(Injectable)]
pub struct ExecuteNodeUseCase {
    database: Arc<Database>,
    node_host: Arc<NodeHostService>,
    node_service: Arc<NodeService>,
}

impl ExecuteNodeUseCase {
    async fn run_execution(&self, execution: &NodeExecution) -> Result<(), Exception> {
        let mut logger = NodeService::open_log(execution.node_id, execution.id)?;
        let flow_id = execution.flow_id;
        let node = self
            .database
            .nodes
            .find_one(Condition::new().eq("flow_id", flow_id).eq("id", execution.node_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Node))?;

        let plugin = Plugin::new(&node.kind)?;
        let context = ExecutionContext {
            host: self.node_host.clone(),
            scope: NodeScope {
                flow_id,
                node_id: node.id,
                execution_id: execution.id,
                kind: node.kind.clone(),
            },
        };
        writeln!(logger, "Executing node {} ({})", node.name, node.kind)?;
        plugin.execute(context, node.properties).await.map_err(Exception::unexpected)
    }

    fn finish_execution(
        &self,
        mut execution: NodeExecution,
        log_path: &Path,
        result: Result<(), Exception>,
    ) -> Result<(), Exception> {
        if let Err(error) = &result
            && let Ok(mut file) = std::fs::File::options().create(true).append(true).open(log_path)
        {
            let _ = writeln!(file, "{error}");
        }

        execution.status =
            if result.is_ok() { ExecutionStatus::Succeeded } else { ExecutionStatus::Failed };
        execution.exit_code = Some(if result.is_ok() { 0 } else { 1 });
        execution.finished_at = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        if !self.node_service.update_execution(execution)? {
            match std::fs::remove_file(log_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}

impl UseCase for ExecuteNodeUseCase {
    type Input = ExecuteNodeInput;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        let Some(execution) = self.node_service.claim_pending_execution(input.node_id)? else {
            return Ok(());
        };
        let log_path = NodeService::get_log_path(execution.node_id, execution.id);
        let result = self.run_execution(&execution).await;
        self.finish_execution(execution, &log_path, result)
    }
}
