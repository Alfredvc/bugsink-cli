use crate::cli::ProjectsCommands;
use crate::client::BugsinkClient;
use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use serde_json::{json, Value};

pub async fn run(
    command: &ProjectsCommands,
    output: &Output,
    url: Option<&str>,
    token: Option<&str>,
    all: bool,
) -> Result<()> {
    let config = Config::resolve(url, token)?;
    let client = BugsinkClient::new(&config.url, &config.token)?;

    match command {
        ProjectsCommands::List { team } => {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(team_id) = team {
                query.push(("team", team_id.to_string()));
            }
            let query_refs: Vec<(&str, &str)> =
                query.iter().map(|(k, v)| (*k, v.as_str())).collect();
            if all {
                let results = client.list_all("projects/", &query_refs).await?;
                output.print(Value::Array(results))
            } else {
                let page = client.list("projects/", &query_refs).await?;
                output.print(json!({
                    "next": page.next,
                    "previous": page.previous,
                    "results": page.results,
                }))
            }
        }
        ProjectsCommands::Get { id } => {
            let project = client.get(&format!("projects/{}/", id)).await?;
            output.print(project)
        }
        ProjectsCommands::Create { team, name } => {
            let body = json!({"team": team, "name": name});
            let created = client.post("projects/", &body).await?;
            output.print(created)
        }
    }
}
