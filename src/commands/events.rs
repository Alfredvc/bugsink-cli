use crate::cli::EventsCommands;
use crate::client::BugsinkClient;
use crate::config::Config;
use crate::output::Output;
use anyhow::Result;
use serde_json::Value;

pub async fn run(
    command: &EventsCommands,
    output: &Output,
    url: Option<&str>,
    token: Option<&str>,
    all: bool,
) -> Result<()> {
    let config = Config::resolve(url, token)?;
    let client = BugsinkClient::new(&config.url, &config.token)?;

    match command {
        EventsCommands::List { issue, order } => {
            let issue_str = issue.to_string();
            let query = vec![("issue", issue_str.as_str()), ("order", order.as_str())];
            if all {
                let results = client.list_all("events/", &query).await?;
                output.print(Value::Array(results))
            } else {
                let page = client.list("events/", &query).await?;
                output.print(serde_json::json!({
                    "next": page.next,
                    "previous": page.previous,
                    "results": page.results,
                }))
            }
        }
        EventsCommands::Get { id } => {
            let event = client.get(&format!("events/{}/", id)).await?;
            output.print(event)
        }
        EventsCommands::Stacktrace { id } => {
            let text = client
                .get_text(&format!("events/{}/stacktrace/", id))
                .await?;
            output.print_raw(&text)
        }
    }
}
