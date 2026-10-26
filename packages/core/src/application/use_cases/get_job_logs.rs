use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::helpers::filesystem::root_dir;
use giong_database::Database;
use giong_database::{Condition, Uuid};
use giong_di::Injectable;
use giong_storage::storage::Storage;
use std::io::ErrorKind;
use std::sync::Arc;

pub struct GetJobLogsInput {
    pub job_id: Uuid,
    pub from_lines: usize,
}

#[derive(Injectable)]
pub struct GetJobLogsUseCase {
    database: Arc<Database>,
    storage: Arc<Storage>,
}

impl UseCase for GetJobLogsUseCase {
    type Input = GetJobLogsInput;
    type Output = Option<String>;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let job_id = &input.job_id;

        if !self.database.jobs.exists(Condition::new().eq("id", *job_id)).await? {
            return Ok(None);
        }

        let log_path = root_dir().join("logs").join(job_id.to_string());
        let content = match tokio::fs::read_to_string(log_path).await {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if let Some(bytes) = self.storage.download(&format!("logs/{job_id}.log")).await? {
                    String::from_utf8(bytes).map_err(Exception::unexpected)?
                } else {
                    String::new()
                }
            }
            Err(error) => return Err(Exception::unexpected(error)),
        };

        Ok(Some(content.split_inclusive('\n').skip(input.from_lines).collect()))
    }
}
