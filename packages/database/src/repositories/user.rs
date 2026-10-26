use super::base::Repository;
use crate::entities::user_entity::UserEntity;

pub trait UserRepository: Repository<UserEntity> {}
