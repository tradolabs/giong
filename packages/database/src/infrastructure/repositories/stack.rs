use super::base::SeaOrmRepository;
use crate::{infrastructure::models::stack, repositories::stack::StackRepository};

pub(crate) type SeaOrmStackRepository = SeaOrmRepository<stack::Entity>;

impl StackRepository for SeaOrmStackRepository {}
