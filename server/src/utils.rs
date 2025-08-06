use crate::db::ClaiDb;
use common::error::ClaiError;
use reqwest::Response;
use tracing::{error, info};

pub struct HttpUtils;

impl HttpUtils {
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

    pub fn network_error(operation: &str) -> impl Fn(reqwest::Error) -> ClaiError + use<'_> {
        move |e| {
            error!("Failed to send request to {}: {}", operation, e);
            ClaiError::Network(e)
        }
    }

    pub fn json_parse_error(operation: &str) -> impl Fn(reqwest::Error) -> ClaiError + use<'_> {
        move |e| {
            error!("Failed to parse {} response: {}", operation, e);
            ClaiError::Network(e)
        }
    }
}

pub fn cleanup_old_sessions() {
    tokio::spawn(async {
        info!("Starting background database cleanup...");
        match ClaiDb::get().cleanup_old_sessions().await {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    info!("Cleaned up {} old database records", deleted_count);
                } else {
                    info!("No old sessions to clean up");
                }
            }
            Err(e) => {
                error!("Failed to cleanup old sessions: {}", e);
            }
        }
    });
}
