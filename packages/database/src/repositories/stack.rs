use super::base::Repository;
use crate::entities::stack_entity::StackEntity;

pub trait StackRepository: Repository<StackEntity> {}
