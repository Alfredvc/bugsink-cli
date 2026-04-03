use crate::output::Output;
use anyhow::Result;

pub async fn run(output: &Output) -> Result<()> {
    output.print(serde_json::json!({"status": "not_implemented"}))
}
