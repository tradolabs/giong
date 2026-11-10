use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::helpers::crypto::encrypt_token;
use crate::config::Config;
use giong_database::Database;
use giong_database::entities::repository_entity::RepositoryEntity;
use giong_database::repositories::types::{condition::Condition, document::Document};
use giong_di::Injectable;
use std::sync::Arc;

pub struct CreateRepositoryInput {
    pub name: String,
    pub url: String,
    pub token: Option<String>,
}

#[derive(Injectable)]
pub struct CreateRepositoryUseCase {
    config: Arc<Config>,
    database: Arc<Database>,
}

impl UseCase for CreateRepositoryUseCase {
    type Input = CreateRepositoryInput;
    type Output = RepositoryEntity;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        if input.name.trim().is_empty() || input.name.chars().any(char::is_control) {
            return Err(Exception::BadRequest(
                "Repository name is required and cannot contain control characters".into(),
            ));
        }
        let url = url::Url::parse(&input.url)
            .map_err(|_| Exception::BadRequest("Enter a valid Git repository URL".into()))?;
        if !matches!(url.scheme(), "https" | "http" | "ssh" | "git")
            || url.host_str().is_none()
            || url.password().is_some()
            || (matches!(url.scheme(), "https" | "http") && !url.username().is_empty())
        {
            return Err(Exception::BadRequest(
                "Use a Git repository URL without embedded credentials".into(),
            ));
        }
        let token = input.token.as_deref().filter(|token| !token.is_empty());
        if token.is_some() && url.scheme() != "https" {
            return Err(Exception::BadRequest(
                "Access tokens require an HTTPS repository URL".into(),
            ));
        }
        let condition = Condition::new().eq("url", input.url.as_str());
        if self.database.repositories.exists(condition.clone()).await? {
            return Err(Exception::Conflict("modules.repository.already_exists"));
        }
        let encrypted_token = token
            .map(|token| {
                let encryption_key = self
                    .config
                    .encryption_key
                    .as_deref()
                    .ok_or_else(|| Exception::BadRequest("Private repositories require GIONG_ENCRYPTION_KEY to be configured on the server".into()))?;
                encrypt_token(token, encryption_key, input.url.as_bytes()).map_err(Exception::from)
            })
            .transpose()?;

        let result = self
            .database
            .repositories
            .create_one(
                Document::new()
                    .set("name", input.name.as_str())
                    .set("url", input.url.as_str())
                    .set("credentials", encrypted_token),
            )
            .await;
        match result {
            Ok(repository) => Ok(repository),
            Err(_) if self.database.repositories.exists(condition).await? => {
                Err(Exception::Conflict("modules.repository.already_exists"))
            }
            Err(error) => Err(error.into()),
        }
    }
}
