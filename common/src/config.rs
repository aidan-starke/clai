use crate::error::{ClaiError, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    /// Server URL for client-server communication
    #[serde(default = "default_server_url")]
    pub clai_server_url: String,

    /// Anthropic API key for Claude integration
    pub anthropic_api_key: String,

    /// Database URL (SQLite file path)
    #[serde(default = "default_database_url")]
    pub database_url: String,

    /// Server host binding address
    #[serde(default = "default_server_host")]
    pub clai_server_host: String,

    /// Server port
    #[serde(default = "default_server_port")]
    pub clai_server_port: u16,

    /// Comma-separated list of allowed CORS origins
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();
        envy::from_env::<Config>()
            .map_err(|e| ClaiError::config(&format!("Failed to load configuration: {}", e)))
    }

    pub fn server_bind_address(&self) -> String {
        format!("{}:{}", self.clai_server_host, self.clai_server_port)
    }
}

fn default_server_url() -> String {
    "http://localhost:3500".to_string()
}

fn default_database_url() -> String {
    "sqlite://clai.db".to_string()
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_server_port() -> u16 {
    3500
}

fn default_allowed_origins() -> String {
    "http://localhost:5173".to_string()
}
