use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct UpdateFlowRequestDto {
    pub name: String,
    #[serde(default)]
    pub inherited_vault_ids: Vec<uuid::Uuid>,
    pub nodes: Vec<giong_node::types::node::Definition>,
}
