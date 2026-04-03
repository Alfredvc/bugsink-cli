use anyhow::{Context, Result};
use crate::cli::AuthCommands;
use crate::client::BugsinkClient;
use crate::config::Config;
use crate::output::Output;
use serde_json::json;
use std::io::{self, BufRead, Write};

pub async fn run(command: &AuthCommands, output: &Output, _url: Option<&str>, _token: Option<&str>) -> Result<()> {
    match command {
        AuthCommands::Login => login(output).await,
        AuthCommands::Status { verify } => status(*verify, output).await,
        AuthCommands::Logout => logout(output).await,
    }
}

async fn login(output: &Output) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Prompt for URL
    write!(stdout, "Bugsink instance URL: ")?;
    stdout.flush()?;
    let mut url = String::new();
    stdin.lock().read_line(&mut url)?;
    let url = url.trim().trim_end_matches('/').to_string();

    if url.is_empty() {
        anyhow::bail!("URL cannot be empty");
    }

    // Open browser to token page
    let token_url = format!("{}/bsmain/auth_tokens/", url);
    eprintln!("Opening {} in your browser...", token_url);
    eprintln!("Create a token and paste it below.");
    eprintln!("Note: Token management requires admin access. If you cannot access this page, ask your Bugsink administrator for a token.");

    if open::that(&token_url).is_err() {
        eprintln!("Could not open browser. Please visit: {}", token_url);
    }

    // Prompt for token
    write!(stdout, "API token: ")?;
    stdout.flush()?;
    let mut token = String::new();
    stdin.lock().read_line(&mut token)?;
    let token = token.trim().to_string();

    if token.is_empty() {
        anyhow::bail!("Token cannot be empty");
    }

    // Verify connection
    eprintln!("Verifying connection...");
    let client = BugsinkClient::new(&url, &token)?;
    client.list("teams/", &[]).await
        .context("Failed to connect to Bugsink. Check your URL and token.")?;

    // Save config
    let config = Config {
        url: Some(url.clone()),
        token: Some(token),
    };
    config.save()?;

    output.print(json!({
        "status": "authenticated",
        "url": url,
    }))?;

    Ok(())
}

async fn status(verify: bool, output: &Output) -> Result<()> {
    let config = Config::load()?;

    match (&config.url, &config.token) {
        (Some(url), Some(_)) => {
            if verify {
                let resolved = Config::resolve(None, None)?;
                let client = BugsinkClient::new(&resolved.url, &resolved.token)?;
                match client.list("teams/", &[]).await {
                    Ok(_) => {
                        output.print(json!({
                            "status": "verified",
                            "url": url,
                        }))?;
                    }
                    Err(e) => {
                        output.print(json!({
                            "status": "error",
                            "url": url,
                            "error": e.to_string(),
                        }))?;
                    }
                }
            } else {
                output.print(json!({
                    "status": "configured",
                    "url": url,
                }))?;
            }
        }
        _ => {
            output.print(json!({
                "status": "not_configured",
            }))?;
        }
    }

    Ok(())
}

async fn logout(output: &Output) -> Result<()> {
    Config::delete()?;
    output.print(json!({
        "status": "logged_out",
    }))?;
    Ok(())
}
