use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryEntity {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    /// Encrypted personal access token, decrypted by core for HTTPS Git authentication.
    pub credentials: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
