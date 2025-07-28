use clap::Parser;
use session_manager::SessionManager;
use std::env;
use std::io::{self, Write};

mod db;
mod server;
mod session_manager;

#[derive(Parser)]
#[command(name = "clai")]
#[command(about = "A CLI tool for clai")]
struct Cli {
    #[arg(long, help = "Resume the last session")]
    resume: bool,
    #[arg(long, help = "Resume a specific named session")]
    session: Option<String>,
    #[arg(long, help = "List all saved sessions")]
    list: bool,
    #[arg(long, help = "Delete a saved session by name")]
    delete: Option<String>,
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

    // Handle list command
    if cli.list {
        session_manager.list_sessions().await?;
        return Ok(());
    }

    // Handle delete command
    if let Some(session_name) = cli.delete {
        session_manager.delete_session(&session_name).await?;
        return Ok(());
    }

    let mut session_id = session_manager.get_or_create_session(cli.resume, cli.session.as_deref()).await?;

    // Start interactive conversation loop
    println!("💬 Chat started! Type 'exit', 'quit', or press Ctrl+C to end the conversation.");
    println!("💾 Use '/save <name>' to save this session with a name.");
    println!("🗑️ Use '/delete <name>' to delete a saved session.");
    println!("📚 Use '/list' to show all saved sessions.");
    println!("🔄 Use '/resume <name>' to switch to a different session.");
    println!("───────────────────────────────────────────────────────────────────");

    loop {
        // Prompt for user input
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let message = input.trim();

                // Check for exit commands
                if message.is_empty() {
                    continue;
                }
                if message.eq_ignore_ascii_case("exit") || message.eq_ignore_ascii_case("quit") {
                    println!("Goodbye! 👋");
                    break;
                }

                // Check for save command
                if message.starts_with("/save ") {
                    let session_name = message.trim_start_matches("/save ").trim();
                    if !session_name.is_empty() {
                        session_manager.save_session(session_id, session_name).await?;
                        continue;
                    } else {
                        println!("Usage: /save <session_name>");
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
                                eprintln!("Failed to delete session: {}", e);
                                continue;
                            }
                        }
                    } else {
                        println!("Usage: /delete <session_name>");
                        continue;
                    }
                }

                // Check for list command
                if message.trim() == "/list" {
                    match session_manager.list_sessions().await {
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Failed to list sessions: {}", e);
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
                                println!("🔄 Switched to session: '{}'", session_name);
                                session_id = new_session_id;
                                continue;
                            }
                            Err(e) => {
                                eprintln!("Failed to resume session: {}", e);
                                continue;
                            }
                        }
                    } else {
                        println!("Usage: /resume <session_name>");
                        continue;
                    }
                }

                // Send message and get response
                let response = session_manager.send_message(session_id, message).await?;
                println!("Claude: {}", response);
                println!("───────────────────────────────────────────────────────────────────");
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    Ok(())
}
