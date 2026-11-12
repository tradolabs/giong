use crate::application::exceptions::base::Exception;
use crate::application::helpers::filesystem::{remove_dir, root_dir};
use crate::application::helpers::nodes::flow_nodes;
use crate::application::use_cases::secrets::{
    GetSecretValueInput, GetSecretValueUseCase, SecretScope,
};
use crate::application::use_cases::use_case::UseCase;
use crate::config::Config;
use crate::infrastructure::adapters::node_storage::NodeStorage;
use anyhow::Result;
use giong_database::Database;
use giong_di::Injectable;
use giong_node::runtime::Plugin;
use giong_node::tools::Tools;
use giong_node::tools::execution::Execution;
use giong_node::tools::secrets::Secrets;
use giong_node::tools::storage::Storage;
use giong_node::types::node::{Flow, OUTPUT_KEY};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

pub struct RunFlowInput {
    pub flow: Flow,
    pub run_id: Option<Uuid>,
    pub scope: Option<String>,
    pub node_id: Option<Uuid>,
    pub log_timestamp: Option<String>,
}
#[derive(Serialize)]
pub struct FlowRun {
    pub id: Uuid,
    pub completed: Vec<String>,
}
#[derive(Injectable)]
pub struct RunFlowUseCase {
    database: Arc<Database>,
    secrets: Arc<GetSecretValueUseCase>,
    config: Arc<Config>,
    node_storage: Arc<NodeStorage>,
}
impl UseCase for RunFlowUseCase {
    type Input = RunFlowInput;
    type Output = FlowRun;
    async fn execute(&self, input: Self::Input) -> Result<FlowRun, Exception> {
        let run_id = input.run_id.unwrap_or_else(Uuid::now_v7);
        let scope =
            input.scope.or_else(|| input.flow.name.clone()).unwrap_or_else(|| run_id.to_string());
        let lock = self.node_storage.flow_lock(&scope).map_err(Exception::unexpected)?;
        let _guard = lock.lock().await;
        let vault_id = input
            .flow
            .vault_id
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|_| Exception::BadRequest("Invalid vault_id".into()))?;
        let inherited_vault_ids = input
            .flow
            .inherited_vault_ids
            .iter()
            .map(|id| Uuid::parse_str(id))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| Exception::BadRequest("Invalid inherited_vault_ids".into()))?;
        crate::application::helpers::secrets::validate_vaults(
            &self.database,
            vault_id,
            &inherited_vault_ids,
        )
        .await?;
        let secrets = self.secrets.clone();
        let runtime = tokio::runtime::Handle::current();
        let secrets = Secrets {
            get: Arc::new(move |key| {
                runtime
                    .block_on(secrets.execute(GetSecretValueInput {
                        scope: SecretScope {
                            vault_id,
                            inherited_vault_ids: inherited_vault_ids.clone(),
                        },
                        key: key.to_owned(),
                    }))
                    .map_err(anyhow::Error::from)
            }),
        };
        let nodes = flow_nodes(input.flow, &scope)?;
        if nodes.is_empty() && input.node_id.is_none() {
            return Ok(FlowRun { id: run_id, completed: vec![] });
        }
        let persistent = Uuid::parse_str(&scope).is_ok();
        let workspace = if persistent {
            root_dir().join("workspaces").join(&scope)
        } else {
            root_dir().join("runs").join(run_id.to_string())
        };
        let selected = input.node_id.map(|id| id.to_string());
        if selected.as_ref().is_some_and(|selected| !nodes.iter().any(|node| &node.id == selected))
        {
            return Err(Exception::NotFound("Node not found in flow"));
        }
        fs::create_dir_all(&workspace).map_err(Exception::unexpected)?;
        let workspace = fs::canonicalize(workspace).map_err(Exception::unexpected)?;
        let log_timestamp = input.log_timestamp;
        let node_storage = self.node_storage.clone();
        let encryption_key = self.config.encryption_key.clone();
        let result = tokio::task::spawn_blocking({
            let workspace = workspace.clone();
            move || -> Result<FlowRun> {
                let kv = node_storage.open()?;
                let outputs = Arc::new(Mutex::new(BTreeMap::new()));
                // Outputs remain private to this flow and survive independent node runs.
                for node in &nodes {
                    let storage = Storage::new(
                        kv.namespace(&node.id),
                        node.id.clone(),
                        encryption_key.as_deref(),
                    )?;
                    if let Some(bytes) = storage.get(OUTPUT_KEY)? {
                        outputs
                            .lock()
                            .map_err(|_| anyhow::anyhow!("Node outputs lock failed"))?
                            .insert(node.definition.name.clone(), serde_json::from_slice(&bytes)?);
                    }
                }
                let mut completed = Vec::new();
                for node in &nodes {
                    if selected.as_ref().is_some_and(|selected| selected != &node.id) {
                        continue;
                    }
                    let directory = Uuid::parse_str(&node.id).unwrap_or_else(|_| Uuid::now_v7());
                    let storage = Storage::new(
                        kv.namespace(&node.id),
                        node.id.clone(),
                        encryption_key.as_deref(),
                    )?;
                    let timestamp = log_timestamp
                        .clone()
                        .unwrap_or_else(crate::application::helpers::node_logs::timestamp);
                    let mut logger =
                        crate::application::helpers::node_logs::open(directory, &timestamp)?;
                    use std::io::Write;
                    writeln!(
                        logger,
                        "Running node {} ({})",
                        node.definition.name, node.definition.kind
                    )?;
                    let events = kv.namespace("node-events");
                    let source = node.id.clone();
                    let event_scope = scope.clone();
                    let events = giong_node::tools::events::Events {
                        emit: Arc::new(move |event| {
                            events.put(
                                Uuid::now_v7().as_bytes(),
                                &serde_json::to_vec(&serde_json::json!({
                                    "source": source, "scope": event_scope, "event": event,
                                }))?,
                            )?;
                            Ok(())
                        }),
                    };
                    let mut plugin = Plugin::load(
                        &node.definition.kind,
                        Tools {
                            events: Some(events),
                            secrets: Some(secrets.clone()),
                            scope: Some(scope.clone()),
                            workspace: Some(workspace.join(format!("node-{directory}"))),
                            storage: Some(storage.clone()),
                            outputs: Some(outputs.clone()),
                            execution: Some(Execution {
                                id: run_id.to_string(),
                                workspace: workspace.clone(),
                                logger,
                            }),
                        },
                    )?;
                    storage.delete(OUTPUT_KEY)?;
                    plugin.configure(&serde_json::to_string(&node.definition.properties)?)?;
                    let output = storage
                        .get(OUTPUT_KEY)?
                        .map(|bytes| serde_json::from_slice(&bytes))
                        .transpose()?
                        .unwrap_or(serde_json::Value::Null);
                    outputs
                        .lock()
                        .map_err(|_| anyhow::anyhow!("Node outputs lock failed"))?
                        .insert(node.definition.name.clone(), output);
                    completed.push(node.definition.name.clone());
                }
                // Outputs may contain secrets; only execution metadata is returned to the caller.
                Ok(FlowRun { id: run_id, completed })
            }
        })
        .await
        .map_err(Exception::unexpected)
        .and_then(|result| result.map_err(Exception::unexpected));
        if !persistent {
            remove_dir(&workspace);
        }
        result
    }
}
