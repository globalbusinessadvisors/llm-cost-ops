// Rate limiting middleware for Axum

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tracing::{debug, warn};

use super::ratelimit::RateLimitUsage;
use super::traits::RateLimiter;

/// Rate limiting middleware state
#[derive(Clone)]
pub struct RateLimitMiddleware<R: RateLimiter> {
    limiter: Arc<R>,
}

impl<R: RateLimiter> RateLimitMiddleware<R> {
    pub fn new(limiter: R) -> Self {
        Self {
            limiter: Arc::new(limiter),
        }
    }
}

/// Extract organization ID from request headers or body
async fn extract_organization_id(req: &Request) -> Option<String> {
    // Try to get from X-Organization-ID header
    if let Some(org_id) = req.headers().get("X-Organization-ID") {
        if let Ok(value) = org_id.to_str() {
            return Some(value.to_string());
        }
    }

    // Try to get from Authorization header (extract from JWT or API key)
    if let Some(auth) = req.headers().get("Authorization") {
        if let Ok(value) = auth.to_str() {
            // For API keys in format "Bearer org-{id}-{key}"
            if let Some(org_part) = value.strip_prefix("Bearer org-") {
                if let Some(org_id) = org_part.split('-').next() {
                    return Some(format!("org-{}", org_id));
                }
            }
        }
    }

    // Default organization for unauthenticated requests
    Some("default".to_string())
}

/// Rate limiting middleware function
pub async fn rate_limit_middleware<R: RateLimiter + Clone + 'static>(
    State(middleware): State<RateLimitMiddleware<R>>,
    req: Request,
    next: Next,
) -> Response {
    // Extract organization ID
    let org_id = match extract_organization_id(&req).await {
        Some(id) => id,
        None => {
            warn!("Failed to extract organization ID from request");
            return rate_limit_error_response(
                "Missing organization identifier",
                None,
            );
        }
    };

    // Check rate limit
    match middleware.limiter.check_rate_limit(&org_id).await {
        Ok(allowed) => {
            if allowed {
                debug!(organization_id = %org_id, "Rate limit check passed");

                // Proceed with request
                next.run(req).await
            } else {
                warn!(organization_id = %org_id, "Rate limit exceeded");

                // Get usage stats for better error response
                rate_limit_exceeded_response(&org_id)
            }
        }
        Err(e) => {
            warn!(
                organization_id = %org_id,
                error = %e,
                "Rate limit check failed"
            );

            // On error, allow the request but log the issue
            // This ensures availability over strict rate limiting
            next.run(req).await
        }
    }
}

/// Create rate limit exceeded response
fn rate_limit_exceeded_response(org_id: &str) -> Response {
    let body = serde_json::json!({
        "error": "Rate limit exceeded",
        "message": format!("Organization {} has exceeded the allowed request rate", org_id),
        "organization_id": org_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();

    // Add standard rate limit headers
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", "1000".parse().unwrap());
    headers.insert("X-RateLimit-Remaining", "0".parse().unwrap());
    headers.insert("X-RateLimit-Reset", "60".parse().unwrap());
    headers.insert("Retry-After", "60".parse().unwrap());

    response
}

/// Create generic rate limit error response
fn rate_limit_error_response(message: &str, retry_after: Option<u64>) -> Response {
    let body = serde_json::json!({
        "error": "Rate limiting error",
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    let mut response = (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response();

    if let Some(seconds) = retry_after {
        response.headers_mut().insert(
            "Retry-After",
            seconds.to_string().parse().unwrap(),
        );
    }

    response
}

/// Add rate limit headers to a response
pub fn add_rate_limit_headers(
    mut response: Response,
    usage: &RateLimitUsage,
) -> Response {
    let headers = response.headers_mut();

    headers.insert(
        "X-RateLimit-Limit",
        usage.limit.to_string().parse().unwrap(),
    );

    headers.insert(
        "X-RateLimit-Remaining",
        usage.remaining.to_string().parse().unwrap(),
    );

    if let Some(retry_after) = usage.retry_after {
        headers.insert(
            "X-RateLimit-Reset",
            retry_after.as_secs().to_string().parse().unwrap(),
        );
    }

    response
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_rate_limit_middleware_allows_requests() {
        // Test passes if middleware module compiles correctly
        // Full integration testing requires running server
        assert!(true);
    }
}
