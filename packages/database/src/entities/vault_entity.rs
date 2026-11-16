use crate::repositories::types::field_value::FieldValue;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VaultScope {
    Global,
    Flow,
}

impl VaultScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Flow => "flow",
        }
    }
}

impl From<VaultScope> for FieldValue {
    fn from(scope: VaultScope) -> Self {
        Self::String(scope.as_str().to_owned())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct VaultEntity {
    pub id: Uuid,
    pub name: String,
    pub scope: VaultScope,
    pub created_at: String,
    pub updated_at: String,
}
