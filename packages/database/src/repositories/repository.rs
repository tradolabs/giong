use super::base::Repository;
use crate::entities::repository_entity::RepositoryEntity;

pub trait RepositoryRepository: Repository<RepositoryEntity> {}
