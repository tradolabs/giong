use serde::Deserialize;
use validator::Validate;
#[derive(Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct UpdateFlowRequestDto {
    #[validate(length(min = 1), non_control_character)]
    pub name: String,
    #[serde(default)]
    pub vault_ids: Vec<uuid::Uuid>,
}
