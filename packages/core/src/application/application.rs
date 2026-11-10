use crate::application::exceptions::base::Exception;
use crate::application::services::secret_service::{
    SecretInfo, SecretScope, SecretService, SecretSetInfo,
};
use crate::application::use_cases::{
    create_job::CreateJobUseCase,
    create_repository::{CreateRepositoryInput, CreateRepositoryUseCase},
    create_secret::{CreateSecretInput, CreateSecretUseCase},
    create_stack::{CreateStackInput, CreateStackUseCase},
    delete_secret::{DeleteSecretInput, DeleteSecretUseCase},
    get_job_logs::{GetJobLogsInput, GetJobLogsUseCase},
    get_jobs::{GetJobsInput, GetJobsUseCase},
    get_repositories::GetRepositoriesUseCase,
    get_secrets::{GetSecretsInput, GetSecretsUseCase},
    get_stacks::GetStacksUseCase,
    poll_repositories::PollRepositoriesUseCase,
    reset_admin_password::ResetAdminPasswordUseCase,
    run_job::{RunJobInput, RunJobUseCase},
    sign_in::{SignInInput, SignInOutput, SignInUseCase},
    update_secret::{UpdateSecretInput, UpdateSecretUseCase},
    update_stack::{UpdateStackInput, UpdateStackUseCase},
    use_case::UseCase,
};
use crate::config::Config;
use giong_database::entities::repository_entity::RepositoryEntity;
use giong_database::entities::{job_entity::JobEntity, stack_entity::StackEntity};
use giong_di::Container;
use std::sync::Arc;
use uuid::Uuid;

pub struct Application {
    config: Arc<Config>,
    container: Arc<Container>,
}

impl Application {
    pub fn new(container: Arc<Container>) -> Result<Self, Exception> {
        let config = container.get::<Config>().map_err(Exception::unexpected)?;
        Ok(Self { config, container })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn run_job(&self, input: RunJobInput) -> Result<(), Exception> {
        self.container.get::<RunJobUseCase>().map_err(Exception::unexpected)?.execute(input).await
    }

    pub async fn poll_repositories(
        &self,
        repository_urls: Option<Vec<String>>,
    ) -> Result<(), Exception> {
        self.container
            .get::<PollRepositoriesUseCase>()
            .map_err(Exception::unexpected)?
            .execute(repository_urls)
            .await
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

    pub async fn get_repositories(&self) -> Result<Vec<RepositoryEntity>, Exception> {
        self.container
            .get::<GetRepositoriesUseCase>()
            .map_err(Exception::unexpected)?
            .execute(())
            .await
    }

    pub async fn create_repository(
        &self,
        input: CreateRepositoryInput,
    ) -> Result<RepositoryEntity, Exception> {
        self.container
            .get::<CreateRepositoryUseCase>()
            .map_err(Exception::unexpected)?
            .execute(input)
            .await
    }

    pub async fn create_stack(&self, input: CreateStackInput) -> Result<StackEntity, Exception> {
        self.container
            .get::<CreateStackUseCase>()
            .map_err(Exception::unexpected)?
            .execute(input)
            .await
    }

    pub async fn get_job_logs(
        &self,
        job_id: &Uuid,
        from_lines: usize,
    ) -> Result<Option<String>, Exception> {
        self.container
            .get::<GetJobLogsUseCase>()
            .map_err(Exception::unexpected)?
            .execute(GetJobLogsInput { job_id: *job_id, from_lines })
            .await
    }

    pub async fn update_stack(&self, input: UpdateStackInput) -> Result<StackEntity, Exception> {
        self.container
            .get::<UpdateStackUseCase>()
            .map_err(Exception::unexpected)?
            .execute(input)
            .await
    }

    pub async fn get_runners(&self) -> Result<Vec<giong_runner::types::Metadata>, Exception> {
        tokio::task::spawn_blocking(giong_runner::plugins::get_plugins)
            .await
            .map_err(Exception::unexpected)?
            .map_err(Exception::unexpected)
    }

    pub async fn get_stacks(&self) -> Result<Vec<StackEntity>, Exception> {
        self.container.get::<GetStacksUseCase>().map_err(Exception::unexpected)?.execute(()).await
    }

    pub async fn get_secrets(&self, stack_id: Uuid) -> Result<Vec<SecretInfo>, Exception> {
        self.container
            .get::<GetSecretsUseCase>()
            .map_err(Exception::unexpected)?
            .execute(GetSecretsInput { stack_id })
            .await
    }

    pub async fn create_secret(
        &self,
        stack_id: Uuid,
        key: String,
        value: String,
    ) -> Result<(), Exception> {
        self.container
            .get::<CreateSecretUseCase>()
            .map_err(Exception::unexpected)?
            .execute(CreateSecretInput { stack_id, key, value })
            .await
    }

    pub async fn update_secret(
        &self,
        stack_id: Uuid,
        key: String,
        value: String,
    ) -> Result<(), Exception> {
        self.container
            .get::<UpdateSecretUseCase>()
            .map_err(Exception::unexpected)?
            .execute(UpdateSecretInput { stack_id, key, value })
            .await
    }

    pub async fn delete_secret(&self, stack_id: Uuid, key: String) -> Result<(), Exception> {
        self.container
            .get::<DeleteSecretUseCase>()
            .map_err(Exception::unexpected)?
            .execute(DeleteSecretInput { stack_id, key })
            .await
    }

    pub async fn get_secret_sets(&self) -> Result<Vec<SecretSetInfo>, Exception> {
        self.container.get::<SecretService>().map_err(Exception::unexpected)?.list_sets().await
    }
    pub async fn create_secret_set(&self, name: String) -> Result<SecretSetInfo, Exception> {
        self.container.get::<SecretService>().map_err(Exception::unexpected)?.create_set(name).await
    }
    pub async fn delete_secret_set(&self, id: Uuid) -> Result<(), Exception> {
        self.container.get::<SecretService>().map_err(Exception::unexpected)?.delete_set(id).await
    }
    pub async fn get_set_secrets(&self, id: Uuid) -> Result<Vec<SecretInfo>, Exception> {
        self.container
            .get::<SecretService>()
            .map_err(Exception::unexpected)?
            .list_secrets(SecretScope::Set(id))
            .await
    }
    pub async fn create_set_secret(
        &self,
        id: Uuid,
        key: String,
        value: String,
    ) -> Result<(), Exception> {
        self.container
            .get::<SecretService>()
            .map_err(Exception::unexpected)?
            .create_secret(SecretScope::Set(id), key, value)
            .await
    }
    pub async fn update_set_secret(
        &self,
        id: Uuid,
        key: String,
        value: String,
    ) -> Result<(), Exception> {
        self.container
            .get::<SecretService>()
            .map_err(Exception::unexpected)?
            .update_secret(SecretScope::Set(id), key, value)
            .await
    }
    pub async fn delete_set_secret(&self, id: Uuid, key: String) -> Result<(), Exception> {
        self.container
            .get::<SecretService>()
            .map_err(Exception::unexpected)?
            .delete_secret(SecretScope::Set(id), key)
            .await
    }
    pub async fn inherit_secret_sets(
        &self,
        stack_id: Uuid,
        sets: Vec<Uuid>,
    ) -> Result<(), Exception> {
        self.container
            .get::<SecretService>()
            .map_err(Exception::unexpected)?
            .update_inherited_sets(stack_id, sets)
            .await
    }

    pub async fn get_jobs(&self, stack_id: Uuid) -> Result<Vec<JobEntity>, Exception> {
        self.container
            .get::<GetJobsUseCase>()
            .map_err(Exception::unexpected)?
            .execute(GetJobsInput { stack_id })
            .await
    }

    pub async fn create_job(&self, stack_id: Uuid) -> Result<JobEntity, Exception> {
        self.container
            .get::<CreateJobUseCase>()
            .map_err(Exception::unexpected)?
            .execute(stack_id)
            .await
    }
}
