use crate::application::exceptions::base::Exception;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct SecretInfo {
    pub id: Uuid,
    pub key: String,
}

#[derive(Injectable)]
pub struct GetSecretsUseCase {
    database: Arc<Database>,
}
impl UseCase for GetSecretsUseCase {
    type Input = Uuid;
    type Output = Vec<SecretInfo>;
    async fn execute(&self, id: Uuid) -> Result<Self::Output, Exception> {
        Ok(self
            .database
            .secrets
            .find(Condition::new().eq("vault_id", id))
            .await?
            .into_iter()
            .map(|secret| SecretInfo { id: secret.id, key: secret.key })
            .collect())
    }
}
