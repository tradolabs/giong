use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::flow::FlowService;
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use std::sync::Arc;

#[derive(Injectable)]
pub struct GetFlowsUseCase {
    database: Arc<Database>,
    flow_service: Arc<FlowService>,
}

impl UseCase for GetFlowsUseCase {
    type Input = ();
    type Output = Vec<FlowEntity>;

    async fn execute(&self, (): Self::Input) -> Result<Self::Output, Exception> {
        let mut flows = self.database.flows.find(Condition::new()).await?;
        self.flow_service.populate_nodes(&mut flows).await?;
        flows.reverse();
        Ok(flows)
    }
}
