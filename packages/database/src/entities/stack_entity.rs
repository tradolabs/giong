use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackEntity {
    pub id: Uuid,
    pub name: String,
    pub configurations: serde_json::Value,
    pub secret_set_id: Option<Uuid>,
    pub inherited_secret_set_ids: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}
