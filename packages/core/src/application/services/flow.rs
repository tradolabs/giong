use crate::application::exceptions::base::Exception;
use crate::application::exceptions::resource::{Resource, ResourceException};
use crate::application::helpers::flow_graph::{FlowGraph, FlowGraphCache};
use crate::application::services::node::NodeService;
use giong_database::Database;
use giong_database::entities::flow_entity::FlowEntity;
use giong_database::entities::node_entity::NodeEntity;
use giong_database::error::Error as DatabaseError;
use giong_database::repositories::types::condition::Condition;
use giong_database::repositories::types::document::Document;
use giong_di::Injectable;
use giong_docker::images;
use giong_kv::kv::Kv;
use giong_node::host::{Definition, Flow, FlowSyncResult, NodeScope, SyncedFlow, Trigger};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Injectable)]
pub struct FlowService {
    database: Arc<Database>,
    kv: Arc<Kv>,
    cache: Arc<FlowGraphCache>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ManagedFlow {
    id: Uuid,
}

struct PlannedNode {
    id: Uuid,
    name: String,
    kind: String,
    properties: serde_json::Value,
    triggers: Vec<Trigger>,
    exists: bool,
}

struct PlannedFlow {
    id: Uuid,
    name: String,
    vault_ids: serde_json::Value,
    exists: bool,
    node_count: u32,
    nodes: Vec<PlannedNode>,
}

impl FlowService {
    pub(crate) async fn get_flow(&self, flow_id: Uuid) -> Result<FlowEntity, Exception> {
        self.database
            .flows
            .find_one(Condition::new().eq("id", flow_id))
            .await?
            .ok_or_else(|| ResourceException::NotFound(Resource::Flow).into())
    }

    pub(crate) async fn get_flow_with_nodes(&self, flow_id: Uuid) -> Result<FlowEntity, Exception> {
        let mut flow = self.get_flow(flow_id).await?;
        self.populate_nodes(std::slice::from_mut(&mut flow)).await?;
        Ok(flow)
    }

    pub(crate) async fn populate_nodes(&self, flows: &mut [FlowEntity]) -> Result<(), Exception> {
        if flows.is_empty() {
            return Ok(());
        }
        let flow_ids = flows.iter().map(|flow| flow.id).collect::<Vec<_>>();
        let nodes = self.database.nodes.find(Condition::new().is_in("flow_id", flow_ids)).await?;
        let mut by_flow: HashMap<Uuid, Vec<NodeEntity>> = HashMap::new();
        for node in nodes {
            by_flow.entry(node.flow_id).or_default().push(node);
        }
        for flow in flows {
            let mut flow_nodes = by_flow.remove(&flow.id).unwrap_or_default();
            flow_nodes.sort_by_key(|node| node.id);
            flow.nodes = flow_nodes;
        }
        Ok(())
    }

    pub(crate) async fn get_event_targets(
        &self,
        flow_id: Uuid,
        source_node_id: Uuid,
    ) -> Result<Vec<Uuid>, Exception> {
        if let Some(graph) = self.cache.graphs.read().await.get(&flow_id).cloned() {
            return Ok(graph.targets(source_node_id));
        }

        let mut graphs = self.cache.graphs.write().await;
        if let Some(graph) = graphs.get(&flow_id) {
            return Ok(graph.targets(source_node_id));
        }
        let nodes = self.database.nodes.find(Condition::new().eq("flow_id", flow_id)).await?;
        let graph = Arc::new(FlowGraph::from_nodes(&nodes)?);
        let targets = graph.targets(source_node_id);
        graphs.insert(flow_id, graph);
        Ok(targets)
    }

    pub(crate) async fn invalidate_graph(&self, flow_id: Uuid) {
        self.cache.graphs.write().await.remove(&flow_id);
    }

    pub(super) async fn sync_flows(
        &self,
        scope: &NodeScope,
        definitions: Vec<Flow>,
    ) -> Result<FlowSyncResult, Exception> {
        if scope.kind != "flow" {
            return Err(Exception::BadRequest("Flow synchronization is not granted".into()));
        }
        if definitions.len() > 128
            || definitions.iter().map(|flow| flow.nodes.len()).sum::<usize>() > 1024
        {
            return Err(Exception::BadRequest("Import exceeds 128 flows or 1024 nodes".into()));
        }
        let runner = self
            .database
            .nodes
            .find_one(
                Condition::new()
                    .eq("id", scope.node_id)
                    .eq("flow_id", scope.flow_id)
                    .eq("kind", scope.kind.as_str()),
            )
            .await?
            .ok_or(ResourceException::NotFound(Resource::Node))?;
        let caller = self
            .database
            .flows
            .find_one(Condition::new().eq("id", runner.flow_id))
            .await?
            .ok_or(ResourceException::NotFound(Resource::Flow))?;
        let allowed: Vec<Uuid> = serde_json::from_value(caller.vault_ids)?;
        let available_vaults: HashMap<_, _> = self
            .database
            .vaults
            .find(Condition::new().is_in("id", allowed))
            .await?
            .into_iter()
            .map(|vault| (vault.name, vault.id))
            .collect();
        let existing_flows = self.database.flows.find(Condition::new()).await?;
        // The registry is outside guest node storage, so a guest cannot claim other flows.
        let registry_store = self.kv.namespace(&format!("flow-imports/{}", runner.id));
        let mut registry: BTreeMap<String, ManagedFlow> = match registry_store.get(b"flows") {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(giong_kv::error::Error::NotFound) => BTreeMap::new(),
            Err(error) => return Err(error.into()),
        };
        let mut names = HashSet::new();
        let mut plan = Vec::new();
        let registered = self.database.plugins.find(Condition::new()).await?;
        let local_image_ids: HashSet<_> =
            images::get_images().await?.into_iter().map(|image| image.id).collect();
        let available_kinds: HashSet<_> = registered
            .into_iter()
            .filter(|plugin| local_image_ids.contains(&plugin.image_id))
            .map(|plugin| plugin.kind)
            .collect();
        for definition in definitions {
            let name = definition.name.as_deref().unwrap_or("").trim().to_owned();
            if name.is_empty() || name.chars().any(char::is_control) || !names.insert(name.clone())
            {
                return Err(Exception::BadRequest(
                    "Imported flows need unique, nonempty names".into(),
                ));
            }
            let managed = registry
                .entry(name.clone())
                .or_insert_with(|| ManagedFlow { id: Uuid::now_v7() })
                .clone();
            if managed.id == caller.id
                || existing_flows.iter().any(|flow| flow.name == name && flow.id != managed.id)
            {
                return Err(Exception::BadRequest(format!(
                    "Flow {name} already exists outside this runner"
                )));
            }
            let current = existing_flows.iter().find(|flow| flow.id == managed.id);
            let mut vault_ids = Vec::new();
            let mut unique_vaults = HashSet::new();
            for vault in &definition.vaults {
                let id = available_vaults.get(vault).copied().ok_or_else(|| {
                    Exception::BadRequest(format!(
                        "Vault {vault} is not attached to the calling flow"
                    ))
                })?;
                if !unique_vaults.insert(id) {
                    return Err(Exception::BadRequest(format!("Duplicate vault {vault}")));
                }
                vault_ids.push(id);
            }
            let existing_nodes =
                self.database.nodes.find(Condition::new().eq("flow_id", managed.id)).await?;
            let incoming_names: HashSet<_> =
                definition.nodes.iter().map(|node| node.name.as_str()).collect();
            let mut merged = Vec::new();
            for node in
                existing_nodes.iter().filter(|node| !incoming_names.contains(node.name.as_str()))
            {
                merged.push(Definition {
                    id: None,
                    name: node.name.clone(),
                    kind: node.kind.clone(),
                    properties: node.properties.clone(),
                    triggers: serde_json::from_value(node.triggers.clone())?,
                });
            }
            NodeService::validate_definitions(merged.iter().chain(&definition.nodes).map(
                |node| {
                    (
                        node.name.as_str(),
                        node.kind.as_str(),
                        &node.properties,
                        node.triggers.as_slice(),
                    )
                },
            ))?;
            let node_count = (merged.len() + definition.nodes.len()) as u32;
            let mut nodes = Vec::new();
            for node in definition.nodes {
                if !available_kinds.contains(&node.kind) {
                    return Err(Exception::BadRequest(format!(
                        "Plugin is not installed: {}",
                        node.kind
                    )));
                }
                let existing = existing_nodes.iter().find(|existing| existing.name == node.name);
                if let Some(id) = &node.id {
                    let id = Uuid::parse_str(id)
                        .map_err(|_| Exception::BadRequest("Invalid node ID".into()))?;
                    if existing.is_none_or(|existing| existing.id != id) {
                        return Err(Exception::BadRequest(
                            "Node IDs must match existing nodes in the managed flow".into(),
                        ));
                    }
                }
                nodes.push(PlannedNode {
                    id: existing.map_or_else(Uuid::now_v7, |node| node.id),
                    name: node.name,
                    kind: node.kind,
                    properties: node.properties,
                    triggers: node.triggers,
                    exists: existing.is_some(),
                });
            }
            plan.push(PlannedFlow {
                id: managed.id,
                name,
                vault_ids: serde_json::to_value(vault_ids)?,
                exists: current.is_some(),
                node_count,
                nodes,
            });
        }
        // Reserve identities before writes. Retries can recover even on backends without transactions.
        registry_store.put(b"flows", &serde_json::to_vec(&registry)?)?;
        let result = self
            .database
            .transaction(async || {
                for flow in &plan {
                    let changes = Document::new()
                        .set("name", flow.name.clone())
                        .set("vault_ids", flow.vault_ids.clone());
                    if flow.exists {
                        if !self
                            .database
                            .flows
                            .update_one(Condition::new().eq("id", flow.id), changes)
                            .await?
                        {
                            return Err(DatabaseError::Unexpected(
                                "Managed flow disappeared during synchronization".into(),
                            ));
                        }
                    } else {
                        self.database.flows.create_one(changes.set("id", flow.id)).await?;
                    }
                    for node in &flow.nodes {
                        let changes = Document::new()
                            .set("name", node.name.clone())
                            .set("kind", node.kind.clone())
                            .set("properties", node.properties.clone())
                            .set(
                                "triggers",
                                serde_json::to_value(&node.triggers).map_err(|error| {
                                    DatabaseError::Unexpected(error.to_string())
                                })?,
                            );
                        if node.exists {
                            if !self
                                .database
                                .nodes
                                .update_one(
                                    Condition::new().eq("id", node.id).eq("flow_id", flow.id),
                                    changes,
                                )
                                .await?
                            {
                                return Err(DatabaseError::Unexpected(
                                    "Managed node disappeared during synchronization".into(),
                                ));
                            }
                        } else {
                            self.database
                                .nodes
                                .create_one(changes.set("id", node.id).set("flow_id", flow.id))
                                .await?;
                        }
                    }
                }
                Ok(())
            })
            .await;
        for flow in &plan {
            self.invalidate_graph(flow.id).await;
        }
        result?;
        Ok(FlowSyncResult {
            flows: plan
                .into_iter()
                .map(|flow| SyncedFlow {
                    id: flow.id.to_string(),
                    name: flow.name,
                    node_count: flow.node_count,
                })
                .collect(),
        })
    }
}
