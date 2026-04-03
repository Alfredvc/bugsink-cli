use assert_cmd::Command;
use flate2::write::GzEncoder;
use flate2::Compression;
use serial_test::serial;
use std::io::Write;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Detect the current platform using the same mapping as the update command.
fn current_platform() -> (&'static str, &'static str) {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        other => panic!("Unsupported OS for test: {}", other),
    };
    let arch = std::env::consts::ARCH; // x86_64 or aarch64
    (os, arch)
}

/// Create a fake .tar.gz archive containing a single file named "bugsink".
fn create_fake_tarball(binary_content: &[u8]) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let mut header = tar::Header::new_gnu();
        header.set_size(binary_content.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append_data(&mut header, "bugsink", binary_content)
            .unwrap();
        builder.into_inner().unwrap();
    }
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes).unwrap();
    encoder.finish().unwrap()
}

#[tokio::test]
#[serial]
async fn test_update_up_to_date() {
    let server = MockServer::start().await;

    // Mock GitHub API: latest release has the same version as current (0.1.0)
    Mock::given(method("GET"))
        .and(path("/repos/Alfredvc/bugsink-cli/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": "v0.1.0",
            "name": "v0.1.0"
        })))
        .expect(1)
        .mount(&server)
        .await;

    // BUGSINK_SELF_PATH must point to a real file outside .cargo/bin
    let tmp_dir = tempfile::tempdir().unwrap();
    let self_path = tmp_dir.path().join("bugsink");
    std::fs::write(&self_path, b"old binary").unwrap();

    let output = Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--json", "update"])
        .env("BUGSINK_GITHUB_API_URL", server.uri())
        .env("BUGSINK_CURRENT_VERSION", "0.1.0")
        .env("BUGSINK_SELF_PATH", self_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "up_to_date");
    assert_eq!(json["version"], "0.1.0");
}

#[tokio::test]
#[serial]
async fn test_update_successful() {
    let server = MockServer::start().await;
    let (os, arch) = current_platform();

    // Mock GitHub API: latest release is newer than current
    Mock::given(method("GET"))
        .and(path("/repos/Alfredvc/bugsink-cli/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag_name": "v0.2.0",
            "name": "v0.2.0"
        })))
        .expect(1)
        .mount(&server)
        .await;

    // Create a fake tarball with known binary content
    let new_binary_content = b"this is the new bugsink binary v0.2.0";
    let tarball = create_fake_tarball(new_binary_content);

    // Mock the download endpoint
    let download_path = format!("/v0.2.0/bugsink-{}-{}.tar.gz", os, arch);
    Mock::given(method("GET"))
        .and(path(&download_path))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(tarball)
                .insert_header("content-type", "application/octet-stream"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // Create a temp file to serve as the current binary
    let tmp_dir = tempfile::tempdir().unwrap();
    let self_path = tmp_dir.path().join("bugsink");
    std::fs::write(&self_path, b"old binary content").unwrap();

    let output = Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--json", "update"])
        .env("BUGSINK_GITHUB_API_URL", server.uri())
        .env("BUGSINK_GITHUB_DOWNLOAD_URL", server.uri())
        .env("BUGSINK_CURRENT_VERSION", "0.1.0")
        .env("BUGSINK_SELF_PATH", self_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["status"], "updated");
    assert_eq!(json["previous_version"], "0.1.0");
    assert_eq!(json["new_version"], "0.2.0");

    // Verify the file was replaced with the new binary content
    let replaced_content = std::fs::read(&self_path).unwrap();
    assert_eq!(replaced_content, new_binary_content);
}

#[tokio::test]
#[serial]
async fn test_update_rate_limit_error() {
    let server = MockServer::start().await;

    // Mock GitHub API returning 403 (rate limit)
    Mock::given(method("GET"))
        .and(path("/repos/Alfredvc/bugsink-cli/releases/latest"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "API rate limit exceeded"
        })))
        .expect(1)
        .mount(&server)
        .await;

    // BUGSINK_SELF_PATH must point to a real file outside .cargo/bin
    let tmp_dir = tempfile::tempdir().unwrap();
    let self_path = tmp_dir.path().join("bugsink");
    std::fs::write(&self_path, b"old binary").unwrap();

    let output = Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--json", "update"])
        .env("BUGSINK_GITHUB_API_URL", server.uri())
        .env("BUGSINK_CURRENT_VERSION", "0.1.0")
        .env("BUGSINK_SELF_PATH", self_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr_json: serde_json::Value = serde_json::from_str(&stderr).unwrap();
    let error_msg = stderr_json["error"].as_str().unwrap();
    assert!(
        error_msg.contains("rate limit"),
        "Expected rate limit message, got: {}",
        error_msg
    );
    assert!(
        error_msg.contains("GITHUB_TOKEN"),
        "Expected GITHUB_TOKEN mention, got: {}",
        error_msg
    );
}

#[tokio::test]
#[serial]
async fn test_update_rate_limit_error_429() {
    let server = MockServer::start().await;

    // Mock GitHub API returning 429 (rate limit)
    Mock::given(method("GET"))
        .and(path("/repos/Alfredvc/bugsink-cli/releases/latest"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "message": "API rate limit exceeded"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let tmp_dir = tempfile::tempdir().unwrap();
    let self_path = tmp_dir.path().join("bugsink");
    std::fs::write(&self_path, b"old binary").unwrap();

    let output = Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--json", "update"])
        .env("BUGSINK_GITHUB_API_URL", server.uri())
        .env("BUGSINK_CURRENT_VERSION", "0.1.0")
        .env("BUGSINK_SELF_PATH", self_path.to_str().unwrap())
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr_json: serde_json::Value = serde_json::from_str(&stderr).unwrap();
    let error_msg = stderr_json["error"].as_str().unwrap();
    assert!(
        error_msg.contains("rate limit"),
        "Expected rate limit message, got: {}",
        error_msg
    );
}

#[tokio::test]
#[serial]
async fn test_update_cargo_install_refusal() {
    // No mock server needed: cargo-install check exits before any API call
    let cargo_path = "/home/testuser/.cargo/bin/bugsink";

    let output = Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--json", "update"])
        .env("BUGSINK_CURRENT_VERSION", "0.1.0")
        .env("BUGSINK_SELF_PATH", cargo_path)
        .output()
        .unwrap();

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr_json: serde_json::Value = serde_json::from_str(&stderr).unwrap();
    let error_msg = stderr_json["error"].as_str().unwrap();
    assert!(
        error_msg.contains("cargo"),
        "Expected cargo install refusal, got: {}",
        error_msg
    );
}
