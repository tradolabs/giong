use super::base_repository;
use crate::{infrastructure::models::stack_model, repositories::stack_repository::StackRepository};

pub(crate) type SeaOrmStackRepository = base_repository::SeaOrmRepository<stack_model::Entity>;

impl StackRepository for SeaOrmStackRepository {}
