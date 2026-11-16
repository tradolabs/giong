use giong_database::entities::node_entity::NodeEntity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(in crate::presentation) struct NodeResponseDto {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub properties: serde_json::Value,
    pub triggers: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<NodeEntity> for NodeResponseDto {
    fn from(entity: NodeEntity) -> Self {
        Self {
            id: entity.id,
            flow_id: entity.flow_id,
            name: entity.name,
            kind: entity.kind,
            properties: entity.properties,
            triggers: entity.triggers,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
