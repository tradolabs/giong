use crate::entities::node_entity::NodeEntity;
use crate::repositories::base::Repository;
pub trait NodeRepository: Repository<NodeEntity> {}
