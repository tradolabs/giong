use crate::entities::secret_entity::SecretEntity;
use crate::repositories::base::Repository;
pub trait SecretRepository: Repository<SecretEntity> {}
