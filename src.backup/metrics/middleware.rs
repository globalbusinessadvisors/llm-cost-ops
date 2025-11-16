// HTTP metrics middleware for Axum

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

use super::collectors::HttpMetrics;

/// Metrics middleware for HTTP requests
pub async fn metrics_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Get request size if available
    if let Some(content_length) = req.headers().get("content-length") {
        if let Ok(size_str) = content_length.to_str() {
            if let Ok(size) = size_str.parse::<u64>() {
                HttpMetrics::record_request_size(size);
            }
        }
    }

    // Process request
    let response = next.run(req).await;

    // Record metrics
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    let status_code = response.status().as_u16();

    HttpMetrics::record_request(&method, &path, status_code, duration_ms);

    // Get response size if available
    if let Some(content_length) = response.headers().get("content-length") {
        if let Ok(size_str) = content_length.to_str() {
            if let Ok(size) = size_str.parse::<u64>() {
                HttpMetrics::record_response_size(size);
            }
        }
    }

    response
}

/// Create a sanitized path for metrics (removes dynamic segments)
pub fn sanitize_path(path: &str) -> String {
    // Replace UUIDs and IDs with placeholders
    let uuid_regex = regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();
    let id_regex = regex::Regex::new(r"/[0-9]+(/|$)").unwrap();

    let sanitized = uuid_regex.replace_all(path, ":uuid");
    let sanitized = id_regex.replace_all(&sanitized, "/:id$1");

    sanitized.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        assert_eq!(
            sanitize_path("/api/organizations/550e8400-e29b-41d4-a716-446655440000/usage"),
            "/api/organizations/:uuid/usage"
        );

        assert_eq!(
            sanitize_path("/api/usage/12345"),
            "/api/usage/:id"
        );

        assert_eq!(
            sanitize_path("/health"),
            "/health"
        );
    }
}
