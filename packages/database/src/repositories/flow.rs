use super::base::Repository;
use crate::entities::flow_entity::FlowEntity;

pub trait FlowRepository: Repository<FlowEntity> {}
