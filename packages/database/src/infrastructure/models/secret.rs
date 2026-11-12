use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "secrets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub vault_id: Uuid,
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
impl From<Model> for crate::entities::secret_entity::SecretEntity {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            vault_id: model.vault_id,
            key: model.key,
            value: model.value,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
