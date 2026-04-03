use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_releases_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/releases/"))
        .and(query_param("project", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null,
            "previous": null,
            "results": [{"id": 1, "version": "1.0.0"}]
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args([
            "--url",
            &server.uri(),
            "--token",
            "t",
            "--json",
            "releases",
            "list",
            "--project",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0.0"));
}

#[tokio::test]
async fn test_releases_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/releases/1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "version": "1.0.0", "date_released": "2026-04-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args([
            "--url",
            &server.uri(),
            "--token",
            "t",
            "--json",
            "releases",
            "get",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0.0"));
}

#[tokio::test]
async fn test_releases_create() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/canonical/0/releases/"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 5, "version": "2.0.0"
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args([
            "--url",
            &server.uri(),
            "--token",
            "t",
            "--json",
            "releases",
            "create",
            "--project",
            "1",
            "--version",
            "2.0.0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2.0.0"));
}
