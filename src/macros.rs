#[macro_export]
macro_rules! write_line {
    ($($arg:tt)*) => {
        {
            let text = format!($($arg)*);
            let styled_text = $crate::utils::auto_style_commands(&text);
            $crate::utils::TERM.get_or_init(|| console::Term::stdout())
                .write_line(&styled_text)
                .unwrap();
        }
    };
}

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