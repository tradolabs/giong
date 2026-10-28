use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Resource {
    pub key: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub config: Value,
}
