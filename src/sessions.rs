use crate::error::{ClaiError, Result};
use crate::types::*;
use reqwest::Client;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SessionManager {
    client: Client,
    server_url: String,
    current_session: Arc<Mutex<Option<i32>>>,
}

impl SessionManager {
    // ===== Construction & State Management =====

    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            client: Client::new(),
            current_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_current_session(&self, session_id: i32) -> Result<()> {
        *self
            .current_session
            .lock()
            .map_err(|_| ClaiError::session("Session state corrupted"))? = Some(session_id);
        Ok(())
    }

    fn require_current_session(&self) -> Result<i32> {
        let session_id = self
            .current_session
            .lock()
            .map_err(|_| ClaiError::session("Session state corrupted"))?
            .ok_or_else(|| ClaiError::session("No current session set"))?;
        Ok(session_id)
    }

    // ===== Initialization =====

    pub async fn init(&self) -> Result<(i32, String)> {
        let (session_id, display_name) = match self.get_last_session().await {
            Ok(session_id) => {
                // Get session info to determine the display name
                match self.get_session_info_by_id(session_id).await {
                    Ok(session) => {
                        let display_name = session
                            .display_name
                            .unwrap_or_else(|| format!("Session {}", session_id));
                        (session_id, display_name)
                    }
                    Err(_) => (session_id, format!("Session {}", session_id)),
                }
            }
            Err(_) => {
                // No last session found, create a new one
                let session_id = self.create_new_session().await?;
                (session_id, format!("Session {}", session_id))
            }
        };

        self.set_current_session(session_id)?;

        Ok((session_id, display_name))
    }

    async fn get_last_session(&self) -> Result<i32> {
        let response = self
            .client
            .get(&format!("{}/sessions/last", self.server_url))
            .send()
            .await?;

        if response.status().is_success() {
            let session: SessionResponse = response.json().await?;
            Ok(session.id)
        } else {
            // No last session found, create a new one
            self.create_new_session().await
        }
    }

    async fn get_session_info_by_id(&self, session_id: i32) -> Result<SessionResponse> {
        let response = self
            .client
            .get(&format!("{}/sessions/{}", self.server_url, session_id))
            .send()
            .await?;

        if response.status().is_success() {
            let session: SessionResponse = response.json().await?;
            Ok(session)
        } else {
            return Err(ClaiError::server(format!(
                "Failed to get session info: {}",
                response.status()
            )));
        }
    }

    // ===== Session CRUD Operations =====

    pub async fn get_session_by_name(&self, session_name: &str) -> Result<i32> {
        let response = self
            .client
            .get(&format!(
                "{}/sessions/by-name/{}",
                self.server_url,
                urlencoding::encode(session_name)
            ))
            .send()
            .await?;

        if response.status().is_success() {
            let session: SessionResponse = response.json().await?;
            Ok(session.id)
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ClaiError::session(format!(
                "Session '{}' not found",
                session_name
            )));
        } else {
            return Err(ClaiError::server(format!(
                "Failed to get session '{}': {}",
                session_name,
                response.status()
            )));
        }
    }

    pub async fn create_new_session(&self) -> Result<i32> {
        let session_name = uuid::Uuid::new_v4().to_string();
        let request = CreateSessionRequest {
            name: session_name,
            display_name: None,
        };

        let response = self
            .client
            .post(&format!("{}/sessions", self.server_url))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let session: SessionResponse = response.json().await?;
            self.set_current_session(session.id)?;
            Ok(session.id)
        } else {
            return Err(ClaiError::server(format!(
                "Failed to create session: {}",
                response.status()
            )));
        }
    }

    pub async fn save_session(&self, display_name: &str) -> Result<()> {
        let session_id = self.require_current_session()?;

        let request = SaveSessionRequest {
            display_name: display_name.to_string(),
        };

        let response = self
            .client
            .patch(&format!("{}/sessions/{}", self.server_url, session_id))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ClaiError::server(format!(
                "Failed to save session: {}",
                response.status()
            )));
        }

        Ok(())
    }

    pub async fn delete_session(&self, session_name: &str) -> Result<()> {
        let response = self
            .client
            .delete(&format!(
                "{}/sessions/by-name/{}",
                self.server_url,
                urlencoding::encode(session_name)
            ))
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ClaiError::session(format!(
                "Session '{}' not found",
                session_name
            )));
        } else if !response.status().is_success() {
            return Err(ClaiError::server(format!(
                "Failed to delete session: {}",
                response.status()
            )));
        }

        Ok(())
    }

    pub async fn set_role(&self, role: Option<String>) -> Result<()> {
        let session_id = self.require_current_session()?;

        let request = SetRoleRequest { role };

        let response = self
            .client
            .put(&format!("{}/sessions/{}/role", self.server_url, session_id))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ClaiError::server(format!(
                "Failed to set role: {}",
                response.status()
            )));
        }

        Ok(())
    }

    pub async fn get_session_info(&self) -> Result<SessionResponse> {
        let session_id = self.require_current_session()?;
        self.get_session_info_by_id(session_id).await
    }

    pub async fn get_sessions(&self) -> Result<Vec<SessionResponse>> {
        let response = self
            .client
            .get(&format!("{}/sessions", self.server_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ClaiError::server(format!(
                "Failed to get sessions: {}",
                response.status()
            )));
        }

        let sessions: Vec<SessionResponse> = response.json().await?;
        Ok(sessions)
    }

    // ===== Model Operations =====

    pub async fn get_available_models(&self) -> Result<Vec<ClaudeModel>> {
        let response = self
            .client
            .get(&format!("{}/models", self.server_url))
            .send()
            .await?;

        if response.status().is_success() {
            let models: Vec<ClaudeModel> = response.json().await?;
            Ok(models)
        } else {
            return Err(ClaiError::server(format!(
                "Failed to get models: {}",
                response.status()
            )));
        }
    }

    pub async fn set_model(&self, model: String) -> Result<()> {
        let session_id = self.require_current_session()?;

        let request = SetModelRequest { model };

        let response = self
            .client
            .put(&format!(
                "{}/sessions/{}/model",
                self.server_url, session_id
            ))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ClaiError::server(format!(
                "Failed to set model: {}",
                response.status()
            )));
        }

        Ok(())
    }

    // ===== Chat Operations =====

    pub async fn send_message(&self, message: &str) -> Result<String> {
        let session_id = self.require_current_session()?;

        let request = ChatRequest {
            message: message.to_string(),
        };

        let response = self
            .client
            .post(&format!("{}/sessions/{}/chat", self.server_url, session_id))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let chat_response: ChatResponse = response.json().await?;
            Ok(chat_response.response)
        } else {
            return Err(ClaiError::server(format!(
                "Failed to send message: {}",
                response.status()
            )));
        }
    }
}
