pub use sea_orm_migration::prelude::*;

pub mod conventions;
mod m20261120_000001_create_vaults;
mod m20261120_000002_create_flows;
mod m20261120_000003_create_nodes;
mod m20261120_000004_create_secrets;
mod m20261120_000005_create_users;
mod m20261120_000006_create_plugins;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261120_000001_create_vaults::Migration),
            Box::new(m20261120_000002_create_flows::Migration),
            Box::new(m20261120_000003_create_nodes::Migration),
            Box::new(m20261120_000004_create_secrets::Migration),
            Box::new(m20261120_000005_create_users::Migration),
            Box::new(m20261120_000006_create_plugins::Migration),
        ]
    }
}
