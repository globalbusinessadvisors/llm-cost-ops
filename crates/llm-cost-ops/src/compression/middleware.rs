// HTTP compression middleware for Axum

use super::{
    codec, CompressionAlgorithm, CompressionConfig, CompressionError,
    CompressionMetrics, CompressionResult,
};
use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use std::str::FromStr;
use std::sync::Arc;
use tower::{Layer, Service};
use tracing::{debug, warn};

/// Compression layer for Axum
#[derive(Clone)]
pub struct CompressionLayer {
    config: Arc<CompressionConfig>,
    metrics: Arc<CompressionMetrics>,
}

impl CompressionLayer {
    /// Create a new compression layer
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config: Arc::new(config),
            metrics: super::metrics::get_metrics(),
        }
    }

    /// Create with custom metrics
    pub fn with_metrics(config: CompressionConfig, metrics: Arc<CompressionMetrics>) -> Self {
        Self {
            config: Arc::new(config),
            metrics,
        }
    }
}

impl<S> Layer<S> for CompressionLayer {
    type Service = CompressionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CompressionService {
            inner,
            config: self.config.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

/// Compression service
#[derive(Clone)]
pub struct CompressionService<S> {
    inner: S,
    config: Arc<CompressionConfig>,
    metrics: Arc<CompressionMetrics>,
}

impl<S> Service<Request> for CompressionService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let config = self.config.clone();
        let metrics = self.metrics.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Handle request decompression if enabled
            let request = if config.compress_requests {
                match decompress_request(request, &config, &metrics).await {
                    Ok(req) => req,
                    Err(e) => {
                        warn!(error = %e, "Failed to decompress request");
                        return Ok(error_response(
                            StatusCode::BAD_REQUEST,
                            "Failed to decompress request body",
                        ));
                    }
                }
            } else {
                request
            };

            // Get Accept-Encoding header before passing request
            let accept_encoding = request
                .headers()
                .get(header::ACCEPT_ENCODING)
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            // Call inner service
            let response = inner.call(request).await?;

            // Handle response compression if enabled
            let response = if config.compress_responses {
                if let Some(accept_encoding) = accept_encoding {
                    compress_response(response, &accept_encoding, &config, &metrics).await
                } else {
                    response
                }
            } else {
                response
            };

            Ok(response)
        })
    }
}

/// Decompress request body
async fn decompress_request(
    request: Request,
    _config: &CompressionConfig,
    metrics: &CompressionMetrics,
) -> CompressionResult<Request> {
    let (mut parts, body) = request.into_parts();

    // Check for Content-Encoding header
    let encoding = parts
        .headers
        .get(header::CONTENT_ENCODING)
        .and_then(|h| h.to_str().ok());

    let encoding = match encoding {
        Some(e) => e,
        None => return Ok(Request::from_parts(parts, body)), // No encoding, return as-is
    };

    // Parse algorithm
    let algorithm = match CompressionAlgorithm::from_str(encoding) {
        Ok(algo) => algo,
        Err(_) => return Ok(Request::from_parts(parts, body)), // Unknown encoding, pass through
    };

    if algorithm == CompressionAlgorithm::Identity {
        return Ok(Request::from_parts(parts, body));
    }

    // Collect body bytes
    let bytes = body
        .collect()
        .await
        .map_err(|e| CompressionError::DecompressionFailed(e.to_string()))?
        .to_bytes();

    // Decompress
    let (decompressed, stats) = codec::decompress(&bytes, algorithm)?;

    // Record metrics
    metrics.record_decompression(&stats);

    // Remove Content-Encoding header
    parts.headers.remove(header::CONTENT_ENCODING);

    // Update Content-Length
    parts.headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&decompressed.len().to_string()).unwrap(),
    );

    debug!(
        algorithm = %algorithm,
        original_size = stats.compressed_size,
        decompressed_size = stats.original_size,
        "Request decompressed"
    );

    Ok(Request::from_parts(parts, Body::from(decompressed)))
}

/// Compress response body
async fn compress_response(
    response: Response,
    accept_encoding: &str,
    config: &CompressionConfig,
    metrics: &CompressionMetrics,
) -> Response {
    // Select compression algorithm
    let algorithm = match config.select_algorithm(Some(accept_encoding)) {
        Some(algo) if algo != CompressionAlgorithm::Identity => algo,
        _ => return response, // No compression
    };

    let (mut parts, body) = response.into_parts();

    // Check if content type should be compressed
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if !config.should_compress_mime_type(content_type) {
        debug!(
            content_type = content_type,
            "Content type not compressible"
        );
        return Response::from_parts(parts, body);
    }

    // Collect body bytes
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            warn!(error = %e, "Failed to collect response body");
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error");
        }
    };

    // Check size thresholds
    if !config.should_compress_size(bytes.len()) {
        debug!(
            size = bytes.len(),
            min_size = config.min_size,
            "Response too small to compress"
        );
        return Response::from_parts(parts, Body::from(bytes));
    }

    // Compress
    let (compressed, stats) = match codec::compress(&bytes, algorithm, config.level) {
        Ok(result) => result,
        Err(e) => {
            warn!(error = %e, algorithm = %algorithm, "Compression failed");
            metrics.record_error(Some(algorithm), "compress");
            return Response::from_parts(parts, Body::from(bytes));
        }
    };

    // Check if compression was beneficial
    if compressed.len() >= bytes.len() {
        debug!(
            original_size = bytes.len(),
            compressed_size = compressed.len(),
            "Compression not beneficial, using original"
        );
        return Response::from_parts(parts, Body::from(bytes));
    }

    // Record metrics
    metrics.record_compression(&stats);

    // Update headers
    parts.headers.insert(
        header::CONTENT_ENCODING,
        HeaderValue::from_str(algorithm.as_str()).unwrap(),
    );

    parts.headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&compressed.len().to_string()).unwrap(),
    );

    // Add Vary header to indicate content negotiation
    parts
        .headers
        .entry(header::VARY)
        .or_insert(HeaderValue::from_static("Accept-Encoding"));

    debug!(
        algorithm = %algorithm,
        original_size = stats.original_size,
        compressed_size = stats.compressed_size,
        ratio = stats.compression_ratio,
        savings_pct = stats.compression_percentage(),
        "Response compressed"
    );

    Response::from_parts(parts, Body::from(compressed))
}

/// Create error response
fn error_response(status: StatusCode, message: &str) -> Response {
    (status, message.to_string()).into_response()
}

/// Create compression layer with default config
pub fn compression_layer() -> CompressionLayer {
    CompressionLayer::new(CompressionConfig::default())
}

/// Middleware function for manual application
pub async fn compression_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = Arc::new(CompressionConfig::default());
    let metrics = super::metrics::get_metrics();

    // Handle request decompression
    let request = if config.compress_requests {
        match decompress_request(request, &config, &metrics).await {
            Ok(req) => req,
            Err(e) => {
                warn!(error = %e, "Failed to decompress request");
                return Ok(error_response(
                    StatusCode::BAD_REQUEST,
                    "Failed to decompress request body",
                ));
            }
        }
    } else {
        request
    };

    // Get Accept-Encoding before passing request
    let accept_encoding = request
        .headers()
        .get(header::ACCEPT_ENCODING)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Call next middleware/handler
    let response = next.run(request).await;

    // Handle response compression
    let response = if config.compress_responses {
        if let Some(accept_encoding) = accept_encoding {
            compress_response(response, &accept_encoding, &config, &metrics).await
        } else {
            response
        }
    } else {
        response
    };

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompressionLevel;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    async fn test_handler() -> ([(header::HeaderName, &'static str); 1], &'static str) {
        ([
            (header::CONTENT_TYPE, "text/plain"),
        ], "Hello, World! This is a test response that should be compressed. Adding more text to ensure it compresses well and the compressed size is smaller than the original.")
    }

    #[tokio::test]
    async fn test_compression_layer_creation() {
        let config = CompressionConfig::default();
        let layer = CompressionLayer::new(config);
        // Should not panic
        drop(layer);
    }

    #[tokio::test]
    async fn test_compression_response() {
        let config = CompressionConfig {
            enabled: true,
            level: CompressionLevel::Default,
            algorithms: vec![CompressionAlgorithm::Gzip],
            min_size: 10, // Low threshold for testing
            max_size: None,
            mime_types: vec!["text/*".to_string()],
            compress_requests: false,
            compress_responses: true,
            buffer_size: 8192,
        };

        let app = Router::new()
            .route("/", get(test_handler))
            .layer(CompressionLayer::new(config));

        let request = Request::builder()
            .uri("/")
            .header(header::ACCEPT_ENCODING, "gzip")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should have Content-Encoding header
        assert_eq!(
            response.headers().get(header::CONTENT_ENCODING).unwrap(),
            "gzip"
        );
    }

    #[tokio::test]
    async fn test_no_compression_without_accept_encoding() {
        let config = CompressionConfig::default();

        let app = Router::new()
            .route("/", get(test_handler))
            .layer(CompressionLayer::new(config));

        let request = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should not have Content-Encoding header
        assert!(response.headers().get(header::CONTENT_ENCODING).is_none());
    }

    #[tokio::test]
    async fn test_compression_with_brotli() {
        let config = CompressionConfig {
            enabled: true,
            level: CompressionLevel::Default,
            algorithms: vec![CompressionAlgorithm::Brotli],
            min_size: 10,
            max_size: None,
            mime_types: vec!["text/*".to_string()],
            compress_requests: false,
            compress_responses: true,
            buffer_size: 8192,
        };

        let app = Router::new()
            .route("/", get(test_handler))
            .layer(CompressionLayer::new(config));

        let request = Request::builder()
            .uri("/")
            .header(header::ACCEPT_ENCODING, "br")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should use brotli
        assert_eq!(
            response.headers().get(header::CONTENT_ENCODING).unwrap(),
            "br"
        );
    }
}
