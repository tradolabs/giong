use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::ports::queue::{events::ExecuteNodeEvent, queue_manager::QueueManager};
use crate::application::services::flow::FlowService;
use crate::application::types::node_execution::{ExecutionStatus, NodeExecution};
use crate::config::Config;
use giong_database::Database;
use giong_database::entities::node_entity::NodeEntity;
use giong_database::repositories::types::condition::Condition;
use giong_di::Injectable;
use giong_kv::kv::Kv;
use giong_node::host::{Flow, FlowSyncResult, NodeOutput, NodeScope, Trigger};
use giong_node::runtime::validate_plugin_kind;
use giong_node::storage::Storage;
use giong_node::types::node::OUTPUT_KEY;
use giong_scheduler::IntervalScheduler;
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

static EXECUTIONS: Mutex<()> = Mutex::new(());
static FLOW_IMPORTS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Clone, Injectable)]
pub struct NodeService {
    database: Arc<Database>,
    flow_service: Arc<FlowService>,
    kv: Arc<Kv>,
    queue: Arc<dyn QueueManager>,
    scheduler: Arc<IntervalScheduler>,
}

impl NodeService {
    pub(crate) fn node_logs_dir(node_id: Uuid) -> PathBuf {
        Config::data_dir().join("logs").join("nodes").join(node_id.to_string())
    }

    pub(crate) fn get_log_path(node_id: Uuid, execution_id: Uuid) -> PathBuf {
        Self::node_logs_dir(node_id).join(format!("{execution_id}.log"))
    }

    pub(crate) fn open_log(node_id: Uuid, execution_id: Uuid) -> std::io::Result<File> {
        fs::create_dir_all(Self::node_logs_dir(node_id))?;
        File::options().create(true).append(true).open(Self::get_log_path(node_id, execution_id))
    }

    pub(crate) fn storage(&self, node_id: Uuid) -> Storage {
        Storage::new(self.kv.namespace(&format!("nodes/{node_id}")))
    }

    pub(crate) fn get_output(&self, node_id: Uuid) -> Result<Option<serde_json::Value>, Exception> {
        let bytes = self.storage(node_id).get(OUTPUT_KEY)?;
        Ok(bytes.map(|bytes| serde_json::from_slice(&bytes)).transpose()?)
    }

    pub(crate) async fn get_flow_outputs(
        &self,
        flow_id: Uuid,
    ) -> Result<BTreeMap<String, NodeOutput>, Exception> {
        let nodes = self.database.nodes.find(Condition::new().eq("flow_id", flow_id)).await?;
        let mut outputs = BTreeMap::new();
        for node in nodes {
            if let Some(output) = self.get_output(node.id)? {
                outputs.insert(node.name, NodeOutput { node_id: node.id, value: output });
            }
        }
        Ok(outputs)
    }

    pub(crate) async fn enqueue_execution(
        &self,
        flow_id: Uuid,
        node_id: Uuid,
    ) -> Result<NodeExecution, Exception> {
        if !self
            .database
            .nodes
            .exists(Condition::new().eq("id", node_id).eq("flow_id", flow_id))
            .await?
        {
            return Err(ResourceException::NotFound(Resource::Node).into());
        }
        let execution = NodeExecution {
            id: Uuid::now_v7(),
            flow_id,
            node_id,
            status: ExecutionStatus::Pending,
            started_at: None,
            finished_at: None,
            exit_code: None,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        self.insert_execution(execution.clone())?;
        self.queue.enqueue(ExecuteNodeEvent { node_id }.into()).await?;
        Ok(execution)
    }

    pub(crate) fn get_executions(&self, node_id: Uuid) -> Result<Vec<NodeExecution>, Exception> {
        match self.kv.namespace(&format!("nodes/{node_id}")).get(b"executions") {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(giong_kv::error::Error::NotFound) => Ok(Vec::new()),
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) fn insert_execution(&self, execution: NodeExecution) -> Result<(), Exception> {
        let _guard = EXECUTIONS.lock().map_err(Exception::unexpected)?;
        let node_id = execution.node_id;
        let mut executions = self.get_executions(node_id)?;
        executions.push(execution);
        executions.sort_by_key(|execution| Reverse(execution.id));
        for old in executions.drain(executions.len().min(10)..) {
            match fs::remove_file(Self::get_log_path(old.node_id, old.id)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        self.kv
            .namespace(&format!("nodes/{node_id}"))
            .put(b"executions", &serde_json::to_vec(&executions)?)?;
        Ok(())
    }

    pub(crate) fn update_execution(&self, execution: NodeExecution) -> Result<bool, Exception> {
        let _guard = EXECUTIONS.lock().map_err(Exception::unexpected)?;
        let node_id = execution.node_id;
        let mut executions = self.get_executions(node_id)?;
        let Some(current) = executions.iter_mut().find(|item| item.id == execution.id) else {
            return Ok(false);
        };
        *current = execution;
        self.kv
            .namespace(&format!("nodes/{node_id}"))
            .put(b"executions", &serde_json::to_vec(&executions)?)?;
        Ok(true)
    }

    pub(crate) fn claim_pending_execution(
        &self,
        node_id: Uuid,
    ) -> Result<Option<NodeExecution>, Exception> {
        let _guard = EXECUTIONS.lock().map_err(Exception::unexpected)?;
        let mut executions = self.get_executions(node_id)?;
        let Some(execution) =
            executions.iter_mut().rev().find(|item| item.status == ExecutionStatus::Pending)
        else {
            return Ok(None);
        };
        execution.status = ExecutionStatus::Executing;
        execution.started_at = Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string());
        let execution = execution.clone();
        self.kv
            .namespace(&format!("nodes/{node_id}"))
            .put(b"executions", &serde_json::to_vec(&executions)?)?;
        Ok(Some(execution))
    }

    pub(crate) async fn find_execution(
        &self,
        execution_id: Uuid,
    ) -> Result<Option<NodeExecution>, Exception> {
        for node in self.database.nodes.find(Condition::new()).await? {
            if let Some(execution) =
                self.get_executions(node.id)?.into_iter().find(|item| item.id == execution_id)
            {
                return Ok(Some(execution));
            }
        }
        Ok(None)
    }

    pub(crate) async fn sync_node(
        &self,
        flow_id: Uuid,
        node_id: Uuid,
        triggers: &[Trigger],
    ) -> Result<NodeEntity, Exception> {
        let node = self
            .database
            .nodes
            .find_one(Condition::new().eq("id", node_id).eq("flow_id", flow_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Node))?;
        self.sync_interval_trigger(flow_id, node.id, triggers).await?;
        self.flow_service.invalidate_graph(flow_id).await;
        Ok(node)
    }

    pub(crate) async fn sync_interval_trigger(
        &self,
        flow_id: Uuid,
        node_id: Uuid,
        triggers: &[Trigger],
    ) -> Result<(), Exception> {
        let interval = triggers
            .iter()
            .filter_map(|trigger| match trigger {
                Trigger::Interval { seconds } => Some(*seconds),
                _ => None,
            })
            .min();
        let Some(seconds) = interval else {
            self.scheduler.unschedule(node_id).await?;
            return Ok(());
        };

        let service = self.clone();
        self.scheduler
            .ensure_scheduled(node_id, seconds, move || {
                let service = service.clone();
                async move {
                    if let Err(error) = service.enqueue_execution(flow_id, node_id).await {
                        eprintln!("Node interval trigger ({node_id}): {error}");
                    }
                }
            })
            .await?;
        Ok(())
    }

    pub(crate) async fn unschedule_node(&self, node_id: Uuid) -> Result<(), Exception> {
        self.scheduler.unschedule(node_id).await?;
        Ok(())
    }

    pub(crate) async fn sync_all_interval_triggers(&self) -> Result<(), Exception> {
        for node in self.database.nodes.find(Condition::new()).await? {
            let triggers: Vec<Trigger> = serde_json::from_value(node.triggers)?;
            self.sync_interval_trigger(node.flow_id, node.id, &triggers).await?;
        }
        Ok(())
    }

    pub(crate) async fn synchronize_flow(
        &self,
        scope: &NodeScope,
        definitions: Vec<Flow>,
    ) -> Result<FlowSyncResult, Exception> {
        let _guard = FLOW_IMPORTS.lock().await;
        let report = self.flow_service.sync_flows(scope, definitions).await?;
        for flow in &report.flows {
            let flow_id = Uuid::parse_str(&flow.id).map_err(Exception::unexpected)?;
            for node in self.database.nodes.find(Condition::new().eq("flow_id", flow_id)).await? {
                let triggers: Vec<Trigger> = serde_json::from_value(node.triggers)?;
                self.sync_interval_trigger(flow_id, node.id, &triggers).await?;
            }
        }
        Ok(report)
    }

    pub(crate) async fn validate_flow_nodes(
        &self,
        flow_id: Uuid,
        exclude_id: Option<Uuid>,
        name: &str,
        kind: &str,
        properties: &serde_json::Value,
        triggers: &[Trigger],
    ) -> Result<(), Exception> {
        let existing = self.database.nodes.find(Condition::new().eq("flow_id", flow_id)).await?;
        let mut nodes = Vec::with_capacity(existing.len() + 1);
        for node in existing.iter().filter(|node| Some(node.id) != exclude_id) {
            let node_triggers: Vec<Trigger> = serde_json::from_value(node.triggers.clone())?;
            nodes.push((
                node.name.as_str(),
                node.kind.as_str(),
                &node.properties,
                Cow::Owned(node_triggers),
            ));
        }
        nodes.push((name, kind, properties, Cow::Borrowed(triggers)));

        Self::validate_definitions(nodes.iter().map(|(name, kind, properties, triggers)| {
            (*name, *kind, *properties, triggers.as_ref())
        }))
    }

    pub(super) fn validate_definitions<'a>(
        nodes: impl IntoIterator<Item = (&'a str, &'a str, &'a serde_json::Value, &'a [Trigger])>,
    ) -> Result<(), Exception> {
        let nodes: Vec<_> = nodes.into_iter().collect();
        let mut names = HashSet::with_capacity(nodes.len());
        for (node_name, node_kind, node_properties, _) in &nodes {
            if node_name.trim().is_empty()
                || node_name.chars().any(char::is_control)
                || !names.insert(*node_name)
                || !node_properties.is_object()
            {
                return Err(Exception::BadRequest(
                    "Nodes need unique names and object properties".into(),
                ));
            }
            validate_plugin_kind(node_kind)
                .map_err(|error| Exception::BadRequest(error.to_string()))?;
        }

        for (_, _, _, node_triggers) in &nodes {
            for trigger in node_triggers.iter() {
                match trigger {
                    Trigger::Interval { seconds }
                        if *seconds == 0 || *seconds > i64::MAX as u64 =>
                    {
                        return Err(Exception::BadRequest(
                            "Trigger interval must be a positive number of seconds".into(),
                        ));
                    }
                    Trigger::Event { node: source, .. } if !names.contains(source.as_str()) => {
                        return Err(Exception::BadRequest(
                            "Event triggers must name a node in this flow".into(),
                        ));
                    }
                    _ => {}
                }
            }
        }

        // A topological order validates event dependencies and prevents trigger loops.
        let mut remaining: Vec<_> =
            nodes.iter().map(|(name, _, _, triggers)| (*name, *triggers)).collect();
        let mut done = HashSet::with_capacity(remaining.len());
        while !remaining.is_empty() {
            let Some(index) = remaining.iter().position(|(_, node_triggers)| {
                node_triggers.iter().all(|trigger| match trigger {
                    Trigger::Event { node: source, .. } => done.contains(source.as_str()),
                    _ => true,
                })
            }) else {
                return Err(Exception::BadRequest("Node triggers contain a cycle".into()));
            };
            let (node_name, _) = remaining.swap_remove(index);
            done.insert(node_name);
        }

        Ok(())
    }
}
