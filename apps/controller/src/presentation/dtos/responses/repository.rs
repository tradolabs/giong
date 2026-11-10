use giong_database::entities::repository_entity::RepositoryEntity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(in crate::presentation) struct RepositoryResponseDto {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub has_credentials: bool,
}

impl From<RepositoryEntity> for RepositoryResponseDto {
    fn from(entity: RepositoryEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            url: entity.url,
            has_credentials: entity.credentials.is_some(),
        }
    }
}
