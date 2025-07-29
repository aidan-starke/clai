#![feature(iter_map_windows)]
use clap::Parser;
use console::style;
use session_manager::SessionManager;
use std::env;
use std::io;

mod db;
mod macros;
mod server;
mod session_manager;
mod utils;

#[derive(Parser)]
#[command(name = "clai")]
#[command(about = "Command Line Artificial Interface (CLAI)")]
struct Cli {
    #[arg(long, help = "Run server")]
    server: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle server mode
    if cli.server {
        return server::run_server().await;
    }

    let server_url = env::var("CLAI_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let session_manager = SessionManager::new(server_url);

    let (mut session_id, mut current_session_name) = session_manager.get_or_create_session(None).await?;

    utils::clear_screen()?;

    write_line!(
        "💬 Chat started! Type {}, {}, or press Ctrl+C to end the conversation.",
        style("exit").red(),
        style("quit").red(),
    );
    write_line!("🧹 Use /clear to clear the screen");
    write_line!("✨ Use /new or /new <name> to create a new session.");
    write_line!("💾 Use /save <name> to save this session with a name.");
    write_line!("🗑️ Use /delete <name> to delete a saved session.");
    write_line!("📚 Use /list to show all saved sessions.");
    write_line!("🔄 Use /resume <name> to switch to a different session.");
    write_line!("🎭 Use /role <role_name> to set role, /role to view current role.");
    write_line!("───────────────────────────────────────────────────────────────────");
    
    utils::write_session_info(&current_session_name, session_id);

    loop {
        // Prompt for user input
        utils::write_prompt(&format!("{} ", style("You:").green().bold()));

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let message = input.trim();

                // Check for exit commands
                if message.is_empty() {
                    continue;
                }
                if message.eq_ignore_ascii_case("exit") || message.eq_ignore_ascii_case("quit") {
                    write_spaced!("Goodbye! 👋");
                    break;
                }

                // Check for save command
                if message.starts_with("/save ") {
                    let session_name = message.trim_start_matches("/save ").trim();
                    if !session_name.is_empty() {
                        session_manager.save_session(session_id, session_name).await?;
                        continue;
                    } else {
                        write_line!("Usage: /save <session_name>");
                        continue;
                    }
                }

                // Check for delete command
                if message.starts_with("/delete ") {
                    let session_name = message.trim_start_matches("/delete ").trim();
                    if !session_name.is_empty() {
                        match session_manager.delete_session(session_name).await {
                            Ok(_) => {
                                continue;
                            }
                            Err(e) => {
                                utils::write_error(&format!("Failed to delete session: {}", e));
                                continue;
                            }
                        }
                    } else {
                        write_line!("Usage: /delete <session_name>");
                        continue;
                    }
                }

                // Check for list command
                if message.trim() == "/list" {
                    match session_manager.list_sessions().await {
                        Ok(_) => continue,
                        Err(e) => {
                            utils::write_error(&format!("Failed to list sessions: {}", e));
                            continue;
                        }
                    }
                }

                // Check for resume command
                if message.starts_with("/resume ") {
                    let session_name = message.trim_start_matches("/resume ").trim();
                    if !session_name.is_empty() {
                        match session_manager.get_session_by_name(session_name).await {
                            Ok(new_session_id) => {
                                write_spaced!("🔄 Switched to session: '{}'", session_name);
                                session_id = new_session_id;
                                current_session_name = session_name.to_string();
                                utils::write_session_info(&current_session_name, session_id);
                                continue;
                            }
                            Err(e) => {
                                utils::write_error(&format!("Failed to resume session: {}", e));
                                continue;
                            }
                        }
                    } else {
                        write_line!("Usage: /resume <session_name>");
                        continue;
                    }
                }

                // Check for role command
                if message.starts_with("/role") {
                    if message == "/role" {
                        // Show current role if just "/role" with no arguments
                        match session_manager.get_session_info(session_id).await {
                            Ok(session) => {
                                if let Some(role) = session.role {
                                    write_line!("🎭 Current role: '{}'", role);
                                } else {
                                    write_line!("🎭 No role set (Claude will respond as default assistant)");
                                }
                                continue;
                            }
                            Err(e) => {
                                utils::write_error(&format!("Failed to get session info: {}", e));
                                continue;
                            }
                        }
                    } else if message.starts_with("/role ") {
                        let role = message.trim_start_matches("/role ").trim();
                        if !role.is_empty() {
                            // Set role
                            match session_manager.set_role(session_id, Some(role.to_string())).await {
                                Ok(_) => {
                                    write_spaced!("🎭 Role set to: '{}'", role);
                                    continue;
                                }
                                Err(e) => {
                                    utils::write_error(&format!("Failed to set role: {}", e));
                                    continue;
                                }
                            }
                        } else {
                            write_line!("Usage: /role <role_name>");
                            continue;
                        }
                    }
                }

                if message.trim() == "/clear" {
                    // Clear the screen
                    utils::clear_screen()?;
                    continue;
                }

                // Check for new command
                if message.starts_with("/new") {
                    if message == "/new" {
                        // Create new session with auto-generated name
                        match session_manager.create_new_session().await {
                            Ok(new_session_id) => {
                                session_id = new_session_id;
                                current_session_name = format!("Session {}", new_session_id);
                                write_spaced!("✨ Created new session (ID: {})", new_session_id);
                                utils::write_session_info(&current_session_name, session_id);
                                continue;
                            }
                            Err(e) => {
                                utils::write_error(&format!("Failed to create new session: {}", e));
                                continue;
                            }
                        }
                    } else if message.starts_with("/new ") {
                        let session_name = message.trim_start_matches("/new ").trim();
                        if !session_name.is_empty() {
                            // Create new session and immediately save it with the given name
                            match session_manager.create_new_session().await {
                                Ok(new_session_id) => {
                                    match session_manager.save_session(new_session_id, session_name).await {
                                        Ok(_) => {
                                            session_id = new_session_id;
                                            current_session_name = session_name.to_string();
                                            write_spaced!("✨ Created and saved new session: '{}'", session_name);
                                            utils::write_session_info(&current_session_name, session_id);
                                            continue;
                                        }
                                        Err(e) => {
                                            utils::write_error(&format!("Failed to save new session: {}", e));
                                            continue;
                                        }
                                    }
                                }
                                Err(e) => {
                                    utils::write_error(&format!("Failed to create new session: {}", e));
                                    continue;
                                }
                            }
                        } else {
                            write_line!("Usage: /new <session_name>");
                            continue;
                        }
                    }
                }

                let bar = indicatif::ProgressBar::new_spinner().with_message(style("Claude is thinking...").blue().to_string());
                bar.enable_steady_tick(std::time::Duration::from_millis(100));
                let response = session_manager.send_message(session_id, message).await?;
                bar.finish_and_clear();

                utils::write_blank_line();
                write_line!("{}", style("Claude:").blue().bold());
                utils::write_blank_line();
                
                // Format the response with proper line breaks
                for line in response.lines() {
                    if line.trim().is_empty() {
                        utils::write_blank_line();
                    } else {
                        write_line!("{}", line);
                    }
                }
                
                utils::write_blank_line();
                write_line!("───────────────────────────────────────────────────────────────────");
                utils::write_blank_line();
            }
            Err(e) => {
                utils::write_error(&format!("Error reading input: {}", e));
                break;
            }
        }
    }

    Ok(())
}
