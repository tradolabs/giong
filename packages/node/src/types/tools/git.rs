use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckoutInputs {
    pub repo_url: String,
    #[serde(default = "default_revision")]
    pub revision: String,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckoutOutput {
    pub repository_path: String,
    pub commit_sha: String,
}

fn default_revision() -> String {
    "HEAD".into()
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangedFilesInputs {
    pub repository_path: String,
    pub from_revision: String,
    pub to_revision: String,
    #[serde(default)]
    pub paths: Vec<String>,
}
