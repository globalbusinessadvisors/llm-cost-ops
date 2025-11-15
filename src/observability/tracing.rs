// Enhanced tracing with correlation IDs and distributed tracing support

use tracing::{Level, Span};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use uuid::Uuid;

use super::config::{TracingConfig, TracingFormat};

/// Correlation ID for distributed tracing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Generate a new correlation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from an existing ID
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Request ID for tracking individual requests
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestId(String);

impl RequestId {
    /// Generate a new request ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from an existing ID
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trace context containing correlation and request IDs
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub correlation_id: CorrelationId,
    pub request_id: RequestId,
    pub parent_span_id: Option<String>,
    pub trace_id: Option<String>,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new() -> Self {
        Self {
            correlation_id: CorrelationId::new(),
            request_id: RequestId::new(),
            parent_span_id: None,
            trace_id: None,
        }
    }

    /// Create with existing IDs
    pub fn with_ids(correlation_id: String, request_id: String) -> Self {
        Self {
            correlation_id: CorrelationId::from_string(correlation_id),
            request_id: RequestId::from_string(request_id),
            parent_span_id: None,
            trace_id: None,
        }
    }

    /// Set parent span ID
    pub fn with_parent_span(mut self, parent_span_id: String) -> Self {
        self.parent_span_id = Some(parent_span_id);
        self
    }

    /// Set trace ID
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize tracing with the given configuration
pub fn init_tracing(config: &TracingConfig) -> Result<(), String> {
    if !config.enabled {
        return Ok(());
    }

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .map_err(|e| format!("Failed to create env filter: {}", e))?;

    // Create the subscriber based on format
    match config.format {
        TracingFormat::Json => {
            let fmt_layer = fmt::layer()
                .json()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(config.include_thread)
                .with_thread_names(config.include_thread)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_span_events(FmtSpan::CLOSE);

            Registry::default()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| format!("Failed to initialize tracing: {}", e))?;
        }
        TracingFormat::Pretty => {
            let fmt_layer = fmt::layer()
                .pretty()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(config.include_thread)
                .with_thread_names(config.include_thread)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_ansi(config.ansi)
                .with_span_events(FmtSpan::CLOSE);

            Registry::default()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| format!("Failed to initialize tracing: {}", e))?;
        }
        TracingFormat::Compact => {
            let fmt_layer = fmt::layer()
                .compact()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(config.include_thread)
                .with_thread_names(config.include_thread)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_ansi(config.ansi)
                .with_span_events(FmtSpan::CLOSE);

            Registry::default()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| format!("Failed to initialize tracing: {}", e))?;
        }
        TracingFormat::Text => {
            let fmt_layer = fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(config.include_thread)
                .with_thread_names(config.include_thread)
                .with_file(config.include_location)
                .with_line_number(config.include_location)
                .with_ansi(config.ansi)
                .with_span_events(FmtSpan::CLOSE);

            Registry::default()
                .with(env_filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| format!("Failed to initialize tracing: {}", e))?;
        }
    }

    Ok(())
}

/// Create a span with trace context
pub fn create_span_with_context(
    level: Level,
    _target: &str,
    _name: &str,
    context: &TraceContext,
) -> Span {
    // Note: Using a generic span since span! macro requires compile-time constants
    // For more specific spans, use the level-specific functions below
    match level {
        Level::TRACE => tracing::trace_span!(
            "operation",
            correlation_id = %context.correlation_id,
            request_id = %context.request_id,
            parent_span_id = ?context.parent_span_id,
            trace_id = ?context.trace_id,
        ),
        Level::DEBUG => tracing::debug_span!(
            "operation",
            correlation_id = %context.correlation_id,
            request_id = %context.request_id,
            parent_span_id = ?context.parent_span_id,
            trace_id = ?context.trace_id,
        ),
        Level::INFO => tracing::info_span!(
            "operation",
            correlation_id = %context.correlation_id,
            request_id = %context.request_id,
            parent_span_id = ?context.parent_span_id,
            trace_id = ?context.trace_id,
        ),
        Level::WARN => tracing::warn_span!(
            "operation",
            correlation_id = %context.correlation_id,
            request_id = %context.request_id,
            parent_span_id = ?context.parent_span_id,
            trace_id = ?context.trace_id,
        ),
        Level::ERROR => tracing::error_span!(
            "operation",
            correlation_id = %context.correlation_id,
            request_id = %context.request_id,
            parent_span_id = ?context.parent_span_id,
            trace_id = ?context.trace_id,
        ),
    }
}

/// Create an info span with context
pub fn info_span_with_context(_target: &str, _name: &str, context: &TraceContext) -> Span {
    tracing::info_span!(
        "operation",
        correlation_id = %context.correlation_id,
        request_id = %context.request_id,
        parent_span_id = ?context.parent_span_id,
        trace_id = ?context.trace_id,
    )
}

/// Create a debug span with context
pub fn debug_span_with_context(_target: &str, _name: &str, context: &TraceContext) -> Span {
    tracing::debug_span!(
        "operation",
        correlation_id = %context.correlation_id,
        request_id = %context.request_id,
        parent_span_id = ?context.parent_span_id,
        trace_id = ?context.trace_id,
    )
}

/// Create a trace span with context
pub fn trace_span_with_context(_target: &str, _name: &str, context: &TraceContext) -> Span {
    tracing::trace_span!(
        "operation",
        correlation_id = %context.correlation_id,
        request_id = %context.request_id,
        parent_span_id = ?context.parent_span_id,
        trace_id = ?context.trace_id,
    )
}

/// Create a warn span with context
pub fn warn_span_with_context(_target: &str, _name: &str, context: &TraceContext) -> Span {
    tracing::warn_span!(
        "operation",
        correlation_id = %context.correlation_id,
        request_id = %context.request_id,
        parent_span_id = ?context.parent_span_id,
        trace_id = ?context.trace_id,
    )
}

/// Create an error span with context
pub fn error_span_with_context(_target: &str, _name: &str, context: &TraceContext) -> Span {
    tracing::error_span!(
        "operation",
        correlation_id = %context.correlation_id,
        request_id = %context.request_id,
        parent_span_id = ?context.parent_span_id,
        trace_id = ?context.trace_id,
    )
}

/// Helper macro for creating spans with automatic target
#[macro_export]
macro_rules! span_with_context {
    ($level:expr, $name:expr, $context:expr) => {
        $crate::observability::tracing::create_span_with_context(
            $level,
            module_path!(),
            $name,
            $context,
        )
    };
}

/// Helper macro for info spans with context
#[macro_export]
macro_rules! info_span_ctx {
    ($name:expr, $context:expr) => {
        $crate::observability::tracing::info_span_with_context(module_path!(), $name, $context)
    };
}

/// Helper macro for debug spans with context
#[macro_export]
macro_rules! debug_span_ctx {
    ($name:expr, $context:expr) => {
        $crate::observability::tracing::debug_span_with_context(module_path!(), $name, $context)
    };
}

/// Extract trace context from HTTP headers
pub fn extract_trace_context_from_headers(
    headers: &axum::http::HeaderMap,
) -> Option<TraceContext> {
    let correlation_id = headers
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let trace_id = headers
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let parent_span_id = headers
        .get("x-parent-span-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // If we have at least one ID, create a context
    if correlation_id.is_some() || request_id.is_some() {
        let mut context = TraceContext::new();

        if let Some(cid) = correlation_id {
            context.correlation_id = CorrelationId::from_string(cid);
        }

        if let Some(rid) = request_id {
            context.request_id = RequestId::from_string(rid);
        }

        if let Some(tid) = trace_id {
            context.trace_id = Some(tid);
        }

        if let Some(psid) = parent_span_id {
            context.parent_span_id = Some(psid);
        }

        Some(context)
    } else {
        None
    }
}

/// Inject trace context into HTTP headers
pub fn inject_trace_context_into_headers(
    headers: &mut axum::http::HeaderMap,
    context: &TraceContext,
) {
    use axum::http::HeaderValue;

    if let Ok(value) = HeaderValue::from_str(context.correlation_id.as_str()) {
        headers.insert("x-correlation-id", value);
    }

    if let Ok(value) = HeaderValue::from_str(context.request_id.as_str()) {
        headers.insert("x-request-id", value);
    }

    if let Some(ref trace_id) = context.trace_id {
        if let Ok(value) = HeaderValue::from_str(trace_id) {
            headers.insert("x-trace-id", value);
        }
    }

    if let Some(ref parent_span_id) = context.parent_span_id {
        if let Ok(value) = HeaderValue::from_str(parent_span_id) {
            headers.insert("x-parent-span-id", value);
        }
    }
}

/// Middleware layer for trace context propagation
#[derive(Clone)]
pub struct TraceContextLayer;

impl TraceContextLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TraceContextLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_generation() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();

        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
    }

    #[test]
    fn test_correlation_id_from_string() {
        let id_str = "test-correlation-id".to_string();
        let id = CorrelationId::from_string(id_str.clone());

        assert_eq!(id.as_str(), "test-correlation-id");
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = RequestId::new();
        let id2 = RequestId::new();

        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
    }

    #[test]
    fn test_trace_context_creation() {
        let context = TraceContext::new();

        assert!(!context.correlation_id.as_str().is_empty());
        assert!(!context.request_id.as_str().is_empty());
        assert!(context.parent_span_id.is_none());
        assert!(context.trace_id.is_none());
    }

    #[test]
    fn test_trace_context_with_ids() {
        let context = TraceContext::with_ids(
            "corr-123".to_string(),
            "req-456".to_string(),
        );

        assert_eq!(context.correlation_id.as_str(), "corr-123");
        assert_eq!(context.request_id.as_str(), "req-456");
    }

    #[test]
    fn test_trace_context_builder() {
        let context = TraceContext::new()
            .with_parent_span("span-123".to_string())
            .with_trace_id("trace-456".to_string());

        assert_eq!(context.parent_span_id, Some("span-123".to_string()));
        assert_eq!(context.trace_id, Some("trace-456".to_string()));
    }

    #[test]
    fn test_extract_trace_context_from_headers() {
        use axum::http::{HeaderMap, HeaderValue};

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-correlation-id",
            HeaderValue::from_static("corr-123"),
        );
        headers.insert("x-request-id", HeaderValue::from_static("req-456"));
        headers.insert("x-trace-id", HeaderValue::from_static("trace-789"));

        let context = extract_trace_context_from_headers(&headers);
        assert!(context.is_some());

        let ctx = context.unwrap();
        assert_eq!(ctx.correlation_id.as_str(), "corr-123");
        assert_eq!(ctx.request_id.as_str(), "req-456");
        assert_eq!(ctx.trace_id, Some("trace-789".to_string()));
    }

    #[test]
    fn test_extract_trace_context_missing_headers() {
        use axum::http::HeaderMap;

        let headers = HeaderMap::new();
        let context = extract_trace_context_from_headers(&headers);
        assert!(context.is_none());
    }

    #[test]
    fn test_inject_trace_context_into_headers() {
        use axum::http::HeaderMap;

        let mut headers = HeaderMap::new();
        let context = TraceContext::with_ids(
            "corr-123".to_string(),
            "req-456".to_string(),
        )
        .with_trace_id("trace-789".to_string());

        inject_trace_context_into_headers(&mut headers, &context);

        assert_eq!(
            headers.get("x-correlation-id").unwrap().to_str().unwrap(),
            "corr-123"
        );
        assert_eq!(
            headers.get("x-request-id").unwrap().to_str().unwrap(),
            "req-456"
        );
        assert_eq!(
            headers.get("x-trace-id").unwrap().to_str().unwrap(),
            "trace-789"
        );
    }

    #[test]
    fn test_correlation_id_display() {
        let id = CorrelationId::from_string("test-id".to_string());
        assert_eq!(format!("{}", id), "test-id");
    }

    #[test]
    fn test_request_id_display() {
        let id = RequestId::from_string("req-id".to_string());
        assert_eq!(format!("{}", id), "req-id");
    }
}
