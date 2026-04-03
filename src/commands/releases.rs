use anyhow::Result;
use crate::cli::ReleasesCommands;
use crate::client::BugsinkClient;
use crate::config::Config;
use crate::output::Output;
use serde_json::{json, Value};

pub async fn run(command: &ReleasesCommands, output: &Output, url: Option<&str>, token: Option<&str>, all: bool) -> Result<()> {
    let config = Config::resolve(url, token)?;
    let client = BugsinkClient::new(&config.url, &config.token)?;

    match command {
        ReleasesCommands::List { project } => {
            let project_str = project.to_string();
            let query = vec![("project", project_str.as_str())];
            if all {
                let results = client.list_all("releases/", &query).await?;
                output.print(Value::Array(results))
            } else {
                let page = client.list("releases/", &query).await?;
                output.print(json!({
                    "next": page.next,
                    "previous": page.previous,
                    "results": page.results,
                }))
            }
        }
        ReleasesCommands::Get { id } => {
            let release = client.get(&format!("releases/{}/", id)).await?;
            output.print(release)
        }
        ReleasesCommands::Create { project, version } => {
            let body = json!({"project": project, "version": version});
            let created = client.post("releases/", &body).await?;
            output.print(created)
        }
    }
}
