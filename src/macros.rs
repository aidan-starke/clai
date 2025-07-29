/// Writes a line of text to the terminal.
///
/// This macro provides a convenient way to output formatted text to the terminal
/// using a consistent terminal instance.
///
/// # Examples
/// ```
/// write_line!("Hello, world!");
/// write_line!("Use /save <name> to save your session");
/// write_line!("User: {} said {}", name, message);
/// write_line!(""); // Blank line
/// ```
#[macro_export]
macro_rules! write_line {
    ($($arg:tt)*) => {
        {
            let text = format!($($arg)*);
            $crate::utils::TERM.get_or_init(|| console::Term::stdout())
                .write_line(&text)
                .unwrap();
        }
    };
}

/// Writes a line of text with blank lines before and after for visual spacing.
///
/// This macro is useful for important messages that need visual emphasis
/// through spacing, such as status updates, confirmations, or notifications.
///
/// # Output Format
/// ```text
/// [blank line]
/// [your text with styling]
/// [blank line]
/// ```
///
/// # Examples
/// ```
/// write_spaced!("✨ Session created successfully!");
/// write_spaced!("🔄 Switched to session: '{}'", session_name);
/// write_spaced!("🎭 Role set to: '{}'", role);
/// ```
///
/// # Use Cases
/// - Success messages that need emphasis
/// - State change notifications  
/// - Important status updates
/// - Farewell messages
#[macro_export]
macro_rules! write_spaced {
    ($($arg:tt)*) => {
        {
            let text = format!($($arg)*);
            $crate::utils::write_spaced_line(&text);
        }
    };
}

/// Handles database operations with consistent error handling and logging.
///
/// This macro wraps database operations to provide standardized error handling
/// for Diesel ORM operations, converting database errors into appropriate HTTP
/// status codes and logging errors for debugging.
///
/// # Parameters
/// - `$operation`: A string literal describing the operation for logging
/// - `$db_op`: The database operation expression that returns a `QueryResult<T>`
///
/// # Return Value
/// - On success: Returns the unwrapped value from the database operation
/// - On `NotFound` error: Returns `HTTP 404 NOT_FOUND` status code
/// - On other errors: Returns `HTTP 500 INTERNAL_SERVER_ERROR` status code
///
/// # Examples
/// ```
/// let session = handle_db_operation!("get session", db.get_session_by_id(id));
/// let message = handle_db_operation!("create message", db.create_message(session_id, "user", content));
/// let sessions = handle_db_operation!("list sessions", db.list_named_sessions());
/// ```
///
/// # Error Handling
/// The macro automatically:
/// - Logs errors with context using the `tracing` crate
/// - Maps `NotFound` errors to HTTP 404 status codes
/// - Maps all other database errors to HTTP 500 status codes
/// - Returns early from the function with the appropriate status code
///
/// # Usage Context
/// This macro should only be used in HTTP handler functions that return
/// `Result<T, axum::http::StatusCode>` as it performs early returns with status codes.
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
