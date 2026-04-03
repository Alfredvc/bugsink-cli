use anyhow::Result;
use serde_json::Value;
use std::io::{self, IsTerminal, Write};

/// Determines output format based on --json flag and TTY detection.
/// - --json flag: always compact JSON
/// - TTY (no flag): pretty-printed JSON
/// - Piped (no flag): compact JSON
pub struct Output {
    force_json: bool,
    fields: Option<Vec<String>>,
}

impl Output {
    pub fn new(force_json: bool, fields: Option<String>) -> Self {
        let fields = fields.map(|f| f.split(',').map(|s| s.trim().to_string()).collect());
        Self { force_json, fields }
    }

    fn is_pretty(&self) -> bool {
        if self.force_json {
            return false;
        }
        io::stdout().is_terminal()
    }

    /// Filter a JSON value to only include specified fields.
    fn filter_fields(&self, value: Value) -> Value {
        let Some(fields) = &self.fields else {
            return value;
        };

        match value {
            Value::Array(arr) => {
                Value::Array(arr.into_iter().map(|v| self.filter_fields(v)).collect())
            }
            Value::Object(map) => {
                let filtered: serde_json::Map<String, Value> = map
                    .into_iter()
                    .filter(|(k, _)| fields.contains(k))
                    .collect();
                Value::Object(filtered)
            }
            other => other,
        }
    }

    /// Print a JSON value to stdout.
    pub fn print(&self, value: Value) -> Result<()> {
        let value = self.filter_fields(value);
        let output = if self.is_pretty() {
            serde_json::to_string_pretty(&value)?
        } else {
            serde_json::to_string(&value)?
        };
        writeln!(io::stdout(), "{}", output)?;
        Ok(())
    }

    /// Print raw text to stdout (used for stacktrace markdown).
    pub fn print_raw(&self, text: &str) -> Result<()> {
        writeln!(io::stdout(), "{}", text)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_fields_object() {
        let output = Output::new(false, Some("id,name".to_string()));
        let input = json!({"id": 1, "name": "test", "extra": "removed"});
        let result = output.filter_fields(input);
        assert_eq!(result, json!({"id": 1, "name": "test"}));
    }

    #[test]
    fn test_filter_fields_array() {
        let output = Output::new(false, Some("id".to_string()));
        let input = json!([{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]);
        let result = output.filter_fields(input);
        assert_eq!(result, json!([{"id": 1}, {"id": 2}]));
    }

    #[test]
    fn test_filter_fields_none_passes_through() {
        let output = Output::new(false, None);
        let input = json!({"id": 1, "name": "test"});
        let result = output.filter_fields(input);
        assert_eq!(result, json!({"id": 1, "name": "test"}));
    }

    #[test]
    fn test_force_json_disables_pretty() {
        let output = Output::new(true, None);
        assert!(!output.is_pretty());
    }
}
