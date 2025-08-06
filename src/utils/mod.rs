pub mod macros;
pub mod terminal;

pub use terminal::*;

use common::{
    config::Config,
    error::{ClaiError, Result},
};

pub async fn ensure_server_running() -> Result<()> {
    let config = Config::load()?;
    let server_url = &config.clai_server_url;

    // Quick health check
    if reqwest::get(&format!("{}/health", server_url))
        .await
        .is_ok()
    {
        return Ok(()); // Server already running
    }

    tokio::spawn(server::run_server(false));

    // Wait for server to be ready, 5 second timeout
    for _ in 0..50 {
        if reqwest::get(&format!("{}/health", server_url))
            .await
            .is_ok()
        {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Err(ClaiError::server("Server failed to start"))
}
