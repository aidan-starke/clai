mod common;

use std::sync::OnceLock;

use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use clai::{db::ClaiDb, server::handlers, types::*};
use serde_json::json;
use tempfile::NamedTempFile;

static INIT: OnceLock<NamedTempFile> = OnceLock::new();

async fn create_test_server() -> TestServer {
    let temp_file = INIT.get_or_init(|| NamedTempFile::new().expect("Failed to create temp file"));
    let database_url = format!("sqlite://{}", temp_file.path().to_string_lossy());
    let _ = ClaiDb::init_with_url(&database_url).await;

    let app = Router::new()
        .route("/health", axum::routing::get(handlers::health))
        .route(
            "/sessions",
            axum::routing::post(handlers::session::create_session),
        )
        .route(
            "/sessions",
            axum::routing::get(handlers::session::list_sessions),
        )
        .route(
            "/sessions/last",
            axum::routing::get(handlers::session::get_last_session),
        )
        .route(
            "/sessions/by-name/{name}",
            axum::routing::get(handlers::session::get_session_by_name)
                .delete(handlers::session::delete_session),
        )
        .route(
            "/sessions/{id}",
            axum::routing::get(handlers::session::get_session_by_id)
                .patch(handlers::session::save_session),
        )
        .route(
            "/sessions/{id}/role",
            axum::routing::put(handlers::session::set_role),
        )
        .route(
            "/sessions/{id}/model",
            axum::routing::put(handlers::session::set_model),
        )
        .route(
            "/sessions/{id}/chat",
            axum::routing::post(handlers::chat::chat),
        )
        .route("/models", axum::routing::get(handlers::models::get_models));

    TestServer::new(app).expect("Failed to create test server")
}

#[tokio::test]
async fn test_health_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_session_operations() {
    let server = create_test_server().await;

    // Test 1: Create session with display name
    let payload1 = json!({
        "name": "test_session",
        "display_name": "TestSession"
    });

    let response1 = server.post("/sessions").json(&payload1).await;
    assert_eq!(response1.status_code(), StatusCode::OK);

    let session1: SessionResponse = response1.json();
    assert_eq!(session1.name, "test_session");
    assert_eq!(session1.display_name, Some("TestSession".to_string()));
    assert!(session1.id > 0);

    // Test 2: Create session with display name
    let payload2 = json!({
        "name": "anonymous_session",
        "display_name": "Anonymous Session"
    });

    let response2 = server.post("/sessions").json(&payload2).await;
    assert_eq!(response2.status_code(), StatusCode::OK);

    let session2: SessionResponse = response2.json();
    assert_eq!(session2.name, "anonymous_session");
    assert_eq!(session2.display_name, Some("Anonymous Session".to_string()));
    assert!(session2.id > 0);
    assert!(session2.id != session1.id);

    // Test 3: Get last session
    let response3 = server.get("/sessions/last").await;
    assert_eq!(response3.status_code(), StatusCode::OK);

    let last_session: SessionResponse = response3.json();
    // Should be the most recently created session
    assert_eq!(last_session.id, session2.id);

    // Test 4: List sessions
    let response4 = server.get("/sessions").await;
    assert_eq!(response4.status_code(), StatusCode::OK);

    let sessions: Vec<SessionResponse> = response4.json();
    assert_eq!(sessions.len(), 2);

    // Test 5: Get session by ID
    let response5 = server.get(&format!("/sessions/{}", session1.id)).await;
    assert_eq!(response5.status_code(), StatusCode::OK);

    let fetched_session: SessionResponse = response5.json();
    assert_eq!(fetched_session.id, session1.id);
    assert_eq!(fetched_session.name, "test_session");

    // Test 6: Save session (rename)
    let save_payload = json!({
        "display_name": "Renamed Session"
    });

    let response6 = server
        .patch(&format!("/sessions/{}", session1.id))
        .json(&save_payload)
        .await;
    assert_eq!(response6.status_code(), StatusCode::OK);

    let saved_session: SessionResponse = response6.json();
    assert_eq!(
        saved_session.display_name,
        Some("Renamed Session".to_string())
    );

    // Test 7: Set session role
    let role_payload = json!({
        "role": "helpful assistant"
    });

    let response7 = server
        .put(&format!("/sessions/{}/role", session1.id))
        .json(&role_payload)
        .await;
    assert_eq!(response7.status_code(), StatusCode::OK);

    let role_session: SessionResponse = response7.json();
    assert_eq!(role_session.role, Some("helpful assistant".to_string()));

    // Test 8: Set session model
    let model_payload = json!({
        "model": "claude-3-sonnet"
    });

    let response8 = server
        .put(&format!("/sessions/{}/model", session1.id))
        .json(&model_payload)
        .await;
    assert_eq!(response8.status_code(), StatusCode::OK);

    let model_session: SessionResponse = response8.json();
    assert_eq!(model_session.model, Some("claude-3-sonnet".to_string()));

    // Test 9: Get session by display name (use the renamed display name)
    let response9 = server.get("/sessions/by-name/Renamed%20Session").await;
    assert_eq!(response9.status_code(), StatusCode::OK);

    // Test 10: Get nonexistent session should return 404
    let response10 = server.get("/sessions/99999").await;
    assert_eq!(response10.status_code(), StatusCode::NOT_FOUND);

    // Test 11: Get nonexistent session by name should return 404
    let response11 = server.get("/sessions/by-name/nonexistent").await;
    assert_eq!(response11.status_code(), StatusCode::NOT_FOUND);

    // Test 12: Delete session by name
    let response12 = server.delete("/sessions/by-name/Renamed%20Session").await;
    assert_eq!(response12.status_code(), StatusCode::OK);

    // Test 13: Verify session was deleted
    let response13 = server.get(&format!("/sessions/{}", session1.id)).await;
    assert_eq!(response13.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_error_cases() {
    let server = create_test_server().await;

    // Test invalid JSON payload
    let response1 = server.post("/sessions").text("invalid json").await;
    assert_eq!(response1.status_code(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

    // Test missing required field
    let payload2 = json!({
        "display_name": "No Name Session"
    });
    let response2 = server.post("/sessions").json(&payload2).await;
    assert_eq!(response2.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}
