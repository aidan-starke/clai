use crate::{config::Config, error::ClaiError, server::HttpUtils, types::*};
use axum::response::Json as JsonResponse;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize)]
struct AnthropicModelsResponse {
    data: Vec<AnthropicModel>,
}

#[derive(Deserialize)]
struct AnthropicModel {
    id: String,
    display_name: String,
    created_at: String,
}

pub async fn get_models() -> Result<JsonResponse<Vec<ClaudeModel>>, ClaiError> {
    info!("Fetching available models from Claude API");

    let config = Config::load()?;

    let client = reqwest::Client::new();

    let response = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", &config.anthropic_api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(HttpUtils::network_error("Claude API"))?;

    HttpUtils::check_response_status("Claude API", &response)?;

    let anthropic_response: AnthropicModelsResponse = response
        .json()
        .await
        .map_err(HttpUtils::json_parse_error("Claude API models"))?;

    let models: Vec<ClaudeModel> = anthropic_response
        .data
        .into_iter()
        .map(|model| ClaudeModel {
            id: model.id,
            display_name: model.display_name,
            created_at: model.created_at,
        })
        .collect();

    info!("Successfully fetched {} models", models.len());

    Ok(JsonResponse(models))
}
