// Metrics collectors for various subsystems

use std::time::Instant;

/// Ingestion metrics
pub struct IngestionMetrics;

impl IngestionMetrics {
    /// Record a successful ingestion request
    pub fn record_success(_organization_id: &str, record_count: usize, duration_ms: f64) {
        metrics::counter!("llm_cost_ops_ingestion_requests_success_total", 1);
        metrics::counter!("llm_cost_ops_ingestion_records_accepted_total", record_count as u64);
        metrics::histogram!("llm_cost_ops_ingestion_duration_ms", duration_ms);
    }

    /// Record a failed ingestion request
    pub fn record_failure(_organization_id: &str, _error_type: &str, duration_ms: f64) {
        metrics::counter!("llm_cost_ops_ingestion_requests_failed_total", 1);
        metrics::histogram!("llm_cost_ops_ingestion_duration_ms", duration_ms);
    }

    /// Record rejected records in batch
    pub fn record_rejected(_organization_id: &str, count: usize) {
        metrics::counter!("llm_cost_ops_ingestion_records_rejected_total", count as u64);
    }

    /// Record validation errors
    pub fn record_validation_error(_organization_id: &str, _error_type: &str) {
        metrics::counter!("llm_cost_ops_ingestion_validation_errors_total", 1);
    }

    /// Record batch size
    pub fn record_batch_size(size: usize) {
        metrics::histogram!("llm_cost_ops_ingestion_batch_size", size as f64);
    }
}

/// Rate limiting metrics
pub struct RateLimitMetrics;

impl RateLimitMetrics {
    /// Record a rate limit check
    pub fn record_check(_organization_id: &str, allowed: bool) {
        if allowed {
            metrics::counter!("llm_cost_ops_ratelimit_checks_allowed_total", 1);
        } else {
            metrics::counter!("llm_cost_ops_ratelimit_checks_blocked_total", 1);
            metrics::counter!("llm_cost_ops_ratelimit_blocks_total", 1);
        }
    }

    /// Record current rate limit usage
    pub fn record_usage(_organization_id: &str, current: u64, limit: u64, remaining: u64) {
        metrics::gauge!("llm_cost_ops_ratelimit_current", current as f64);
        metrics::gauge!("llm_cost_ops_ratelimit_limit", limit as f64);
        metrics::gauge!("llm_cost_ops_ratelimit_remaining", remaining as f64);

        let usage_percent = if limit > 0 {
            (current as f64 / limit as f64) * 100.0
        } else {
            0.0
        };

        metrics::gauge!("llm_cost_ops_ratelimit_usage_percent", usage_percent);
    }

    /// Record rate limiter error
    pub fn record_error(_organization_id: &str, _error_type: &str) {
        metrics::counter!("llm_cost_ops_ratelimit_errors_total", 1);
    }
}

/// Storage metrics
pub struct StorageMetrics;

impl StorageMetrics {
    /// Record a storage operation
    pub fn record_operation(_operation: &str, success: bool, duration_ms: f64) {
        if success {
            metrics::counter!("llm_cost_ops_storage_operations_success_total", 1);
        } else {
            metrics::counter!("llm_cost_ops_storage_operations_failed_total", 1);
        }
        metrics::histogram!("llm_cost_ops_storage_duration_ms", duration_ms);
    }

    /// Record database connection pool stats
    pub fn record_pool_stats(active: usize, idle: usize, max: usize) {
        metrics::gauge!("llm_cost_ops_storage_pool_active", active as f64);
        metrics::gauge!("llm_cost_ops_storage_pool_idle", idle as f64);
        metrics::gauge!("llm_cost_ops_storage_pool_max", max as f64);

        let utilization = if max > 0 {
            (active as f64 / max as f64) * 100.0
        } else {
            0.0
        };
        metrics::gauge!("llm_cost_ops_storage_pool_utilization_percent", utilization);
    }

    /// Record query execution
    pub fn record_query(_query_type: &str, duration_ms: f64) {
        metrics::histogram!("llm_cost_ops_storage_query_duration_ms", duration_ms);
        metrics::counter!("llm_cost_ops_storage_queries_total", 1);
    }
}

/// HTTP request metrics
pub struct HttpMetrics;

impl HttpMetrics {
    /// Record an HTTP request
    pub fn record_request(_method: &str, _path: &str, status_code: u16, duration_ms: f64) {
        metrics::counter!("llm_cost_ops_http_requests_total", 1);
        metrics::histogram!("llm_cost_ops_http_request_duration_ms", duration_ms);

        // Record by status class (2xx, 4xx, 5xx)
        let status_class = status_code / 100;
        match status_class {
            2 => metrics::counter!("llm_cost_ops_http_requests_2xx_total", 1),
            4 => metrics::counter!("llm_cost_ops_http_requests_4xx_total", 1),
            5 => metrics::counter!("llm_cost_ops_http_requests_5xx_total", 1),
            _ => {}
        }
    }

    /// Record request size
    pub fn record_request_size(size_bytes: u64) {
        metrics::histogram!("llm_cost_ops_http_request_size_bytes", size_bytes as f64);
    }

    /// Record response size
    pub fn record_response_size(size_bytes: u64) {
        metrics::histogram!("llm_cost_ops_http_response_size_bytes", size_bytes as f64);
    }

    /// Record active connections
    pub fn record_active_connections(count: usize) {
        metrics::gauge!("llm_cost_ops_http_active_connections", count as f64);
    }
}

/// System metrics
pub struct SystemMetrics;

impl SystemMetrics {
    /// Record application uptime
    pub fn record_uptime(uptime_seconds: u64) {
        metrics::gauge!("llm_cost_ops_uptime_seconds", uptime_seconds as f64);
    }

    /// Record memory usage
    pub fn record_memory_usage(used_bytes: u64, available_bytes: u64) {
        metrics::gauge!("llm_cost_ops_memory_used_bytes", used_bytes as f64);
        metrics::gauge!("llm_cost_ops_memory_available_bytes", available_bytes as f64);
    }

    /// Record CPU usage
    pub fn record_cpu_usage(usage_percent: f64) {
        metrics::gauge!("llm_cost_ops_cpu_usage_percent", usage_percent);
    }
}

/// Helper function to record ingestion request
pub fn record_ingestion_request<F, T>(organization_id: &str, record_count: usize, f: F) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    IngestionMetrics::record_success(organization_id, record_count, duration_ms);

    result
}

/// Helper function to record rate limit check
pub fn record_rate_limit_check(organization_id: &str, allowed: bool) {
    RateLimitMetrics::record_check(organization_id, allowed);
}

/// Helper function to record storage operation
pub fn record_storage_operation<F, T>(operation: &str, f: F) -> crate::domain::Result<T>
where
    F: FnOnce() -> crate::domain::Result<T>,
{
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    let success = result.is_ok();
    StorageMetrics::record_operation(operation, success, duration_ms);

    result
}

/// Timer guard that automatically records duration when dropped
pub struct MetricsTimer {
    start: Instant,
    metric_name: &'static str,
}

impl MetricsTimer {
    /// Create a new timer
    pub fn new(metric_name: &'static str) -> Self {
        Self {
            start: Instant::now(),
            metric_name,
        }
    }
}

impl Drop for MetricsTimer {
    fn drop(&mut self) {
        let duration_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        metrics::histogram!(self.metric_name, duration_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingestion_metrics() {
        IngestionMetrics::record_success("org-test", 10, 25.5);
        IngestionMetrics::record_failure("org-test", "validation", 15.2);
        IngestionMetrics::record_rejected("org-test", 2);
        IngestionMetrics::record_validation_error("org-test", "invalid_tokens");
        IngestionMetrics::record_batch_size(50);
    }

    #[test]
    fn test_rate_limit_metrics() {
        RateLimitMetrics::record_check("org-test", true);
        RateLimitMetrics::record_check("org-test", false);
        RateLimitMetrics::record_usage("org-test", 750, 1000, 250);
        RateLimitMetrics::record_error("org-test", "redis_connection");
    }

    #[test]
    fn test_storage_metrics() {
        StorageMetrics::record_operation("insert", true, 5.2);
        StorageMetrics::record_operation("select", false, 12.8);
        StorageMetrics::record_pool_stats(5, 10, 20);
        StorageMetrics::record_query("usage_insert", 3.4);
    }

    #[test]
    fn test_http_metrics() {
        HttpMetrics::record_request("POST", "/v1/usage", 200, 45.3);
        HttpMetrics::record_request("GET", "/health", 200, 1.2);
        HttpMetrics::record_request_size(1024);
        HttpMetrics::record_response_size(512);
        HttpMetrics::record_active_connections(42);
    }

    #[test]
    fn test_metrics_timer() {
        let _timer = MetricsTimer::new("test_operation");
        // Timer will record duration when dropped
    }
}
