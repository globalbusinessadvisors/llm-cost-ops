// API server configuration and setup

use axum::{middleware, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use super::{middleware as api_middleware, routes};

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    /// Server host
    pub host: String,

    /// Server port
    pub port: u16,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Enable CORS
    pub enable_cors: bool,

    /// Enable request logging
    pub enable_logging: bool,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            request_timeout_secs: 30,
            enable_cors: true,
            enable_logging: true,
        }
    }
}

impl ApiServerConfig {
    /// Get socket address
    pub fn socket_addr(&self) -> Result<SocketAddr, String> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e| format!("Invalid socket address: {}", e))
    }
}

/// API server
pub struct ApiServer {
    config: ApiServerConfig,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(config: ApiServerConfig) -> Self {
        Self { config }
    }

    /// Build the router
    pub fn build_router(&self) -> Router {
        create_api_router(&self.config)
    }

    /// Run the server
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.config.socket_addr()?;
        let app = self.build_router();

        tracing::info!("Starting API server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Create API router with middleware
pub fn create_api_router(config: &ApiServerConfig) -> Router {
    let mut router = routes::create_routes();

    // Add middleware layers
    let middleware_stack = ServiceBuilder::new()
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.request_timeout_secs,
        )))
        .layer(middleware::from_fn(api_middleware::request_id_middleware));

    router = router.layer(middleware_stack);

    // Add CORS if enabled
    if config.enable_cors {
        router = router.layer(CorsLayer::permissive());
    }

    // Add tracing if enabled
    if config.enable_logging {
        router = router.layer(TraceLayer::new_for_http());
    }

    router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ApiServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_socket_addr() {
        let config = ApiServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            ..Default::default()
        };

        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:3000");
    }

    #[test]
    fn test_server_creation() {
        let config = ApiServerConfig::default();
        let server = ApiServer::new(config);
        let _router = server.build_router();
    }
}
