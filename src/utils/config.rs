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
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self> {
        envy::from_env::<Config>()
            .map_err(|e| ClaiError::config(&format!("Failed to load configuration: {}", e)))
    }

    /// Get server bind address
    pub fn server_bind_address(&self) -> String {
        format!("{}:{}", self.clai_server_host, self.clai_server_port)
    }
}

// Default functions for serde
fn default_server_url() -> String {
    crate::constants::DEFAULT_SERVER_URL.to_string()
}

fn default_database_url() -> String {
    "clai.db".to_string()
}

fn default_server_host() -> String {
    crate::constants::DEFAULT_SERVER_HOST.to_string()
}

fn default_server_port() -> u16 {
    crate::constants::DEFAULT_SERVER_PORT
}
