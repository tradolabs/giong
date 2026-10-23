use anyhow::Result;
use sea_orm::{
    DatabaseConnection, DbBackend, SqlxSqliteConnector,
    sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{str::FromStr, sync::Arc, time::Duration};

pub(super) async fn connect_sqlite(database_url: &str) -> Result<DatabaseConnection> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    Ok(SqlxSqliteConnector::from_sqlx_sqlite_pool(pool))
}

pub(super) async fn connect_d1(database_url: &str) -> Result<DatabaseConnection> {
    let driver = super::drivers::d1::D1::new(database_url)?;
    Ok(sea_orm::Database::connect_proxy(DbBackend::Sqlite, Arc::new(Box::new(driver))).await?)
}
