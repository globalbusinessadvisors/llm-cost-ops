// Health check system for monitoring service health

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use super::config::HealthConfig;

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,

    /// Service is degraded but functional
    Degraded,

    /// Service is unhealthy
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub check_duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl ComponentHealth {
    /// Create a healthy component result
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            last_check: chrono::Utc::now(),
            check_duration_ms: 0,
            details: None,
        }
    }

    /// Create a degraded component result
    pub fn degraded(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            last_check: chrono::Utc::now(),
            check_duration_ms: 0,
            details: None,
        }
    }

    /// Create an unhealthy component result
    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            last_check: chrono::Utc::now(),
            check_duration_ms: 0,
            details: None,
        }
    }

    /// Add details
    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    /// Add a single detail
    pub fn with_detail(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let mut details = self.details.unwrap_or_default();
        details.insert(key.into(), value);
        self.details = Some(details);
        self
    }

    /// Set check duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.check_duration_ms = duration_ms;
        self
    }
}

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of the component being checked
    fn name(&self) -> &str;

    /// Perform the health check
    async fn check(&self) -> ComponentHealth;

    /// Check if this is a critical component
    fn is_critical(&self) -> bool {
        false
    }
}

/// Overall system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: Vec<ComponentHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl SystemHealth {
    /// Determine overall status from components
    pub fn determine_status(components: &[ComponentHealth], critical_names: &[String]) -> HealthStatus {
        let mut has_degraded = false;

        for component in components {
            // Check if component is critical
            let is_critical = critical_names.contains(&component.name);

            match component.status {
                HealthStatus::Unhealthy if is_critical => {
                    // Critical component unhealthy = system unhealthy
                    return HealthStatus::Unhealthy;
                }
                HealthStatus::Unhealthy => {
                    // Non-critical component unhealthy = degraded
                    has_degraded = true;
                }
                HealthStatus::Degraded => {
                    has_degraded = true;
                }
                HealthStatus::Healthy => {}
            }
        }

        if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Health checker manager
pub struct HealthChecker {
    checks: Arc<RwLock<Vec<Arc<dyn HealthCheck>>>>,
    config: HealthConfig,
    start_time: Instant,
    critical_components: Arc<RwLock<Vec<String>>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthConfig) -> Self {
        Self {
            checks: Arc::new(RwLock::new(Vec::new())),
            config,
            start_time: Instant::now(),
            critical_components: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a health check
    pub async fn register(&self, check: Arc<dyn HealthCheck>) {
        let mut checks = self.checks.write().await;

        // If this is a critical check, add to critical list
        if check.is_critical() {
            let mut critical = self.critical_components.write().await;
            critical.push(check.name().to_string());
        }

        checks.push(check);
    }

    /// Mark a component as critical
    pub async fn mark_critical(&self, component_name: impl Into<String>) {
        let mut critical = self.critical_components.write().await;
        let name = component_name.into();
        if !critical.contains(&name) {
            critical.push(name);
        }
    }

    /// Perform all health checks
    pub async fn check_health(&self) -> SystemHealth {
        let checks = self.checks.read().await.clone();
        let critical = self.critical_components.read().await.clone();

        // Run all checks concurrently
        let check_futures: Vec<_> = checks
            .iter()
            .map(|check| async move {
                let start = Instant::now();
                let mut result = check.check().await;
                result.check_duration_ms = start.elapsed().as_millis() as u64;
                result
            })
            .collect();

        let components = futures::future::join_all(check_futures).await;

        let status = SystemHealth::determine_status(&components, &critical);

        SystemHealth {
            status,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            components,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check liveness (basic check that service is running)
    pub async fn check_liveness(&self) -> HealthStatus {
        // Liveness is just a ping - if we can respond, we're alive
        HealthStatus::Healthy
    }

    /// Check readiness (service is ready to accept traffic)
    pub async fn check_readiness(&self) -> HealthStatus {
        let health = self.check_health().await;

        // Ready if not unhealthy (degraded is acceptable for readiness)
        match health.status {
            HealthStatus::Healthy | HealthStatus::Degraded => HealthStatus::Healthy,
            HealthStatus::Unhealthy => HealthStatus::Unhealthy,
        }
    }

    /// Get configuration
    pub fn config(&self) -> &HealthConfig {
        &self.config
    }
}

/// Database health check
pub struct DatabaseHealthCheck {
    name: String,
    critical: bool,
}

impl DatabaseHealthCheck {
    pub fn new(name: impl Into<String>, critical: bool) -> Self {
        Self {
            name: name.into(),
            critical,
        }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> ComponentHealth {
        // In a real implementation, this would check database connectivity
        // For now, we'll simulate it
        ComponentHealth::healthy(&self.name)
            .with_detail("type", serde_json::json!("database"))
    }

    fn is_critical(&self) -> bool {
        self.critical
    }
}

/// Cache health check
pub struct CacheHealthCheck {
    name: String,
    critical: bool,
}

impl CacheHealthCheck {
    pub fn new(name: impl Into<String>, critical: bool) -> Self {
        Self {
            name: name.into(),
            critical,
        }
    }
}

#[async_trait]
impl HealthCheck for CacheHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> ComponentHealth {
        ComponentHealth::healthy(&self.name)
            .with_detail("type", serde_json::json!("cache"))
    }

    fn is_critical(&self) -> bool {
        self.critical
    }
}

/// External service health check
pub struct ExternalServiceHealthCheck {
    name: String,
    endpoint: String,
    critical: bool,
}

impl ExternalServiceHealthCheck {
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>, critical: bool) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            critical,
        }
    }
}

#[async_trait]
impl HealthCheck for ExternalServiceHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> ComponentHealth {
        // In a real implementation, this would check the external service
        ComponentHealth::healthy(&self.name)
            .with_detail("type", serde_json::json!("external_service"))
            .with_detail("endpoint", serde_json::json!(&self.endpoint))
    }

    fn is_critical(&self) -> bool {
        self.critical
    }
}

/// Custom function-based health check
pub struct FunctionHealthCheck<F>
where
    F: Fn() -> ComponentHealth + Send + Sync,
{
    name: String,
    check_fn: F,
    critical: bool,
}

impl<F> FunctionHealthCheck<F>
where
    F: Fn() -> ComponentHealth + Send + Sync,
{
    pub fn new(name: impl Into<String>, check_fn: F, critical: bool) -> Self {
        Self {
            name: name.into(),
            check_fn,
            critical,
        }
    }
}

#[async_trait]
impl<F> HealthCheck for FunctionHealthCheck<F>
where
    F: Fn() -> ComponentHealth + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> ComponentHealth {
        (self.check_fn)()
    }

    fn is_critical(&self) -> bool {
        self.critical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[test]
    fn test_component_health_creation() {
        let health = ComponentHealth::healthy("test");
        assert_eq!(health.name, "test");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.message.is_none());

        let degraded = ComponentHealth::degraded("test", "some issue");
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert_eq!(degraded.message, Some("some issue".to_string()));

        let unhealthy = ComponentHealth::unhealthy("test", "critical issue");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(unhealthy.message, Some("critical issue".to_string()));
    }

    #[test]
    fn test_component_health_with_details() {
        let mut details = HashMap::new();
        details.insert("key".to_string(), serde_json::json!("value"));

        let health = ComponentHealth::healthy("test").with_details(details);
        assert!(health.details.is_some());
        assert_eq!(
            health.details.unwrap().get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_system_health_determine_status() {
        let components = vec![
            ComponentHealth::healthy("db"),
            ComponentHealth::healthy("cache"),
        ];

        let status = SystemHealth::determine_status(&components, &vec![]);
        assert_eq!(status, HealthStatus::Healthy);

        let components = vec![
            ComponentHealth::healthy("db"),
            ComponentHealth::degraded("cache", "slow"),
        ];

        let status = SystemHealth::determine_status(&components, &vec![]);
        assert_eq!(status, HealthStatus::Degraded);

        let components = vec![
            ComponentHealth::unhealthy("db", "down"),
            ComponentHealth::healthy("cache"),
        ];

        let critical = vec!["db".to_string()];
        let status = SystemHealth::determine_status(&components, &critical);
        assert_eq!(status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_health_checker_creation() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        let health = checker.check_health().await;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.components.len(), 0);
    }

    #[tokio::test]
    async fn test_health_checker_register() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        let db_check = Arc::new(DatabaseHealthCheck::new("database", true));
        checker.register(db_check).await;

        let health = checker.check_health().await;
        assert_eq!(health.components.len(), 1);
        assert_eq!(health.components[0].name, "database");
    }

    #[tokio::test]
    async fn test_health_checker_liveness() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        let status = checker.check_liveness().await;
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_checker_readiness() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        let status = checker.check_readiness().await;
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_database_health_check() {
        let check = DatabaseHealthCheck::new("test_db", true);
        assert_eq!(check.name(), "test_db");
        assert!(check.is_critical());

        let health = check.check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_cache_health_check() {
        let check = CacheHealthCheck::new("test_cache", false);
        assert_eq!(check.name(), "test_cache");
        assert!(!check.is_critical());

        let health = check.check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_external_service_health_check() {
        let check = ExternalServiceHealthCheck::new(
            "external_api",
            "https://api.example.com",
            true,
        );
        assert_eq!(check.name(), "external_api");
        assert!(check.is_critical());

        let health = check.check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_function_health_check() {
        let check = FunctionHealthCheck::new(
            "custom",
            || ComponentHealth::healthy("custom"),
            false,
        );

        let health = check.check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_mark_critical() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        checker.mark_critical("database").await;

        let critical = checker.critical_components.read().await;
        assert!(critical.contains(&"database".to_string()));
    }
}
