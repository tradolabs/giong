use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct UpdateStackRequestDto {
    pub name: String,
    pub configurations: serde_json::Value,
}
