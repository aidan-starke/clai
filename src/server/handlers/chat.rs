use crate::db::ClaiDb;
use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: i32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

pub async fn chat(Path(session_id): Path<i32>, Json(payload): Json<ChatRequest>) -> Result<JsonResponse<ChatResponse>, StatusCode> {
    info!("Chat request for session {} with message: {}", session_id, payload.message);

    let api_key = env::var("ANTHROPIC_API_KEY").map_err(|e| {
        error!("Failed to get ANTHROPIC_API_KEY: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let client = reqwest::Client::new();
    let mut db = ClaiDb::new();

    // Get conversation history (before storing the new message)
    let messages = match db.get_session_messages(session_id) {
        Ok(msgs) => {
            // Convert database messages to Claude API format
            let mut claude_messages = Vec::new();
            for msg in msgs {
                claude_messages.push(ClaudeMessage {
                    role: msg.role,
                    content: msg.content,
                });
            }
            // Add the new user message
            claude_messages.push(ClaudeMessage {
                role: "user".to_string(),
                content: payload.message.clone(),
            });
            claude_messages
        }
        Err(e) => {
            error!("Failed to get session messages: {}", e);
            // Fallback to just the current message
            vec![ClaudeMessage {
                role: "user".to_string(),
                content: payload.message.clone(),
            }]
        }
    };

    // Store user message in database after getting history
    if let Err(e) = db.create_message(session_id, "user", &payload.message) {
        error!("Failed to store user message: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let claude_request = ClaudeRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        max_tokens: 1000,
        messages,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&claude_request)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to Claude API: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !response.status().is_success() {
        error!("Claude API returned error status: {}", response.status());
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let claude_response: ClaudeResponse = response.json().await.map_err(|e| {
        error!("Failed to parse Claude API response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let text = claude_response.content.into_iter().map(|c| c.text).collect::<Vec<_>>().join("");

    // Store Claude's response in database
    if let Err(e) = db.create_message(session_id, "assistant", &text) {
        error!("Failed to store Claude response: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("Successfully processed chat request for session {}", session_id);

    let chat_response = ChatResponse { response: text };

    Ok(JsonResponse(chat_response))
}
