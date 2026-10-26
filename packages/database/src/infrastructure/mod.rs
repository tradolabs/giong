pub mod database;
mod error;
pub mod models;
pub mod repositories;

pub use migrations;

pub(crate) async fn connect(
    database_url: &str,
) -> Result<crate::database::Database, sea_orm::DbErr> {
    let database = database::database::Database::open(database_url).await?;
    Ok(std::sync::Arc::new(database).into_database())
}
