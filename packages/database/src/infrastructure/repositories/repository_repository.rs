use super::base_repository;
use crate::{
    infrastructure::models::repository_model,
    repositories::repository_repository::RepositoryRepository,
};

pub(crate) type SeaOrmRepository = base_repository::SeaOrmRepository<repository_model::Entity>;

impl RepositoryRepository for SeaOrmRepository {}
