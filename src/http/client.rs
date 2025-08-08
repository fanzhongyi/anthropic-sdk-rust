use reqwest::{Client, Request, Response, RequestBuilder};
use serde_json::Value;
use crate::config::ClientConfig;
use crate::http::auth::AuthHandler;
use crate::types::errors::{AnthropicError, Result};
use crate::types::shared::RequestId;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: ClientConfig,
    auth: AuthHandler,
    beta_headers: HashMap<String, String>,
}

impl HttpClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        // Validate configuration before creating client
        config.validate()?;

        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| AnthropicError::Connection { message: e.to_string() })?;

        let auth = AuthHandler::with_method(config.api_key.clone(), config.auth_method.clone());

        Ok(Self {
            client,
            config,
            auth,
            beta_headers: HashMap::new(),
        })
    }

    /// Send a prepared request with authentication and error handling
    pub async fn send(&self, mut request: Request) -> Result<Response> {
        // Add authentication headers
        let headers = request.headers_mut();
        self.auth.add_auth_headers(headers)?;

        // Add beta headers
        for (key, value) in &self.beta_headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                reqwest::header::HeaderValue::from_str(value).unwrap()
            );
        }

        let response = self.client.execute(request).await
            .map_err(|e| AnthropicError::Connection { message: e.to_string() })?;

        self.handle_response_status(response).await
    }

    /// Create a GET request builder
    pub fn get(&self, url: &str) -> RequestBuilder {
        self.client.get(url)
    }

    /// Create a POST request builder
    pub fn post(&self, url: &str) -> RequestBuilder {
        self.client.post(url)
    }

    /// Create a PUT request builder
    pub fn put(&self, url: &str) -> RequestBuilder {
        self.client.put(url)
    }

    /// Create a DELETE request builder
    pub fn delete(&self, url: &str) -> RequestBuilder {
        self.client.delete(url)
    }

    /// Build a full URL from a path
    pub fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
    }

    /// Handle response status codes and convert to appropriate errors
    async fn handle_response_status(&self, response: Response) -> Result<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let status_code = status.as_u16();

        // Try to extract error message from response body
        let error_message = match response.text().await {
            Ok(body) => {
                // Try to parse as JSON and extract error message
                match serde_json::from_str::<Value>(&body) {
                    Ok(json) => {
                        json.get("error")
                            .and_then(|e| e.get("message"))
                            .and_then(|m| m.as_str())
                            .unwrap_or(&body)
                            .to_string()
                    }
                    Err(_) => body,
                }
            }
            Err(_) => format!("HTTP {}: {}", status_code, status.canonical_reason().unwrap_or("Unknown")),
        };

        Err(AnthropicError::from_status(status_code, error_message))
    }

    /// Extract request ID from response headers
    pub fn extract_request_id(&self, response: &Response) -> Option<RequestId> {
        response.headers()
            .get("request-id")
            .and_then(|value| value.to_str().ok())
            .map(|id| RequestId::new(id.to_string()))
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Get the current configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Get the underlying reqwest client
    ///
    /// This is useful for creating custom requests or integrating with other libraries.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Add a beta header for experimental features
    pub fn with_beta_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.beta_headers.insert(name.into(), value.into());
    }

    /// Add extended cache TTL beta header
    pub fn with_extended_cache_ttl(&mut self) {
        self.beta_headers.insert("extended-cache-ttl-2025-04-11".to_string(), "true".to_string());
    }

    /// Add interleaved thinking beta header
    pub fn with_interleaved_thinking(&mut self) {
        self.beta_headers.insert("interleaved-thinking-2025-05-14".to_string(), "true".to_string());
    }

    /// Create a new client with beta headers
    pub fn with_beta_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.beta_headers = headers;
        self
    }
}
