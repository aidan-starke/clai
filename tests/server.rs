use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use axum_test::TestServer;
use std::sync::OnceLock;
use test_macros::db_test;

use clai::{server::handlers, types::*};

mod common;

static TEST_SERVER: OnceLock<TestServer> = OnceLock::new();

fn get_test_server() -> &'static TestServer {
    TEST_SERVER.get_or_init(|| {
        let app = Router::new()
            .route("/health", get(handlers::health))
            .route("/sessions", post(handlers::session::create_session))
            .route("/sessions", get(handlers::session::list_sessions))
            .route("/sessions/last", get(handlers::session::get_last_session))
            .route(
                "/sessions/by-name/{name}",
                get(handlers::session::get_session_by_name),
            )
            .route("/sessions/{id}", get(handlers::session::get_session_by_id))
            .route("/sessions/{id}", patch(handlers::session::save_session))
            .route("/sessions/{id}/role", put(handlers::session::set_role))
            .route("/sessions/{id}/model", put(handlers::session::set_model))
            .route(
                "/sessions/by-name/{name}",
                delete(handlers::session::delete_session),
            );

        TestServer::new(app).expect("Failed to create test server")
    })
}

#[tokio::test]
async fn test_health_endpoint() {
    let server = get_test_server();

    let response = server.get("/health").await;

    response.assert_status_ok();
}

#[db_test]
async fn test_create_session() {
    let server = get_test_server();

    let create_request = CreateSessionRequest {
        name: "test-session-123".to_string(),
        display_name: Some("My Test Session".to_string()),
    };

    let response = server.post("/sessions").json(&create_request).await;

    response.assert_status_ok();

    let session: SessionResponse = response.json();
    assert_eq!(session.name, "test-session-123");
    assert_eq!(session.display_name, Some("My Test Session".to_string()));
}

#[db_test]
async fn test_get_session_by_id() {
    let server = get_test_server();

    let create_request = CreateSessionRequest {
        name: "get-by-id-test".to_string(),
        display_name: Some("Get By ID Test".to_string()),
    };

    let create_response = server.post("/sessions").json(&create_request).await;

    let created_session: SessionResponse = create_response.json();

    let get_response = server
        .get(&format!("/sessions/{}", created_session.id))
        .await;

    get_response.assert_status_ok();

    let retrieved_session: SessionResponse = get_response.json();
    assert_eq!(retrieved_session.id, created_session.id);
    assert_eq!(retrieved_session.name, "get-by-id-test");
}

#[db_test]
async fn test_get_session_by_name() {
    let server = get_test_server();

    let create_request = CreateSessionRequest {
        name: "get-by-name-test".to_string(),
        display_name: Some("Get By Name Test".to_string()),
    };

    let create_response = server.post("/sessions").json(&create_request).await;

    let created_session: SessionResponse = create_response.json();

    let get_response = server.get("/sessions/by-name/Get%20By%20Name%20Test").await;

    get_response.assert_status_ok();

    let retrieved_session: SessionResponse = get_response.json();
    assert_eq!(retrieved_session.id, created_session.id);
    assert_eq!(
        retrieved_session.display_name,
        Some("Get By Name Test".to_string())
    );
}

#[db_test]
async fn test_session_not_found() {
    let server = get_test_server();

    let response = server.get("/sessions/99999").await;

    response.assert_status_not_found();
}

#[db_test]
async fn test_save_session() {
    let server = get_test_server();

    let create_request = CreateSessionRequest {
        name: "save-test".to_string(),
        display_name: None,
    };

    let create_response = server.post("/sessions").json(&create_request).await;

    let created_session: SessionResponse = create_response.json();

    let save_request = SaveSessionRequest {
        display_name: "Saved Session".to_string(),
    };

    let save_response = server
        .patch(&format!("/sessions/{}", created_session.id))
        .json(&save_request)
        .await;

    save_response.assert_status_ok();

    let saved_session: SessionResponse = save_response.json();
    assert_eq!(
        saved_session.display_name,
        Some("Saved Session".to_string())
    );
}

#[db_test]
async fn test_set_role() {
    let server = get_test_server();

    let create_request = CreateSessionRequest {
        name: "role-test".to_string(),
        display_name: Some("Role Test".to_string()),
    };

    let create_response = server.post("/sessions").json(&create_request).await;

    let created_session: SessionResponse = create_response.json();

    let role_request = SetRoleRequest {
        role: Some("helpful assistant".to_string()),
    };

    let role_response = server
        .put(&format!("/sessions/{}/role", created_session.id))
        .json(&role_request)
        .await;

    role_response.assert_status_ok();

    let updated_session: SessionResponse = role_response.json();
    assert_eq!(updated_session.role, Some("helpful assistant".to_string()));
}

#[db_test]
async fn test_set_model() {
    let server = get_test_server();

    let create_request = CreateSessionRequest {
        name: "model-test".to_string(),
        display_name: Some("Model Test".to_string()),
    };

    let create_response = server.post("/sessions").json(&create_request).await;

    let created_session: SessionResponse = create_response.json();

    let model_request = SetModelRequest {
        model: "claude-3-opus".to_string(),
    };

    let model_response = server
        .put(&format!("/sessions/{}/model", created_session.id))
        .json(&model_request)
        .await;

    model_response.assert_status_ok();

    let updated_session: SessionResponse = model_response.json();
    assert_eq!(updated_session.model, Some("claude-3-opus".to_string()));
}

#[db_test]
async fn test_list_sessions() {
    let server = get_test_server();

    let requests = vec![
        CreateSessionRequest {
            name: "temp1".to_string(),
            display_name: None,
        },
        CreateSessionRequest {
            name: "temp2".to_string(),
            display_name: Some("Named Session 1".to_string()),
        },
        CreateSessionRequest {
            name: "temp3".to_string(),
            display_name: Some("Named Session 2".to_string()),
        },
    ];

    for request in requests {
        server.post("/sessions").json(&request).await;
    }

    let list_response = server.get("/sessions").await;

    list_response.assert_status_ok();

    let sessions: Vec<SessionResponse> = list_response.json();
    assert_eq!(sessions.len(), 2);

    let display_names: Vec<_> = sessions
        .iter()
        .filter_map(|s| s.display_name.as_ref())
        .collect();

    assert!(display_names.contains(&&"Named Session 1".to_string()));
    assert!(display_names.contains(&&"Named Session 2".to_string()));
}

#[db_test]
async fn test_delete_session() {
    let server = get_test_server();

    let create_request = CreateSessionRequest {
        name: "delete-test".to_string(),
        display_name: Some("To Be Deleted".to_string()),
    };

    let create_response = server.post("/sessions").json(&create_request).await;

    let created_session: SessionResponse = create_response.json();

    let delete_response = server.delete("/sessions/by-name/To%20Be%20Deleted").await;

    delete_response.assert_status_ok();

    let get_response = server
        .get(&format!("/sessions/{}", created_session.id))
        .await;
    get_response.assert_status_not_found();
}
