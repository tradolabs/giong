use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "
                UPDATE nodes
                SET kind = CASE kind
                    WHEN 'runners/flow' THEN 'flow'
                    WHEN 'runners/kubectl' THEN 'kubectl'
                END
                WHERE kind IN ('runners/flow', 'runners/kubectl')
                ",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "
                UPDATE nodes
                SET kind = CASE kind
                    WHEN 'flow' THEN 'runners/flow'
                    WHEN 'kubectl' THEN 'runners/kubectl'
                END
                WHERE kind IN ('flow', 'kubectl')
                ",
            )
            .await?;
        Ok(())
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(false)
    }
}
