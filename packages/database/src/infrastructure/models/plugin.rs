use crate::entities::plugin_entity::PluginEntity;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "plugins")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub kind: String,
    pub image: String,
    pub image_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for PluginEntity {
    fn from(model: Model) -> Self {
        Self {
            kind: model.kind,
            image: model.image,
            image_id: model.image_id,
            name: model.name,
            version: model.version,
            description: model.description,
        }
    }
}
