use super::base_repository::BaseRepository;
use crate::entities::stack_entity::StackEntity;

pub trait StackRepository: BaseRepository<StackEntity> {}
