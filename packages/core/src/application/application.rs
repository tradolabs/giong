use crate::application::exceptions::base::Exception;
use crate::application::use_cases::{
    create_job::CreateJobUseCase,
    create_repository::{CreateRepositoryInput, CreateRepositoryUseCase},
    create_stack::{CreateStackInput, CreateStackUseCase},
    get_job_logs::{GetJobLogsInput, GetJobLogsUseCase},
    get_jobs::{GetJobsInput, GetJobsUseCase},
    get_stacks::GetStacksUseCase,
    poll_repositories::PollRepositoriesUseCase,
    reset_admin_password::ResetAdminPasswordUseCase,
    run_job::{RunJobInput, RunJobUseCase},
    sign_in::{SignInInput, SignInOutput, SignInUseCase},
    use_case::UseCase,
};
use crate::config::Config;
use giong_database::{JobEntity, StackEntity};
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

    pub async fn add_repo(
        &self,
        name: &str,
        url: &str,
        token: Option<&str>,
    ) -> Result<(), Exception> {
        self.container
            .get::<CreateRepositoryUseCase>()
            .map_err(Exception::unexpected)?
            .execute(CreateRepositoryInput {
                name: name.to_owned(),
                url: url.to_owned(),
                token: token.map(str::to_owned),
            })
            .await
    }

    pub async fn add_stack(
        &self,
        name: &str,
        repo_url: &str,
        branch: &str,
        folder_path: &str,
    ) -> Result<(), Exception> {
        self.container
            .get::<CreateStackUseCase>()
            .map_err(Exception::unexpected)?
            .execute(CreateStackInput {
                name: name.to_owned(),
                repo_url: repo_url.to_owned(),
                branch: branch.to_owned(),
                folder_path: folder_path.to_owned(),
            })
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

    pub async fn get_stacks(&self) -> Result<Vec<StackEntity>, Exception> {
        self.container.get::<GetStacksUseCase>().map_err(Exception::unexpected)?.execute(()).await
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
