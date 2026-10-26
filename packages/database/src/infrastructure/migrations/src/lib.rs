pub use sea_orm_migration::prelude::*;

pub mod conventions;

mod m20261021_000000_initialize_database;
mod m20261021_000001_create_table_repositories;
mod m20261021_000002_create_table_stacks;
mod m20261021_000003_create_table_jobs;
mod m20261024_000004_create_table_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261021_000000_initialize_database::Migration),
            Box::new(m20261021_000001_create_table_repositories::Migration),
            Box::new(m20261021_000002_create_table_stacks::Migration),
            Box::new(m20261021_000003_create_table_jobs::Migration),
            Box::new(m20261024_000004_create_table_users::Migration),
        ]
    }
}
