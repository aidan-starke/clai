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
        let status_code: StatusCode = self.into();
        status_code.into_response()
    }
}
