use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub kind: String,
    pub name: String,
    pub version: String,
    pub description: String,
}
