use crate::{session_manager::SessionManager, utils, write_line, write_spaced};

pub enum CommandResult {
    Continue,
    UpdateSession { id: i32 },
}

pub struct CommandHandler {
    session_manager: SessionManager,
}

impl CommandHandler {
    pub fn new(session_manager: SessionManager) -> Self {
        Self { session_manager }
    }

    pub async fn handle_command(&self, message: &str, session_id: i32) -> anyhow::Result<CommandResult> {
        match message {
            "/clear" => self.handle_clear().await,
            cmd if cmd.starts_with("/new") => self.handle_new(cmd).await,
            cmd if cmd.starts_with("/save ") => self.handle_save(cmd, session_id).await,
            cmd if cmd.starts_with("/delete ") => self.handle_delete(cmd).await,
            "/list" => self.handle_list().await,
            cmd if cmd.starts_with("/resume ") => self.handle_resume(cmd).await,
            cmd if cmd.starts_with("/role") => self.handle_role(cmd, session_id).await,
            _ => Ok(CommandResult::Continue),
        }
    }

    async fn handle_clear(&self) -> anyhow::Result<CommandResult> {
        utils::clear_screen()?;
        Ok(CommandResult::Continue)
    }

    async fn handle_new(&self, cmd: &str) -> anyhow::Result<CommandResult> {
        if cmd == "/new" {
            match self.session_manager.create_new_session().await {
                Ok(new_session_id) => {
                    let session_name = format!("Session {}", new_session_id);
                    write_spaced!("✨ Created new session (ID: {})", new_session_id);
                    utils::write_session_info(new_session_id, &session_name);
                    Ok(CommandResult::UpdateSession { id: new_session_id })
                }
                Err(e) => {
                    utils::write_error(&format!("Failed to create new session: {}", e));
                    Ok(CommandResult::Continue)
                }
            }
        } else {
            let session_name = cmd.trim_start_matches("/new ").trim();
            if session_name.is_empty() {
                write_line!("Usage: /new <session_name>");
                return Ok(CommandResult::Continue);
            }

            match self.session_manager.create_new_session().await {
                Ok(new_session_id) => match self.session_manager.save_session(new_session_id, session_name).await {
                    Ok(_) => {
                        write_spaced!("✨ Created and saved new session: '{}'", session_name);
                        utils::write_session_info(new_session_id, &session_name);
                        Ok(CommandResult::UpdateSession { id: new_session_id })
                    }
                    Err(e) => {
                        utils::write_error(&format!("Failed to save new session: {}", e));
                        Ok(CommandResult::Continue)
                    }
                },
                Err(e) => {
                    utils::write_error(&format!("Failed to create new session: {}", e));
                    Ok(CommandResult::Continue)
                }
            }
        }
    }

    async fn handle_save(&self, cmd: &str, session_id: i32) -> anyhow::Result<CommandResult> {
        let session_name = cmd.trim_start_matches("/save ").trim();
        if session_name.is_empty() {
            write_line!("Usage: /save <session_name>");
            return Ok(CommandResult::Continue);
        }

        match self.session_manager.save_session(session_id, session_name).await {
            Ok(_) => Ok(CommandResult::Continue),
            Err(e) => {
                utils::write_error(&format!("Failed to save session: {}", e));
                Ok(CommandResult::Continue)
            }
        }
    }

    async fn handle_delete(&self, cmd: &str) -> anyhow::Result<CommandResult> {
        let session_name = cmd.trim_start_matches("/delete ").trim();
        if session_name.is_empty() {
            write_line!("Usage: /delete <session_name>");
            return Ok(CommandResult::Continue);
        }

        match self.session_manager.delete_session(session_name).await {
            Ok(_) => Ok(CommandResult::Continue),
            Err(e) => {
                utils::write_error(&format!("Failed to delete session: {}", e));
                Ok(CommandResult::Continue)
            }
        }
    }

    async fn handle_list(&self) -> anyhow::Result<CommandResult> {
        match self.session_manager.list_sessions().await {
            Ok(_) => Ok(CommandResult::Continue),
            Err(e) => {
                utils::write_error(&format!("Failed to list sessions: {}", e));
                Ok(CommandResult::Continue)
            }
        }
    }

    async fn handle_resume(&self, cmd: &str) -> anyhow::Result<CommandResult> {
        let session_name = cmd.trim_start_matches("/resume ").trim();
        if session_name.is_empty() {
            write_line!("Usage: /resume <session_name>");
            return Ok(CommandResult::Continue);
        }

        match self.session_manager.get_session_by_name(session_name).await {
            Ok(new_session_id) => {
                write_spaced!("🔄 Switched to session: '{}'", session_name);
                utils::write_session_info(new_session_id, &session_name);
                Ok(CommandResult::UpdateSession { id: new_session_id })
            }
            Err(e) => {
                utils::write_error(&format!("Failed to resume session: {}", e));
                Ok(CommandResult::Continue)
            }
        }
    }

    async fn handle_role(&self, cmd: &str, session_id: i32) -> anyhow::Result<CommandResult> {
        if cmd == "/role" {
            match self.session_manager.get_session_info(session_id).await {
                Ok(session) => {
                    if let Some(role) = session.role {
                        write_line!("🎭 Current role: '{}'", role);
                    } else {
                        write_line!("🎭 No role set (Claude will respond as default assistant)");
                    }
                    Ok(CommandResult::Continue)
                }
                Err(e) => {
                    utils::write_error(&format!("Failed to get session info: {}", e));
                    Ok(CommandResult::Continue)
                }
            }
        } else {
            let role = cmd.trim_start_matches("/role ").trim();
            if role.is_empty() {
                write_line!("Usage: /role <role_name>");
                return Ok(CommandResult::Continue);
            }

            match self.session_manager.set_role(session_id, Some(role.to_string())).await {
                Ok(_) => {
                    write_spaced!("🎭 Role set to: '{}'", role);
                    Ok(CommandResult::Continue)
                }
                Err(e) => {
                    utils::write_error(&format!("Failed to set role: {}", e));
                    Ok(CommandResult::Continue)
                }
            }
        }
    }
}

