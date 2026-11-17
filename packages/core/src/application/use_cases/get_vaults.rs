use crate::application::exceptions::base::Exception;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::entities::vault_entity::{VaultEntity, VaultScope};
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;

#[derive(Injectable)]
pub struct GetVaultsUseCase {
    database: Arc<Database>,
}

impl UseCase for GetVaultsUseCase {
    type Input = ();
    type Output = Vec<VaultEntity>;
    async fn execute(&self, _: ()) -> Result<Self::Output, Exception> {
        let vaults =
            self.database.vaults.find(Condition::new().eq("scope", VaultScope::Global)).await?;
        Ok(vaults)
    }
}
