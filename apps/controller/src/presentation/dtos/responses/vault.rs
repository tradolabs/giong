use giong_database::entities::vault_entity::VaultEntity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(in crate::presentation) struct VaultResponseDto {
    pub id: Uuid,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<VaultEntity> for VaultResponseDto {
    fn from(entity: VaultEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
