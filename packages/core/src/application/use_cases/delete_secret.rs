use crate::application::exceptions::base::Exception;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteSecretInput {
    pub vault_id: Uuid,
    pub secret_id: Uuid,
}

#[derive(Injectable)]
pub struct DeleteSecretUseCase {
    database: Arc<Database>,
}

impl UseCase for DeleteSecretUseCase {
    type Input = DeleteSecretInput;
    type Output = ();
    async fn execute(&self, input: DeleteSecretInput) -> Result<(), Exception> {
        let DeleteSecretInput { vault_id, secret_id } = input;
        self.database
            .secrets
            .delete_one(Condition::new().eq("vault_id", vault_id).eq("id", secret_id))
            .await?;
        Ok(())
    }
}
