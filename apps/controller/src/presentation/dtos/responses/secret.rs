use giong_database::entities::secret_entity::SecretEntity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(in crate::presentation) struct SecretResponseDto {
    pub id: Uuid,
    pub key: String,
}

impl From<SecretEntity> for SecretResponseDto {
    fn from(entity: SecretEntity) -> Self {
        Self { id: entity.id, key: entity.key }
    }
}
