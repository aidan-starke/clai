use clap::Parser;

#[derive(Parser)]
#[command(name = "clai")]
#[command(about = "A CLI tool for clai")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Test, // Add your commands here
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(command) => {
            match command {
                Commands::Test => {
                    println!("Running test command...");
                    // Add your test command logic here
                }
            }
        }
        None => {
            println!("Use --help for usage information");
        }
    }

    Ok(())
}
