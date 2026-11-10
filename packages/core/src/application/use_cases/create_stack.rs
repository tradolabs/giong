use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::Database;
use giong_database::entities::stack_entity::StackEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;

pub struct CreateStackInput {
    pub name: String,
    pub repo_url: String,
    pub branch: String,
    pub folder_path: String,
}

#[derive(Injectable)]
pub struct CreateStackUseCase {
    database: Arc<Database>,
}

impl UseCase for CreateStackUseCase {
    type Input = CreateStackInput;
    type Output = StackEntity;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        if input.name.trim().is_empty() || input.name.chars().any(char::is_control) {
            return Err(Exception::BadRequest(
                "Stack name is required and cannot contain control characters".into(),
            ));
        }
        if input.branch.trim().is_empty()
            || input.branch.chars().any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(Exception::BadRequest(
                "Branch is required and cannot contain whitespace".into(),
            ));
        }
        if !self
            .database
            .repositories
            .exists(Condition::new().eq("url", input.repo_url.as_str()))
            .await?
        {
            return Err(Exception::NotFound("modules.repository.not_found"));
        }
        let condition = Condition::new().eq("name", input.name.as_str());
        if self.database.stacks.exists(condition.clone()).await? {
            return Err(Exception::Conflict("modules.stack.already_exists"));
        }
        let result = self
            .database
            .stacks
            .create_one(
                Document::new()
                    .set("name", input.name.as_str())
                    .set("repo_url", input.repo_url.as_str())
                    .set("branch", input.branch.as_str())
                    .set("folder_path", input.folder_path.as_str()),
            )
            .await;
        match result {
            Ok(stack) => Ok(stack),
            Err(_) if self.database.stacks.exists(condition).await? => {
                Err(Exception::Conflict("modules.stack.already_exists"))
            }
            Err(error) => Err(error.into()),
        }
    }
}
