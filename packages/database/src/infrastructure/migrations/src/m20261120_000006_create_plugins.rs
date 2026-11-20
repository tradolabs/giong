use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("plugins")
                    .col(ColumnDef::new("kind").text().not_null().primary_key())
                    .col(ColumnDef::new("image").text().not_null())
                    .col(ColumnDef::new("image_id").text().not_null())
                    .col(ColumnDef::new("name").text().not_null())
                    .col(ColumnDef::new("version").text().not_null())
                    .col(ColumnDef::new("description").text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("plugins").to_owned()).await
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(true)
    }
}
