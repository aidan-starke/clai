use crate::db::ClaiDb;
use crate::handle_db_operation;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
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

    // Get session info to check for role
    let session = handle_db_operation!("get session info", db.get_session_by_id(session_id));

    // Get conversation history (before storing the new message)
    let mut messages = match db.get_session_messages(session_id) {
        Ok(msgs) => {
            // Convert database messages to Claude API format
            let mut claude_messages = Vec::new();
            for msg in msgs {
                claude_messages.push(ClaudeMessage {
                    role: msg.role,
                    content: msg.content,
                });
            }
            claude_messages
        }
        Err(e) => {
            error!("Failed to get session messages: {}", e);
            // Fallback to empty conversation history
            Vec::new()
        }
    };

    // Add the new user message
    messages.push(ClaudeMessage {
        role: "user".to_string(),
        content: payload.message.clone(),
    });

    // Prepare system message if role is set
    let system_message = session.role.as_ref().map(|role| {
        format!(
            "You are a {}. Please respond in character and provide expertise relevant to this role.",
            role
        )
    });

    let claude_request = ClaudeRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        max_tokens: 1000,
        messages,
        system: system_message,
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

    handle_db_operation!("store user message", db.create_message(session_id, "user", &payload.message));

    let text = claude_response.content.into_iter().map(|c| c.text).collect::<Vec<_>>().join("");
    handle_db_operation!("store assistant message", db.create_message(session_id, "assistant", &text));

    info!("Successfully processed chat request for session {}", session_id);

    let chat_response = ChatResponse { response: text };

    Ok(JsonResponse(chat_response))
}
