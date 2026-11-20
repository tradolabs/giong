use crate::application::exceptions::base::Exception;
use crate::application::use_cases::use_case::UseCase;
use giong_database::Database;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_node::runtime::validate_plugin_kind;
use std::sync::Arc;

#[derive(Injectable)]
pub struct RemovePluginUseCase {
    database: Arc<Database>,
}

impl UseCase for RemovePluginUseCase {
    type Input = String;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Exception> {
        validate_plugin_kind(&input).map_err(|error| Exception::BadRequest(error.to_string()))?;
        self.database.plugins.delete_one(Condition::new().eq("kind", input)).await?;
        Ok(())
    }
}
