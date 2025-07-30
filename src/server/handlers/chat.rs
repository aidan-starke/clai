use crate::db::ClaiDb;
use crate::error::ClaiError;
use crate::handle_db_operation;
use crate::utils::types::*;
use axum::{
    extract::{Json, Path},
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

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

pub async fn chat(
    Path(session_id): Path<i32>,
    Json(payload): Json<ChatRequest>,
) -> std::result::Result<JsonResponse<ChatResponse>, ClaiError> {
    info!(
        "Chat request for session {} with message: {}",
        session_id, payload.message
    );

    let api_key = env::var("ANTHROPIC_API_KEY").map_err(|e| {
        error!("Failed to get ANTHROPIC_API_KEY: {}", e);
        ClaiError::config("ANTHROPIC_API_KEY environment variable not set")
    })?;

    let client = reqwest::Client::new();

    let (session, messages) = {
        let mut db = ClaiDb::get();

        let session = handle_db_operation!("get session info", db.get_session_by_id(session_id));

        let messages = match db.get_session_messages(session_id) {
            Ok(msgs) => {
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
                Vec::new()
            }
        };

        let mut messages = messages;
        messages.push(ClaudeMessage {
            role: "user".to_string(),
            content: payload.message.clone(),
        });

        (session, messages)
    };

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
            ClaiError::Network(e)
        })?;

    if !response.status().is_success() {
        error!("Claude API returned error status: {}", response.status());
        return Err(ClaiError::server(format!("Claude API returned error status: {}", response.status())));
    }

    let claude_response: ClaudeResponse = response.json().await.map_err(|e| {
        error!("Failed to parse Claude API response: {}", e);
        ClaiError::Network(e)
    })?;

    let text = claude_response
        .content
        .into_iter()
        .map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    let mut db = ClaiDb::get();
    handle_db_operation!(
        "store user message",
        db.create_message(session_id, "user", &payload.message)
    );
    handle_db_operation!(
        "store assistant message",
        db.create_message(session_id, "assistant", &text)
    );

    info!(
        "Successfully processed chat request for session {}",
        session_id
    );

    let chat_response = ChatResponse { response: text };

    Ok(JsonResponse(chat_response))
}
