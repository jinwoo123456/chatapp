pub use sea_orm_migration::prelude::*;

mod m2025_09_15_000001_init;
mod m2025_09_15_000002_recreate_users;
mod m2025_09_15_000003_add_profile_fields;
mod m2025_09_16_000004_room_read;

#[derive(DeriveMigrationName)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m2025_09_15_000001_init::Migration),
            Box::new(m2025_09_15_000002_recreate_users::Migration),
            Box::new(m2025_09_15_000003_add_profile_fields::Migration),
            Box::new(m2025_09_16_000004_room_read::Migration),
        ]
    }
}
