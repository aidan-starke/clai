use console::{style, Term};
use std::sync::OnceLock;

use common::error::Result;

use crate::write_line;

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
