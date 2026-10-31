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
                    .table("secrets")
                    .if_not_exists()
                    .col(ColumnDef::new("id").uuid().not_null().primary_key())
                    .col(ColumnDef::new("stack_id").uuid().not_null())
                    .col(ColumnDef::new("key").string().not_null())
                    .col(ColumnDef::new("value").text().not_null())
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
                                "secrets",
                                &["stack_id"],
                                "stacks",
                                &["id"],
                            ))
                            .from("secrets", "stack_id")
                            .to("stacks", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name(conventions::unique_constraint_name(
                                "secrets",
                                &["stack_id", "key"],
                            ))
                            .col("stack_id")
                            .col("key")
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("secrets").if_exists().to_owned()).await
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}
