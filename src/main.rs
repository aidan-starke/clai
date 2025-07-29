#![feature(iter_map_windows)]
use clap::Parser;
use commands::{CommandHandler, CommandResult};
use console::style;
use session_manager::SessionManager;
use std::env;
use utils::COMMANDS;

mod commands;
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
        return server::run_server(true).await;
    } else {
        utils::ensure_server_running().await?;
    }

    let server_url = env::var("CLAI_SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let session_manager = SessionManager::new(server_url);
    let command_handler = CommandHandler::new(session_manager.clone());

    let (mut session_id, session_name) = session_manager.get_or_create_session(None).await?;

    utils::clear_screen()?;

    write_line!(
        "💬 Chat started! Type {}, {}, or press {} to end the conversation.",
        style("exit").red(),
        style("quit").red(),
        style("Ctrl+C").red()
    );
    utils::write_command_help();
    write_line!("───────────────────────────────────────────────────────────────────");

    utils::write_session_info(session_id, &session_name);

    loop {
        match utils::read_input_with_autocomplete() {
            Ok(input) => {
                let message = input.trim();

                if message.is_empty() {
                    continue;
                }

                // Handle exit commands
                if message.eq_ignore_ascii_case("exit") || message.eq_ignore_ascii_case("quit") {
                    write_spaced!("Goodbye! 👋");
                    break;
                }

                // Handle slash commands
                if message.starts_with('/') {
                    write_line!("");
                    write_line!("You entered: {}", style(message).yellow().bold());

                    // Check if it's a valid command
                    let is_valid_command = COMMANDS
                        .iter()
                        .any(|&cmd| message == cmd || message.starts_with(&format!("{} ", cmd)));

                    if !is_valid_command {
                        utils::write_command_help();
                        continue;
                    }

                    // Handle the command
                    match command_handler.handle_command(message, session_id).await? {
                        CommandResult::Continue => continue,
                        CommandResult::UpdateSession { id } => {
                            session_id = id;
                            continue;
                        }
                    }
                }

                // Handle chat message
                let bar = indicatif::ProgressBar::new_spinner().with_message(style("Claude is thinking...").blue().to_string());
                bar.enable_steady_tick(std::time::Duration::from_millis(100));
                let response = session_manager.send_message(session_id, message).await?;
                bar.finish_and_clear();

                write_spaced!("{}", style("Claude:").blue().bold());

                // Format the response with proper line breaks
                for line in response.lines() {
                    if line.trim().is_empty() {
                        write_line!("");
                    } else {
                        write_line!("{}", line);
                    }
                }

                write_spaced!("───────────────────────────────────────────────────────────────────");
            }
            Err(e) => {
                utils::write_error(&format!("Error reading input: {}", e));
                break;
            }
        }
    }

    Ok(())
}

