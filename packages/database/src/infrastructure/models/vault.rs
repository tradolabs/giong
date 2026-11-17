use crate::entities::vault_entity::{VaultEntity, VaultScope};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "vaults")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub scope: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for VaultEntity {
    fn from(model: Model) -> Self {
        let scope = match model.scope.as_str() {
            "flow" => VaultScope::Flow,
            _ => VaultScope::Global,
        };
        Self {
            id: model.id,
            name: model.name,
            scope,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
