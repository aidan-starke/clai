use axum::{
    routing::{get, post, put},
    Router,
};
use tracing::info;

use crate::{config::Config, db::ClaiDb, error::Result, server::handlers, utils};

pub async fn run_server(debug_mode: bool) -> Result<()> {
    if debug_mode {
        tracing_subscriber::fmt().init();
    }

    info!("clai server starting...");

    // Initialize database and run migrations
    ClaiDb::init()?;

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

    let config = Config::load()?;
    let listener = tokio::net::TcpListener::bind(config.server_bind_address()).await?;
    info!("Server running on {}", config.server_bind_address());

    axum::serve(listener, app).await?;

    Ok(())
}
