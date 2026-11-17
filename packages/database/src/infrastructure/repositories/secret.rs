use crate::infrastructure::models::secret;
use crate::infrastructure::repositories::base::SeaOrmRepository;
use crate::repositories::secret::SecretRepository;
pub(crate) type SeaOrmSecretRepository = SeaOrmRepository<secret::Entity>;
impl SecretRepository for SeaOrmSecretRepository {}
