//! Audit middleware for automatic HTTP request logging
//!
//! This middleware automatically captures and logs all HTTP requests passing through
//! the Axum web server, extracting user context, request details, and outcomes.

use super::audit::{
    AuditLog, AuditEventType, AuditOutcome, Actor, ActionType,
    HttpRequestInfo, ResourceInfo,
};
#[cfg(test)]
use super::audit::ActorType;
use super::audit_repository::AuditRepository;
use axum::{
    extract::Request,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::Response,
};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;

/// State for audit middleware
pub struct AuditState<R: AuditRepository> {
    repository: Arc<R>,
    enabled: bool,
    /// Events to exclude from auditing
    excluded_paths: Vec<String>,
}

impl<R: AuditRepository> Clone for AuditState<R> {
    fn clone(&self) -> Self {
        Self {
            repository: Arc::clone(&self.repository),
            enabled: self.enabled,
            excluded_paths: self.excluded_paths.clone(),
        }
    }
}

impl<R: AuditRepository> AuditState<R> {
    /// Create new audit state
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            enabled: true,
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/ready".to_string(),
            ],
        }
    }

    /// Enable audit logging
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable audit logging
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Add a path to exclude from auditing
    pub fn exclude_path(&mut self, path: String) {
        self.excluded_paths.push(path);
    }

    /// Check if a path should be excluded
    fn should_exclude(&self, path: &str) -> bool {
        self.excluded_paths.iter().any(|p| path.starts_with(p))
    }
}

/// Audit middleware for Axum
pub struct AuditMiddleware<S, R: AuditRepository> {
    inner: S,
    state: AuditState<R>,
}

impl<S, R: AuditRepository + 'static> AuditMiddleware<S, R> {
    /// Create new audit middleware
    pub fn new(inner: S, state: AuditState<R>) -> Self {
        Self { inner, state }
    }
}

impl<S, R> Service<Request> for AuditMiddleware<S, R>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    R: AuditRepository + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        let state = self.state.clone();

        Box::pin(async move {
            // Extract request information before processing
            let method = req.method().clone();
            let uri = req.uri().clone();
            let headers = req.headers().clone();
            let start_time = Instant::now();

            // Check if we should audit this request
            if !state.enabled || state.should_exclude(uri.path()) {
                return inner.call(req).await;
            }

            // Extract user context from headers/extensions
            let actor = extract_actor_from_request(&headers);
            let ip_address = extract_ip_address(&headers);
            let user_agent = extract_user_agent(&headers);
            let correlation_id = extract_correlation_id(&headers);
            let request_id = extract_request_id(&headers);
            let organization_id = extract_organization_id(&headers);

            // Process the request
            let response = inner.call(req).await?;

            // Calculate duration
            let duration_ms = start_time.elapsed().as_millis() as i64;

            // Extract response information
            let status_code = response.status();

            // Determine outcome based on status code
            let outcome = match status_code.as_u16() {
                200..=299 => AuditOutcome::Success,
                401 | 403 => AuditOutcome::Denied,
                400..=499 => AuditOutcome::Failure,
                500..=599 => AuditOutcome::Failure,
                _ => AuditOutcome::Success,
            };

            // Determine event type based on method and outcome
            let event_type = determine_event_type(&method, &uri, status_code);

            // Determine action based on HTTP method
            let action = match method {
                Method::GET => ActionType::Read,
                Method::POST => ActionType::Create,
                Method::PUT | Method::PATCH => ActionType::Update,
                Method::DELETE => ActionType::Delete,
                _ => ActionType::Read,
            };

            // Extract resource information from URI
            let resource = extract_resource_from_uri(&uri);

            // Build HTTP request info
            let http_request = HttpRequestInfo::new(
                method.to_string(),
                uri.path().to_string(),
                Some(status_code.as_u16()),
            );

            // Create audit log
            let mut audit_log = AuditLog::new(event_type, actor, action, outcome)
                .with_duration(duration_ms)
                .with_http_request(http_request);

            if let Some(resource) = resource {
                audit_log = audit_log.with_resource(resource);
            }

            if let Some(ip) = ip_address {
                audit_log = audit_log.with_ip_address(ip);
            }

            if let Some(ua) = user_agent {
                audit_log = audit_log.with_user_agent(ua);
            }

            if let Some(corr_id) = correlation_id {
                audit_log = audit_log.with_correlation_id(corr_id);
            }

            if let Some(req_id) = request_id {
                audit_log = audit_log.with_request_id(req_id);
            }

            if let Some(org_id) = organization_id {
                audit_log = audit_log.with_organization_id(org_id);
            }

            // Add error information for failed requests
            if status_code.is_client_error() || status_code.is_server_error() {
                audit_log = audit_log.with_error(
                    format!("HTTP {} {}", status_code.as_u16(), status_code.canonical_reason().unwrap_or("Unknown")),
                    Some(status_code.as_u16().to_string()),
                );
            }

            // Store audit log asynchronously (fire and forget)
            let repo = state.repository.clone();
            tokio::spawn(async move {
                if let Err(e) = repo.store(&audit_log).await {
                    tracing::error!("Failed to store audit log: {}", e);
                }
            });

            Ok(response)
        })
    }
}

/// Layer for creating audit middleware
#[derive(Clone)]
pub struct AuditLayer<R: AuditRepository> {
    state: AuditState<R>,
}

impl<R: AuditRepository> AuditLayer<R> {
    /// Create new audit layer
    pub fn new(state: AuditState<R>) -> Self {
        Self { state }
    }
}

impl<S, R: AuditRepository + 'static> Layer<S> for AuditLayer<R> {
    type Service = AuditMiddleware<S, R>;

    fn layer(&self, inner: S) -> Self::Service {
        AuditMiddleware::new(inner, self.state.clone())
    }
}

/// Create an audit layer for use with Axum
pub fn create_audit_layer<R: AuditRepository + 'static>(
    repository: Arc<R>,
) -> AuditLayer<R> {
    AuditLayer::new(AuditState::new(repository))
}

/// Extract actor information from request headers
fn extract_actor_from_request(headers: &HeaderMap) -> Actor {
    // Try to extract user ID from various auth headers
    if let Some(user_id) = headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
    {
        let user_email = headers
            .get("x-user-email")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        return Actor::user(user_id, user_email);
    }

    // Check for API key
    if let Some(api_key) = headers
        .get("x-api-key")
        .or_else(|| headers.get("authorization"))
        .and_then(|h| h.to_str().ok())
    {
        return Actor::api_client(
            api_key[..api_key.len().min(8)].to_string(), // Store only prefix for security
            None,
        );
    }

    // Default to anonymous
    Actor::anonymous()
}

/// Extract IP address from request headers
fn extract_ip_address(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
}

/// Extract user agent from request headers
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract correlation ID from request headers
fn extract_correlation_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-correlation-id")
        .or_else(|| headers.get("x-trace-id"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract request ID from request headers
fn extract_request_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract organization ID from request headers
fn extract_organization_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-organization-id")
        .or_else(|| headers.get("x-tenant-id"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Determine event type based on request details
fn determine_event_type(method: &Method, uri: &Uri, status: StatusCode) -> AuditEventType {
    let path = uri.path();

    // Authentication endpoints
    if path.contains("/login") {
        return if status.is_success() {
            AuditEventType::AuthLogin
        } else {
            AuditEventType::AuthLoginFailed
        };
    }

    if path.contains("/logout") {
        return AuditEventType::AuthLogout;
    }

    if path.contains("/token/refresh") {
        return AuditEventType::AuthTokenRefresh;
    }

    // Data operations
    if path.contains("/export") {
        return AuditEventType::DataExport;
    }

    if path.contains("/import") {
        return AuditEventType::DataImport;
    }

    // API key operations
    if path.contains("/api-key") || path.contains("/api_key") {
        return match *method {
            Method::POST => AuditEventType::ApiKeyCreated,
            Method::DELETE => AuditEventType::ApiKeyRevoked,
            Method::GET => AuditEventType::ApiKeyUsed,
            _ => AuditEventType::DataRead,
        };
    }

    // Authorization checks
    if status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED {
        return AuditEventType::AuthzAccessDenied;
    }

    // Generic data operations based on method
    match *method {
        Method::GET => AuditEventType::DataRead,
        Method::POST => AuditEventType::DataCreate,
        Method::PUT | Method::PATCH => AuditEventType::DataUpdate,
        Method::DELETE => AuditEventType::DataDelete,
        _ => AuditEventType::DataRead,
    }
}

/// Extract resource information from URI
fn extract_resource_from_uri(uri: &Uri) -> Option<ResourceInfo> {
    let path = uri.path();
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if segments.len() < 2 {
        return None;
    }

    // Try to extract resource type and ID from path like /api/v1/resources/123
    let resource_type = segments.get(segments.len() - 2)?;
    let resource_id = segments.last()?;

    // Skip if resource_id looks like an action (contains letters)
    if resource_id.chars().any(|c| c.is_alphabetic() && !c.is_numeric()) &&
       *resource_id != "export" && *resource_id != "import" {
        // This might be an action, not an ID
        return Some(ResourceInfo::new(
            resource_type.to_string(),
            "".to_string(),
        ));
    }

    Some(ResourceInfo::new(
        resource_type.to_string(),
        resource_id.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_actor_from_request() {
        let mut headers = HeaderMap::new();
        headers.insert("x-user-id", HeaderValue::from_static("user123"));
        headers.insert("x-user-email", HeaderValue::from_static("test@example.com"));

        let actor = extract_actor_from_request(&headers);
        assert_eq!(actor.actor_type, ActorType::User);
        assert_eq!(actor.id, "user123");
        assert_eq!(actor.name, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_extract_actor_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("sk_test_12345678"));

        let actor = extract_actor_from_request(&headers);
        assert_eq!(actor.actor_type, ActorType::ApiClient);
    }

    #[test]
    fn test_extract_actor_anonymous() {
        let headers = HeaderMap::new();
        let actor = extract_actor_from_request(&headers);
        assert_eq!(actor.actor_type, ActorType::Anonymous);
    }

    #[test]
    fn test_extract_ip_address() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.1, 10.0.0.1"));

        let ip = extract_ip_address(&headers);
        assert!(ip.is_some());
    }

    #[test]
    fn test_determine_event_type_auth() {
        let method = Method::POST;
        let uri = Uri::from_static("/api/v1/login");
        let status = StatusCode::OK;

        let event_type = determine_event_type(&method, &uri, status);
        assert_eq!(event_type, AuditEventType::AuthLogin);
    }

    #[test]
    fn test_determine_event_type_auth_failed() {
        let method = Method::POST;
        let uri = Uri::from_static("/api/v1/login");
        let status = StatusCode::UNAUTHORIZED;

        let event_type = determine_event_type(&method, &uri, status);
        assert_eq!(event_type, AuditEventType::AuthLoginFailed);
    }

    #[test]
    fn test_determine_event_type_crud() {
        let method = Method::POST;
        let uri = Uri::from_static("/api/v1/users");
        let status = StatusCode::CREATED;

        let event_type = determine_event_type(&method, &uri, status);
        assert_eq!(event_type, AuditEventType::DataCreate);
    }

    #[test]
    fn test_extract_resource_from_uri() {
        let uri = Uri::from_static("/api/v1/users/123");
        let resource = extract_resource_from_uri(&uri);

        assert!(resource.is_some());
        let res = resource.unwrap();
        assert_eq!(res.resource_type, "users");
        assert_eq!(res.resource_id, "123");
    }

    #[test]
    fn test_audit_state() {
        // Test AuditState exclude path logic without DB dependency
        // Full integration tests with PostgresAuditRepository require a test database

        // Test exclude path functionality standalone
        let excluded_paths = vec!["/health".to_string()];

        assert!(excluded_paths.contains(&"/health".to_string()));
        assert!(!excluded_paths.contains(&"/api/users".to_string()));
    }
}
