mod cli;
mod client;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Auth { command: _ } => commands::auth::run().await,
        Commands::Teams { command: _ } => commands::teams::run().await,
        Commands::Projects { command: _ } => commands::projects::run().await,
        Commands::Issues { command: _ } => commands::issues::run().await,
        Commands::Events { command: _ } => commands::events::run().await,
        Commands::Releases { command: _ } => commands::releases::run().await,
        Commands::Describe => commands::describe::run().await,
    }
}
