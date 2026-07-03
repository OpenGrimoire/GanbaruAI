use reqwest::{header::HeaderMap, Method, StatusCode, Url};
use serde_json::{json, Value};
use sqlx::__rt::sleep;
use std::time::{Duration, Instant};

pub(super) const NOTION_API_VERSION: &str = "2026-03-11";

const NOTION_API_BASE_URL: &str = "https://api.notion.com/v1";
const MAX_RETRY_ATTEMPTS: usize = 3;
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(350);

#[derive(Default)]
pub(super) struct NotionApiStats {
    pub(super) request_count: i64,
    pub(super) retry_count: i64,
    pub(super) rate_limit_count: i64,
}

pub(super) struct NotionApiClient {
    http: reqwest::Client,
    token: String,
    page_size: i64,
    last_request_at: Option<Instant>,
    stats: NotionApiStats,
}

#[derive(Debug)]
pub(super) struct NotionApiError {
    pub(super) status: Option<StatusCode>,
    pub(super) message: String,
}

impl NotionApiError {
    fn request(message: impl Into<String>) -> Self {
        Self {
            status: None,
            message: message.into(),
        }
    }

    fn response(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status: Some(status),
            message: message.into(),
        }
    }
}

impl NotionApiClient {
    pub(super) fn new(token: String, page_size: i64) -> Result<Self, String> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("create Notion API HTTP client: {e}"))?;
        Ok(Self {
            http,
            token,
            page_size,
            last_request_at: None,
            stats: NotionApiStats::default(),
        })
    }

    pub(super) fn stats(&self) -> &NotionApiStats {
        &self.stats
    }

    pub(super) async fn retrieve_page(&mut self, page_id: &str) -> Result<Value, NotionApiError> {
        self.send_json(Method::GET, &format!("/pages/{page_id}"), None, None)
            .await
    }

    pub(super) async fn retrieve_data_source(
        &mut self,
        data_source_id: &str,
    ) -> Result<Value, NotionApiError> {
        self.send_json(
            Method::GET,
            &format!("/data_sources/{data_source_id}"),
            None,
            None,
        )
        .await
    }

    pub(super) async fn list_block_children(
        &mut self,
        block_id: &str,
    ) -> Result<Vec<Value>, NotionApiError> {
        self.paginated_get(
            &format!("/blocks/{block_id}/children"),
            "block children",
            Some("block"),
            &[],
        )
        .await
    }

    pub(super) async fn list_comments(
        &mut self,
        block_id: &str,
    ) -> Result<Vec<Value>, NotionApiError> {
        self.paginated_get(
            "/comments",
            "comments",
            Some("comment"),
            &[("block_id", block_id)],
        )
        .await
    }

    pub(super) async fn list_users(&mut self) -> Result<Vec<Value>, NotionApiError> {
        self.paginated_get("/users", "users", Some("user"), &[])
            .await
    }

    pub(super) async fn query_data_source(
        &mut self,
        data_source_id: &str,
    ) -> Result<Vec<Value>, NotionApiError> {
        let mut results = Vec::new();
        let mut start_cursor: Option<String> = None;
        loop {
            let mut body = json!({ "page_size": self.page_size });
            if let Some(cursor) = start_cursor.as_ref() {
                body["start_cursor"] = Value::String(cursor.clone());
            }
            let page = self
                .send_json(
                    Method::POST,
                    &format!("/data_sources/{data_source_id}/query"),
                    None,
                    Some(body),
                )
                .await?;
            append_page_results(&mut results, &page, "data source rows")?;
            start_cursor = next_cursor(&page, "data source rows")?;
            if start_cursor.is_none() {
                break;
            }
        }
        Ok(results)
    }

    async fn paginated_get(
        &mut self,
        path: &str,
        label: &'static str,
        result_type: Option<&'static str>,
        extra_query: &[(&str, &str)],
    ) -> Result<Vec<Value>, NotionApiError> {
        let mut results = Vec::new();
        let mut start_cursor: Option<String> = None;
        loop {
            let mut owned_query = vec![("page_size".to_string(), self.page_size.to_string())];
            if let Some(cursor) = start_cursor.as_ref() {
                owned_query.push(("start_cursor".to_string(), cursor.clone()));
            }
            for (key, value) in extra_query {
                owned_query.push(((*key).to_string(), (*value).to_string()));
            }
            let borrowed_query = owned_query
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect::<Vec<_>>();
            let page = self
                .send_json(Method::GET, path, Some(&borrowed_query), None)
                .await?;
            let initial_len = results.len();
            append_page_results(&mut results, &page, label)?;
            if let Some(expected_type) = result_type {
                for result in results.iter().skip(initial_len) {
                    if result.get("object").and_then(Value::as_str) != Some(expected_type) {
                        return Err(NotionApiError::request(format!(
                            "Notion {label} response included an unexpected object type"
                        )));
                    }
                }
            }
            start_cursor = next_cursor(&page, label)?;
            if start_cursor.is_none() {
                break;
            }
        }
        Ok(results)
    }

    async fn send_json(
        &mut self,
        method: Method,
        path: &str,
        query: Option<&[(&str, &str)]>,
        body: Option<Value>,
    ) -> Result<Value, NotionApiError> {
        let mut url = Url::parse(&format!("{NOTION_API_BASE_URL}{path}"))
            .map_err(|e| NotionApiError::request(format!("build Notion API URL: {e}")))?;
        if let Some(query_items) = query {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query_items {
                pairs.append_pair(key, value);
            }
        }
        let mut attempt = 0;
        loop {
            self.pace_request().await;
            self.stats.request_count += 1;
            let mut request = self
                .http
                .request(method.clone(), url.clone())
                .bearer_auth(&self.token)
                .header("Notion-Version", NOTION_API_VERSION)
                .header("Accept", "application/json");
            if let Some(payload) = body.clone() {
                request = request.json(&payload);
            }
            let response = request
                .send()
                .await
                .map_err(|e| NotionApiError::request(format!("send Notion API request: {e}")))?;
            let status = response.status();
            let headers = response.headers().clone();
            let text = response
                .text()
                .await
                .map_err(|e| NotionApiError::request(format!("read Notion API response: {e}")))?;
            if status.is_success() {
                return serde_json::from_str(&text).map_err(|e| {
                    NotionApiError::response(status, format!("parse Notion API JSON: {e}"))
                });
            }
            let message = error_message_from_body(&text).unwrap_or_else(|| status.to_string());
            if let Some(delay) = retry_delay(status, &headers, attempt) {
                self.stats.retry_count += 1;
                if is_rate_limited(status) {
                    self.stats.rate_limit_count += 1;
                }
                attempt += 1;
                sleep(delay).await;
                continue;
            }
            return Err(NotionApiError::response(status, message));
        }
    }

    async fn pace_request(&mut self) {
        if let Some(last_request_at) = self.last_request_at {
            let elapsed = last_request_at.elapsed();
            if elapsed < MIN_REQUEST_INTERVAL {
                sleep(MIN_REQUEST_INTERVAL - elapsed).await;
            }
        }
        self.last_request_at = Some(Instant::now());
    }
}

fn append_page_results(
    results: &mut Vec<Value>,
    page: &Value,
    label: &str,
) -> Result<(), NotionApiError> {
    let Some(items) = page.get("results").and_then(Value::as_array) else {
        return Err(NotionApiError::request(format!(
            "Notion {label} response did not include results"
        )));
    };
    results.extend(items.iter().cloned());
    Ok(())
}

fn next_cursor(page: &Value, label: &str) -> Result<Option<String>, NotionApiError> {
    if page.get("has_more").and_then(Value::as_bool) != Some(true) {
        return Ok(None);
    }
    page.get("next_cursor")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            NotionApiError::request(format!(
                "Notion {label} response had has_more without next_cursor"
            ))
        })
        .map(Some)
}

pub(super) fn retry_delay(
    status: StatusCode,
    headers: &HeaderMap,
    attempt: usize,
) -> Option<Duration> {
    if attempt >= MAX_RETRY_ATTEMPTS {
        return None;
    }
    if is_rate_limited(status) {
        let seconds = headers
            .get("Retry-After")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(1);
        return Some(Duration::from_secs(seconds));
    }
    if matches!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    ) {
        return Some(Duration::from_millis(250 * 2_u64.pow(attempt as u32)));
    }
    None
}

fn is_rate_limited(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.as_u16() == 529
}

fn error_message_from_body(body: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    value
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    fn retry_delay_respects_retry_after_for_rate_limits() {
        let mut headers = HeaderMap::new();
        headers.insert("Retry-After", HeaderValue::from_static("2"));
        let delay = retry_delay(StatusCode::TOO_MANY_REQUESTS, &headers, 0);
        assert_eq!(delay, Some(Duration::from_secs(2)));
    }

    #[test]
    fn retry_delay_backs_off_for_transient_server_errors() {
        let headers = HeaderMap::new();
        let delay = retry_delay(StatusCode::SERVICE_UNAVAILABLE, &headers, 1);
        assert_eq!(delay, Some(Duration::from_millis(500)));
    }

    #[test]
    fn next_cursor_requires_cursor_when_response_has_more() {
        let page = json!({ "has_more": true, "results": [] });
        let error = next_cursor(&page, "blocks").expect_err("cursor should be required");
        assert!(error.message.contains("next_cursor"));
    }
}
