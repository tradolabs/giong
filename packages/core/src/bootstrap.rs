use crate::application::application::Application;
use crate::application::helpers::flow_graph::FlowGraphCache;
use crate::application::ports::queue::queue_manager::QueueManager;
use crate::application::services::flow_service::FlowService;
use crate::application::services::node_service::NodeService;
use crate::application::services::secret_service::SecretService;
use crate::application::use_cases::create_node::CreateNodeUseCase;
use crate::application::use_cases::create_secret::CreateSecretUseCase;
use crate::application::use_cases::create_vault::CreateVaultUseCase;
use crate::application::use_cases::delete_flow::DeleteFlowUseCase;
use crate::application::use_cases::delete_node::DeleteNodeUseCase;
use crate::application::use_cases::delete_secret::DeleteSecretUseCase;
use crate::application::use_cases::delete_vault::DeleteVaultUseCase;
use crate::application::use_cases::get_node_details::GetNodeDetailsUseCase;
use crate::application::use_cases::get_secret::GetSecretUseCase;
use crate::application::use_cases::get_secrets::GetSecretsUseCase;
use crate::application::use_cases::get_vaults::GetVaultsUseCase;
use crate::application::use_cases::resume_pending_executions::ResumePendingExecutionsUseCase;
use crate::application::use_cases::update_node::UpdateNodeUseCase;
use crate::application::use_cases::update_secret::UpdateSecretUseCase;
use crate::application::use_cases::update_vault::UpdateVaultUseCase;
use crate::application::use_cases::{
    create_flow::CreateFlowUseCase, execute_node::ExecuteNodeUseCase, get_flows::GetFlowsUseCase,
    get_node_logs::GetNodeLogsUseCase, reset_admin_password::ResetAdminPasswordUseCase,
    sign_in::SignInUseCase, update_flow::UpdateFlowUseCase,
};
use crate::config::Config;
use crate::infrastructure::adapters::queue_manager::QueueManagerImpl;
use crate::presentation::queues::execute_node;
use giong_database::Database;
use giong_di::Container;
use giong_kv::kv::Kv;
use giong_publisher::Publisher;
use giong_scheduler::IntervalScheduler;
use std::{fs, sync::Arc};

fn ensure_data_dir() -> anyhow::Result<std::path::PathBuf> {
    let data_dir = Config::data_dir();
    fs::create_dir_all(data_dir.join("logs"))?;
    fs::create_dir_all(data_dir.join("executions"))?;
    Ok(data_dir)
}

pub async fn ensure_initialized(config: Config) -> anyhow::Result<Arc<Application>> {
    let data_dir = ensure_data_dir()?;
    let database_url = config.database_url.clone().unwrap_or_else(|| {
        format!("sqlite://{}?mode=rwc", data_dir.join("giong.sqlite3").display())
    });
    let database = Database::new(&database_url).await?;
    let kv = Kv::new(data_dir.join("nodes.redb"))?;
    let publisher = Arc::new(Publisher::new().await?);
    let scheduler = IntervalScheduler::new().await?;

    let mut container = Container::new();
    container
        .register(config)
        .register(database)
        .register(kv)
        .register(scheduler)
        .register(FlowGraphCache::default())
        .register(Arc::new(QueueManagerImpl::new(publisher.clone())) as Arc<dyn QueueManager>)
        .register_type::<NodeService>()
        .register_type::<FlowService>()
        .register_type::<SecretService>()
        .register_type::<ResumePendingExecutionsUseCase>()
        .register_type::<CreateVaultUseCase>()
        .register_type::<GetVaultsUseCase>()
        .register_type::<GetSecretUseCase>()
        .register_type::<UpdateVaultUseCase>()
        .register_type::<DeleteVaultUseCase>()
        .register_type::<GetSecretsUseCase>()
        .register_type::<CreateSecretUseCase>()
        .register_type::<UpdateSecretUseCase>()
        .register_type::<DeleteSecretUseCase>()
        .register_type::<CreateFlowUseCase>()
        .register_type::<UpdateFlowUseCase>()
        .register_type::<CreateNodeUseCase>()
        .register_type::<UpdateNodeUseCase>()
        .register_type::<DeleteFlowUseCase>()
        .register_type::<DeleteNodeUseCase>()
        .register_type::<GetNodeLogsUseCase>()
        .register_type::<GetNodeDetailsUseCase>()
        .register_type::<GetFlowsUseCase>()
        .register_type::<ResetAdminPasswordUseCase>()
        .register_type::<ExecuteNodeUseCase>()
        .register_type::<SignInUseCase>();

    let container = Arc::new(container);
    container.get::<NodeService>()?.sync_all_interval_triggers().await?;
    let application = Arc::new(Application::new(Arc::clone(&container))?);
    publisher.with_state(container).handler(execute_node::handle).register().await?;

    Ok(application)
}
