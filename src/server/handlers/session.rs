use crate::db::ClaiDb;
use crate::server::utils;
use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::Json as JsonResponse,
};
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
}

pub async fn create_session(Json(payload): Json<CreateSessionRequest>) -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Creating new session with name: {}", payload.name);

    let mut db = ClaiDb::new();

    match db.create_session(&payload.name, payload.display_name.as_deref()) {
        Ok(session) => {
            info!("Created session with ID: {}, name: {}", session.id, session.name);

            utils::cleanup_old_sessions();

            let response = SessionResponse {
                id: session.id,
                name: session.name,
                display_name: session.display_name,
            };
            Ok(JsonResponse(response))
        }
        Err(e) => {
            tracing::error!("Failed to create session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_last_session() -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Getting last session");

    let mut db = ClaiDb::new();

    match db.get_last_session() {
        Ok(session) => {
            info!("Found last session: ID {}, name: {}", session.id, session.name);

            // Update timestamp to mark as recently accessed
            if let Err(e) = db.update_session_timestamp(session.id) {
                tracing::warn!("Failed to update session timestamp: {}", e);
            }

            let response = SessionResponse {
                id: session.id,
                name: session.name,
                display_name: session.display_name,
            };
            Ok(JsonResponse(response))
        }
        Err(DieselError::NotFound) => {
            info!("No sessions found");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to get last session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct SaveSessionRequest {
    pub display_name: String,
}

pub async fn save_session(
    Path(session_id): Path<i32>,
    Json(payload): Json<SaveSessionRequest>,
) -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Saving session {} with display name: {}", session_id, payload.display_name);

    let mut db = ClaiDb::new();

    match db.update_session_display_name(session_id, &payload.display_name) {
        Ok(session) => {
            info!("Saved session {} as '{}'", session_id, payload.display_name);
            let response = SessionResponse {
                id: session.id,
                name: session.name,
                display_name: session.display_name,
            };
            Ok(JsonResponse(response))
        }
        Err(e) => {
            tracing::error!("Failed to save session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_sessions() -> Result<JsonResponse<Vec<SessionResponse>>, StatusCode> {
    info!("Listing named sessions");

    let mut db = ClaiDb::new();

    match db.list_named_sessions() {
        Ok(sessions) => {
            let response: Vec<SessionResponse> = sessions
                .into_iter()
                .map(|session| SessionResponse {
                    id: session.id,
                    name: session.name,
                    display_name: session.display_name,
                })
                .collect();

            info!("Found {} named sessions", response.len());
            Ok(JsonResponse(response))
        }
        Err(e) => {
            tracing::error!("Failed to list sessions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_session_by_name(Path(name): Path<String>) -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Getting session by name: {}", name);

    let mut db = ClaiDb::new();

    match db.get_session_by_name(&name) {
        Ok(session) => {
            info!("Found session: {} (ID: {})", name, session.id);

            // Update timestamp to mark as recently accessed
            if let Err(e) = db.update_session_timestamp(session.id) {
                tracing::warn!("Failed to update session timestamp: {}", e);
            }

            let response = SessionResponse {
                id: session.id,
                name: session.name,
                display_name: session.display_name,
            };
            Ok(JsonResponse(response))
        }
        Err(DieselError::NotFound) => {
            info!("Session '{}' not found", name);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to get session by name: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_session(Path(name): Path<String>) -> Result<JsonResponse<()>, StatusCode> {
    info!("Deleting session by name: {}", name);

    let mut db = ClaiDb::new();

    match db.delete_session_by_name(&name) {
        Ok(_) => {
            info!("Successfully deleted session '{}'", name);
            Ok(JsonResponse(()))
        }
        Err(DieselError::NotFound) => {
            info!("Session '{}' not found", name);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to delete session '{}': {}", name, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
