use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_issues_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/issues/"))
        .and(query_param("project", "1"))
        .and(query_param("sort", "digest_order"))
        .and(query_param("order", "asc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null,
            "previous": null,
            "results": [{"id": 42, "title": "NullPointerException"}]
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
            "issues",
            "list",
            "--project",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("NullPointerException"));
}

#[tokio::test]
async fn test_issues_list_custom_sort() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/issues/"))
        .and(query_param("project", "1"))
        .and(query_param("sort", "last_seen"))
        .and(query_param("order", "asc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null, "previous": null, "results": []
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
            "issues",
            "list",
            "--project",
            "1",
            "--sort",
            "last_seen",
            "--order",
            "asc",
        ])
        .assert()
        .success();
}

#[tokio::test]
async fn test_issues_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/issues/42/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 42, "title": "NullPointerException", "status": "open"
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
            "issues",
            "get",
            "42",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("NullPointerException"));
}
