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
            $crate::utils::get_term()
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
/// for Diesel ORM operations, converting database errors into ClaiError types
/// and logging errors for debugging.
///
/// # Parameters
/// - `$operation`: A string literal describing the operation for logging
/// - `$db_op`: The database operation expression that returns a `QueryResult<T>`
///
/// # Return Value
/// - On success: Returns the unwrapped value from the database operation
/// - On error: Returns a `ClaiError::Database` variant with proper logging
///
/// # Examples
/// ```
/// let session = handle_db_operation!("get session", ClaiDb::get_session_by_id(id))?;
/// let message = handle_db_operation!("create message", ClaiDb::create_message(session_id, "user", content))?;
/// let sessions = handle_db_operation!("list sessions", ClaiDb::list_named_sessions())?;
/// ```
///
/// # Error Handling
/// The macro automatically:
/// - Logs errors with context using the `tracing` crate
/// - Wraps diesel errors in ClaiError::Database for consistent error handling
/// - Returns the error for the caller to handle appropriately
///
/// # Usage Context
/// This macro should be used in functions that return `crate::error::Result<T>`
/// and the result should be handled with the `?` operator.
#[macro_export]
macro_rules! handle_db_operation {
    ($operation:literal, $db_op:expr) => {
        match $db_op {
            Ok(value) => value,
            Err(e) => {
                // Log the operation context if it's a database error
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
