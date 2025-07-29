use console::{style, Term};
use std::sync::OnceLock;

use crate::write_line;

pub static TERM: OnceLock<Term> = OnceLock::new();

pub fn clear_screen() -> anyhow::Result<()> {
    TERM.get_or_init(|| Term::stdout()).clear_screen()?;
    Ok(())
}

pub fn write_spaced_line(text: &str) {
    write_line!("");
    write_line!("{}", text);
    write_line!("");
}

pub fn write_prompt(text: &str) {
    TERM.get_or_init(|| Term::stdout()).write_str(text).unwrap();
}

pub fn write_error(text: &str) {
    write_line!("");
    write_line!("❌ {}", style(text).red());
    write_line!("");
}

pub fn write_session_info(session_name: &str, session_id: i32) {
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
    write_line!("{}", style("💡 Available commands:").yellow().bold());

    let commands = [
        ("🧹 /clear", "Clear the screen"),
        ("✨ /new [name]", "Create a new session"),
        ("💾 /save <name>", "Save current session with a name"),
        ("🗑️ /delete <name>", "Delete a saved session"),
        ("📚 /list", "Show all saved sessions"),
        ("🔄 /resume <name>", "Switch to a different session"),
        ("🎭 /role [role]", "Set or view current role"),
    ];

    for (command, description) in &commands {
        write_line!("  {} - {}", style(command).yellow(), style(description).dim());
    }
    write_line!("");
}

pub fn auto_style_commands(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();

    // Track which words have been processed as pairs
    let mut processed = vec![false; words.len()];

    // First pass: handle command + argument pairs using map_windows
    let pairs: Vec<_> = words
        .iter()
        .enumerate()
        .map_windows(|[(i, first), (j, second)]| {
            if first.starts_with('/') && first.len() > 1 && second.starts_with('<') && second.ends_with('>') {
                processed[*i] = true;
                processed[*j] = true;
                Some((*i, format!("{} {}", style(first).yellow(), style(second).yellow())))
            } else {
                None
            }
        })
        .filter_map(|x| x)
        .collect();

    // Build result with pairs and individual words
    let mut res = Vec::new();

    for (i, word) in words.iter().enumerate() {
        if processed[i] {
            // Check if this is the start of a pair
            if let Some((_, styled_pair)) = pairs.iter().find(|(pair_i, _)| *pair_i == i) {
                res.push(styled_pair.clone());
            }
            // Skip if this word was part of a pair
        } else {
            // Handle individual words
            if word.starts_with('/') || word.starts_with("Ctrl") && word.len() > 1 {
                res.push(style(word).yellow().to_string());
            } else {
                res.push(word.to_string());
            }
        }
    }

    res.join(" ")
}
