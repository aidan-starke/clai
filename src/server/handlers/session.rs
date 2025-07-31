use crate::{db::ClaiDb, error::ClaiError, handle_db_operation, types::*, utils};
use axum::{
    extract::{Json, Path},
    response::Json as JsonResponse,
};
use tracing::{info, warn};

/// Creates a JSON response from a session model.
/// Takes a session from database models and returns Ok(JsonResponse(SessionResponse)).
macro_rules! json_response {
    ($session:expr) => {
        Ok(JsonResponse(SessionResponse {
            id: $session.id,
            name: $session.name,
            display_name: $session.display_name,
            role: $session.role,
            model: $session.model,
        }))
    };
}

// CREATE operations
pub async fn create_session(
    Json(payload): Json<CreateSessionRequest>,
) -> Result<JsonResponse<SessionResponse>, ClaiError> {
    info!("Creating new session with name: {}", payload.name);

    let session = handle_db_operation!(
        "create session",
        ClaiDb::create_session(&payload.name, payload.display_name.as_deref())
    );

    info!(
        "Created session with ID: {}, name: {}",
        session.id, session.name
    );

    utils::cleanup_old_sessions();

    json_response!(session)
}

// READ operations
pub async fn get_last_session() -> Result<JsonResponse<SessionResponse>, ClaiError> {
    info!("Getting last session");

    let session = handle_db_operation!("get last session", ClaiDb::get_last_session());

    // Update timestamp to mark as recently accessed
    if let Err(e) = ClaiDb::update_session_timestamp(session.id) {
        warn!("Failed to update session timestamp: {}", e);
    }

    info!(
        "Found last session: ID {}, name: {}",
        session.id, session.name
    );

    json_response!(session)
}

pub async fn get_session_by_name(
    Path(name): Path<String>,
) -> Result<JsonResponse<SessionResponse>, ClaiError> {
    info!("Getting session by name: {}", name);

    let session = handle_db_operation!("get session by name", ClaiDb::get_session_by_name(&name));

    info!("Found session: {} (ID: {})", name, session.id);

    // Update timestamp to mark as recently accessed
    if let Err(e) = ClaiDb::update_session_timestamp(session.id) {
        warn!("Failed to update session timestamp: {}", e);
    }

    json_response!(session)
}

pub async fn list_sessions() -> Result<JsonResponse<Vec<SessionResponse>>, ClaiError> {
    info!("Listing named sessions");

    let sessions = handle_db_operation!("list sessions", ClaiDb::list_named_sessions());

    let response: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|session| SessionResponse {
            id: session.id,
            name: session.name,
            display_name: session.display_name,
            role: session.role,
            model: session.model,
        })
        .collect();

    info!("Found {} named sessions", response.len());

    Ok(JsonResponse(response))
}

pub async fn get_session_by_id(
    Path(session_id): Path<i32>,
) -> Result<JsonResponse<SessionResponse>, ClaiError> {
    info!("Getting session by ID: {}", session_id);

    let session = handle_db_operation!("get session by id", ClaiDb::get_session_by_id(session_id));

    info!("Found session: ID {}, name: {}", session_id, session.name);

    json_response!(session)
}

// UPDATE operations
pub async fn save_session(
    Path(session_id): Path<i32>,
    Json(payload): Json<SaveSessionRequest>,
) -> Result<JsonResponse<SessionResponse>, ClaiError> {
    info!(
        "Saving session {} with display name: {}",
        session_id, payload.display_name
    );

    let session = handle_db_operation!(
        "save session",
        ClaiDb::update_session_display_name(session_id, &payload.display_name)
    );

    info!("Saved session {} as '{}'", session_id, payload.display_name);

    json_response!(session)
}

pub async fn set_role(
    Path(session_id): Path<i32>,
    Json(payload): Json<SetRoleRequest>,
) -> Result<JsonResponse<SessionResponse>, ClaiError> {
    info!(
        "Setting role for session {} to: {:?}",
        session_id, payload.role
    );

    let session = handle_db_operation!(
        "set session role",
        ClaiDb::update_session_role(session_id, payload.role.as_deref())
    );

    info!("Updated session {} role to: {:?}", session_id, payload.role);

    json_response!(session)
}

pub async fn set_model(
    Path(session_id): Path<i32>,
    Json(payload): Json<SetModelRequest>,
) -> Result<JsonResponse<SessionResponse>, ClaiError> {
    info!(
        "Setting model for session {} to: {}",
        session_id, payload.model
    );

    let session = handle_db_operation!(
        "set session model",
        ClaiDb::update_session_model(session_id, &payload.model)
    );

    info!("Updated session {} model to: {}", session_id, payload.model);

    json_response!(session)
}

// DELETE operations
pub async fn delete_session(Path(name): Path<String>) -> Result<JsonResponse<()>, ClaiError> {
    info!("Deleting session by name: {}", name);

    handle_db_operation!("delete session", ClaiDb::delete_session_by_name(&name));

    info!("Successfully deleted session '{}'", name);

    Ok(JsonResponse(()))
}
