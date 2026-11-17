use crate::entities::vault_entity::VaultEntity;
use crate::repositories::base::Repository;
pub trait VaultRepository: Repository<VaultEntity> {}
