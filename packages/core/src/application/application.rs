use crate::application::exceptions::base::Exception;
use crate::application::use_cases::{
    get_flows::GetFlowsUseCase,
    reset_admin_password::ResetAdminPasswordUseCase,
    sign_in::{SignInInput, SignInOutput, SignInUseCase},
    use_case::UseCase,
};
use crate::config::Config;
use giong_database::entities::flow_entity::FlowEntity;
use giong_di::Container;
use std::sync::Arc;

pub struct Application {
    config: Arc<Config>,
    container: Arc<Container>,
}

impl Application {
    pub fn new(container: Arc<Container>) -> Result<Self, Exception> {
        let config = container.get::<Config>().map_err(Exception::unexpected)?;
        Ok(Self { config, container })
    }

    pub async fn execute<U: UseCase + Send + Sync + 'static>(
        &self,
        input: U::Input,
    ) -> Result<U::Output, Exception> {
        self.container.get::<U>().map_err(Exception::unexpected)?.execute(input).await
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn reset_admin_password(&self) -> Result<String, Exception> {
        self.container
            .get::<ResetAdminPasswordUseCase>()
            .map_err(Exception::unexpected)?
            .execute(())
            .await
    }

    pub async fn sign_in(&self, username: &str, password: &str) -> Result<SignInOutput, Exception> {
        self.container
            .get::<SignInUseCase>()
            .map_err(Exception::unexpected)?
            .execute(SignInInput { username: username.to_owned(), password: password.to_owned() })
            .await
    }

    pub async fn get_flows(&self) -> Result<Vec<FlowEntity>, Exception> {
        self.container.get::<GetFlowsUseCase>().map_err(Exception::unexpected)?.execute(()).await
    }
}
