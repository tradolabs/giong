use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::Database;
use giong_database::entities::stack_entity::StackEntity;
use giong_database::enums::job_status::JobStatus;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateStackInput {
    pub stack_id: Uuid,
    pub repo_url: Option<String>,
    pub folder_path: Option<String>,
}

#[derive(Injectable)]
pub struct UpdateStackUseCase {
    database: Arc<Database>,
}

impl UseCase for UpdateStackUseCase {
    type Input = UpdateStackInput;
    type Output = StackEntity;

    async fn execute(&self, input: Self::Input) -> Result<StackEntity, Exception> {
        let stack = self
            .database
            .stacks
            .find_one(Condition::new().eq("id", input.stack_id))
            .await?
            .ok_or(Exception::NotFound("modules.stack.not_found"))?;
        let repo_url = input.repo_url.unwrap_or_else(|| stack.repo_url.clone());
        let folder_path = input.folder_path.unwrap_or_else(|| stack.folder_path.clone());
        if !self.database.repositories.exists(Condition::new().eq("url", repo_url.as_str())).await?
        {
            return Err(Exception::NotFound("modules.repository.not_found"));
        }
        let repository_changed = repo_url != stack.repo_url;
        if repository_changed
            && self
                .database
                .jobs
                .exists(
                    Condition::new()
                        .eq("stack_id", input.stack_id)
                        .is_in("status", [JobStatus::Pending, JobStatus::Running]),
                )
                .await?
        {
            return Err(Exception::Conflict("modules.job.active_job_exists"));
        }
        let mut changes = Document::new()
            .set("repo_url", repo_url)
            .set("folder_path", folder_path);
        if repository_changed {
            changes = changes.set("last_commit", None::<String>);
        }
        let updated = self
            .database
            .stacks
            .update_one(Condition::new().eq("id", input.stack_id), changes)
            .await?;
        if !updated {
            return Err(Exception::NotFound("modules.stack.not_found"));
        }
        self.database
            .stacks
            .find_one(Condition::new().eq("id", input.stack_id))
            .await?
            .ok_or(Exception::NotFound("modules.stack.not_found"))
    }
}
