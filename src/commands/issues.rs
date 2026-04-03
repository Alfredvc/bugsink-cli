use crate::cli::IssuesCommands;
use crate::client::BugsinkClient;
use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use serde_json::Value;

pub async fn run(
    command: &IssuesCommands,
    output: &Output,
    url: Option<&str>,
    token: Option<&str>,
    all: bool,
) -> Result<()> {
    let config = Config::resolve(url, token)?;
    let client = BugsinkClient::new(&config.url, &config.token)?;

    match command {
        IssuesCommands::List {
            project,
            sort,
            order,
        } => {
            let project_str = project.to_string();
            let query = vec![
                ("project", project_str.as_str()),
                ("sort", sort.as_str()),
                ("order", order.as_str()),
            ];
            if all {
                let results = client.list_all("issues/", &query).await?;
                output.print(Value::Array(results))
            } else {
                let page = client.list("issues/", &query).await?;
                output.print(serde_json::json!({
                    "next": page.next,
                    "previous": page.previous,
                    "results": page.results,
                }))
            }
        }
        IssuesCommands::Get { id } => {
            let issue = client.get(&format!("issues/{}/", id)).await?;
            output.print(issue)
        }
    }
}
