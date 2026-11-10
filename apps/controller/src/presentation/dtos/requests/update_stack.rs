use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::presentation) struct UpdateStackRequestDto {
    pub repo_url: Option<String>,
    pub folder_path: Option<String>,
}
