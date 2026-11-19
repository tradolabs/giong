use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct KubectlProperties {
    pub source: String,
    pub path: Option<String>,
    pub values_file: Option<String>,
    pub namespace: Option<String>,
    pub release: Option<String>,
    pub kubeconfig: Option<serde_json::Value>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct DeploymentState {
    pub revision: String,
    pub configuration: String,
}

#[derive(Serialize)]
pub(crate) struct KubectlOutput {
    pub exit_code: i32,
    pub skipped: bool,
    pub revision: String,
}
