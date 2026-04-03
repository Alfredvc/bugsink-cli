use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub url: Option<String>,
    pub token: Option<String>,
}

/// Resolved config with all values guaranteed present.
/// Built from: CLI flags > env vars > config file.
#[derive(Debug)]
pub struct ResolvedConfig {
    pub url: String,
    pub token: String,
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("bugsink");
        Ok(dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    pub fn load() -> Result<Config> {
        Self::load_from(&Self::config_path()?)
    }

    pub fn load_from(path: &std::path::Path) -> Result<Config> {
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
        self.save_to(&Self::config_path()?)
    }

    pub fn save_to(&self, path: &std::path::Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }

    pub fn delete() -> Result<()> {
        let path = Self::config_path()?;
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete config file: {}", path.display()))?;
        }
        Ok(())
    }

    /// Resolve config by merging CLI flags > env vars > config file.
    /// Returns error if url or token cannot be resolved.
    pub fn resolve(cli_url: Option<&str>, cli_token: Option<&str>) -> Result<ResolvedConfig> {
        let file_config = Self::load().unwrap_or_default();
        Self::resolve_with(cli_url, cli_token, file_config)
    }

    fn resolve_with(
        cli_url: Option<&str>,
        cli_token: Option<&str>,
        file_config: Config,
    ) -> Result<ResolvedConfig> {
        let url = cli_url
            .map(|s| s.to_string())
            .or_else(|| std::env::var("BUGSINK_URL").ok())
            .or(file_config.url)
            .context("Bugsink URL not configured. Run `bugsink auth login` or set BUGSINK_URL.")?;

        let token = cli_token
            .map(|s| s.to_string())
            .or_else(|| std::env::var("BUGSINK_TOKEN").ok())
            .or(file_config.token)
            .context(
                "Bugsink token not configured. Run `bugsink auth login` or set BUGSINK_TOKEN.",
            )?;

        // Normalize URL: strip trailing slash
        let url = url.trim_end_matches('/').to_string();

        Ok(ResolvedConfig { url, token })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    fn test_load_missing_file_returns_default() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.json");
        let config = Config::load_from(&path).unwrap();
        assert!(config.url.is_none());
        assert!(config.token.is_none());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");

        let config = Config {
            url: Some("https://bugsink.example.com".to_string()),
            token: Some("abc123".to_string()),
        };
        config.save_to(&path).unwrap();

        let loaded = Config::load_from(&path).unwrap();
        assert_eq!(loaded.url.as_deref(), Some("https://bugsink.example.com"));
        assert_eq!(loaded.token.as_deref(), Some("abc123"));
    }

    #[test]
    #[serial]
    fn test_resolve_cli_flags_take_priority() {
        std::env::remove_var("BUGSINK_URL");
        std::env::remove_var("BUGSINK_TOKEN");

        let resolved =
            Config::resolve(Some("https://cli-flag.example.com"), Some("cli-token")).unwrap();

        assert_eq!(resolved.url, "https://cli-flag.example.com");
        assert_eq!(resolved.token, "cli-token");
    }

    #[test]
    #[serial]
    fn test_resolve_strips_trailing_slash() {
        std::env::remove_var("BUGSINK_URL");
        std::env::remove_var("BUGSINK_TOKEN");

        let resolved = Config::resolve(Some("https://example.com/"), Some("token")).unwrap();

        assert_eq!(resolved.url, "https://example.com");
    }

    #[test]
    #[serial]
    fn test_resolve_missing_url_errors() {
        std::env::remove_var("BUGSINK_URL");
        std::env::remove_var("BUGSINK_TOKEN");

        let result = Config::resolve_with(None, Some("token"), Config::default());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("URL not configured"));
    }

    #[test]
    #[serial]
    fn test_resolve_missing_token_errors() {
        std::env::remove_var("BUGSINK_URL");
        std::env::remove_var("BUGSINK_TOKEN");

        let result = Config::resolve_with(Some("https://example.com"), None, Config::default());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("token not configured"));
    }
}
