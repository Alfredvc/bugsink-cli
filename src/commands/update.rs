use crate::output::Output;
use anyhow::{bail, Context, Result};
use semver::Version;
use serde_json::Value;
use std::io::Write;

const GITHUB_REPO: &str = "Alfredvc/bugsink-cli";

/// Get the current version, allowing override via env var for testing.
fn current_version() -> Result<Version> {
    let version_str = std::env::var("BUGSINK_CURRENT_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
    Version::parse(&version_str)
        .with_context(|| format!("Failed to parse current version: {}", version_str))
}

/// Get the resolved path of the current executable, allowing override via env var for testing.
fn self_exe_path() -> Result<std::path::PathBuf> {
    if let Ok(override_path) = std::env::var("BUGSINK_SELF_PATH") {
        return Ok(std::path::PathBuf::from(override_path));
    }
    std::env::current_exe()
        .context("Failed to determine current executable path")?
        .canonicalize()
        .context("Failed to canonicalize current executable path")
}

/// Check if the binary was installed via cargo (path contains `.cargo/bin`).
fn is_cargo_installed(path: &std::path::Path) -> bool {
    let components: Vec<_> = path.components().collect();
    components.windows(2).any(|pair| {
        matches!(
            (&pair[0], &pair[1]),
            (
                std::path::Component::Normal(a),
                std::path::Component::Normal(b)
            ) if *a == ".cargo" && *b == "bin"
        )
    })
}

/// Get the GitHub API base URL, allowing override via env var for testing.
fn github_api_base_url() -> String {
    std::env::var("BUGSINK_GITHUB_API_URL")
        .unwrap_or_else(|_| "https://api.github.com".to_string())
}

/// Build a reqwest client for GitHub API requests.
fn build_github_client() -> Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("bugsink-cli"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );

    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        let auth_value = format!("Bearer {}", token);
        let mut header_val = reqwest::header::HeaderValue::from_str(&auth_value)
            .context("Invalid GITHUB_TOKEN format")?;
        header_val.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, header_val);
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .context("Failed to build HTTP client")
}

/// Fetch the latest release tag from GitHub. Returns the tag_name string (e.g. "v0.2.0").
async fn fetch_latest_release(client: &reqwest::Client) -> Result<Value> {
    let url = format!(
        "{}/repos/{}/releases/latest",
        github_api_base_url(),
        GITHUB_REPO
    );

    let response = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch latest release from {}", url))?;

    let status = response.status();

    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::TOO_MANY_REQUESTS
    {
        bail!(
            "GitHub API rate limit exceeded (HTTP {}). \
             Set the GITHUB_TOKEN environment variable with a personal access token \
             to increase the rate limit.",
            status.as_u16()
        );
    }

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        bail!(
            "GitHub API error (HTTP {}): {}",
            status.as_u16(),
            body
        );
    }

    response
        .json::<Value>()
        .await
        .context("Failed to parse GitHub release response as JSON")
}

/// Parse a version from a GitHub release tag_name (strips leading 'v' if present).
fn parse_release_version(release: &Value) -> Result<Version> {
    let tag_name = release["tag_name"]
        .as_str()
        .context("GitHub release response missing 'tag_name' field")?;

    let version_str = tag_name.strip_prefix('v').unwrap_or(tag_name);
    Version::parse(version_str)
        .with_context(|| format!("Release tag '{}' is not a valid semver version", tag_name))
}

/// Detect the current platform and return (os, arch) strings suitable for artifact names.
/// Maps "macos" to "darwin". Only supports linux/macos + x86_64/aarch64.
fn detect_platform() -> Result<(&'static str, &'static str)> {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        other => bail!(
            "Self-update is not supported on this platform (OS: {})",
            other
        ),
    };

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => bail!(
            "Self-update is not supported on this platform (arch: {})",
            other
        ),
    };

    Ok((os, arch))
}

/// Get the GitHub download base URL, allowing override via env var for testing.
fn github_download_base_url() -> String {
    std::env::var("BUGSINK_GITHUB_DOWNLOAD_URL")
        .unwrap_or_else(|_| format!("https://github.com/{}/releases/download", GITHUB_REPO))
}

/// Build the download URL for a release artifact.
fn download_url(version: &Version, os: &str, arch: &str) -> String {
    format!(
        "{}/v{}/bugsink-{}-{}.tar.gz",
        github_download_base_url(),
        version,
        os,
        arch
    )
}

/// Download the tarball to a temporary file in the given directory.
async fn download_tarball(
    client: &reqwest::Client,
    url: &str,
    dir: &std::path::Path,
    os: &str,
    arch: &str,
) -> Result<tempfile::NamedTempFile> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to download release from {}", url))?;

    let status = response.status();
    if !status.is_success() {
        bail!(
            "No matching release artifact for this platform (OS: {}, arch: {}). \
             Expected asset at {}",
            os,
            arch,
            url
        );
    }

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read release download from {}", url))?;

    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .context("Failed to create temporary file for download")?;

    tmp.write_all(&bytes)
        .context("Failed to write downloaded data to temporary file")?;
    tmp.flush()
        .context("Failed to flush temporary file")?;

    Ok(tmp)
}

/// Extract the `bugsink` binary from a .tar.gz archive.
/// Returns the path to the extracted binary (a new temp file in the same directory).
fn extract_binary(
    tarball: &std::path::Path,
    dir: &std::path::Path,
) -> Result<tempfile::NamedTempFile> {
    let file = std::fs::File::open(tarball)
        .with_context(|| format!("Failed to open tarball: {}", tarball.display()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    let mut extracted = tempfile::NamedTempFile::new_in(dir)
        .context("Failed to create temporary file for extracted binary")?;

    let mut found = false;
    for entry_result in archive.entries().context("Failed to read tar entries")? {
        let mut entry = entry_result.context("Failed to read tar entry")?;
        let path = entry
            .path()
            .context("Failed to read entry path from tar archive")?;

        // The archive should contain a single file named "bugsink" at the root.
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if file_name == "bugsink" {
            std::io::copy(&mut entry, &mut extracted)
                .context("Failed to extract bugsink binary from archive")?;
            found = true;
            break;
        }
    }

    if !found {
        bail!("Corrupt archive: no 'bugsink' binary found in the release tarball");
    }

    extracted
        .flush()
        .context("Failed to flush extracted binary")?;

    Ok(extracted)
}

/// Replace the current binary with the new one. Sets executable permissions on Unix.
fn replace_binary(
    new_binary: tempfile::NamedTempFile,
    target_path: &std::path::Path,
) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(new_binary.path(), permissions)
            .context("Failed to set executable permissions on new binary")?;
    }

    // Persist (rename) the temp file to the target path.
    // NamedTempFile::persist does an atomic rename on the same filesystem.
    new_binary.persist(target_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to replace binary at '{}': {}. Check file permissions.",
            target_path.display(),
            e.error
        )
    })?;

    Ok(())
}

pub async fn run(output: &Output) -> Result<()> {
    // Step 1: Detect current version
    let current = current_version()?;

    // Step 2: Detect installation method
    let exe_path = self_exe_path()?;
    if is_cargo_installed(&exe_path) {
        bail!(
            "This binary appears to be installed via cargo ({}). \
             Please update using: cargo install --git https://github.com/{}",
            exe_path.display(),
            GITHUB_REPO
        );
    }

    // Step 3: Fetch latest release from GitHub
    let client = build_github_client()?;
    let release = fetch_latest_release(&client).await?;

    // Step 4: Compare versions
    let latest = parse_release_version(&release)?;
    if latest <= current {
        return output.print(serde_json::json!({
            "status": "up_to_date",
            "version": current.to_string()
        }));
    }

    // Step 5: Detect platform
    let (os, arch) = detect_platform()?;

    // Step 6: Download tarball
    let url = download_url(&latest, os, arch);
    let exe_dir = exe_path
        .parent()
        .context("Failed to determine parent directory of current executable")?;
    let tarball = download_tarball(&client, &url, exe_dir, os, arch).await?;

    // Step 7: Extract binary
    let new_binary = extract_binary(tarball.path(), exe_dir)?;

    // Clean up tarball temp file explicitly (extracted binary is what we need)
    drop(tarball);

    // Step 8: Replace binary
    replace_binary(new_binary, &exe_path)?;

    // Step 9: Output
    output.print(serde_json::json!({
        "status": "updated",
        "previous_version": current.to_string(),
        "new_version": latest.to_string()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Version comparison tests ---

    #[test]
    fn test_version_current_less_than_latest() {
        let current = Version::parse("0.1.0").unwrap();
        let latest = Version::parse("0.2.0").unwrap();
        assert!(latest > current);
    }

    #[test]
    fn test_version_current_equals_latest() {
        let current = Version::parse("0.1.0").unwrap();
        let latest = Version::parse("0.1.0").unwrap();
        assert!(latest <= current);
    }

    #[test]
    fn test_version_current_greater_than_latest() {
        let current = Version::parse("0.3.0").unwrap();
        let latest = Version::parse("0.2.0").unwrap();
        assert!(latest <= current);
    }

    #[test]
    fn test_parse_release_version_with_v_prefix() {
        let release = serde_json::json!({"tag_name": "v1.2.3"});
        let version = parse_release_version(&release).unwrap();
        assert_eq!(version, Version::parse("1.2.3").unwrap());
    }

    #[test]
    fn test_parse_release_version_without_v_prefix() {
        let release = serde_json::json!({"tag_name": "1.2.3"});
        let version = parse_release_version(&release).unwrap();
        assert_eq!(version, Version::parse("1.2.3").unwrap());
    }

    #[test]
    fn test_parse_release_version_missing_tag_name() {
        let release = serde_json::json!({"name": "some release"});
        let result = parse_release_version(&release);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing 'tag_name'")
        );
    }

    #[test]
    fn test_parse_release_version_invalid_semver() {
        let release = serde_json::json!({"tag_name": "not-a-version"});
        let result = parse_release_version(&release);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("not a valid semver")
        );
    }

    // --- Platform detection tests ---

    #[test]
    fn test_detect_platform_returns_supported() {
        // This test runs on the current platform, so it should succeed on CI (linux/macos).
        let result = detect_platform();
        match std::env::consts::OS {
            "macos" => {
                let (os, _arch) = result.unwrap();
                assert_eq!(os, "darwin");
            }
            "linux" => {
                let (os, _arch) = result.unwrap();
                assert_eq!(os, "linux");
            }
            _ => {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_platform_mapping_in_download_url() {
        let version = Version::parse("0.2.0").unwrap();
        let url = download_url(&version, "darwin", "aarch64");
        assert_eq!(
            url,
            "https://github.com/Alfredvc/bugsink-cli/releases/download/v0.2.0/bugsink-darwin-aarch64.tar.gz"
        );

        let url = download_url(&version, "linux", "x86_64");
        assert_eq!(
            url,
            "https://github.com/Alfredvc/bugsink-cli/releases/download/v0.2.0/bugsink-linux-x86_64.tar.gz"
        );
    }

    // --- Cargo-install detection tests ---

    #[test]
    fn test_cargo_installed_detected() {
        let path = std::path::PathBuf::from("/home/user/.cargo/bin/bugsink");
        assert!(is_cargo_installed(&path));
    }

    #[test]
    fn test_cargo_installed_not_detected_for_normal_path() {
        let path = std::path::PathBuf::from("/usr/local/bin/bugsink");
        assert!(!is_cargo_installed(&path));
    }

    #[test]
    fn test_cargo_installed_not_detected_for_local_bin() {
        let path = std::path::PathBuf::from("/home/user/.local/bin/bugsink");
        assert!(!is_cargo_installed(&path));
    }

    #[test]
    fn test_cargo_installed_detected_nested() {
        let path = std::path::PathBuf::from("/Users/someone/.cargo/bin/bugsink");
        assert!(is_cargo_installed(&path));
    }

    // --- Extract binary tests ---

    #[test]
    fn test_extract_binary_valid_archive() {
        let dir = tempfile::tempdir().unwrap();
        let tarball_path = dir.path().join("test.tar.gz");

        // Create a tar.gz with a "bugsink" entry
        let tarball_file = std::fs::File::create(&tarball_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(tarball_file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);

        let content = b"fake binary content";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append_data(&mut header, "bugsink", &content[..])
            .unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();

        let result = extract_binary(&tarball_path, dir.path());
        assert!(result.is_ok());

        let extracted = result.unwrap();
        let extracted_content = std::fs::read(extracted.path()).unwrap();
        assert_eq!(extracted_content, content);
    }

    #[test]
    fn test_extract_binary_missing_entry() {
        let dir = tempfile::tempdir().unwrap();
        let tarball_path = dir.path().join("test.tar.gz");

        // Create a tar.gz without a "bugsink" entry
        let tarball_file = std::fs::File::create(&tarball_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(tarball_file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);

        let content = b"some other file";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "not-bugsink", &content[..])
            .unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();

        let result = extract_binary(&tarball_path, dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Corrupt archive"));
    }
}
