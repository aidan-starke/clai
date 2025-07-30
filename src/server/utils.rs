use crate::error::ClaiError;
use reqwest::Response;
use tracing::error;

/// Common HTTP response utilities for handlers
pub struct HttpUtils;

impl HttpUtils {
    /// Check if HTTP response is successful, return ClaiError if not
    pub fn check_response_status(operation: &str, response: &Response) -> Result<(), ClaiError> {
        if !response.status().is_success() {
            error!("{} returned error status: {}", operation, response.status());
            return Err(ClaiError::server(format!(
                "{} returned error status: {}",
                operation,
                response.status()
            )));
        }
        Ok(())
    }

    /// Create a map_err closure for network requests
    pub fn network_error(operation: &str) -> impl Fn(reqwest::Error) -> ClaiError + use<'_> {
        move |e| {
            error!("Failed to send request to {}: {}", operation, e);
            ClaiError::Network(e)
        }
    }

    /// Create a map_err closure for JSON parsing
    pub fn json_parse_error(operation: &str) -> impl Fn(reqwest::Error) -> ClaiError + use<'_> {
        move |e| {
            error!("Failed to parse {} response: {}", operation, e);
            ClaiError::Network(e)
        }
    }

    /// Load configuration with proper error handling
    pub fn load_config() -> Result<crate::config::Config, ClaiError> {
        crate::config::Config::load().map_err(|e| {
            error!("Failed to load configuration: {}", e);
            e
        })
    }
}
