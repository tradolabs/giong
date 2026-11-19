pub use sea_orm_migration::prelude::*;

pub mod conventions;
mod m20261112_000001_initial;
mod m20261119_000002_remove_trigger_event_name;
mod m20261119_000003_flatten_builtin_plugin_kinds;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261112_000001_initial::Migration),
            Box::new(m20261119_000002_remove_trigger_event_name::Migration),
            Box::new(m20261119_000003_flatten_builtin_plugin_kinds::Migration),
        ]
    }
}
