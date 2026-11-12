use crate::application::application::Application;
use crate::application::helpers::filesystem::root_dir;
use crate::application::ports::queue::queue_manager::QueueManager;
use crate::application::use_cases::delete_flow::DeleteFlowUseCase;
use crate::application::use_cases::node_triggers::{
    DispatchNodeTriggersUseCase, ResumePendingRunsUseCase, TriggerNodeUseCase,
};
use crate::application::use_cases::run_flow::RunFlowUseCase;
use crate::application::use_cases::secrets::{
    DeleteSecretUseCase, GetSecretValueUseCase, GetSecretsUseCase, PutSecretUseCase,
};
use crate::application::use_cases::update_flow_vaults::UpdateFlowVaultsUseCase;
use crate::application::use_cases::vaults::{
    CreateVaultUseCase, DeleteVaultUseCase, GetVaultsUseCase, UpdateVaultUseCase,
};
use crate::application::use_cases::{
    create_node_run::CreateNodeRunUseCase, get_flows::GetFlowsUseCase,
    get_node_run_logs::GetNodeRunLogsUseCase, get_node_runs::GetNodeRunsUseCase,
    reset_admin_password::ResetAdminPasswordUseCase, run_node::RunNodeUseCase,
    sign_in::SignInUseCase, upsert_flow::UpsertFlowUseCase,
};
use crate::config::Config;
use crate::infrastructure::adapters::node_runs::NodeRuns;
use crate::infrastructure::adapters::node_storage::NodeStorage;
use crate::infrastructure::adapters::queue_manager::QueueManagerImpl;
use crate::presentation::queues::run_node;
use giong_database::Database;
use giong_di::Container;
use giong_publisher::Publisher;
use std::{fs, sync::Arc};

pub async fn ensure_initialized(config: Config) -> anyhow::Result<Arc<Application>> {
    let data_dir = root_dir();
    fs::create_dir_all(data_dir.join("logs"))?;
    fs::create_dir_all(data_dir.join("runs"))?;

    let database_url = config.database_url.clone().unwrap_or_else(|| {
        format!("sqlite://{}?mode=rwc", data_dir.join("giong.sqlite3").display())
    });
    let database = Database::new(&database_url).await?;
    let node_storage = NodeStorage::new(data_dir.join("nodes.redb"));
    let publisher = Arc::new(Publisher::new().await?);

    let mut container = Container::new();
    container
        .register(config)
        .register(database)
        .register(node_storage)
        .register(Arc::new(QueueManagerImpl::new(publisher.clone())) as Arc<dyn QueueManager>)
        .register_type::<NodeRuns>()
        .register_type::<RunFlowUseCase>()
        .register_type::<DispatchNodeTriggersUseCase>()
        .register_type::<ResumePendingRunsUseCase>()
        .register_type::<TriggerNodeUseCase>()
        .register_type::<CreateVaultUseCase>()
        .register_type::<GetVaultsUseCase>()
        .register_type::<UpdateVaultUseCase>()
        .register_type::<DeleteVaultUseCase>()
        .register_type::<GetSecretsUseCase>()
        .register_type::<PutSecretUseCase>()
        .register_type::<DeleteSecretUseCase>()
        .register_type::<GetSecretValueUseCase>()
        .register_type::<UpsertFlowUseCase>()
        .register_type::<UpdateFlowVaultsUseCase>()
        .register_type::<DeleteFlowUseCase>()
        .register_type::<GetNodeRunLogsUseCase>()
        .register_type::<GetNodeRunsUseCase>()
        .register_type::<GetFlowsUseCase>()
        .register_type::<ResetAdminPasswordUseCase>()
        .register_type::<RunNodeUseCase>()
        .register_type::<CreateNodeRunUseCase>()
        .register_type::<SignInUseCase>();

    let container = Arc::new(container);
    let application = Arc::new(Application::new(Arc::clone(&container))?);
    publisher.with_state(container).handler(run_node::handle).register().await?;

    Ok(application)
}
