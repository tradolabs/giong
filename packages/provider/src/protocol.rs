use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub name: String,
    pub package: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(alias = "config")]
    pub inputs: Value,
}

impl Resource {
    pub fn urn(&self) -> String {
        format!("{}::{}", self.package, self.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeStatus {
    Created,
    Updated,
    Unchanged,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub urn: String,
    pub inputs: Value,
}
