use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "secret_sets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: Option<String>,
    pub r#type: String,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
impl From<Model> for crate::entities::secret_set_entity::SecretSetEntity {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            r#type: model.r#type,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
