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
                    .col(ColumnDef::new("id").uuid().not_null().primary_key())
                    .col(ColumnDef::new("vault_id").uuid().not_null())
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
                    .index(
                        Index::create()
                            .name(conventions::unique_constraint_name(
                                "secrets",
                                &["vault_id", "key"],
                            ))
                            .col("vault_id")
                            .col("key")
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(conventions::foreign_key_name(
                                "secrets",
                                &["vault_id"],
                                "vaults",
                                &["id"],
                            ))
                            .from("secrets", "vault_id")
                            .to("vaults", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("secrets").to_owned()).await
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}
