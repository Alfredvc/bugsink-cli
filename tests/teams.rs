use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_teams_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/teams/"))
        .and(header("Authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null,
            "previous": null,
            "results": [
                {"id": 1, "name": "Backend"},
                {"id": 2, "name": "Frontend"},
            ]
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--url", &server.uri(), "--token", "test-token", "--json", "teams", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Backend"))
        .stdout(predicate::str::contains("Frontend"));
}

#[tokio::test]
async fn test_teams_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/teams/1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1, "name": "Backend", "slug": "backend"
        })))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--url", &server.uri(), "--token", "test-token", "--json", "teams", "get", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Backend"));
}
