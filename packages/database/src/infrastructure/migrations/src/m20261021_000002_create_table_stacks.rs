use crate::conventions;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("stacks")
                    .if_not_exists()
                    .col(ColumnDef::new("id").uuid().not_null().primary_key())
                    .col(ColumnDef::new("name").string().not_null())
                    .col(ColumnDef::new("repo_url").string().not_null())
                    .col(ColumnDef::new("branch").string().not_null())
                    .col(ColumnDef::new("folder_path").string().not_null())
                    .col(ColumnDef::new("last_commit").string())
                    .col(
                        ColumnDef::new("created_at")
                            .string()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new("updated_at")
                            .string()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .index(
                        Index::create()
                            .name(conventions::unique_constraint_name("stacks", &["name"]))
                            .table("stacks")
                            .col("name")
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("stacks").if_exists().to_owned()).await?;

        Ok(())
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}
