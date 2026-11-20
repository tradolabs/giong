use giong_node::host::Trigger;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct UpdateNodeRequestDto {
    #[validate(length(min = 1), non_control_character)]
    pub name: String,
    #[serde(rename = "type")]
    #[validate(length(min = 1), non_control_character)]
    pub kind: String,
    #[serde(default = "empty_properties")]
    pub properties: serde_json::Value,
    #[serde(default)]
    pub triggers: Vec<Trigger>,
}

fn empty_properties() -> serde_json::Value {
    serde_json::json!({})
}
