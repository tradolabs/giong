use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::helpers::filesystem::root_dir;
use crate::infrastructure::adapters::node_runs::NodeRuns;
use giong_di::Injectable;
use std::io::ErrorKind;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetNodeRunLogsInput {
    pub run_id: Uuid,
    pub from_lines: usize,
}

#[derive(Injectable)]
pub struct GetNodeRunLogsUseCase {
    runs: Arc<NodeRuns>,
}

impl UseCase for GetNodeRunLogsUseCase {
    type Input = GetNodeRunLogsInput;
    type Output = Option<String>;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let run_id = &input.run_id;

        let Some(run) = self.runs.get(*run_id)? else {
            return Ok(None);
        };
        let log_path = if let Some(timestamp) = run.log_timestamp {
            crate::application::helpers::node_logs::path(run.node_id, &timestamp)
        } else {
            root_dir().join("logs").join(run_id.to_string())
        };
        let content = match tokio::fs::read_to_string(log_path).await {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            Err(error) => return Err(Exception::unexpected(error)),
        };

        Ok(Some(content.split_inclusive('\n').skip(input.from_lines).collect()))
    }
}
