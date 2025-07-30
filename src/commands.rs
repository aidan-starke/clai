use crate::{
    sessions::SessionManager,
    utils::{
        self,
        constants::{COMMANDS, DEFAULT_MODEL},
    },
    write_line, write_spaced,
};

pub struct CommandHandler {
    session_manager: SessionManager,
}

impl CommandHandler {
    pub fn new(session_manager: SessionManager) -> Self {
        Self { session_manager }
    }

    // ===== Command Dispatch =====

    pub async fn handle_command(&self, message: &str) {
        let is_valid_command = COMMANDS
            .iter()
            .any(|&cmd| message == cmd || message.starts_with(&format!("{} ", cmd)));

        if !is_valid_command {
            utils::write_command_help();
            return;
        }

        match message {
            "/clear" => self.handle_clear(),
            cmd if cmd.starts_with("/new") => self.handle_new(cmd).await,
            cmd if cmd.starts_with("/save ") => self.handle_save(cmd).await,
            cmd if cmd.starts_with("/delete ") => self.handle_delete(cmd).await,
            "/list" => self.handle_list().await,
            cmd if cmd.starts_with("/resume ") => self.handle_resume(cmd).await,
            cmd if cmd.starts_with("/role") => self.handle_role(cmd).await,
            cmd if cmd.starts_with("/model") => self.handle_model(cmd).await,
            _ => panic!("Unhandled command: {}", message),
        }
    }

    // ===== Session Lifecycle Management =====

    fn handle_clear(&self) {
        let _ = utils::clear_screen();
    }

    async fn handle_new(&self, cmd: &str) {
        if cmd == "/new" {
            match self.session_manager.create_new_session().await {
                Ok(new_session_id) => {
                    let session_name = format!("Session {}", new_session_id);
                    write_spaced!("✨ Created new session (ID: {})", new_session_id);
                    utils::write_session_info(new_session_id, &session_name);
                    return;
                }
                Err(e) => {
                    utils::write_error(&format!("Failed to create new session: {}", e));
                    return;
                }
            }
        }

        let session_name = cmd.trim_start_matches("/new ").trim();
        if session_name.is_empty() {
            write_line!("Usage: /new <session_name>");
            return;
        }

        match self.session_manager.create_new_session().await {
            Ok(new_session_id) => match self.session_manager.save_session(session_name).await {
                Ok(_) => {
                    write_spaced!("✨ Created and saved new session: '{}'", session_name);
                    utils::write_session_info(new_session_id, &session_name);
                    ()
                }
                Err(e) => {
                    utils::write_error(&format!("Failed to save new session: {}", e));
                    ()
                }
            },
            Err(e) => {
                utils::write_error(&format!("Failed to create new session: {}", e));
                ()
            }
        }
    }

    async fn handle_save(&self, cmd: &str) {
        let session_name = cmd.trim_start_matches("/save ").trim();
        if session_name.is_empty() {
            write_line!("Usage: /save <session_name>");
            return;
        }

        match self.session_manager.save_session(session_name).await {
            Ok(_) => {
                write_spaced!("✅ Session saved as '{}'", session_name);
                ()
            }
            Err(e) => {
                utils::write_error(&format!("Failed to save session: {}", e));
                ()
            }
        }
    }

    async fn handle_delete(&self, cmd: &str) {
        let session_name = cmd.trim_start_matches("/delete ").trim();
        if session_name.is_empty() {
            write_line!("Usage: /delete <session_name>");
            return;
        }

        // Check if user is trying to delete the current session
        match self.session_manager.get_session_info().await {
            Ok(current_session) => {
                if let Some(current_display_name) = &current_session.display_name {
                    if current_display_name == session_name {
                        utils::write_error("Cannot delete the current session. Switch to another session first using /resume or /new.");
                        return;
                    }
                }
            }
            Err(e) => {
                utils::write_error(&format!("Failed to get current session info: {}", e));
                return;
            }
        }

        match self.session_manager.delete_session(session_name).await {
            Ok(_) => {
                write_spaced!("🗑️ Session '{}' deleted successfully", session_name);
                ()
            }
            Err(e) => {
                utils::write_error(&format!("Failed to delete session: {}", e));
                ()
            }
        }
    }

    async fn handle_resume(&self, cmd: &str) {
        let session_name = cmd.trim_start_matches("/resume ").trim();
        if session_name.is_empty() {
            write_line!("Usage: /resume <session_name>");
            return;
        }

        match self.session_manager.get_session_by_name(session_name).await {
            Ok(new_session_id) => {
                self.session_manager.set_current_session(new_session_id);
                write_spaced!("🔄 Switched to session: '{}'", session_name);
                utils::write_session_info(new_session_id, &session_name);
                ()
            }
            Err(e) => {
                utils::write_error(&format!("Failed to resume session: {}", e));
                ()
            }
        }
    }

    // ===== Session Query Operations =====

    async fn handle_list(&self) {
        match self.session_manager.get_sessions().await {
            Ok(sessions) => {
                if sessions.is_empty() {
                    write_line!("No saved sessions found.");
                    return;
                }

                write_line!("");
                write_line!("📚 Saved Sessions:");
                write_line!("─────────────────");
                for session in sessions {
                    if let Some(display_name) = &session.display_name {
                        if let Some(role) = &session.role {
                            write_line!("• {} (ID: {}) 🎭 {}", display_name, session.id, role);
                        } else {
                            write_line!("• {} (ID: {})", display_name, session.id);
                        }
                    }
                }
                write_line!("");
                ()
            }
            Err(e) => {
                utils::write_error(&format!("Failed to list sessions: {}", e));
                ()
            }
        }
    }

    async fn handle_role(&self, cmd: &str) {
        if cmd == "/role" {
            match self.session_manager.get_session_info().await {
                Ok(session) => {
                    if let Some(role) = session.role {
                        write_line!("🎭 Current role: '{}'", role);
                    } else {
                        write_line!("🎭 No role set (Claude will respond as default assistant)");
                    }
                    return;
                }
                Err(e) => {
                    utils::write_error(&format!("Failed to get session info: {}", e));
                    return;
                }
            }
        }

        let role = cmd.trim_start_matches("/role ").trim();
        if role.is_empty() {
            write_line!("Usage: /role <role_name>");
            return;
        }

        match self.session_manager.set_role(Some(role.to_string())).await {
            Ok(_) => {
                write_spaced!("🎭 Role set to: '{}'", role);
                ()
            }
            Err(e) => {
                utils::write_error(&format!("Failed to set role: {}", e));
                ()
            }
        }
    }

    async fn handle_model(&self, cmd: &str) {
        if cmd == "/model" {
            // Show current model and available models
            match self.session_manager.get_session_info().await {
                Ok(session) => {
                    let current_model = session.model.as_deref().unwrap_or(DEFAULT_MODEL);
                    write_line!("🤖 Current model: '{}'", current_model);

                    write_line!("");
                    write_line!("📋 Available models:");
                    write_line!("─────────────────");

                    match self.session_manager.get_available_models().await {
                        Ok(models) => {
                            for (i, model) in models.iter().enumerate() {
                                let indicator = if model.id == current_model {
                                    "→"
                                } else {
                                    " "
                                };
                                write_line!(
                                    "{} {}. {} - {}",
                                    indicator,
                                    i + 1,
                                    model.id,
                                    model.display_name
                                );
                            }
                            write_line!("");
                            write_line!("💡 Use '/model <number>' to select a model");
                        }
                        Err(e) => {
                            utils::write_error(&format!("Failed to fetch available models: {}", e));
                        }
                    }
                }
                Err(e) => {
                    utils::write_error(&format!("Failed to get session info: {}", e));
                }
            }
            return;
        }

        let model_input = cmd.trim_start_matches("/model ").trim();
        if model_input.is_empty() {
            write_line!("Usage: /model <number>");
            return;
        }

        // Parse model selection (by number)
        match model_input.parse::<usize>() {
            Ok(model_index) => match self.session_manager.get_available_models().await {
                Ok(models) => {
                    if model_index == 0 || model_index > models.len() {
                        utils::write_error(
                            "Invalid model number. Use '/model' to see available options.",
                        );
                        return;
                    }

                    let selected_model = &models[model_index - 1];
                    match self
                        .session_manager
                        .set_model(selected_model.id.clone())
                        .await
                    {
                        Ok(_) => {
                            write_spaced!("🤖 Model set to: '{}'", selected_model.display_name);
                        }
                        Err(e) => {
                            utils::write_error(&format!("Failed to set model: {}", e));
                        }
                    }
                }
                Err(e) => {
                    utils::write_error(&format!("Failed to fetch available models: {}", e));
                }
            },
            Err(_) => {
                utils::write_error(
                    "Please provide a valid number. Use '/model' to see available options.",
                );
            }
        }
    }
}
