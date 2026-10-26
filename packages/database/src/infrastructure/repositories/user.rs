use super::base::SeaOrmRepository;
use crate::{infrastructure::models::user, repositories::user::UserRepository};

pub(crate) type SeaOrmUserRepository = SeaOrmRepository<user::Entity>;

impl UserRepository for SeaOrmUserRepository {}
