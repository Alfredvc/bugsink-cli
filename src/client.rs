use anyhow::{bail, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct PaginatedResponse {
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<Value>,
}

pub struct BugsinkClient {
    http: reqwest::Client,
    base_url: String,
}

impl BugsinkClient {
    pub fn new(url: &str, token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bearer {}", token);
        let mut header_val = HeaderValue::from_str(&auth_value).context("Invalid token format")?;
        header_val.set_sensitive(true);
        headers.insert(AUTHORIZATION, header_val);

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        let base_url = url.to_string();

        Ok(Self { http, base_url })
    }

    fn api_url(&self, path: &str) -> String {
        format!(
            "{}/api/canonical/0/{}",
            self.base_url,
            path.trim_start_matches('/')
        )
    }

    /// GET a single resource, returns the JSON value.
    pub async fn get(&self, path: &str) -> Result<Value> {
        let url = self.api_url(path);
        let response = self
            .http
            .get(&url)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .with_context(|| format!("Request failed: GET {}", url))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("API error ({}): {}", status, body);
        }

        response
            .json::<Value>()
            .await
            .with_context(|| format!("Failed to parse JSON from: GET {}", url))
    }

    /// GET a paginated list. Returns one page of results.
    pub async fn list(&self, path: &str, query: &[(&str, &str)]) -> Result<PaginatedResponse> {
        let url = self.api_url(path);
        let response = self
            .http
            .get(&url)
            .header(ACCEPT, "application/json")
            .query(query)
            .send()
            .await
            .with_context(|| format!("Request failed: GET {}", url))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("API error ({}): {}", status, body);
        }

        response
            .json::<PaginatedResponse>()
            .await
            .with_context(|| format!("Failed to parse paginated response from: GET {}", url))
    }

    /// GET all pages of a paginated list.
    pub async fn list_all(&self, path: &str, query: &[(&str, &str)]) -> Result<Vec<Value>> {
        let mut all_results = Vec::new();

        // First page
        let first_page = self.list(path, query).await?;
        all_results.extend(first_page.results);
        let mut next_url = first_page.next;

        // Follow next pages
        while let Some(url) = next_url {
            let response = self
                .http
                .get(&url)
                .header(ACCEPT, "application/json")
                .send()
                .await
                .with_context(|| format!("Request failed: GET {}", url))?;

            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                bail!("API error ({}): {}", status, body);
            }

            let page: PaginatedResponse = response
                .json()
                .await
                .with_context(|| format!("Failed to parse paginated response from: GET {}", url))?;
            all_results.extend(page.results);
            next_url = page.next;
        }

        Ok(all_results)
    }

    /// POST a resource with a JSON body.
    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = self.api_url(path);
        let response = self
            .http
            .post(&url)
            .header(ACCEPT, "application/json")
            .json(body)
            .send()
            .await
            .with_context(|| format!("Request failed: POST {}", url))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("API error ({}): {}", status, body);
        }

        response
            .json::<Value>()
            .await
            .with_context(|| format!("Failed to parse JSON from: POST {}", url))
    }

    /// GET raw text (used for stacktrace markdown endpoint).
    pub async fn get_text(&self, path: &str) -> Result<String> {
        let url = self.api_url(path);
        let response = self
            .http
            .get(&url)
            .header("Accept", "text/plain")
            .send()
            .await
            .with_context(|| format!("Request failed: GET {}", url))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("API error ({}): {}", status, body);
        }

        response
            .text()
            .await
            .with_context(|| format!("Failed to read text from: GET {}", url))
    }

    /// Fetch the raw OpenAPI schema JSON.
    pub async fn get_schema(&self) -> Result<Value> {
        let url = format!("{}/api/canonical/0/schema/", self.base_url);
        let response = self
            .http
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .with_context(|| format!("Request failed: GET {}", url))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("API error ({}): {}", status, body);
        }

        response
            .json::<Value>()
            .await
            .with_context(|| format!("Failed to parse schema from: GET {}", url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/canonical/0/teams/1/"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": 1, "name": "Test Team"})),
            )
            .mount(&server)
            .await;

        let client = BugsinkClient::new(&server.uri(), "test-token").unwrap();
        let result = client.get("teams/1/").await.unwrap();
        assert_eq!(result["id"], 1);
        assert_eq!(result["name"], "Test Team");
    }

    #[tokio::test]
    async fn test_get_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/canonical/0/teams/999/"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
            .mount(&server)
            .await;

        let client = BugsinkClient::new(&server.uri(), "test-token").unwrap();
        let result = client.get("teams/999/").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));
    }

    #[tokio::test]
    async fn test_list_with_pagination() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/canonical/0/teams/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "next": null,
                "previous": null,
                "results": [
                    {"id": 1, "name": "Team A"},
                    {"id": 2, "name": "Team B"},
                ]
            })))
            .mount(&server)
            .await;

        let client = BugsinkClient::new(&server.uri(), "test-token").unwrap();
        let page = client.list("teams/", &[]).await.unwrap();
        assert_eq!(page.results.len(), 2);
        assert!(page.next.is_none());
    }

    #[tokio::test]
    async fn test_list_with_query_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/canonical/0/issues/"))
            .and(query_param("project", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "next": null,
                "previous": null,
                "results": [{"id": 1}]
            })))
            .mount(&server)
            .await;

        let client = BugsinkClient::new(&server.uri(), "test-token").unwrap();
        let page = client.list("issues/", &[("project", "5")]).await.unwrap();
        assert_eq!(page.results.len(), 1);
    }

    #[tokio::test]
    async fn test_list_all_multi_page() {
        let server = MockServer::start().await;
        let page2_url = format!("{}/api/canonical/0/teams/?page=2", server.uri());

        Mock::given(method("GET"))
            .and(path("/api/canonical/0/teams/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "next": page2_url,
                "previous": null,
                "results": [{"id": 1, "name": "Team A"}]
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
                "results": [{"id": 2, "name": "Team B"}]
            })))
            .expect(1)
            .with_priority(1)
            .mount(&server)
            .await;

        let client = BugsinkClient::new(&server.uri(), "test-token").unwrap();
        let results = client.list_all("teams/", &[]).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["name"], "Team A");
        assert_eq!(results[1]["name"], "Team B");
    }

    #[tokio::test]
    async fn test_post_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/canonical/0/projects/"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"id": 10, "name": "New Project"})),
            )
            .mount(&server)
            .await;

        let client = BugsinkClient::new(&server.uri(), "test-token").unwrap();
        let body = serde_json::json!({"team": 1, "name": "New Project"});
        let result = client.post("projects/", &body).await.unwrap();
        assert_eq!(result["id"], 10);
    }
}
