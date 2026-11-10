use giong_database::entities::stack_entity::StackEntity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(in crate::presentation) struct StackResponseDto {
    pub id: Uuid,
    pub name: String,
    pub configurations: serde_json::Value,
    pub secret_set_id: Option<Uuid>,
    pub inherited_secret_set_ids: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl From<StackEntity> for StackResponseDto {
    fn from(entity: StackEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            configurations: entity.configurations,
            secret_set_id: entity.secret_set_id,
            inherited_secret_set_ids: entity.inherited_secret_set_ids,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
