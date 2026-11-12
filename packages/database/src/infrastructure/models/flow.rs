use uuid::Uuid;

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "flows")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub vault_id: Option<Uuid>,
    pub inherited_vault_ids: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::entities::flow_entity::FlowEntity {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            vault_id: model.vault_id,
            inherited_vault_ids: model.inherited_vault_ids,
            nodes: Vec::new(),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
