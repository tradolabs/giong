use super::base::SeaOrmRepository;
use crate::infrastructure::models::secret_set;
use crate::repositories::secret_set::SecretSetRepository;
pub(crate) type SeaOrmSecretSetRepository = SeaOrmRepository<secret_set::Entity>;
impl SecretSetRepository for SeaOrmSecretSetRepository {}
