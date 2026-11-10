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
                    .col(ColumnDef::new("secret_set_id").uuid())
                    .col(ColumnDef::new("inherited_secret_set_ids").json().not_null().default("[]"))
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
                    .foreign_key(
                        ForeignKey::create()
                            .from("stacks", "secret_set_id")
                            .to("secret_sets", "id")
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .index(
                        Index::create()
                            .name(conventions::unique_constraint_name("stacks", &["secret_set_id"]))
                            .col("secret_set_id")
                            .unique(),
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

        manager.get_connection().execute_unprepared(
            "CREATE TRIGGER stacks_cleanup_secrets AFTER DELETE ON stacks WHEN OLD.secret_set_id IS NOT NULL BEGIN DELETE FROM secret_sets WHERE id = OLD.secret_set_id AND type = 'local'; END"
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS stacks_cleanup_secrets")
            .await?;
        manager.drop_table(Table::drop().table("stacks").if_exists().to_owned()).await?;

        Ok(())
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}
