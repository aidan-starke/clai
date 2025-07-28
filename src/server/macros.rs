#[macro_export]
macro_rules! handle_db_operation {
    ($operation:literal, $db_op:expr) => {
        match $db_op {
            Ok(value) => value,
            Err(diesel::result::Error::NotFound) => {
                tracing::error!("Not found during {}", $operation);
                return Err(axum::http::StatusCode::NOT_FOUND);
            }
            Err(e) => {
                tracing::error!("Database error during {}: {}", $operation, e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };
}
