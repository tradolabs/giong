use uuid::Uuid;

pub struct SecretEntity {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}
