use clai::db::ClaiDb;
use std::env;
use tempfile::NamedTempFile;

pub fn setup_test_db() -> NamedTempFile {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    env::set_var("DATABASE_URL", temp_file.path().to_str().unwrap());
    env::set_var("ANTHROPIC_API_KEY", "sk-test-key-for-tests");

    ClaiDb::init().expect("Failed to initialize test database");

    temp_file
}
