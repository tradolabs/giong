use uuid::Uuid;

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "stacks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub repo_url: String,
    pub branch: String,
    pub folder_path: String,
    pub last_commit: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::repository::Entity",
        from = "Column::RepoUrl",
        to = "super::repository::Column::Url"
    )]
    Repository,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::entities::stack_entity::StackEntity {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            repo_url: model.repo_url,
            branch: model.branch,
            folder_path: model.folder_path,
            last_commit: model.last_commit,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
