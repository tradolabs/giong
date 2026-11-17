use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Reserved node storage key for the output published by execute.
pub const OUTPUT_KEY: &str = "__giong/output";

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

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    Interval { seconds: u64 },
    Event { node: String },
}

impl<'de> Deserialize<'de> for Trigger {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
        enum Input {
            Interval {
                seconds: u64,
            },
            Event {
                node: String,
                // Existing stored triggers and flow files may still include an event name.
                #[serde(default, rename = "event")]
                _legacy_event: serde::de::IgnoredAny,
            },
        }

        Ok(match Input::deserialize(deserializer)? {
            Input::Interval { seconds } => Self::Interval { seconds },
            Input::Event { node, .. } => Self::Event { node },
        })
    }
}
