use thiserror::Error;

/// Main error type for CLAI application
#[derive(Error, Debug)]
pub enum ClaiError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Network/HTTP client errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Server/API errors
    #[error("Server error: {message}")]
    Server { message: String },

    /// Session management errors
    #[error("Session error: {message}")]
    Session { message: String },

    /// Terminal/IO errors
    #[error("Terminal I/O error: {0}")]
    Terminal(#[from] std::io::Error),

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// MCP protocol errors
    #[error("MCP protocol error: {0}")]
    McpProtocol(#[from] rmcp::ServiceError),
}

impl ClaiError {
    pub fn server<S: Into<String>>(message: S) -> Self {
        ClaiError::Server {
            message: message.into(),
        }
    }

    pub fn session<S: Into<String>>(message: S) -> Self {
        ClaiError::Session {
            message: message.into(),
        }
    }

    pub fn config<S: Into<String>>(message: S) -> Self {
        ClaiError::Config {
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, ClaiError>;

impl From<ClaiError> for axum::http::StatusCode {
    fn from(error: ClaiError) -> Self {
        match error {
            ClaiError::Database(sea_orm::DbErr::RecordNotFound(_)) => {
                axum::http::StatusCode::NOT_FOUND
            }
            ClaiError::Network(_) => axum::http::StatusCode::BAD_GATEWAY,
            ClaiError::Session { .. } => axum::http::StatusCode::BAD_REQUEST,
            ClaiError::Serialization(_) => axum::http::StatusCode::BAD_REQUEST,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for ClaiError {
    fn into_response(self) -> axum::response::Response {
        let error_message = self.to_string();
        tracing::error!("Handler error: {}", error_message);

        let status_code: axum::http::StatusCode = self.into();
        (status_code, error_message).into_response()
    }
}
