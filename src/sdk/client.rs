//! SDK client implementation with builder pattern and async/await support

use crate::sdk::config::{ClientConfig, RetryConfig};
use crate::sdk::error::{SdkError, SdkResult};
use crate::sdk::retry::RetryPolicy;
use crate::sdk::telemetry::TelemetryCollector;
use crate::sdk::types::*;
use reqwest::{Client as HttpClient, Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, instrument, warn};
use url::Url;

/// Main SDK client for LLM-CostOps API
#[derive(Clone)]
pub struct CostOpsClient {
    /// HTTP client
    http_client: HttpClient,

    /// Configuration
    config: Arc<ClientConfig>,

    /// Retry policy
    retry_policy: RetryPolicy,

    /// Telemetry collector
    telemetry: TelemetryCollector,

    /// Rate limiter semaphore
    rate_limiter: Option<Arc<Semaphore>>,
}

impl CostOpsClient {
    /// Create a new client from configuration
    pub fn new(config: ClientConfig) -> SdkResult<Self> {
        config.validate()?;

        let http_client = HttpClient::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(config.pool_config.max_idle)
            .pool_idle_timeout(config.pool_config.idle_timeout)
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| SdkError::config(format!("Failed to build HTTP client: {}", e)))?;

        let retry_policy = RetryPolicy::new(config.retry_config.clone());

        let telemetry = TelemetryCollector::new(
            "llm_cost_ops_sdk",
            config.telemetry_config.metrics_enabled,
        );

        let rate_limiter = if config.rate_limit_config.enabled {
            config
                .rate_limit_config
                .requests_per_second
                .map(|rps| Arc::new(Semaphore::new(rps)))
        } else {
            None
        };

        info!("SDK client initialized");

        Ok(Self {
            http_client,
            config: Arc::new(config),
            retry_policy,
            telemetry,
            rate_limiter,
        })
    }

    /// Create a new client builder
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Submit usage data
    #[instrument(skip(self, request), fields(org_id = %request.organization_id))]
    pub async fn submit_usage(&self, request: UsageRequest) -> SdkResult<UsageResponse> {
        debug!("Submitting usage data");
        self.post("/api/v1/usage", &request).await
    }

    /// Get cost data
    #[instrument(skip(self, request), fields(org_id = %request.organization_id))]
    pub async fn get_costs(&self, request: CostRequest) -> SdkResult<CostResponse> {
        debug!("Fetching cost data");
        self.post("/api/v1/costs/query", &request).await
    }

    /// Get cost forecast
    #[instrument(skip(self, request), fields(org_id = %request.organization_id))]
    pub async fn get_forecast(&self, request: ForecastRequest) -> SdkResult<ForecastResponse> {
        debug!("Fetching forecast data");
        self.post("/api/v1/forecasts", &request).await
    }

    /// Get health status
    #[instrument(skip(self))]
    pub async fn health(&self) -> SdkResult<HealthResponse> {
        debug!("Checking health");
        self.get("/health").await
    }

    /// Generic GET request
    async fn get<T: DeserializeOwned>(&self, path: &str) -> SdkResult<T> {
        self.request::<(), T>(Method::GET, path, None).await
    }

    /// Generic POST request
    async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> SdkResult<T> {
        self.request(Method::POST, path, Some(body)).await
    }

    /// Generic HTTP request with retry logic
    async fn request<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> SdkResult<T> {
        let timer = self.telemetry.start_timer();

        // Rate limiting
        if let Some(limiter) = &self.rate_limiter {
            match limiter.try_acquire() {
                Ok(_permit) => {
                    // Permit acquired, proceed with request
                }
                Err(_) => {
                    self.telemetry.record_rate_limit_hit();
                    warn!("Rate limit exceeded, waiting for permit");
                    let _permit = limiter.acquire().await.map_err(|e| {
                        SdkError::internal(format!("Failed to acquire rate limit permit: {}", e))
                    })?;
                }
            }
        }

        // Execute request with retry
        let result = self
            .retry_policy
            .execute(|| async {
                self.execute_request(method.clone(), path, body).await
            })
            .await;

        match result {
            Ok(response) => {
                timer.success();
                Ok(response)
            }
            Err(e) => {
                timer.failure();
                Err(e)
            }
        }
    }

    /// Execute a single HTTP request
    async fn execute_request<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> SdkResult<T> {
        let url = self.build_url(path)?;

        let mut request_builder = self.http_client.request(method.clone(), url.clone());

        // Add authentication
        request_builder = request_builder.header("X-API-Key", &self.config.api_key);

        // Add default headers
        for (key, value) in &self.config.default_headers {
            request_builder = request_builder.header(key, value);
        }

        // Add body if present
        if let Some(body) = body {
            request_builder = request_builder.json(body);
        }

        debug!(
            "Executing {} request to {}",
            method,
            url
        );

        let response = request_builder
            .send()
            .await
            .map_err(|e| {
                error!("Request failed: {}", e);
                SdkError::Network(e.to_string())
            })?;

        self.handle_response(response).await
    }

    /// Handle HTTP response
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> SdkResult<T> {
        let status = response.status();

        debug!("Received response with status: {}", status);

        if status.is_success() {
            response.json::<T>().await.map_err(|e| {
                error!("Failed to deserialize response: {}", e);
                SdkError::Network(e.to_string())
            })
        } else {
            // Try to extract error details
            let status_code = status.as_u16();
            let error_body = response
                .json::<serde_json::Value>()
                .await
                .ok();

            let message = error_body
                .as_ref()
                .and_then(|v| v.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            error!(
                "API error: {} - {}",
                status_code, message
            );

            Err(SdkError::api(status_code, message, error_body))
        }
    }

    /// Build full URL from path
    fn build_url(&self, path: &str) -> SdkResult<Url> {
        self.config
            .base_url
            .join(path.trim_start_matches('/'))
            .map_err(|e| SdkError::config(format!("Invalid URL path: {}", e)))
    }

    /// Get client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Get telemetry collector
    pub fn telemetry(&self) -> &TelemetryCollector {
        &self.telemetry
    }
}

/// Builder for CostOpsClient
#[derive(Default)]
pub struct ClientBuilder {
    config_builder: Option<crate::sdk::config::ClientConfigBuilder>,
}

impl ClientBuilder {
    /// Set the base URL
    pub fn base_url(mut self, url: impl AsRef<str>) -> SdkResult<Self> {
        let builder = self
            .config_builder
            .take()
            .unwrap_or_default()
            .base_url(url)?;
        self.config_builder = Some(builder);
        Ok(self)
    }

    /// Set the API key
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        let builder = self
            .config_builder
            .take()
            .unwrap_or_default()
            .api_key(key);
        self.config_builder = Some(builder);
        self
    }

    /// Set the timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        let builder = self
            .config_builder
            .take()
            .unwrap_or_default()
            .timeout(timeout);
        self.config_builder = Some(builder);
        self
    }

    /// Set the retry configuration
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        let builder = self
            .config_builder
            .take()
            .unwrap_or_default()
            .retry_config(config);
        self.config_builder = Some(builder);
        self
    }

    /// Add a default header
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let builder = self
            .config_builder
            .take()
            .unwrap_or_default()
            .add_header(key, value);
        self.config_builder = Some(builder);
        self
    }

    /// Build the client
    pub fn build(self) -> SdkResult<CostOpsClient> {
        let config = self
            .config_builder
            .unwrap_or_default()
            .build()?;

        CostOpsClient::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_client_builder() {
        let result = CostOpsClient::builder()
            .base_url("https://api.example.com")
            .unwrap()
            .api_key("test-key")
            .timeout(Duration::from_secs(60))
            .build();

        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_builder_missing_api_key() {
        let result = CostOpsClient::builder()
            .base_url("https://api.example.com")
            .unwrap()
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_url_building() {
        let config = ClientConfig::builder()
            .base_url("https://api.example.com")
            .unwrap()
            .api_key("test")
            .build()
            .unwrap();

        let client = CostOpsClient::new(config).unwrap();
        let url = client.build_url("/api/v1/usage").unwrap();
        assert_eq!(url.as_str(), "https://api.example.com/api/v1/usage");
    }
}
