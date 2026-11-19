use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE nodes
                SET triggers = (
                    SELECT json_group_array(json(trigger))
                    FROM (
                        SELECT CASE
                            WHEN entry.type = 'object'
                                 AND json_extract(entry.value, '$.type') = 'event'
                            THEN json_remove(entry.value, '$.event')
                            ELSE nodes.triggers -> entry.key
                        END AS trigger
                        FROM json_each(nodes.triggers) AS entry
                        ORDER BY entry.key
                    )
                )
                WHERE EXISTS (
                    SELECT 1
                    FROM json_each(nodes.triggers) AS entry
                    WHERE CASE WHEN entry.type = 'object' THEN
                        json_extract(entry.value, '$.type') = 'event'
                        AND json_type(entry.value, '$.event') IS NOT NULL
                    END
                )
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Older versions also accept the normalized trigger shape.
        Ok(())
    }

    fn use_transaction(&self) -> Option<bool> {
        Some(false)
    }
}
