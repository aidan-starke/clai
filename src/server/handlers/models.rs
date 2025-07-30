use crate::error::ClaiError;
use crate::utils::types::*;
use axum::response::Json as JsonResponse;
use serde::Deserialize;
use std::env;
use tracing::{error, info};

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

pub async fn get_models() -> std::result::Result<JsonResponse<Vec<ClaudeModel>>, ClaiError> {
    info!("Fetching available models from Claude API");

    let api_key = env::var("ANTHROPIC_API_KEY").map_err(|e| {
        error!("Failed to get ANTHROPIC_API_KEY: {}", e);
        ClaiError::config("ANTHROPIC_API_KEY environment variable not set")
    })?;

    let client = reqwest::Client::new();

    let response = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await
        .map_err(|e| {
            error!("Failed to fetch models from Claude API: {}", e);
            ClaiError::Network(e)
        })?;

    if !response.status().is_success() {
        error!("Claude API returned error status: {}", response.status());
        return Err(ClaiError::server(format!(
            "Claude API returned error status: {}",
            response.status()
        )));
    }

    let anthropic_response: AnthropicModelsResponse = response.json().await.map_err(|e| {
        error!("Failed to parse Claude API models response: {}", e);
        ClaiError::Network(e)
    })?;

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
