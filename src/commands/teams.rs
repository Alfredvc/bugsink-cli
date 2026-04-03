use crate::cli::TeamsCommands;
use crate::client::BugsinkClient;
use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use serde_json::Value;

pub async fn run(
    command: &TeamsCommands,
    output: &Output,
    url: Option<&str>,
    token: Option<&str>,
    all: bool,
) -> Result<()> {
    let config = Config::resolve(url, token)?;
    let client = BugsinkClient::new(&config.url, &config.token)?;

    match command {
        TeamsCommands::List => {
            if all {
                let results = client.list_all("teams/", &[]).await?;
                output.print(Value::Array(results))
            } else {
                let page = client.list("teams/", &[]).await?;
                output.print(serde_json::json!({
                    "next": page.next,
                    "previous": page.previous,
                    "results": page.results,
                }))
            }
        }
        TeamsCommands::Get { id } => {
            let team = client.get(&format!("teams/{}/", id)).await?;
            output.print(team)
        }
    }
}
