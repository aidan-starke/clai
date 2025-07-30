use axum::{
    routing::{get, post, put},
    Router,
};
use tracing::info;

use crate::error::Result;
use crate::server::handlers;
use crate::utils::{
    self,
    constants::{DEFAULT_SERVER_HOST, DEFAULT_SERVER_PORT},
};

pub async fn run_server(debug_mode: bool) -> Result<()> {
    dotenv::dotenv().ok();

    if debug_mode {
        tracing_subscriber::fmt().init();
    }

    info!("clai server starting...");

    utils::cleanup_old_sessions();

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/sessions", post(handlers::session::create_session))
        .route("/sessions", get(handlers::session::list_sessions))
        .route("/sessions/last", get(handlers::session::get_last_session))
        .route(
            "/sessions/by-name/{name}",
            get(handlers::session::get_session_by_name).delete(handlers::session::delete_session),
        )
        .route(
            "/sessions/{id}",
            get(handlers::session::get_session_by_id).patch(handlers::session::save_session),
        )
        .route("/sessions/{id}/role", put(handlers::session::set_role))
        .route("/sessions/{id}/model", put(handlers::session::set_model))
        .route("/sessions/{id}/chat", post(handlers::chat::chat))
        .route("/models", get(handlers::models::get_models));

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", DEFAULT_SERVER_HOST, DEFAULT_SERVER_PORT))
            .await?;
    info!(
        "Server running on http://{}:{}",
        DEFAULT_SERVER_HOST, DEFAULT_SERVER_PORT
    );

    axum::serve(listener, app).await?;

    Ok(())
}
