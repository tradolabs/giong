use super::base::SeaOrmRepository;
use crate::{infrastructure::models::flow, repositories::flow::FlowRepository};

pub(crate) type SeaOrmFlowRepository = SeaOrmRepository<flow::Entity>;

impl FlowRepository for SeaOrmFlowRepository {}
