use super::base::SeaOrmRepository;
use crate::{infrastructure::models::repository, repositories::repository::RepositoryRepository};

pub(crate) type SeaOrmRepositoryRepository = SeaOrmRepository<repository::Entity>;

impl RepositoryRepository for SeaOrmRepositoryRepository {}
