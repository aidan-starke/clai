use crate::types::*;
use anyhow::Result;
use reqwest::Client;
use std::cell::Cell;

#[derive(Clone)]
pub struct SessionManager {
    client: Client,
    server_url: String,
    current_session: Cell<Option<i32>>,
}

impl SessionManager {
    pub fn new(server_url: String) -> Self {
        Self {
            client: Client::new(),
            server_url,
            current_session: Cell::new(None),
        }
    }

    pub fn set_current_session(&self, session_id: i32) {
        self.current_session.set(Some(session_id));
    }

    pub fn require_current_session(&self) -> Result<i32> {
        self.current_session
            .get()
            .ok_or_else(|| anyhow::anyhow!("No current session set"))
    }

    pub async fn init(&self, session_name: Option<&str>) -> Result<(i32, String)> {
        let (session_id, display_name) = if let Some(name) = session_name {
            // Try to get session by name
            let session_id = self.get_session_by_name(name).await?;
            (session_id, name.to_string())
        } else {
            // Try to get the last session
            match self.get_last_session().await {
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
            }
        };

        self.set_current_session(session_id);

        Ok((session_id, display_name))
    }

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
            println!("Resuming session: {} (ID: {})", session_name, session.id);
            Ok(session.id)
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Session '{}' not found", session_name);
        } else {
            anyhow::bail!(
                "Failed to get session '{}': {}",
                session_name,
                response.status()
            );
        }
    }

    pub async fn get_last_session(&self) -> Result<i32> {
        let response = self
            .client
            .get(&format!("{}/sessions/last", self.server_url))
            .send()
            .await?;

        if response.status().is_success() {
            let session: SessionResponse = response.json().await?;
            let display_name = session.display_name.as_deref().unwrap_or(&session.name);
            println!("Resuming session: {} (ID: {})", display_name, session.id);
            Ok(session.id)
        } else {
            // No last session found, create a new one
            self.create_new_session().await
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
            println!("Created new session: {} (ID: {})", session.name, session.id);
            self.set_current_session(session.id);
            Ok(session.id)
        } else {
            anyhow::bail!("Failed to create session: {}", response.status());
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

        if response.status().is_success() {
            println!("✅ Session saved as '{}'", display_name);
        } else {
            anyhow::bail!("Failed to save session: {}", response.status());
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

        if response.status().is_success() {
            println!("🗑️ Session '{}' deleted successfully", session_name);
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Session '{}' not found", session_name);
        } else {
            anyhow::bail!("Failed to delete session: {}", response.status());
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

        if response.status().is_success() {
            // Success message is handled by the caller
        } else {
            anyhow::bail!("Failed to set role: {}", response.status());
        }

        Ok(())
    }

    pub async fn get_session_info(&self) -> Result<SessionResponse> {
        let session_id = self.require_current_session()?;
        self.get_session_info_by_id(session_id).await
    }

    pub async fn get_session_info_by_id(&self, session_id: i32) -> Result<SessionResponse> {
        let response = self
            .client
            .get(&format!("{}/sessions/{}", self.server_url, session_id))
            .send()
            .await?;

        if response.status().is_success() {
            let session: SessionResponse = response.json().await?;
            Ok(session)
        } else {
            anyhow::bail!("Failed to get session info: {}", response.status());
        }
    }

    pub async fn list_sessions(&self) -> Result<()> {
        let response = self
            .client
            .get(&format!("{}/sessions", self.server_url))
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list sessions: {}", response.status());
        }

        let sessions: Vec<SessionResponse> = response.json().await?;

        if sessions.is_empty() {
            println!("No saved sessions found.");
            return Ok(());
        }

        println!("📚 Saved Sessions:");
        println!("─────────────────");
        for session in sessions {
            if let Some(display_name) = &session.display_name {
                if let Some(role) = &session.role {
                    println!("• {} (ID: {}) 🎭 {}", display_name, session.id, role);
                } else {
                    println!("• {} (ID: {})", display_name, session.id);
                }
            }
        }

        Ok(())
    }

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
            anyhow::bail!("Failed to send message: {}", response.status());
        }
    }
}
