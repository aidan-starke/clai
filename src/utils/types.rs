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
