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
                    .table("nodes")
                    .col(ColumnDef::new("id").uuid().not_null().primary_key())
                    .col(ColumnDef::new("flow_id").uuid().not_null())
                    .col(ColumnDef::new("name").string().not_null())
                    .col(ColumnDef::new("kind").string().not_null())
                    .col(ColumnDef::new("properties").json().not_null().default("{}"))
                    .col(ColumnDef::new("triggers").json().not_null().default("[]"))
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
                                "nodes",
                                &["flow_id", "name"],
                            ))
                            .col("flow_id")
                            .col("name")
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(conventions::foreign_key_name(
                                "nodes",
                                &["flow_id"],
                                "flows",
                                &["id"],
                            ))
                            .from("nodes", "flow_id")
                            .to("flows", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("nodes").to_owned()).await
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}
