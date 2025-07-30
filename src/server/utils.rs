use crate::error::ClaiError;
use reqwest::Response;
use tracing::error;

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
