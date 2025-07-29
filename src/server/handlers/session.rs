use crate::db::ClaiDb;
use crate::handle_db_operation;
use crate::server::utils;
use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::Json as JsonResponse,
};
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
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct SaveSessionRequest {
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct SetRoleRequest {
    pub role: Option<String>,
}

// CREATE operations
pub async fn create_session(Json(payload): Json<CreateSessionRequest>) -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Creating new session with name: {}", payload.name);

    let mut db = ClaiDb::new();

    let session = handle_db_operation!("create session", db.create_session(&payload.name, payload.display_name.as_deref()));

    info!("Created session with ID: {}, name: {}", session.id, session.name);

    utils::cleanup_old_sessions();

    let response = SessionResponse {
        id: session.id,
        name: session.name,
        display_name: session.display_name,
        role: session.role,
    };

    Ok(JsonResponse(response))
}

// READ operations
pub async fn get_last_session() -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Getting last session");

    let mut db = ClaiDb::new();

    let session = handle_db_operation!("get last session", db.get_last_session());

    info!("Found last session: ID {}, name: {}", session.id, session.name);

    // Update timestamp to mark as recently accessed
    if let Err(e) = db.update_session_timestamp(session.id) {
        tracing::warn!("Failed to update session timestamp: {}", e);
    }

    let response = SessionResponse {
        id: session.id,
        name: session.name,
        display_name: session.display_name,
        role: session.role,
    };

    Ok(JsonResponse(response))
}

pub async fn get_session_by_name(Path(name): Path<String>) -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Getting session by name: {}", name);

    let mut db = ClaiDb::new();

    let session = handle_db_operation!("get session by name", db.get_session_by_name(&name));

    info!("Found session: {} (ID: {})", name, session.id);

    // Update timestamp to mark as recently accessed
    if let Err(e) = db.update_session_timestamp(session.id) {
        tracing::warn!("Failed to update session timestamp: {}", e);
    }

    let response = SessionResponse {
        id: session.id,
        name: session.name,
        display_name: session.display_name,
        role: session.role,
    };

    Ok(JsonResponse(response))
}

pub async fn list_sessions() -> Result<JsonResponse<Vec<SessionResponse>>, StatusCode> {
    info!("Listing named sessions");

    let mut db = ClaiDb::new();

    let sessions = handle_db_operation!("list sessions", db.list_named_sessions());

    let response: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|session| SessionResponse {
            id: session.id,
            name: session.name,
            display_name: session.display_name,
            role: session.role,
        })
        .collect();

    info!("Found {} named sessions", response.len());

    Ok(JsonResponse(response))
}

pub async fn get_session_by_id(Path(session_id): Path<i32>) -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Getting session by ID: {}", session_id);

    let mut db = ClaiDb::new();

    let session = handle_db_operation!("get session by id", db.get_session_by_id(session_id));

    info!("Found session: ID {}, name: {}", session_id, session.name);

    let response = SessionResponse {
        id: session.id,
        name: session.name,
        display_name: session.display_name,
        role: session.role,
    };

    Ok(JsonResponse(response))
}

// UPDATE operations
pub async fn save_session(
    Path(session_id): Path<i32>,
    Json(payload): Json<SaveSessionRequest>,
) -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Saving session {} with display name: {}", session_id, payload.display_name);

    let mut db = ClaiDb::new();

    let session = handle_db_operation!("save session", db.update_session_display_name(session_id, &payload.display_name));

    info!("Saved session {} as '{}'", session_id, payload.display_name);

    let response = SessionResponse {
        id: session.id,
        name: session.name,
        display_name: session.display_name,
        role: session.role,
    };

    Ok(JsonResponse(response))
}

pub async fn set_role(
    Path(session_id): Path<i32>,
    Json(payload): Json<SetRoleRequest>,
) -> Result<JsonResponse<SessionResponse>, StatusCode> {
    info!("Setting role for session {} to: {:?}", session_id, payload.role);

    let mut db = ClaiDb::new();

    let session = handle_db_operation!("set session role", db.update_session_role(session_id, payload.role.as_deref()));

    info!("Updated session {} role to: {:?}", session_id, payload.role);

    let response = SessionResponse {
        id: session.id,
        name: session.name,
        display_name: session.display_name,
        role: session.role,
    };

    Ok(JsonResponse(response))
}

// DELETE operations
pub async fn delete_session(Path(name): Path<String>) -> Result<JsonResponse<()>, StatusCode> {
    info!("Deleting session by name: {}", name);

    let mut db = ClaiDb::new();

    handle_db_operation!("delete session", db.delete_session_by_name(&name));

    info!("Successfully deleted session '{}'", name);

    Ok(JsonResponse(()))
}
