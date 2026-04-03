use anyhow::Result;
use crate::client::BugsinkClient;
use crate::config::Config;
use crate::output::Output;

pub async fn run(output: &Output, url: Option<&str>, token: Option<&str>) -> Result<()> {
    let config = Config::resolve(url, token)?;
    let client = BugsinkClient::new(&config.url, &config.token)?;
    let schema = client.get_schema().await?;
    output.print(schema)
}
