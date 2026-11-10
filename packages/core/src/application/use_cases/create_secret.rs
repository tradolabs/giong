use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::secret_service::{SecretScope, SecretService};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateSecretInput {
    pub stack_id: Uuid,
    pub key: String,
    pub value: String,
}
#[derive(Injectable)]
pub struct CreateSecretUseCase {
    secret_service: Arc<SecretService>,
}
impl UseCase for CreateSecretUseCase {
    type Input = CreateSecretInput;
    type Output = ();
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        self.secret_service
            .create_secret(SecretScope::Stack(input.stack_id), input.key, input.value)
            .await
    }
}
