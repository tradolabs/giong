use giong_database::entities::stack_entity::StackEntity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(in crate::presentation) struct StackResponseDto {
    pub id: Uuid,
    pub name: String,
    pub repo_url: String,
    pub branch: String,
    pub folder_path: String,
    pub inherited_secret_set_ids: serde_json::Value,
    pub last_commit: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<StackEntity> for StackResponseDto {
    fn from(entity: StackEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            repo_url: entity.repo_url,
            branch: entity.branch,
            folder_path: entity.folder_path,
            inherited_secret_set_ids: entity.inherited_secret_set_ids,
            last_commit: entity.last_commit,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
