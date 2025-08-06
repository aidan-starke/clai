use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tempfile::NamedTempFile;

#[macro_export]
macro_rules! test_with {
    ($test_name:ident, $get_item:expr, |$item:ident| $test_body:block) => {
        #[tokio::test]
        async fn $test_name() {
            let $item = $get_item;
            $test_body
        }
    };
}

#[async_trait]
pub trait Get {
    async fn get() -> &'static Self;
}

pub struct TestDb {
    temp_file: NamedTempFile,
    connection: DatabaseConnection,
}

impl TestDb {
    pub async fn new() -> Self {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let database_url = format!("sqlite://{}", temp_file.path().to_string_lossy());

        let connection = Database::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        Migrator::up(&connection, None)
            .await
            .expect("Failed to run migrations");

        Self {
            temp_file,
            connection,
        }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub fn database_url(&self) -> String {
        format!("sqlite://{}", self.temp_file.path().to_string_lossy())
    }
}
