/// Handles database operations with consistent error handling and logging
#[macro_export]
macro_rules! handle_db_operation {
    ($operation:literal, $db_op:expr) => {
        match $db_op {
            Ok(value) => value,
            Err(e) => {
                if let common::error::ClaiError::Database(ref db_err) = e {
                    tracing::error!("Database error during {}: {}", $operation, db_err);
                } else {
                    tracing::error!("Error during {}: {}", $operation, e);
                }
                return Err(e);
            }
        }
    };
}
