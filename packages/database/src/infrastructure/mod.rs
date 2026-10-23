pub mod database;
pub mod models;
pub mod repositories;

pub use migrations;

pub(crate) async fn connect(database_url: &str) -> anyhow::Result<crate::Database> {
    Ok(std::rc::Rc::new(database::Database::open(database_url).await?).into_database())
}
