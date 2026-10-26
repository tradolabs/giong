use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::helpers::crypto::encrypt_token;
use crate::config::Config;
use anyhow::Context;
use giong_database::Database;
use giong_database::Document;
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
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        let encrypted_token = input
            .token
            .as_deref()
            .map(|token| {
                let encryption_key = self
                    .config
                    .encryption_key
                    .as_deref()
                    .context("Encryption key is not configured")?;
                encrypt_token(token, encryption_key, input.url.as_bytes())
            })
            .transpose()?;

        self.database
            .repositories
            .create_one(
                Document::new()
                    .set("name", input.name.as_str())
                    .set("url", input.url.as_str())
                    .set("credentials", encrypted_token),
            )
            .await?;

        Ok(())
    }
}
