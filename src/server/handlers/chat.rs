use crate::server::HttpUtils;
use crate::{
    constants::{CLAUDE_MAX_TOKENS, DEFAULT_MODEL},
    db::ClaiDb,
    error::ClaiError,
    handle_db_operation,
    types::*,
};
use axum::{
    extract::{Json, Path},
    response::Json as JsonResponse,
};
use serde::{Deserialize, Serialize};
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

    let config = HttpUtils::load_config()?;

    let client = reqwest::Client::new();

    let session = handle_db_operation!("get session info", ClaiDb::get_session_by_id(session_id));

    let mut messages = match ClaiDb::get_session_messages(session_id) {
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
        model: session
            .model
            .as_deref()
            .unwrap_or(DEFAULT_MODEL)
            .to_string(),
        max_tokens: CLAUDE_MAX_TOKENS,
        messages,
        system: system_message,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &config.anthropic_api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&claude_request)
        .send()
        .await
        .map_err(HttpUtils::network_error("Claude API"))?;

    HttpUtils::check_response_status("Claude API", &response)?;

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .map_err(HttpUtils::json_parse_error("Claude API"))?;

    let text = claude_response
        .content
        .into_iter()
        .map(|c| c.text)
        .collect::<Vec<_>>()
        .join("");

    handle_db_operation!(
        "store user message",
        ClaiDb::create_message(session_id, "user", &payload.message)
    );
    handle_db_operation!(
        "store assistant message",
        ClaiDb::create_message(session_id, "assistant", &text)
    );

    info!(
        "Successfully processed chat request for session {}",
        session_id
    );

    let chat_response = ChatResponse { response: text };

    Ok(JsonResponse(chat_response))
}
