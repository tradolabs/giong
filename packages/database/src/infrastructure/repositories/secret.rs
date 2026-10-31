use super::base::SeaOrmRepository;
use crate::infrastructure::models::secret;
use crate::repositories::secret::SecretRepository;

pub(crate) type SeaOrmSecretRepository = SeaOrmRepository<secret::Entity>;

impl SecretRepository for SeaOrmSecretRepository {}
