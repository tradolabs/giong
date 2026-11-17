use sea_orm::entity::prelude::*;
use uuid::Uuid;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub flow_id: Uuid,
    pub name: String,
    pub kind: String,
    pub properties: serde_json::Value,
    pub triggers: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
impl From<Model> for crate::entities::node_entity::NodeEntity {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            flow_id: model.flow_id,
            name: model.name,
            kind: model.kind,
            properties: model.properties,
            triggers: model.triggers,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
