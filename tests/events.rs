use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_events_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/events/"))
        .and(query_param("issue", "42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null,
            "previous": null,
            "results": [{"id": "evt-abc", "timestamp": "2026-04-01T12:00:00Z"}]
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
            "events",
            "list",
            "--issue",
            "42",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("evt-abc"));
}

#[tokio::test]
async fn test_events_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/events/evt-abc/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "evt-abc", "data": {"exception": "NullPointerException"}
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
            "events",
            "get",
            "evt-abc",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("NullPointerException"));
}

#[tokio::test]
async fn test_events_stacktrace() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/events/evt-abc/stacktrace/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("## Stacktrace\n\n```\nFile \"app.py\", line 42\n```"),
        )
        .mount(&server)
        .await;

    Command::cargo_bin("bugsink")
        .unwrap()
        .args([
            "--url",
            &server.uri(),
            "--token",
            "t",
            "events",
            "stacktrace",
            "evt-abc",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stacktrace"))
        .stdout(predicate::str::contains("app.py"));
}
