use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowEntity {
    pub id: Uuid,
    pub name: String,
    pub vault_id: Option<Uuid>,
    pub inherited_vault_ids: serde_json::Value,
    pub nodes: Vec<crate::entities::node_entity::NodeEntity>,
    pub created_at: String,
    pub updated_at: String,
}
