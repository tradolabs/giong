use serde::Deserialize;

pub struct LocalImage {
    pub id: String,
    pub reference: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DockerImage {
    pub id: Option<String>,
    pub repo_tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct DockerMessage {
    pub message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DockerPullProgress {
    pub error: Option<String>,
    pub error_detail: Option<DockerMessage>,
}
