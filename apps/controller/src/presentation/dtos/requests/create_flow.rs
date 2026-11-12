use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct CreateFlowRequestDto {
    pub name: String,
    #[serde(default)]
    pub inherited_vault_ids: Vec<uuid::Uuid>,
    #[serde(default)]
    pub nodes: Vec<giong_node::types::node::Definition>,
}
