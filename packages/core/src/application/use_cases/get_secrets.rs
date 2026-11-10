use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::secret_service::{SecretInfo, SecretScope, SecretService};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetSecretsInput {
    pub stack_id: Uuid,
}
#[derive(Injectable)]
pub struct GetSecretsUseCase {
    secret_service: Arc<SecretService>,
}
impl UseCase for GetSecretsUseCase {
    type Input = GetSecretsInput;
    type Output = Vec<SecretInfo>;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        self.secret_service.list_secrets(SecretScope::Stack(input.stack_id)).await
    }
}
