use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub configuration_schema: Value,
    pub defaults: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Plan {
    Docker {
        dockerfile: String,
        #[serde(default)]
        dockerignore: Option<String>,
        #[serde(default)]
        args: Vec<String>,
    },
}

pub struct Secret {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub repo_url: String,
    #[serde(default = "default_revision")]
    pub revision: String,
    pub path: String,
}

fn default_revision() -> String {
    "HEAD".into()
}

pub fn configuration_sources(configurations: &Value) -> Result<Vec<Source>, crate::error::Error> {
    use crate::error::Error;
    if configurations.get("repositories").is_some() {
        return Err(Error::Unexpected(
            "Define repository URLs directly in configuration.source".into(),
        ));
    }
    let value = configurations
        .pointer("/configuration/source")
        .ok_or_else(|| Error::Unexpected("configuration.source is required".into()))?;
    let sources: Vec<Source> = if value.is_array() {
        serde_json::from_value(value.clone())
    } else {
        serde_json::from_value(value.clone()).map(|source| vec![source])
    }
    .map_err(|error| Error::Unexpected(error.to_string()))?;
    if sources.is_empty()
        || sources.iter().any(|source| {
            source.repo_url.trim().is_empty()
                || source.revision.trim().is_empty()
                || source.path.trim().is_empty()
        })
    {
        return Err(Error::Unexpected(
            "At least one source with repo_url, revision and path is required".into(),
        ));
    }
    Ok(sources)
}
