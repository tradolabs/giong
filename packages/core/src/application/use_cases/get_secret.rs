use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::secret_entity::SecretEntity;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetSecretInput {
    pub vault_id: Uuid,
    pub secret_id: Uuid,
}

#[derive(Injectable)]
pub struct GetSecretUseCase {
    database: Arc<Database>,
}

impl UseCase for GetSecretUseCase {
    type Input = GetSecretInput;
    type Output = SecretEntity;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        self.database
            .secrets
            .find_one(Condition::new().eq("vault_id", input.vault_id).eq("id", input.secret_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Secret))
            .map_err(Exception::from)
    }
}
