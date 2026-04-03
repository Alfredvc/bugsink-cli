use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{header, method, path, query_param};
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
        .args([
            "--url",
            &server.uri(),
            "--token",
            "test-token",
            "--json",
            "teams",
            "list",
        ])
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
        .args([
            "--url",
            &server.uri(),
            "--token",
            "test-token",
            "--json",
            "teams",
            "get",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Backend"));
}

#[tokio::test]
async fn test_teams_list_all() {
    let server = MockServer::start().await;
    let page2_url = format!("{}/api/canonical/0/teams/?page=2", server.uri());

    Mock::given(method("GET"))
        .and(path("/api/canonical/0/teams/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": page2_url,
            "previous": null,
            "results": [{"id": 1, "name": "Backend"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/canonical/0/teams/"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null,
            "previous": null,
            "results": [{"id": 2, "name": "Frontend"}]
        })))
        .expect(1)
        .with_priority(1)
        .mount(&server)
        .await;

    // --all should return a flat array with results from both pages
    let output = Command::cargo_bin("bugsink")
        .unwrap()
        .args([
            "--url",
            &server.uri(),
            "--token",
            "test-token",
            "--json",
            "--all",
            "teams",
            "list",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // --all returns a flat array, not a paginated envelope
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "Backend");
    assert_eq!(arr[1]["name"], "Frontend");
}

#[tokio::test]
async fn test_teams_list_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/canonical/0/teams/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "next": null,
            "previous": null,
            "results": [
                {"id": 1, "name": "Backend", "slug": "backend"},
                {"id": 2, "name": "Frontend", "slug": "frontend"},
            ]
        })))
        .mount(&server)
        .await;

    // --all --fields should return a flat array with only specified fields
    let output = Command::cargo_bin("bugsink")
        .unwrap()
        .args([
            "--url",
            &server.uri(),
            "--token",
            "test-token",
            "--json",
            "--all",
            "--fields",
            "id,name",
            "teams",
            "list",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let arr = json.as_array().unwrap();
    // Should have id and name but NOT slug
    assert_eq!(arr[0]["id"], 1);
    assert_eq!(arr[0]["name"], "Backend");
    assert!(arr[0].get("slug").is_none());
    assert_eq!(arr[1]["id"], 2);
    assert!(arr[1].get("slug").is_none());
}
