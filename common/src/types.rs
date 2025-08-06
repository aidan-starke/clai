use serde::{Deserialize, Serialize};

// Session types
#[derive(Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub display_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SaveSessionRequest {
    pub display_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct SetRoleRequest {
    pub role: Option<String>,
}

// Chat types
#[derive(Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
}

// Model types
#[derive(Serialize, Deserialize)]
pub struct SetModelRequest {
    pub model: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClaudeModel {
    pub id: String,
    pub display_name: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ClaudeModel>,
}
