// Structured logging system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn, debug, trace};

use super::tracing::{CorrelationId, RequestId, TraceContext};

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub correlation_id: Option<String>,
    pub request_id: Option<String>,
    pub component: String,
    pub operation: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            correlation_id: None,
            request_id: None,
            component: component.into(),
            operation: None,
            fields: HashMap::new(),
            error: None,
            duration_ms: None,
        }
    }

    /// Add trace context
    pub fn with_context(mut self, context: &TraceContext) -> Self {
        self.correlation_id = Some(context.correlation_id.as_str().to_string());
        self.request_id = Some(context.request_id.as_str().to_string());
        self
    }

    /// Add correlation ID
    pub fn with_correlation_id(mut self, correlation_id: CorrelationId) -> Self {
        self.correlation_id = Some(correlation_id.as_str().to_string());
        self
    }

    /// Add request ID
    pub fn with_request_id(mut self, request_id: RequestId) -> Self {
        self.request_id = Some(request_id.as_str().to_string());
        self
    }

    /// Add operation name
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Add a field
    pub fn with_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.insert(key.into(), value);
        self
    }

    /// Add multiple fields
    pub fn with_fields(mut self, fields: HashMap<String, serde_json::Value>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Add error information
    pub fn with_error(mut self, error: impl std::fmt::Display) -> Self {
        self.error = Some(error.to_string());
        self
    }

    /// Add duration
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Log the entry using tracing
    pub fn log(&self) {
        let msg = self.format_message();

        match self.level {
            LogLevel::Trace => trace!("{}", msg),
            LogLevel::Debug => debug!("{}", msg),
            LogLevel::Info => info!("{}", msg),
            LogLevel::Warn => warn!("{}", msg),
            LogLevel::Error => error!("{}", msg),
        }
    }

    /// Format the message with all context
    fn format_message(&self) -> String {
        let mut parts = vec![self.message.clone()];

        if let Some(ref op) = self.operation {
            parts.push(format!("operation={}", op));
        }

        if let Some(ref cid) = self.correlation_id {
            parts.push(format!("correlation_id={}", cid));
        }

        if let Some(ref rid) = self.request_id {
            parts.push(format!("request_id={}", rid));
        }

        if let Some(duration) = self.duration_ms {
            parts.push(format!("duration_ms={}", duration));
        }

        for (key, value) in &self.fields {
            parts.push(format!("{}={}", key, value));
        }

        if let Some(ref err) = self.error {
            parts.push(format!("error={}", err));
        }

        parts.join(" | ")
    }
}

/// Structured logger
pub struct StructuredLogger {
    component: String,
    context: Option<TraceContext>,
}

impl StructuredLogger {
    /// Create a new structured logger for a component
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            context: None,
        }
    }

    /// Create with trace context
    pub fn with_context(component: impl Into<String>, context: TraceContext) -> Self {
        Self {
            component: component.into(),
            context: Some(context),
        }
    }

    /// Log a trace message
    pub fn trace(&self, message: impl Into<String>) -> LogEntry {
        let mut entry = LogEntry::new(LogLevel::Trace, self.component.clone(), message);
        if let Some(ref ctx) = self.context {
            entry = entry.with_context(ctx);
        }
        entry
    }

    /// Log a debug message
    pub fn debug(&self, message: impl Into<String>) -> LogEntry {
        let mut entry = LogEntry::new(LogLevel::Debug, self.component.clone(), message);
        if let Some(ref ctx) = self.context {
            entry = entry.with_context(ctx);
        }
        entry
    }

    /// Log an info message
    pub fn info(&self, message: impl Into<String>) -> LogEntry {
        let mut entry = LogEntry::new(LogLevel::Info, self.component.clone(), message);
        if let Some(ref ctx) = self.context {
            entry = entry.with_context(ctx);
        }
        entry
    }

    /// Log a warning message
    pub fn warn(&self, message: impl Into<String>) -> LogEntry {
        let mut entry = LogEntry::new(LogLevel::Warn, self.component.clone(), message);
        if let Some(ref ctx) = self.context {
            entry = entry.with_context(ctx);
        }
        entry
    }

    /// Log an error message
    pub fn error(&self, message: impl Into<String>) -> LogEntry {
        let mut entry = LogEntry::new(LogLevel::Error, self.component.clone(), message);
        if let Some(ref ctx) = self.context {
            entry = entry.with_context(ctx);
        }
        entry
    }

    /// Set trace context
    pub fn set_context(&mut self, context: TraceContext) {
        self.context = Some(context);
    }

    /// Clear trace context
    pub fn clear_context(&mut self) {
        self.context = None;
    }

    /// Get a reference to the current context
    pub fn context(&self) -> Option<&TraceContext> {
        self.context.as_ref()
    }
}

/// Helper macros for structured logging
#[macro_export]
macro_rules! log_info {
    ($logger:expr, $msg:expr $(, $key:expr => $value:expr)*) => {{
        let mut entry = $logger.info($msg);
        $(
            entry = entry.with_field($key, serde_json::json!($value));
        )*
        entry.log();
    }};
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $msg:expr, $err:expr $(, $key:expr => $value:expr)*) => {{
        let mut entry = $logger.error($msg).with_error($err);
        $(
            entry = entry.with_field($key, serde_json::json!($value));
        )*
        entry.log();
    }};
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $msg:expr $(, $key:expr => $value:expr)*) => {{
        let mut entry = $logger.warn($msg);
        $(
            entry = entry.with_field($key, serde_json::json!($value));
        )*
        entry.log();
    }};
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $msg:expr $(, $key:expr => $value:expr)*) => {{
        let mut entry = $logger.debug($msg);
        $(
            entry = entry.with_field($key, serde_json::json!($value));
        )*
        entry.log();
    }};
}

/// Performance logger for tracking operation durations
pub struct PerformanceLogger {
    logger: StructuredLogger,
    operation: String,
    start: std::time::Instant,
}

impl PerformanceLogger {
    /// Start tracking an operation
    pub fn start(logger: &StructuredLogger, operation: impl Into<String>) -> Self {
        Self {
            logger: StructuredLogger {
                component: logger.component.clone(),
                context: logger.context.clone(),
            },
            operation: operation.into(),
            start: std::time::Instant::now(),
        }
    }

    /// Complete the operation and log the duration
    pub fn complete(self) -> u64 {
        let duration = self.start.elapsed().as_millis() as u64;
        self.logger
            .info(format!("Operation completed: {}", self.operation))
            .with_operation(&self.operation)
            .with_duration_ms(duration)
            .log();
        duration
    }

    /// Complete with success
    pub fn complete_with_success(self, message: impl Into<String>) -> u64 {
        let duration = self.start.elapsed().as_millis() as u64;
        self.logger
            .info(message)
            .with_operation(&self.operation)
            .with_duration_ms(duration)
            .with_field("status", serde_json::json!("success"))
            .log();
        duration
    }

    /// Complete with error
    pub fn complete_with_error(self, error: impl std::fmt::Display) -> u64 {
        let duration = self.start.elapsed().as_millis() as u64;
        self.logger
            .error(format!("Operation failed: {}", self.operation))
            .with_operation(&self.operation)
            .with_duration_ms(duration)
            .with_error(error)
            .with_field("status", serde_json::json!("error"))
            .log();
        duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "test_component", "test message");

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.component, "test_component");
        assert_eq!(entry.message, "test message");
        assert!(entry.correlation_id.is_none());
        assert!(entry.request_id.is_none());
    }

    #[test]
    fn test_log_entry_with_context() {
        let context = TraceContext::with_ids("corr-123".to_string(), "req-456".to_string());
        let entry = LogEntry::new(LogLevel::Info, "test_component", "test message")
            .with_context(&context);

        assert_eq!(entry.correlation_id, Some("corr-123".to_string()));
        assert_eq!(entry.request_id, Some("req-456".to_string()));
    }

    #[test]
    fn test_log_entry_with_fields() {
        let mut fields = HashMap::new();
        fields.insert("key1".to_string(), serde_json::json!("value1"));
        fields.insert("key2".to_string(), serde_json::json!(42));

        let entry = LogEntry::new(LogLevel::Info, "test", "message")
            .with_fields(fields.clone());

        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields.get("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(entry.fields.get("key2"), Some(&serde_json::json!(42)));
    }

    #[test]
    fn test_log_entry_with_error() {
        let entry = LogEntry::new(LogLevel::Error, "test", "operation failed")
            .with_error("something went wrong");

        assert_eq!(entry.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_log_entry_with_duration() {
        let entry = LogEntry::new(LogLevel::Info, "test", "operation completed")
            .with_duration_ms(123);

        assert_eq!(entry.duration_ms, Some(123));
    }

    #[test]
    fn test_structured_logger_creation() {
        let logger = StructuredLogger::new("test_component");
        assert_eq!(logger.component, "test_component");
        assert!(logger.context.is_none());
    }

    #[test]
    fn test_structured_logger_with_context() {
        let context = TraceContext::new();
        let logger = StructuredLogger::with_context("test_component", context);
        assert!(logger.context.is_some());
    }

    #[test]
    fn test_structured_logger_log_methods() {
        let logger = StructuredLogger::new("test");

        let trace_entry = logger.trace("trace message");
        assert_eq!(trace_entry.level, LogLevel::Trace);

        let debug_entry = logger.debug("debug message");
        assert_eq!(debug_entry.level, LogLevel::Debug);

        let info_entry = logger.info("info message");
        assert_eq!(info_entry.level, LogLevel::Info);

        let warn_entry = logger.warn("warn message");
        assert_eq!(warn_entry.level, LogLevel::Warn);

        let error_entry = logger.error("error message");
        assert_eq!(error_entry.level, LogLevel::Error);
    }

    #[test]
    fn test_structured_logger_context_methods() {
        let mut logger = StructuredLogger::new("test");
        assert!(logger.context().is_none());

        let context = TraceContext::new();
        logger.set_context(context);
        assert!(logger.context().is_some());

        logger.clear_context();
        assert!(logger.context().is_none());
    }

    #[test]
    fn test_performance_logger() {
        let logger = StructuredLogger::new("test");
        let perf = PerformanceLogger::start(&logger, "test_operation");

        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = perf.complete();

        assert!(duration >= 10);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "trace");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Error.to_string(), "error");
    }

    #[test]
    fn test_log_entry_format_message() {
        let entry = LogEntry::new(LogLevel::Info, "test", "test message")
            .with_operation("test_op")
            .with_field("key1", serde_json::json!("value1"))
            .with_duration_ms(100);

        let formatted = entry.format_message();
        assert!(formatted.contains("test message"));
        assert!(formatted.contains("operation=test_op"));
        assert!(formatted.contains("duration_ms=100"));
    }
}
