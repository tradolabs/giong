use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum SecretValue {
    Literal(String),
    Reference { secret: SecretReference },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum SecretReference {
    Key(String),
    Scoped { key: String, vault: Option<String> },
}

impl SecretReference {
    pub fn key(&self) -> &str {
        match self {
            Self::Key(key) | Self::Scoped { key, .. } => key,
        }
    }

    pub fn vault(&self) -> Option<&str> {
        match self {
            Self::Key(_) => None,
            Self::Scoped { vault, .. } => vault.as_deref(),
        }
    }
}
