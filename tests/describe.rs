use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_describe() {
    let server = MockServer::start().await;

    let schema = serde_json::json!({
        "openapi": "3.0.3",
        "info": {"title": "Bugsink API", "version": "0.1.0"},
        "paths": {
            "/api/canonical/0/teams/": {
                "get": {"summary": "List teams"}
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/canonical/0/schema/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(schema))
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args(["--url", &server.uri(), "--token", "t", "--json", "describe"])
        .assert()
        .success()
        .stdout(predicate::str::contains("openapi"))
        .stdout(predicate::str::contains("List teams"));
}
