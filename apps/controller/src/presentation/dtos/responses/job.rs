use giong_core::JobEntity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(in crate::presentation) struct JobResponseDto {
    pub id: Uuid,
    pub stack_id: Uuid,
    pub commit_sha: String,
    pub status: &'static str,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub exit_code: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<JobEntity> for JobResponseDto {
    fn from(entity: JobEntity) -> Self {
        Self {
            id: entity.id,
            stack_id: entity.stack_id,
            commit_sha: entity.commit_sha,
            status: entity.status.as_str(),
            started_at: entity.started_at,
            finished_at: entity.finished_at,
            exit_code: entity.exit_code,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}
