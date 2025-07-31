/// Writes a line of text to the terminal
#[macro_export]
macro_rules! write_line {
    ($($arg:tt)*) => {
        {
            let text = format!($($arg)*);
            if let Err(e) = $crate::utils::get_term().write_line(&text) {
                eprintln!("Warning: Failed to write to terminal: {}", e);
            }
        }
    };
}

/// Writes a line of text with blank lines before and after for visual spacing
#[macro_export]
macro_rules! write_spaced {
    ($($arg:tt)*) => {
        {
            let text = format!($($arg)*);
            $crate::utils::write_spaced_line(&text);
        }
    };
}

/// Handles database operations with consistent error handling and logging
#[macro_export]
macro_rules! handle_db_operation {
    ($operation:literal, $db_op:expr) => {
        match $db_op {
            Ok(value) => value,
            Err(e) => {
                if let $crate::error::ClaiError::Database(ref diesel_err) = e {
                    tracing::error!("Database error during {}: {}", $operation, diesel_err);
                } else {
                    tracing::error!("Error during {}: {}", $operation, e);
                }
                return Err(e);
            }
        }
    };
}
