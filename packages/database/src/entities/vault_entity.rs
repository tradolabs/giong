use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct VaultEntity {
    pub id: Uuid,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}
