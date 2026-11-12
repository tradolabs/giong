use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum SecretValue {
    Literal(String),
    Reference { secret: String },
}
