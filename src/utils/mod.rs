pub mod constants;
pub mod error;
pub mod macros;
pub mod types;

use console::{style, Term};
use std::{env, sync::OnceLock};

use crate::{db::ClaiDb, error::Result, server, write_line};

pub static TERM: OnceLock<Term> = OnceLock::new();

pub fn get_term() -> &'static Term {
    TERM.get_or_init(|| Term::stdout())
}

pub fn clear_screen() -> Result<()> {
    get_term().clear_screen()?;
    Ok(())
}

pub fn write_spaced_line(text: &str) {
    write_line!("");
    write_line!("{}", text);
    write_line!("");
}

pub fn write_error(text: &str) {
    write_line!("");
    write_line!("❌ {}", style(text).red());
    write_line!("");
}

pub fn write_session_info(session_id: i32, session_name: &str) {
    write_line!("");
    write_line!(
        "📍 Current session: {} (ID: {})",
        style(session_name).cyan().bold(),
        style(session_id.to_string()).dim()
    );
    write_line!("");
}

pub fn write_command_help() {
    write_line!("");
    write_line!("💡 Available commands:");

    let commands = [
        ("🧹 /clear", "Clear the screen"),
        ("✨ /new [name]", "Create a new session"),
        ("💾 /save <name>", "Save current session with a name"),
        ("🗑️ /delete <name>", "Delete a saved session"),
        ("📚 /list", "Show all saved sessions"),
        ("🔄 /resume <name>", "Switch to a different session"),
        ("🎭 /role [role]", "Set or view current role"),
        (
            "🤖 /model [number]",
            "View or set the AI model for this session",
        ),
    ];

    for (command, description) in &commands {
        write_line!(
            "  {} - {}",
            style(command).yellow(),
            style(description).dim()
        );
    }
    write_line!("");
}

pub async fn ensure_server_running() -> Result<()> {
    let server_url =
        env::var("CLAI_SERVER_URL").unwrap_or_else(|_| constants::DEFAULT_SERVER_URL.to_string());

    // Quick health check
    if reqwest::get(&format!("{}/health", server_url))
        .await
        .is_ok()
    {
        return Ok(()); // Server already running
    }

    // Start server
    tokio::spawn(server::run_server(false));

    // Wait for server to be ready, 5 second timeout
    for _ in 0..50 {
        if reqwest::get(&format!("{}/health", server_url))
            .await
            .is_ok()
        {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Err(crate::error::ClaiError::server("Server failed to start"))
}

pub fn cleanup_old_sessions() {
    tokio::spawn(async {
        tracing::info!("Starting background database cleanup...");
        match ClaiDb::cleanup_old_sessions() {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    tracing::info!("Cleaned up {} old database records", deleted_count);
                } else {
                    tracing::info!("No old sessions to clean up");
                }
            }
            Err(e) => {
                tracing::error!("Failed to cleanup old sessions: {}", e);
            }
        }
    });
}
