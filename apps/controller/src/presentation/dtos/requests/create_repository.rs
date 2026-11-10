use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct CreateRepositoryRequestDto {
    pub name: String,
    pub url: String,
    pub token: Option<String>,
}
