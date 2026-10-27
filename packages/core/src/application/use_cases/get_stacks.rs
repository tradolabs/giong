use super::use_case::UseCase;
use crate::application::exceptions::base::Exception;
use giong_database::{Condition, Database, StackEntity};
use giong_di::Injectable;
use std::sync::Arc;

#[derive(Injectable)]
pub struct GetStacksUseCase {
    database: Arc<Database>,
}

impl UseCase for GetStacksUseCase {
    type Input = ();
    type Output = Vec<StackEntity>;

    async fn execute(&self, (): Self::Input) -> Result<Self::Output, Exception> {
        let mut stacks = self.database.stacks.find(Condition::new()).await?;
        stacks.reverse();
        Ok(stacks)
    }
}
