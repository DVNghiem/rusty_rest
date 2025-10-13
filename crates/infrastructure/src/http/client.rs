use anyhow;
use reqwest::{Client, StatusCode, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT}};
use serde::{Deserialize, Serialize};
use shared::{AppResult, AppError, retry_async, RetryPolicy};
use std::time::Duration;
use tracing::{instrument, info, warn, error};
use uuid::Uuid;

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub base_url: Option<String>,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub default_headers: HeaderMap,
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        Self {
            base_url: None,
            timeout: Duration::from_secs(30),
            retry_policy: RetryPolicy::default(),
            default_headers: headers,
            user_agent: "MyProject-HttpClient/1.0".to_string(),
        }
    }
}

/// Enterprise-grade HTTP client with observability and resilience patterns
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    config: HttpClientConfig,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(config: HttpClientConfig) -> AppResult<Self> {
        let mut client_builder = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent);

        // Add default headers
        let mut headers = config.default_headers.clone();
        headers.insert(USER_AGENT, HeaderValue::from_str(&config.user_agent).unwrap());
        client_builder = client_builder.default_headers(headers);

        let client = client_builder
            .build()
            .map_err(|e| AppError::HttpClient(e))?;

        Ok(Self { client, config })
    }

    /// Create client with default config
    pub fn default() -> AppResult<Self> {
        Self::new(HttpClientConfig::default())
    }

    /// GET request
    #[instrument(skip(self))]
    pub async fn get(&self, url: &str) -> AppResult<HttpResponse> {
        self.request(HttpMethod::Get, url, None::<&()>, None).await
    }

    /// GET request with headers
    #[instrument(skip(self, headers))]
    pub async fn get_with_headers(&self, url: &str, headers: Option<HeaderMap>) -> AppResult<HttpResponse> {
        self.request(HttpMethod::Get, url, None::<&()>, headers).await
    }

    /// POST request with JSON body
    #[instrument(skip(self, body))]
    pub async fn post<T>(&self, url: &str, body: &T) -> AppResult<HttpResponse>
    where
        T: Serialize,
    {
        self.request(HttpMethod::Post, url, Some(body), None).await
    }

    /// POST request with JSON body and headers
    #[instrument(skip(self, body, headers))]
    pub async fn post_with_headers<T>(
        &self,
        url: &str,
        body: &T,
        headers: Option<HeaderMap>,
    ) -> AppResult<HttpResponse>
    where
        T: Serialize,
    {
        self.request(HttpMethod::Post, url, Some(body), headers).await
    }

    /// PUT request with JSON body
    #[instrument(skip(self, body))]
    pub async fn put<T>(&self, url: &str, body: &T) -> AppResult<HttpResponse>
    where
        T: Serialize,
    {
        self.request(HttpMethod::Put, url, Some(body), None).await
    }

    /// PATCH request with JSON body
    #[instrument(skip(self, body))]
    pub async fn patch<T>(&self, url: &str, body: &T) -> AppResult<HttpResponse>
    where
        T: Serialize,
    {
        self.request(HttpMethod::Patch, url, Some(body), None).await
    }

    /// DELETE request
    #[instrument(skip(self))]
    pub async fn delete(&self, url: &str) -> AppResult<HttpResponse> {
        self.request(HttpMethod::Delete, url, None::<&()>, None).await
    }

    /// Generic request method with retry logic
    #[instrument(skip(self, body, headers))]
    async fn request<T>(
        &self,
        method: HttpMethod,
        url: &str,
        body: Option<&T>,
        headers: Option<HeaderMap>,
    ) -> AppResult<HttpResponse>
    where
        T: Serialize,
    {
        let full_url = self.build_url(url);
        let request_id = Uuid::new_v4().to_string();

        info!(
            request_id = %request_id,
            method = %method,
            url = %full_url,
            "Making HTTP request"
        );

        let result = retry_async(
            || async {
                self.execute_request(&method, &full_url, body, headers.as_ref(), &request_id)
                    .await
            },
            &self.config.retry_policy,
        )
        .await;

        match &result {
            Ok(response) => {
                info!(
                    request_id = %request_id,
                    status = %response.status,
                    "HTTP request completed successfully"
                );
            }
            Err(e) => {
                error!(
                    request_id = %request_id,
                    error = %e,
                    "HTTP request failed"
                );
            }
        }

        result
    }

    async fn execute_request<T>(
        &self,
        method: &HttpMethod,
        url: &str,
        body: Option<&T>,
        headers: Option<&HeaderMap>,
        request_id: &str,
    ) -> AppResult<HttpResponse>
    where
        T: Serialize,
    {
        let mut request_builder = match method {
            HttpMethod::Get => self.client.get(url),
            HttpMethod::Post => self.client.post(url),
            HttpMethod::Put => self.client.put(url),
            HttpMethod::Patch => self.client.patch(url),
            HttpMethod::Delete => self.client.delete(url),
        };

        // Add request ID header for tracing
        request_builder = request_builder.header("X-Request-ID", request_id);

        // Add custom headers if provided
        if let Some(headers) = headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        // Add body for POST/PUT/PATCH requests
        if let Some(body) = body {
            let json_body = serde_json::to_string(body)
                .map_err(|e| AppError::Internal(anyhow::Error::from(e)))?;
            request_builder = request_builder.body(json_body);
        }

        let response = request_builder.send().await?;
        let status = response.status();
        let headers = response.headers().clone();

        // Read response body
        let body_bytes = response.bytes().await?;
        let body_text = String::from_utf8_lossy(&body_bytes).to_string();

        // Check if request was successful
        if !status.is_success() {
            warn!(
                request_id = %request_id,
                status = %status,
                body = %body_text,
                "HTTP request returned error status"
            );

            return Err(AppError::Internal(anyhow::anyhow!(
                "HTTP {} error: {}", status, body_text
            )));
        }

        Ok(HttpResponse {
            status,
            headers,
            body: body_text,
            request_id: request_id.to_string(),
        })
    }

    fn build_url(&self, path: &str) -> String {
        match &self.config.base_url {
            Some(base) => {
                if path.starts_with("http://") || path.starts_with("https://") {
                    path.to_string()
                } else {
                    format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))
                }
            }
            None => path.to_string(),
        }
    }
}

/// HTTP response wrapper
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: String,
    pub request_id: String,
}

impl HttpResponse {
    /// Parse response body as JSON
    pub fn json<T>(&self) -> AppResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_str(&self.body)
            .map_err(|e| AppError::Internal(anyhow::Error::from(e)))
    }

    /// Get response body as text
    pub fn text(&self) -> &str {
        &self.body
    }

    /// Check if response was successful
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Get status code
    pub fn status(&self) -> StatusCode {
        self.status
    }
}

#[derive(Debug, Clone)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
        }
    }
}

/// Authentication helpers
impl HttpClient {
    /// Create client with bearer token authentication
    pub fn with_bearer_token(token: &str) -> AppResult<Self> {
        let mut config = HttpClientConfig::default();
        config.default_headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid bearer token")))?,
        );
        Self::new(config)
    }

    /// Create client with API key authentication
    pub fn with_api_key(api_key: &str, header_name: &str) -> AppResult<Self> {
        let mut config = HttpClientConfig::default();
        let header_name_parsed = reqwest::header::HeaderName::from_bytes(header_name.as_bytes())
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid header name")))?;
        config.default_headers.insert(
            header_name_parsed,
            HeaderValue::from_str(api_key)
                .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid API key")))?,
        );
        Self::new(config)
    }

    /// Add authorization header to existing client
    pub fn add_auth_header(&mut self, name: &str, value: &str) -> AppResult<()> {
        let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid header name")))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid header value")))?;
        
        self.config.default_headers.insert(header_name, header_value);
        Ok(())
    }
}

/// Service-specific HTTP clients
pub struct ExternalServiceClient {
    client: HttpClient,
    base_url: String,
}

impl ExternalServiceClient {
    pub fn new(base_url: String, api_key: Option<String>) -> AppResult<Self> {
        let mut config = HttpClientConfig::default();
        config.base_url = Some(base_url.clone());

        if let Some(key) = api_key {
            config.default_headers.insert(
                "X-API-Key",
                HeaderValue::from_str(&key)
                    .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid API key")))?,
            );
        }

        let client = HttpClient::new(config)?;
        Ok(Self { client, base_url })
    }

    /// Health check for external service
    pub async fn health_check(&self) -> AppResult<bool> {
        match self.client.get("/health").await {
            Ok(response) => Ok(response.is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Generic GET request to external service
    pub async fn get<T>(&self, endpoint: &str) -> AppResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.client.get(endpoint).await?;
        response.json()
    }

    /// Generic POST request to external service
    pub async fn post<T, R>(&self, endpoint: &str, body: &T) -> AppResult<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let response = self.client.post(endpoint, body).await?;
        response.json()
    }
}

/// Circuit breaker pattern for external services
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub retry_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(30),
            retry_timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, rejecting requests
    HalfOpen, // Testing if service is back
}

pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    config: CircuitBreakerConfig,
    last_failure_time: Option<std::time::Instant>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            config,
            last_failure_time: None,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.config.retry_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
        self.last_failure_time = None;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        if self.failure_count >= self.config.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }

    pub fn state(&self) -> &CircuitBreakerState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_building() {
        let config = HttpClientConfig {
            base_url: Some("https://api.example.com".to_string()),
            ..Default::default()
        };
        let client = HttpClient::new(config).unwrap();
        
        assert_eq!(client.build_url("/users"), "https://api.example.com/users");
        assert_eq!(client.build_url("users"), "https://api.example.com/users");
        assert_eq!(client.build_url("https://other.com/test"), "https://other.com/test");
    }

    #[test]
    fn test_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_secs(30),
            retry_timeout: Duration::from_secs(60),
        };
        let mut cb = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(*cb.state(), CircuitBreakerState::Closed);
        assert!(cb.can_execute());

        // Record failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(*cb.state(), CircuitBreakerState::Closed);

        // Exceed threshold
        cb.record_failure();
        assert_eq!(*cb.state(), CircuitBreakerState::Open);
        assert!(!cb.can_execute());

        // Success resets the breaker
        cb.record_success();
        assert_eq!(*cb.state(), CircuitBreakerState::Closed);
        assert!(cb.can_execute());
    }
}