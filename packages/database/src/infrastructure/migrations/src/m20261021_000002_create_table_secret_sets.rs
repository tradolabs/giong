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
                    .table("secret_sets")
                    .if_not_exists()
                    .col(ColumnDef::new("id").uuid().not_null().primary_key())
                    .col(ColumnDef::new("name").string())
                    .col(ColumnDef::new("type").string().not_null().default("shared"))
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
                            .name(conventions::unique_constraint_name("secret_sets", &["name"]))
                            .col("name")
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("secret_sets").if_exists().to_owned()).await
    }
    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}
