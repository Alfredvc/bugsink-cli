mod cli;
mod client;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use output::Output;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let output = Output::new(cli.json, cli.fields.clone());

    let url_ref = cli.url.as_deref();
    let token_ref = cli.token.as_deref();
    let all = cli.all;

    let result = match &cli.command {
        Commands::Auth { command } => {
            commands::auth::run(command, &output, url_ref, token_ref).await
        }
        Commands::Teams { command } => {
            commands::teams::run(command, &output, url_ref, token_ref, all).await
        }
        Commands::Projects { command } => {
            commands::projects::run(command, &output, url_ref, token_ref, all).await
        }
        Commands::Issues { command } => {
            commands::issues::run(command, &output, url_ref, token_ref, all).await
        }
        Commands::Events { command } => {
            commands::events::run(command, &output, url_ref, token_ref, all).await
        }
        Commands::Releases { command } => {
            commands::releases::run(command, &output, url_ref, token_ref, all).await
        }
        Commands::Describe => commands::describe::run(&output, url_ref, token_ref).await,
        Commands::Update => commands::update::run(&output).await,
    };

    if let Err(e) = result {
        let error_json = serde_json::json!({"error": e.to_string()});
        eprintln!(
            "{}",
            serde_json::to_string(&error_json).unwrap_or_else(|_| e.to_string())
        );
        std::process::exit(1);
    }

    Ok(())
}
