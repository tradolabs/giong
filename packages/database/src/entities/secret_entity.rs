use uuid::Uuid;

pub struct SecretEntity {
    pub id: Uuid,
    pub secret_set_id: Uuid,
    pub key: String,
    /// Encrypted text value.
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}
