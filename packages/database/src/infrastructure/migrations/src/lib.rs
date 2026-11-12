pub use sea_orm_migration::prelude::*;

pub mod conventions;
mod m20261112_000001_initial;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20261112_000001_initial::Migration)]
    }
}
