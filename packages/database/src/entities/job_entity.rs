use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobEntity {
    pub id: Uuid,
    pub stack_id: Uuid,
    pub commit_sha: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub exit_code: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}
