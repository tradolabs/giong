use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackEntity {
    pub id: Uuid,
    pub name: String,
    pub repo_url: String,
    pub branch: String,
    pub folder_path: String,
    pub last_commit: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
