use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::secret_service::{SecretScope, SecretService};
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteSecretInput {
    pub stack_id: Uuid,
    pub key: String,
}
#[derive(Injectable)]
pub struct DeleteSecretUseCase {
    secret_service: Arc<SecretService>,
}
impl UseCase for DeleteSecretUseCase {
    type Input = DeleteSecretInput;
    type Output = ();
    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        self.secret_service.delete_secret(SecretScope::Stack(input.stack_id), input.key).await
    }
}
