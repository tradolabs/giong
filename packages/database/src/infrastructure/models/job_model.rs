use uuid::Uuid;

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "jobs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub stack_id: Uuid,
    pub commit_sha: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub exit_code: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::stack_model::Entity",
        from = "Column::StackId",
        to = "super::stack_model::Column::Id"
    )]
    Stack,
}

impl Related<super::stack_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Stack.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for crate::entities::job_entity::JobEntity {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            stack_id: model.stack_id,
            commit_sha: model.commit_sha,
            status: model.status,
            started_at: model.started_at,
            finished_at: model.finished_at,
            exit_code: model.exit_code,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
