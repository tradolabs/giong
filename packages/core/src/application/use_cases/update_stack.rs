use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use crate::application::services::stack_service::StackService;
use giong_database::entities::stack_entity::StackEntity;
use giong_di::Injectable;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateStackInput {
    pub stack_id: Uuid,
    pub name: String,
    pub configurations: serde_json::Value,
}
#[derive(Injectable)]
pub struct UpdateStackUseCase {
    stack_service: Arc<StackService>,
}
impl UseCase for UpdateStackUseCase {
    type Input = UpdateStackInput;
    type Output = StackEntity;
    async fn execute(&self, input: Self::Input) -> Result<StackEntity, Exception> {
        self.stack_service.save(Some(input.stack_id), input.name, input.configurations).await
    }
}
