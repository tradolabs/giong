use super::base::Repository;
use crate::entities::secret_set_entity::SecretSetEntity;
pub trait SecretSetRepository: Repository<SecretSetEntity> {}
