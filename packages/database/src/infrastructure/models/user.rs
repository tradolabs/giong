use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::entities::user_entity::UserEntity {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
            password_hash: model.password_hash,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
