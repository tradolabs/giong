use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct CreateStackRequestDto {
    pub name: String,
    pub repo_url: String,
    pub branch: String,
    pub folder_path: String,
}
