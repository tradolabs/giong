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
                    .table("jobs")
                    .if_not_exists()
                    .col(ColumnDef::new("id").uuid().not_null().primary_key())
                    .col(ColumnDef::new("stack_id").uuid().not_null())
                    .col(ColumnDef::new("commit_sha").string().not_null())
                    .col(ColumnDef::new("status").string().not_null())
                    .col(
                        ColumnDef::new("started_at")
                            .string()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new("finished_at").string())
                    .col(ColumnDef::new("exit_code").integer())
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
                    .foreign_key(
                        ForeignKey::create()
                            .name(conventions::foreign_key_name(
                                "jobs",
                                &["stack_id"],
                                "stacks",
                                &["id"],
                            ))
                            .from("jobs", "stack_id")
                            .to("stacks", "id"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("jobs").if_exists().to_owned()).await?;

        Ok(())
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}
