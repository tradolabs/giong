use crate::application::application::Application;
use crate::application::helpers::filesystem::root_dir;
use crate::application::ports::queue::queue_manager::QueueManager;
use crate::application::use_cases::{
    create_repository::CreateRepositoryUseCase, create_stack::CreateStackUseCase,
    get_job_logs::GetJobLogsUseCase, poll_repositories::PollRepositoriesUseCase,
    reset_admin_password::ResetAdminPasswordUseCase, run_job::RunJobUseCase,
    sign_in::SignInUseCase,
};
use crate::config::Config;
use crate::infrastructure::adapters::queue_manager::QueueManagerImpl;
use crate::presentation::queues::run_job;
use giong_database::Database;
use giong_di::Container;
use giong_publisher::Publisher;
use giong_storage::storage::Storage;
use std::{fs, sync::Arc};

pub async fn ensure_initialized(mut config: Config) -> anyhow::Result<Arc<Application>> {
    let data_dir = root_dir();
    fs::create_dir_all(data_dir.join("logs"))?;
    fs::create_dir_all(data_dir.join("runs"))?;

    let database_url = config.database_url.clone().unwrap_or_else(|| {
        format!("sqlite://{}?mode=rwc", data_dir.join("giong.sqlite3").display())
    });
    let storage_url = config
        .storage_url
        .get_or_insert_with(|| format!("local://{}", data_dir.join("storage").display()));

    let database = Database::new(&database_url).await?;
    let storage = Storage::new(storage_url)?;
    let publisher = Arc::new(Publisher::new().await?);

    let mut container = Container::new();
    container
        .register(config)
        .register(database)
        .register(storage)
        .register(Arc::new(QueueManagerImpl::new(publisher.clone())) as Arc<dyn QueueManager>)
        .register_type::<CreateRepositoryUseCase>()
        .register_type::<CreateStackUseCase>()
        .register_type::<GetJobLogsUseCase>()
        .register_type::<PollRepositoriesUseCase>()
        .register_type::<ResetAdminPasswordUseCase>()
        .register_type::<RunJobUseCase>()
        .register_type::<SignInUseCase>();

    let container = Arc::new(container);
    let application = Arc::new(Application::new(Arc::clone(&container))?);
    publisher.with_state(container).handler(run_job::handle).register().await?;

    Ok(application)
}
