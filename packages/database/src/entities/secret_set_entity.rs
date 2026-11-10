use uuid::Uuid;

pub struct SecretSetEntity {
    pub id: Uuid,
    pub name: Option<String>,
    pub r#type: String,
    pub created_at: String,
    pub updated_at: String,
}
