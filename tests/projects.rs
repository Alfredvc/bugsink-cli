use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_projects_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/projects/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null,
            "previous": null,
            "results": [{"id": 1, "name": "My App"}]
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--url", &server.uri(), "--token", "t", "--json", "projects", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("My App"));
}

#[tokio::test]
async fn test_projects_list_by_team() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/projects/"))
        .and(query_param("team", "3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null,
            "previous": null,
            "results": [{"id": 5, "name": "Team Project"}]
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--url", &server.uri(), "--token", "t", "--json", "projects", "list", "--team", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Team Project"));
}

#[tokio::test]
async fn test_projects_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/projects/1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "name": "My App", "slug": "my-app"
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--url", &server.uri(), "--token", "t", "--json", "projects", "get", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-app"));
}

#[tokio::test]
async fn test_projects_create() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/canonical/0/projects/"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": 10, "name": "New Project"
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--url", &server.uri(), "--token", "t", "--json", "projects", "create", "--team", "1", "--name", "New Project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("New Project"));
}
