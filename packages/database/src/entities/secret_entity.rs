use uuid::Uuid;

pub struct SecretEntity {
    pub id: Uuid,
    pub stack_id: Uuid,
    pub key: String,
    /// Encrypted by core using the stack ID and key as associated data.
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}
