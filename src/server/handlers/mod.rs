pub mod chat;
pub mod models;
pub mod session;

use crate::error::ClaiError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub async fn health() -> StatusCode {
    StatusCode::OK
}

// Convert ClaiError to axum Response for HTTP handlers
impl IntoResponse for ClaiError {
    fn into_response(self) -> Response {
        let error_message = self.to_string();
        tracing::error!("Handler error: {}", error_message);

        let status_code: StatusCode = self.into();
        (status_code, error_message).into_response()
    }
}
