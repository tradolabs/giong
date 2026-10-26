use uuid::Uuid;

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "repositories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(unique)]
    pub url: String,
    /// Encrypted credentials for Git authentication.
    pub credentials: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::entities::repository_entity::RepositoryEntity {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            url: model.url,
            credentials: model.credentials,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
