use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::stack_service::StackService;
use giong_database::entities::stack_entity::StackEntity;
use giong_di::Injectable;
use std::sync::Arc;

pub struct CreateStackInput {
    pub name: String,
    pub configurations: serde_json::Value,
}
#[derive(Injectable)]
pub struct CreateStackUseCase {
    stack_service: Arc<StackService>,
}
impl UseCase for CreateStackUseCase {
    type Input = CreateStackInput;
    type Output = StackEntity;
    async fn execute(&self, input: Self::Input) -> Result<StackEntity, Exception> {
        self.stack_service.save(None, input.name, input.configurations).await
    }
}
