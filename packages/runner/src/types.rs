use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub configuration_schema: Value,
    pub defaults: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Plan {
    Docker {
        dockerfile: String,
        #[serde(default)]
        dockerignore: Option<String>,
        #[serde(default)]
        args: Vec<String>,
    },
}

pub struct Secret {
    pub key: String,
    pub value: String,
}
