use clap::Parser;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};
use tracing::debug;

mod db;
mod server;

#[derive(Parser)]
#[command(name = "clai")]
#[command(about = "A CLI tool for clai")]
struct Cli {
    #[arg(long, help = "Resume the last session instead of creating a new one")]
    resume: bool,
    #[arg(long, help = "Resume a specific named session")]
    session: Option<String>,
    #[arg(long, help = "List all saved sessions")]
    list: bool,
    #[arg(long, help = "Delete a saved session by name")]
    delete: Option<String>,
    #[arg(long, help = "Run in server mode")]
    server: bool,
    #[arg(help = "Optional initial message to send to Claude")]
    message: Option<String>,
}

#[derive(Serialize)]
struct CreateSessionRequest {
    name: String,
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct SessionResponse {
    id: i32,
    name: String,
    display_name: Option<String>,
}

#[derive(Serialize)]
struct ChatRequest {
    message: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    response: String,
}

#[derive(Serialize)]
struct SaveSessionRequest {
    display_name: String,
}

async fn get_or_create_session(
    client: &reqwest::Client,
    server_url: &str,
    resume: bool,
    session_name: Option<&str>,
) -> anyhow::Result<i32> {
    if let Some(name) = session_name {
        // Try to get session by name
        let response = client
            .get(&format!(
                "{}/sessions/by-name/{}",
                server_url,
                urlencoding::encode(name)
            ))
            .send()
            .await?;

        if response.status().is_success() {
            let session: SessionResponse = response.json().await?;
            println!("Resuming session: {} (ID: {})", name, session.id);
            return Ok(session.id);
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Session '{}' not found", name);
        } else {
            anyhow::bail!("Failed to get session '{}': {}", name, response.status());
        }
    }

    if resume {
        // Try to get the last session
        let response = client
            .get(&format!("{}/sessions/last", server_url))
            .send()
            .await?;

        if response.status().is_success() {
            let session: SessionResponse = response.json().await?;
            let display_name = session.display_name.as_deref().unwrap_or(&session.name);
            println!("Resuming session: {} (ID: {})", display_name, session.id);
            return Ok(session.id);
        }
    }

    // Create a new session
    let session_name = uuid::Uuid::new_v4().to_string();
    let request = CreateSessionRequest {
        name: session_name,
        display_name: None,
    };

    let response = client
        .post(&format!("{}/sessions", server_url))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let session: SessionResponse = response.json().await?;
        println!("Created new session: {} (ID: {})", session.name, session.id);
        Ok(session.id)
    } else {
        anyhow::bail!("Failed to create session: {}", response.status());
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle server mode
    if cli.server {
        return server::run_server().await;
    }

    let server_url =
        env::var("CLAI_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let client = reqwest::Client::new();

    // Handle list command
    if cli.list {
        list_sessions(&client, &server_url).await?;
        return Ok(());
    }

    // Handle delete command
    if let Some(session_name) = cli.delete {
        delete_session(&client, &server_url, &session_name).await?;
        return Ok(());
    }

    let mut session_id =
        get_or_create_session(&client, &server_url, cli.resume, cli.session.as_deref()).await?;

    // Send initial message if provided
    if let Some(initial_message) = cli.message {
        debug!("Sending initial message: {}", initial_message);
        send_message(&client, &server_url, session_id, &initial_message).await?;
    }

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
                        save_session(&client, &server_url, session_id, session_name).await?;
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
                        match delete_session(&client, &server_url, session_name).await {
                            Ok(_) => {
                                println!("Session '{}' has been deleted.", session_name);
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
                    match list_sessions(&client, &server_url).await {
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
                        match get_session_by_name(&client, &server_url, session_name).await {
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
                send_message(&client, &server_url, session_id, message).await?;
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    Ok(())
}

async fn send_message(
    client: &reqwest::Client,
    server_url: &str,
    session_id: i32,
    message: &str,
) -> anyhow::Result<()> {
    let request = ChatRequest {
        message: message.to_string(),
    };

    let response = client
        .post(&format!("{}/sessions/{}/chat", server_url, session_id))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        let chat_response: ChatResponse = response.json().await?;
        println!("Claude: {}", chat_response.response);
        println!("───────────────────────────────────────────────────────────────────");
    } else {
        eprintln!("Failed to send message: {}", response.status());
    }

    Ok(())
}

async fn save_session(
    client: &reqwest::Client,
    server_url: &str,
    session_id: i32,
    display_name: &str,
) -> anyhow::Result<()> {
    let request = SaveSessionRequest {
        display_name: display_name.to_string(),
    };

    let response = client
        .patch(&format!("{}/sessions/{}", server_url, session_id))
        .json(&request)
        .send()
        .await?;

    if response.status().is_success() {
        println!("✅ Session saved as '{}'", display_name);
    } else {
        eprintln!("Failed to save session: {}", response.status());
    }

    Ok(())
}

async fn delete_session(
    client: &reqwest::Client,
    server_url: &str,
    session_name: &str,
) -> anyhow::Result<()> {
    let response = client
        .delete(&format!(
            "{}/sessions/by-name/{}",
            server_url,
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

async fn get_session_by_name(
    client: &reqwest::Client,
    server_url: &str,
    session_name: &str,
) -> anyhow::Result<i32> {
    let response = client
        .get(&format!(
            "{}/sessions/by-name/{}",
            server_url,
            urlencoding::encode(session_name)
        ))
        .send()
        .await?;

    if response.status().is_success() {
        let session: SessionResponse = response.json().await?;
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

async fn list_sessions(client: &reqwest::Client, server_url: &str) -> anyhow::Result<()> {
    let response = client
        .get(&format!("{}/sessions", server_url))
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
            println!("• {} (ID: {})", display_name, session.id);
        }
    }
    println!("\nUse --session <name> to resume a specific session");
    println!("Use --resume to continue the most recent session");

    Ok(())
}
