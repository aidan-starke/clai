pub mod db;
pub mod handlers;
mod macros;
mod utils;

use axum::{
    routing::{get, post, put},
    Router,
};
use db::ClaiDb;
use tower_http::cors::CorsLayer;
use tracing::info;

use common::{config::Config, error::Result};

pub async fn run_server(debug_mode: bool) -> Result<()> {
    if debug_mode {
        tracing_subscriber::fmt().init();
    }

    info!("clai server starting...");

    // Initialize database and run migrations
    ClaiDb::init().await?;

    utils::cleanup_old_sessions();

    let config = Config::load()?;

    // Parse allowed origins from config (comma-separated)
    let allowed_origins: Vec<_> = config
        .allowed_origins
        .split(',')
        .map(|s| s.trim().parse().expect("Invalid origin URL"))
        .collect();

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

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
        .route("/models", get(handlers::models::get_models))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(config.server_bind_address()).await?;
    info!("Server running on {}", config.server_bind_address());

    axum::serve(listener, app).await?;

    Ok(())
}
