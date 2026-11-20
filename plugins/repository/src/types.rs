use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryInput {
    pub repo_url: String,
    #[serde(default = "default_revision")]
    pub revision: String,
    #[serde(default)]
    pub token: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepositoryOutput {
    pub repository_path: String,
}

fn default_revision() -> String {
    "HEAD".into()
}
