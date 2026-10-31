use super::base::Repository;
use crate::entities::secret_entity::SecretEntity;
pub trait SecretRepository: Repository<SecretEntity> {}
