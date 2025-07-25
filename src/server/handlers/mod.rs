pub mod chat;
pub mod session;

use axum::http::StatusCode;

pub async fn health() -> StatusCode {
    StatusCode::OK
}
