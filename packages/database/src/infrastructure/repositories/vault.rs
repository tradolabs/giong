use crate::infrastructure::models::vault;
use crate::infrastructure::repositories::base::SeaOrmRepository;
use crate::repositories::vault::VaultRepository;
pub(crate) type SeaOrmVaultRepository = SeaOrmRepository<vault::Entity>;
impl VaultRepository for SeaOrmVaultRepository {}
