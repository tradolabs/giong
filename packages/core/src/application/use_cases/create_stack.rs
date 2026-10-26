use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::Database;
use giong_database::Document;
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
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        self.database
            .stacks
            .create_one(
                Document::new()
                    .set("name", input.name.as_str())
                    .set("repo_url", input.repo_url.as_str())
                    .set("branch", input.branch.as_str())
                    .set("folder_path", input.folder_path.as_str()),
            )
            .await?;

        Ok(())
    }
}
