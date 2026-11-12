use giong_database::entities::flow_entity::FlowEntity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(in crate::presentation) struct FlowResponseDto {
    pub id: Uuid,
    pub name: String,
    pub vault_id: Option<Uuid>,
    pub inherited_vault_ids: serde_json::Value,
    pub nodes: Vec<giong_database::entities::node_entity::NodeEntity>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<FlowEntity> for FlowResponseDto {
    fn from(entity: FlowEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            nodes: entity.nodes,
            vault_id: entity.vault_id,
            inherited_vault_ids: entity.inherited_vault_ids,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
