use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::services::flow::FlowService;
use crate::application::services::node::NodeService;
use crate::application::services::secret::SecretService;
use crate::application::types::node_execution::{ExecutionStatus, NodeExecution};
use crate::application::use_cases::use_case::UseCase;
use crate::config::Config;
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::entities::node_entity::NodeEntity;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_kv::kv::Kv;
use giong_node::runtime::Plugin;
use giong_node::tools::Tools;
use giong_node::tools::events::Events;
use giong_node::tools::execution::Execution;
use giong_node::tools::secrets::Secrets;
use giong_node::tools::storage::Storage;
use giong_node::types::node::OUTPUT_KEY;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct ExecuteNodeInput {
    pub node_id: Uuid,
}

#[derive(Injectable)]
pub struct ExecuteNodeUseCase {
    kv: Arc<Kv>,
    database: Arc<Database>,
    secret_service: Arc<SecretService>,
    flow_service: Arc<FlowService>,
    node_service: Arc<NodeService>,
}

impl ExecuteNodeUseCase {
    fn prepare_log_file(log_path: &Path) -> Result<(), Exception> {
        let directory = log_path.parent().expect("Node log has a parent directory");
        std::fs::create_dir_all(directory)?;
        std::fs::OpenOptions::new().create(true).append(true).open(log_path)?;

        Ok(())
    }

    fn build_secrets_tool(&self, flow: &FlowEntity) -> Result<Secrets, Exception> {
        let inherited_vault_ids: Vec<Uuid> =
            serde_json::from_value(flow.inherited_vault_ids.clone())?;
        let vault_id = flow.vault_id;
        let secret_service = Arc::clone(&self.secret_service);
        let runtime = tokio::runtime::Handle::current();

        Ok(Secrets {
            get: Arc::new(move |key| {
                runtime
                    .block_on(secret_service.get_secret_value(
                        Some(vault_id),
                        &inherited_vault_ids,
                        key,
                    ))
                    .map_err(anyhow::Error::from)
            }),
        })
    }

    fn build_events_tool(&self, flow_id: Uuid, source_node_id: Uuid) -> Events {
        let flow_service = Arc::clone(&self.flow_service);
        let node_service = Arc::clone(&self.node_service);
        let runtime = tokio::runtime::Handle::current();

        Events {
            emit: Arc::new(move |_event| {
                runtime.block_on(async {
                    let targets = flow_service.get_event_targets(flow_id, source_node_id).await?;
                    for target_id in targets {
                        node_service.enqueue_execution(flow_id, target_id).await?;
                    }
                    Ok::<_, Exception>(())
                })?;
                Ok(())
            }),
        }
    }

    fn collect_flow_outputs(
        kv: &Kv,
        nodes: &[NodeEntity],
    ) -> anyhow::Result<Arc<Mutex<BTreeMap<String, serde_json::Value>>>> {
        let mut outputs = BTreeMap::new();
        for source in nodes {
            let storage = Storage::new(kv.namespace(&format!("nodes/{}", source.id)));
            if let Some(bytes) = storage.get(OUTPUT_KEY)? {
                outputs.insert(source.name.clone(), serde_json::from_slice(&bytes)?);
            }
        }
        Ok(Arc::new(Mutex::new(outputs)))
    }

    async fn run_execution(
        &self,
        execution: &NodeExecution,
        log_path: &Path,
    ) -> Result<(), Exception> {
        Self::prepare_log_file(log_path)?;

        let flow = self
            .database
            .flows
            .find_one(Condition::new().eq("id", execution.flow_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Flow))?;
        let nodes = self.database.nodes.find(Condition::new().eq("flow_id", flow.id)).await?;
        let node = nodes
            .iter()
            .find(|node| node.id == execution.node_id)
            .cloned()
            .ok_or(ResourceException::NotFound(Resource::Node))?;

        let secrets = self.build_secrets_tool(&flow)?;
        let events = self.build_events_tool(flow.id, node.id);

        let workspace = Config::data_dir().join("workspaces").join(flow.id.to_string());
        std::fs::create_dir_all(&workspace)?;
        let workspace = std::fs::canonicalize(workspace)?;

        let kv = Arc::clone(&self.kv);
        let log_path = log_path.to_path_buf();
        let execution_id = execution.id;
        let flow_id = flow.id;

        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let outputs = Self::collect_flow_outputs(&kv, &nodes)?;
            let storage = Storage::new(kv.namespace(&format!("nodes/{}", node.id)));
            let mut logger =
                std::fs::OpenOptions::new().create(true).append(true).open(log_path)?;
            writeln!(logger, "Executing node {} ({})", node.name, node.kind)?;

            let mut plugin = Plugin::load(
                &node.kind,
                Tools {
                    events: Some(events),
                    secrets: Some(secrets),
                    scope: Some(flow_id.to_string()),
                    workspace: Some(workspace.join(format!("node-{}", node.id))),
                    storage: Some(storage.clone()),
                    outputs: Some(outputs),
                    execution: Some(Execution { id: execution_id.to_string(), workspace, logger }),
                },
            )?;
            storage.delete(OUTPUT_KEY)?;
            plugin.execute(&serde_json::to_string(&node.properties)?)?;
            Ok(())
        })
        .await
        .map_err(Exception::unexpected)?
        .map_err(Exception::unexpected)
    }

    fn finish_execution(
        &self,
        mut execution: NodeExecution,
        log_path: &Path,
        result: Result<(), Exception>,
    ) -> Result<(), Exception> {
        if let Err(error) = &result
            && let Ok(mut file) = std::fs::File::options().create(true).append(true).open(log_path)
        {
            let _ = writeln!(file, "{error}");
        }

        execution.status =
            if result.is_ok() { ExecutionStatus::Succeeded } else { ExecutionStatus::Failed };
        execution.exit_code = Some(if result.is_ok() { 0 } else { 1 });
        execution.finished_at = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        if !self.node_service.update_execution(execution)? {
            match std::fs::remove_file(log_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}

impl UseCase for ExecuteNodeUseCase {
    type Input = ExecuteNodeInput;
    type Output = ();

    async fn execute(&self, input: Self::Input) -> Result<(), Exception> {
        let Some(execution) = self.node_service.claim_pending_execution(input.node_id)? else {
            return Ok(());
        };
        let log_path = Config::data_dir()
            .join("logs")
            .join("nodes")
            .join(execution.node_id.to_string())
            .join(format!("{}.log", execution.id));
        let result = self.run_execution(&execution, &log_path).await;
        self.finish_execution(execution, &log_path, result)
    }
}
