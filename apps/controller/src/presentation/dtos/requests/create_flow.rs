use serde::Deserialize;
use validator::Validate;
#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct CreateFlowRequestDto {
    #[validate(length(min = 1), non_control_character)]
    pub name: String,
    #[serde(default)]
    pub inherited_vault_ids: Vec<uuid::Uuid>,
}
