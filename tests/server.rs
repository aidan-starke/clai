pub mod utils;

use async_trait::async_trait;
use axum::{http::StatusCode, Router};
use axum_test::TestServer;
use common::types::*;
use serde_json::json;
use server::{db::ClaiDb, handlers};
use tokio::sync::OnceCell;
use utils::{Get, TestDb};

static TEST_DB: OnceCell<TestDb> = OnceCell::const_new();

#[async_trait]
impl Get for TestDb {
    async fn get() -> &'static TestDb {
        TEST_DB.get_or_init(|| async { TestDb::new().await }).await
    }
}

async fn create_test_server() -> TestServer {
    let test_db = TestDb::get().await;
    let _ = ClaiDb::init_with_url(&test_db.database_url()).await;

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

test_with!(test_health_endpoint, create_test_server().await, |server| {
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);
});

test_with!(
    test_create_session_with_display_name,
    create_test_server().await,
    |server| {
        let payload = json!({
            "name": "test_session",
            "display_name": "Test Session"
        });

        let response = server.post("/sessions").json(&payload).await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let session: SessionResponse = response.json();
        assert_eq!(session.name, "test_session");
        assert_eq!(session.display_name, Some("Test Session".to_string()));
        assert!(session.id > 0);
    }
);

test_with!(
    test_create_session_without_display_name,
    create_test_server().await,
    |server| {
        let payload = json!({
            "name": "anonymous_session"
        });

        let response = server.post("/sessions").json(&payload).await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let session: SessionResponse = response.json();
        assert_eq!(session.name, "anonymous_session");
        assert_eq!(session.display_name, None);
        assert!(session.id > 0);
    }
);

test_with!(
    test_get_session_by_id,
    create_test_server().await,
    |server| {
        // Create a session first
        let payload = json!({
            "name": "test_session",
            "display_name": "Test Session"
        });

        let create_response = server.post("/sessions").json(&payload).await;
        let created_session: SessionResponse = create_response.json();

        // Get the session by ID
        let response = server
            .get(&format!("/sessions/{}", created_session.id))
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let fetched_session: SessionResponse = response.json();
        assert_eq!(fetched_session.id, created_session.id);
        assert_eq!(fetched_session.name, "test_session");
        assert_eq!(
            fetched_session.display_name,
            Some("Test Session".to_string())
        );
    }
);

test_with!(
    test_get_nonexistent_session_by_id,
    create_test_server().await,
    |server| {
        let response = server.get("/sessions/99999").await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
);

test_with!(
    test_get_last_session,
    create_test_server().await,
    |server| {
        // Create two sessions
        let payload1 = json!({
            "name": "session1",
            "display_name": "First Session"
        });

        let payload2 = json!({
            "name": "session2",
            "display_name": "Second Session"
        });

        let response1 = server.post("/sessions").json(&payload1).await;
        let _session1: SessionResponse = response1.json();

        let response2 = server.post("/sessions").json(&payload2).await;
        let _session2: SessionResponse = response2.json();

        // Get the last session (should be session2 since it was created after session1)
        let response = server.get("/sessions/last").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let last_session: SessionResponse = response.json();
        assert!(last_session.id > 0);
        assert!(last_session.display_name.is_some());
    }
);

test_with!(test_list_sessions, create_test_server().await, |server| {
    // Create two named sessions
    let payload1 = json!({
        "name": "session1",
        "display_name": "Named Session 1"
    });

    let payload2 = json!({
        "name": "session2",
        "display_name": "Named Session 2"
    });

    server.post("/sessions").json(&payload1).await;
    server.post("/sessions").json(&payload2).await;

    // List sessions (only returns named sessions)
    let response = server.get("/sessions").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let sessions: Vec<SessionResponse> = response.json();
    // Should have at least our 2 sessions (may have more from other tests)
    assert!(sessions.len() >= 2);
    assert!(sessions.iter().all(|s| s.display_name.is_some()));

    // Verify our specific sessions are present
    let our_session_names: Vec<&str> = sessions
        .iter()
        .filter_map(|s| s.display_name.as_deref())
        .collect();
    assert!(our_session_names.contains(&"Named Session 1"));
    assert!(our_session_names.contains(&"Named Session 2"));
});

test_with!(test_save_session, create_test_server().await, |server| {
    // Create a session
    let payload = json!({
        "name": "test_session",
        "display_name": "Original Name"
    });

    let create_response = server.post("/sessions").json(&payload).await;
    let session: SessionResponse = create_response.json();

    // Update the display name
    let save_payload = json!({
        "display_name": "Updated Name"
    });

    let response = server
        .patch(&format!("/sessions/{}", session.id))
        .json(&save_payload)
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let updated_session: SessionResponse = response.json();
    assert_eq!(
        updated_session.display_name,
        Some("Updated Name".to_string())
    );
    assert_eq!(updated_session.id, session.id);
});

test_with!(
    test_set_session_role,
    create_test_server().await,
    |server| {
        // Create a session
        let payload = json!({
            "name": "test_session",
            "display_name": "Test Session"
        });

        let create_response = server.post("/sessions").json(&payload).await;
        let session: SessionResponse = create_response.json();

        // Set the role
        let role_payload = json!({
            "role": "helpful assistant"
        });

        let response = server
            .put(&format!("/sessions/{}/role", session.id))
            .json(&role_payload)
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let updated_session: SessionResponse = response.json();
        assert_eq!(updated_session.role, Some("helpful assistant".to_string()));
    }
);

test_with!(
    test_set_session_model,
    create_test_server().await,
    |server| {
        // Create a session
        let payload = json!({
            "name": "test_session",
            "display_name": "Test Session"
        });

        let create_response = server.post("/sessions").json(&payload).await;
        let session: SessionResponse = create_response.json();

        // Set the model
        let model_payload = json!({
            "model": "claude-3-sonnet"
        });

        let response = server
            .put(&format!("/sessions/{}/model", session.id))
            .json(&model_payload)
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let updated_session: SessionResponse = response.json();
        assert_eq!(updated_session.model, Some("claude-3-sonnet".to_string()));
    }
);

test_with!(
    test_get_session_by_name,
    create_test_server().await,
    |server| {
        // Create a session
        let payload = json!({
            "name": "test_session",
            "display_name": "Test Session"
        });

        server.post("/sessions").json(&payload).await;

        // Get session by display name
        let response = server.get("/sessions/by-name/Test%20Session").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let session: SessionResponse = response.json();
        assert_eq!(session.display_name, Some("Test Session".to_string()));
    }
);

test_with!(
    test_get_nonexistent_session_by_name,
    create_test_server().await,
    |server| {
        let response = server.get("/sessions/by-name/nonexistent").await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
);

test_with!(
    test_delete_session_by_name,
    create_test_server().await,
    |server| {
        // Create a session
        let payload = json!({
            "name": "test_session",
            "display_name": "To Delete"
        });

        let create_response = server.post("/sessions").json(&payload).await;
        let session: SessionResponse = create_response.json();

        // Delete the session by name
        let response = server.delete("/sessions/by-name/To%20Delete").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // Verify it's deleted
        let get_response = server.get(&format!("/sessions/{}", session.id)).await;
        assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
    }
);

test_with!(
    test_invalid_json_payload,
    create_test_server().await,
    |server| {
        let response = server.post("/sessions").text("invalid json").await;
        assert_eq!(response.status_code(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
);

test_with!(
    test_missing_required_field,
    create_test_server().await,
    |server| {
        let payload = json!({
            "display_name": "No Name Session"
        });

        let response = server.post("/sessions").json(&payload).await;
        assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
    }
);
