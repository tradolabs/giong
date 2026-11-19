#[cfg(feature = "host")]
use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(feature = "host")]
use std::{collections::BTreeMap, fs::File};
#[cfg(feature = "host")]
use uuid::Uuid;

#[cfg(feature = "host")]
/// Identity of the flow and node for a single execution.
pub struct NodeScope {
    pub flow_id: Uuid,
    pub node_id: Uuid,
    pub execution_id: Uuid,
    pub kind: String,
}

#[cfg(feature = "host")]
pub struct NodeOutput {
    pub node_id: Uuid,
    pub value: Value,
}

#[cfg(feature = "host")]
/// Application operations available to a node execution.
pub trait NodeHost: Send + Sync {
    fn storage(&self, scope: &NodeScope) -> Storage;
    fn open_log(&self, scope: &NodeScope) -> anyhow::Result<File>;
    fn outputs(&self, scope: &NodeScope) -> anyhow::Result<BTreeMap<String, NodeOutput>>;

    fn get_secret(
        &self,
        scope: &NodeScope,
        vault: Option<&str>,
        key: &str,
    ) -> anyhow::Result<String>;
    fn emit_changed(&self, scope: &NodeScope) -> anyhow::Result<()>;

    /// Apply flow definitions with the permissions of the current execution.
    fn synchronize_flow(
        &self,
        scope: &NodeScope,
        flows: Vec<Flow>,
    ) -> anyhow::Result<FlowSyncResult>;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default = "empty_properties", alias = "inputs")]
    pub properties: Value,
    #[serde(default)]
    pub triggers: Vec<Trigger>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flow {
    #[serde(default)]
    pub vault_id: Option<String>,
    #[serde(default)]
    pub inherited_vault_ids: Vec<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub nodes: Vec<Definition>,
}

fn empty_properties() -> Value {
    serde_json::json!({})
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    Interval { seconds: u64 },
    Event { node: String },
}

#[derive(Serialize, Deserialize)]
pub struct FlowSyncResult {
    pub flows: Vec<SyncedFlow>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncedFlow {
    pub id: String,
    pub name: String,
    pub node_count: u32,
}
