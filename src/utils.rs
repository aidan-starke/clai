use console::{style, Key, Term};
use std::{env, sync::OnceLock};

use crate::{server, write_line};

pub const COMMANDS: [&str; 7] = ["/clear", "/new", "/save", "/delete", "/list", "/resume", "/role"];

pub static TERM: OnceLock<Term> = OnceLock::new();

pub fn get_term() -> &'static Term {
    TERM.get_or_init(|| Term::stdout())
}

pub fn clear_screen() -> anyhow::Result<()> {
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
    write_line!("💡 Available commands:");

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

pub fn read_input_with_autocomplete() -> Result<String, std::io::Error> {
    let term = get_term();
    let mut input = String::new();
    let mut show_dropdown = false;
    let mut selected_index = 0;
    let mut dropdown_lines = 0;

    let commands = ["/clear", "/new", "/save", "/delete", "/list", "/resume", "/role"];

    loop {
        // Clear any existing dropdown
        if dropdown_lines > 0 {
            for _ in 0..dropdown_lines {
                term.move_cursor_down(1)?;
                term.clear_line()?;
            }
            for _ in 0..dropdown_lines {
                term.move_cursor_up(1)?;
            }
            dropdown_lines = 0;
        }

        // Show current input
        term.clear_line()?;
        term.write_str(&format!("You: {}", input))?;

        // Show dropdown if user typed '/'
        if show_dropdown {
            let filtered_commands: Vec<&str> = commands.iter().filter(|cmd| cmd.starts_with(&input)).copied().collect();

            if !filtered_commands.is_empty() {
                dropdown_lines = filtered_commands.len();
                term.write_line("")?;
                for (i, cmd) in filtered_commands.iter().enumerate() {
                    if i == selected_index {
                        term.write_line(&format!("  → {}", style(cmd).bold()))?;
                    } else {
                        term.write_line(&format!("    {}", style(cmd).dim()))?;
                    }
                }
                // Move cursor back to input line
                for _ in 0..dropdown_lines {
                    term.move_cursor_up(1)?;
                }
                term.move_cursor_up(1)?; // Go above the empty line too
            }
        }

        // Read next key
        let key = term.read_key()?;

        match key {
            Key::Enter => {
                // Select command if dropdown is showing
                if show_dropdown {
                    let filtered_commands: Vec<&str> = commands.iter().filter(|cmd| cmd.starts_with(&input)).copied().collect();

                    if !filtered_commands.is_empty() && selected_index < filtered_commands.len() {
                        input = filtered_commands[selected_index].to_string();
                    }
                }

                // Clear dropdown and finalize
                if dropdown_lines > 0 {
                    for _ in 0..dropdown_lines {
                        term.move_cursor_down(1)?;
                        term.clear_line()?;
                    }
                    for _ in 0..dropdown_lines {
                        term.move_cursor_up(1)?;
                    }
                    term.move_cursor_down(1)?; // Move past the empty line
                    term.clear_line()?;
                    term.move_cursor_up(1)?;
                }

                term.clear_line()?;
                term.write_str(&format!("You: {}", input))?;
                term.write_line("")?;
                return Ok(input);
            }
            Key::Escape => {
                show_dropdown = false;
                selected_index = 0;
                // Dropdown will be cleared on next loop iteration
            }
            Key::ArrowDown => {
                if show_dropdown {
                    let filtered_commands: Vec<&str> = commands.iter().filter(|cmd| cmd.starts_with(&input)).copied().collect();
                    if !filtered_commands.is_empty() {
                        selected_index = (selected_index + 1) % filtered_commands.len();
                    }
                }
            }
            Key::ArrowUp => {
                if show_dropdown {
                    let filtered_commands: Vec<&str> = commands.iter().filter(|cmd| cmd.starts_with(&input)).copied().collect();
                    if !filtered_commands.is_empty() {
                        selected_index = if selected_index == 0 {
                            filtered_commands.len() - 1
                        } else {
                            selected_index - 1
                        };
                    }
                }
            }
            Key::Backspace => {
                if !input.is_empty() {
                    input.pop();
                    if !input.starts_with('/') {
                        show_dropdown = false;
                        selected_index = 0;
                    }
                }
            }
            Key::Char(c) => {
                input.push(c);
                if input.starts_with('/') {
                    show_dropdown = true;
                    selected_index = 0;
                } else {
                    show_dropdown = false;
                }
            }
            _ => {}
        }
    }
}

pub async fn ensure_server_running() -> anyhow::Result<()> {
    let server_url = env::var("CLAI_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Quick health check
    if reqwest::get(&format!("{}/health", server_url)).await.is_ok() {
        return Ok(()); // Server already running
    }

    // Start server
    tokio::spawn(server::run_server(false));

    // Wait for server to be ready, 5 second timeout
    for _ in 0..50 {
        if reqwest::get(&format!("{}/health", server_url)).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Err(anyhow::anyhow!("Server failed to start"))
}
