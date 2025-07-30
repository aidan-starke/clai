use thiserror::Error;

/// Main error type for CLAI application
#[derive(Error, Debug)]
pub enum ClaiError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

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

}

impl ClaiError {
    /// Create a new server error
    pub fn server<S: Into<String>>(message: S) -> Self {
        ClaiError::Server {
            message: message.into(),
        }
    }

    /// Create a new session error
    pub fn session<S: Into<String>>(message: S) -> Self {
        ClaiError::Session {
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        ClaiError::Config {
            message: message.into(),
        }
    }

}

/// Convenient Result type alias
pub type Result<T> = std::result::Result<T, ClaiError>;

/// Convert ClaiError to HTTP status codes for server responses
impl From<ClaiError> for axum::http::StatusCode {
    fn from(error: ClaiError) -> Self {
        match error {
            ClaiError::Database(diesel::result::Error::NotFound) => {
                axum::http::StatusCode::NOT_FOUND
            }
            ClaiError::Database(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ClaiError::Network(_) => axum::http::StatusCode::BAD_GATEWAY,
            ClaiError::Server { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ClaiError::Session { .. } => axum::http::StatusCode::BAD_REQUEST,
            ClaiError::Terminal(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ClaiError::Config { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ClaiError::Serialization(_) => axum::http::StatusCode::BAD_REQUEST,
        }
    }
}

