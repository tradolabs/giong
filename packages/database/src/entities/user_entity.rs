use uuid::Uuid;

pub struct UserEntity {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
}
