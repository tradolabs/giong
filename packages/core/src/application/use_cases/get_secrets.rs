use crate::application::exceptions::base::Exception;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::secret_entity::SecretEntity;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct GetSecretsUseCase {
    database: Arc<Database>,
}
impl UseCase for GetSecretsUseCase {
    type Input = Uuid;
    type Output = Vec<SecretEntity>;
    async fn execute(&self, id: Uuid) -> Result<Self::Output, Exception> {
        let secrets = self.database.secrets.find(Condition::new().eq("vault_id", id)).await?;
        Ok(secrets)
    }
}
