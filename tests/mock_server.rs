// Mock Server for Testing External Dependencies
// Provides controllable responses for LLM providers, Observatory, and Registry

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;

/// Mock server state with configurable responses
#[derive(Clone)]
pub struct MockServerState {
    /// Stored responses for different endpoints
    responses: Arc<Mutex<HashMap<String, MockResponse>>>,
    /// Request counter for verification
    request_counts: Arc<Mutex<HashMap<String, usize>>>,
    /// Latency simulation (milliseconds)
    latency_ms: Arc<Mutex<u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse {
    pub status_code: u16,
    pub body: serde_json::Value,
    pub headers: HashMap<String, String>,
}

impl MockServerState {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            request_counts: Arc::new(Mutex::new(HashMap::new())),
            latency_ms: Arc::new(Mutex::new(0)),
        }
    }

    /// Set a mock response for a specific endpoint
    pub fn set_response(&self, endpoint: String, response: MockResponse) {
        let mut responses = self.responses.lock().unwrap();
        responses.insert(endpoint, response);
    }

    /// Get request count for an endpoint
    pub fn get_request_count(&self, endpoint: &str) -> usize {
        let counts = self.request_counts.lock().unwrap();
        *counts.get(endpoint).unwrap_or(&0)
    }

    /// Set simulated latency
    pub fn set_latency(&self, ms: u64) {
        *self.latency_ms.lock().unwrap() = ms;
    }

    /// Increment request counter
    fn increment_count(&self, endpoint: &str) {
        let mut counts = self.request_counts.lock().unwrap();
        *counts.entry(endpoint.to_string()).or_insert(0) += 1;
    }

    /// Get stored response
    fn get_response(&self, endpoint: &str) -> Option<MockResponse> {
        let responses = self.responses.lock().unwrap();
        responses.get(endpoint).cloned()
    }

    /// Get simulated latency
    async fn simulate_latency(&self) {
        let latency = *self.latency_ms.lock().unwrap();
        if latency > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(latency)).await;
        }
    }
}

impl Default for MockServerState {
    fn default() -> Self {
        Self::new()
    }
}

// Mock LLM Provider Endpoints

#[derive(Debug, Deserialize)]
struct CompletionRequest {
    model: String,
    prompt: Option<String>,
    messages: Option<Vec<serde_json::Value>>,
    max_tokens: Option<u64>,
}

async fn mock_completion(
    State(state): State<MockServerState>,
    Json(req): Json<CompletionRequest>,
) -> impl IntoResponse {
    state.increment_count("/completions");
    state.simulate_latency().await;

    // Check for mock response
    if let Some(response) = state.get_response("/completions") {
        return (
            StatusCode::from_u16(response.status_code).unwrap(),
            Json(response.body),
        );
    }

    // Default mock response
    let response = json!({
        "id": "mock-completion-001",
        "model": req.model,
        "usage": {
            "prompt_tokens": 1000,
            "completion_tokens": 500,
            "total_tokens": 1500,
            "cached_tokens": 0
        },
        "choices": [{
            "message": {
                "role": "assistant",
                "content": "Mock response from test server"
            },
            "finish_reason": "stop"
        }]
    });

    (StatusCode::OK, Json(response))
}

// Mock Observatory Metrics Endpoint

#[derive(Debug, Serialize)]
struct MetricsEvent {
    timestamp: String,
    provider: String,
    model: String,
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
    request_id: String,
}

async fn mock_metrics_stream(State(state): State<MockServerState>) -> impl IntoResponse {
    state.increment_count("/metrics/stream");

    // Check for mock response
    if let Some(response) = state.get_response("/metrics/stream") {
        return (
            StatusCode::from_u16(response.status_code).unwrap(),
            Json(response.body),
        );
    }

    // Return mock metrics
    let events = vec![
        MetricsEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            prompt_tokens: 1500,
            completion_tokens: 800,
            total_tokens: 2300,
            request_id: "mock-req-001".to_string(),
        },
        MetricsEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet".to_string(),
            prompt_tokens: 2000,
            completion_tokens: 1000,
            total_tokens: 3000,
            request_id: "mock-req-002".to_string(),
        },
    ];

    (StatusCode::OK, Json(events))
}

// Mock Registry Endpoints

#[derive(Debug, Serialize)]
struct ProviderInfo {
    id: String,
    name: String,
    models: Vec<ModelInfo>,
    status: String,
}

#[derive(Debug, Serialize)]
struct ModelInfo {
    name: String,
    context_window: u64,
    supports_streaming: bool,
    pricing: PricingInfo,
}

#[derive(Debug, Serialize)]
struct PricingInfo {
    input_price_per_million: f64,
    output_price_per_million: f64,
    currency: String,
    effective_date: String,
}

async fn mock_get_provider(
    State(state): State<MockServerState>,
    Path(provider_id): Path<String>,
) -> impl IntoResponse {
    state.increment_count(&format!("/providers/{}", provider_id));
    state.simulate_latency().await;

    // Check for mock response
    let endpoint = format!("/providers/{}", provider_id);
    if let Some(response) = state.get_response(&endpoint) {
        return (
            StatusCode::from_u16(response.status_code).unwrap(),
            Json(response.body),
        );
    }

    // Default mock provider
    let provider = ProviderInfo {
        id: provider_id.clone(),
        name: format!("Mock {}", provider_id),
        models: vec![ModelInfo {
            name: "mock-model".to_string(),
            context_window: 8192,
            supports_streaming: true,
            pricing: PricingInfo {
                input_price_per_million: 10.0,
                output_price_per_million: 30.0,
                currency: "USD".to_string(),
                effective_date: "2025-01-01".to_string(),
            },
        }],
        status: "active".to_string(),
    };

    (StatusCode::OK, Json(provider))
}

async fn mock_list_providers(State(state): State<MockServerState>) -> impl IntoResponse {
    state.increment_count("/providers");

    let providers = vec!["openai", "anthropic", "google", "azure"];
    let provider_list: Vec<_> = providers
        .iter()
        .map(|&id| ProviderInfo {
            id: id.to_string(),
            name: format!("Mock {}", id),
            models: vec![],
            status: "active".to_string(),
        })
        .collect();

    (StatusCode::OK, Json(provider_list))
}

// Mock Rate Limiting Endpoint

async fn mock_rate_limit(State(state): State<MockServerState>) -> impl IntoResponse {
    state.increment_count("/rate-limit");

    // Simulate rate limiting by checking request count
    let count = state.get_request_count("/rate-limit");

    if count > 100 {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "Rate limit exceeded",
                "retry_after": 60
            })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "remaining": 100 - count
        })),
    )
}

// Mock Error Scenarios

async fn mock_server_error(State(state): State<MockServerState>) -> impl IntoResponse {
    state.increment_count("/error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "Mock internal server error"
        })),
    )
}

async fn mock_timeout(State(state): State<MockServerState>) -> impl IntoResponse {
    state.increment_count("/timeout");
    // Simulate very slow response
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "healthy"})))
}

/// Create the mock server router
pub fn create_mock_router(state: MockServerState) -> Router {
    Router::new()
        // Health
        .route("/health", get(health_check))
        // LLM Provider endpoints
        .route("/v1/completions", post(mock_completion))
        .route("/v1/chat/completions", post(mock_completion))
        // Observatory endpoints
        .route("/metrics/stream", get(mock_metrics_stream))
        // Registry endpoints
        .route("/providers", get(mock_list_providers))
        .route("/providers/:id", get(mock_get_provider))
        // Testing endpoints
        .route("/rate-limit", get(mock_rate_limit))
        .route("/error", get(mock_server_error))
        .route("/timeout", get(mock_timeout))
        .with_state(state)
}

/// Start the mock server on a random port
pub async fn start_mock_server() -> (MockServerState, SocketAddr) {
    let state = MockServerState::new();
    let app = create_mock_router(state.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (state, addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest;

    #[tokio::test]
    async fn test_mock_server_starts() {
        let (state, addr) = start_mock_server().await;

        let url = format!("http://{}/health", addr);
        let response = reqwest::get(&url).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(state.get_request_count("/health"), 0); // Health doesn't increment
    }

    #[tokio::test]
    async fn test_mock_completion_endpoint() {
        let (state, addr) = start_mock_server().await;

        let client = reqwest::Client::new();
        let url = format!("http://{}/v1/completions", addr);

        let request = json!({
            "model": "gpt-4",
            "prompt": "Test prompt",
            "max_tokens": 100
        });

        let response = client.post(&url).json(&request).send().await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(state.get_request_count("/completions"), 1);

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["model"], "gpt-4");
        assert!(body["usage"]["total_tokens"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_mock_custom_response() {
        let (state, addr) = start_mock_server().await;

        // Set a custom response
        state.set_response(
            "/completions".to_string(),
            MockResponse {
                status_code: 200,
                body: json!({
                    "custom": "response",
                    "usage": {
                        "total_tokens": 9999
                    }
                }),
                headers: HashMap::new(),
            },
        );

        let client = reqwest::Client::new();
        let url = format!("http://{}/v1/completions", addr);

        let response = client
            .post(&url)
            .json(&json!({"model": "test"}))
            .send()
            .await
            .unwrap();

        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["custom"], "response");
        assert_eq!(body["usage"]["total_tokens"], 9999);
    }

    #[tokio::test]
    async fn test_mock_latency_simulation() {
        let (state, addr) = start_mock_server().await;

        // Set 500ms latency
        state.set_latency(500);

        let start = std::time::Instant::now();

        let url = format!("http://{}/providers", addr);
        let _response = reqwest::get(&url).await.unwrap();

        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() >= 500, "Latency simulation failed");
    }

    #[tokio::test]
    async fn test_mock_rate_limiting() {
        let (state, addr) = start_mock_server().await;

        let url = format!("http://{}/rate-limit", addr);

        // Make requests until rate limited
        for i in 0..105 {
            let response = reqwest::get(&url).await.unwrap();

            if i < 100 {
                assert_eq!(response.status(), StatusCode::OK);
            } else {
                assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
            }
        }

        assert!(state.get_request_count("/rate-limit") > 100);
    }

    #[tokio::test]
    async fn test_mock_error_response() {
        let (_state, addr) = start_mock_server().await;

        let url = format!("http://{}/error", addr);
        let response = reqwest::get(&url).await.unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_mock_metrics_stream() {
        let (state, addr) = start_mock_server().await;

        let url = format!("http://{}/metrics/stream", addr);
        let response = reqwest::get(&url).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(state.get_request_count("/metrics/stream"), 1);

        let events: Vec<MetricsEvent> = response.json().await.unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].provider, "openai");
        assert_eq!(events[1].provider, "anthropic");
    }

    #[tokio::test]
    async fn test_mock_provider_lookup() {
        let (_state, addr) = start_mock_server().await;

        let url = format!("http://{}/providers/openai", addr);
        let response = reqwest::get(&url).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let provider: ProviderInfo = response.json().await.unwrap();
        assert_eq!(provider.id, "openai");
        assert!(!provider.models.is_empty());
    }
}
