pub use sea_orm_migration::prelude::*;

mod m20250805_150840_create_sessions;
mod m20250805_150850_create_messages;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250805_150840_create_sessions::Migration),
            Box::new(m20250805_150850_create_messages::Migration),
        ]
    }
}
