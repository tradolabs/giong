use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::node::NodeService;
use giong_di::Injectable;
use std::io::ErrorKind;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetNodeLogsInput {
    pub execution_id: Uuid,
    pub from_lines: usize,
}

#[derive(Injectable)]
pub struct GetNodeLogsUseCase {
    node_service: Arc<NodeService>,
}

impl UseCase for GetNodeLogsUseCase {
    type Input = GetNodeLogsInput;
    type Output = Option<String>;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let Some(execution) = self.node_service.find_execution(input.execution_id).await? else {
            return Ok(None);
        };
        let path = NodeService::get_log_path(execution.node_id, execution.id);
        let content = match tokio::fs::read_to_string(path).await {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            Err(error) => return Err(Exception::unexpected(error)),
        };

        Ok(Some(content.split_inclusive('\n').skip(input.from_lines).collect()))
    }
}
