use super::base_repository::BaseRepository;
use crate::entities::repository_entity::RepositoryEntity;

pub trait RepositoryRepository: BaseRepository<RepositoryEntity> {}
