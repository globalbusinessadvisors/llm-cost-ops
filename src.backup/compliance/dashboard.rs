//! Compliance dashboard for real-time monitoring
//!
//! Provides real-time compliance metrics, policy status, and security alerts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Removed unused imports

/// Dashboard error types
#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Data not available: {0}")]
    DataNotAvailable(String),

    #[error("Invalid metric: {0}")]
    InvalidMetric(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type DashboardResult<T> = Result<T, DashboardError>;

/// Overall compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceStatus {
    Compliant,
    Warning,
    Critical,
    Unknown,
}

impl ComplianceStatus {
    pub fn from_score(score: f64) -> Self {
        if score >= 95.0 {
            ComplianceStatus::Compliant
        } else if score >= 80.0 {
            ComplianceStatus::Warning
        } else if score > 0.0 {
            ComplianceStatus::Critical
        } else {
            ComplianceStatus::Unknown
        }
    }
}

/// Compliance score calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScore {
    pub overall_score: f64,
    pub policy_score: f64,
    pub audit_score: f64,
    pub security_score: f64,
    pub data_governance_score: f64,
    pub calculated_at: DateTime<Utc>,
}

impl ComplianceScore {
    pub fn calculate(
        policy_score: f64,
        audit_score: f64,
        security_score: f64,
        data_governance_score: f64,
    ) -> Self {
        let overall_score = (policy_score + audit_score + security_score + data_governance_score) / 4.0;

        Self {
            overall_score,
            policy_score,
            audit_score,
            security_score,
            data_governance_score,
            calculated_at: Utc::now(),
        }
    }

    pub fn status(&self) -> ComplianceStatus {
        ComplianceStatus::from_score(self.overall_score)
    }
}

/// Trend data for metrics over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub timestamps: Vec<DateTime<Utc>>,
    pub values: Vec<f64>,
    pub trend_direction: TrendDirection,
    pub change_percentage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

impl TrendData {
    pub fn new(timestamps: Vec<DateTime<Utc>>, values: Vec<f64>) -> Self {
        let (trend_direction, change_percentage) = Self::calculate_trend(&values);

        Self {
            timestamps,
            values,
            trend_direction,
            change_percentage,
        }
    }

    fn calculate_trend(values: &[f64]) -> (TrendDirection, f64) {
        if values.len() < 2 {
            return (TrendDirection::Stable, 0.0);
        }

        let first = values.first().unwrap();
        let last = values.last().unwrap();

        if first == &0.0 {
            return (TrendDirection::Stable, 0.0);
        }

        let change = ((last - first) / first) * 100.0;

        let direction = if change > 5.0 {
            TrendDirection::Up
        } else if change < -5.0 {
            TrendDirection::Down
        } else {
            TrendDirection::Stable
        };

        (direction, change)
    }
}

/// Policy compliance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetric {
    pub total_policies: usize,
    pub active_policies: usize,
    pub draft_policies: usize,
    pub deprecated_policies: usize,
    pub policies_needing_review: usize,
    pub policies_by_type: HashMap<String, usize>,
    pub policy_violations_24h: usize,
    pub compliance_rate: f64,
}

/// Audit log metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetric {
    pub total_events_24h: usize,
    pub total_events_7d: usize,
    pub failed_auth_attempts_24h: usize,
    pub access_denials_24h: usize,
    pub high_severity_events_24h: usize,
    pub events_by_type: HashMap<String, usize>,
    pub unique_users_24h: usize,
    pub anomalies_detected: usize,
}

/// Security metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetric {
    pub open_incidents: usize,
    pub incidents_24h: usize,
    pub incidents_7d: usize,
    pub critical_incidents: usize,
    pub mean_time_to_detect: f64,
    pub mean_time_to_resolve: f64,
    pub security_score: f64,
    pub vulnerabilities: Vec<Vulnerability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: Uuid,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub remediation_status: String,
}

/// GDPR compliance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprMetric {
    pub open_requests: usize,
    pub requests_24h: usize,
    pub requests_7d: usize,
    pub overdue_requests: usize,
    pub average_response_time_hours: f64,
    pub requests_by_type: HashMap<String, usize>,
    pub data_breach_count_30d: usize,
}

/// Data retention metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionMetric {
    pub total_data_sets: usize,
    pub compliant_data_sets: usize,
    pub non_compliant_data_sets: usize,
    pub data_sets_expiring_7d: usize,
    pub data_sets_expiring_30d: usize,
    pub legal_holds_active: usize,
    pub compliance_rate: f64,
}

/// Alert metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMetric {
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub info_alerts: usize,
    pub recent_alerts: Vec<Alert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub refresh_interval_seconds: u64,
    pub retention_days: u32,
    pub alert_thresholds: AlertThresholds,
    pub enabled_widgets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub failed_auth_threshold: usize,
    pub access_denial_threshold: usize,
    pub incident_threshold: usize,
    pub compliance_score_threshold: f64,
    pub gdpr_response_time_threshold_hours: f64,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 30,
            retention_days: 90,
            alert_thresholds: AlertThresholds {
                failed_auth_threshold: 10,
                access_denial_threshold: 5,
                incident_threshold: 3,
                compliance_score_threshold: 80.0,
                gdpr_response_time_threshold_hours: 72.0,
            },
            enabled_widgets: vec![
                "compliance_score".to_string(),
                "policy_metrics".to_string(),
                "audit_metrics".to_string(),
                "security_metrics".to_string(),
                "gdpr_metrics".to_string(),
                "retention_metrics".to_string(),
                "alerts".to_string(),
            ],
        }
    }
}

/// Complete dashboard metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    pub compliance_score: ComplianceScore,
    pub policy_metrics: PolicyMetric,
    pub audit_metrics: AuditMetric,
    pub security_metrics: SecurityMetric,
    pub gdpr_metrics: GdprMetric,
    pub retention_metrics: RetentionMetric,
    pub alert_metrics: AlertMetric,
    pub trends: HashMap<String, TrendData>,
    pub last_updated: DateTime<Utc>,
}

/// Compliance dashboard manager
pub struct ComplianceDashboard {
    config: DashboardConfig,
    metrics_cache: Arc<RwLock<Option<DashboardMetrics>>>,
    alerts: Arc<RwLock<Vec<Alert>>>,
}

impl ComplianceDashboard {
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            metrics_cache: Arc::new(RwLock::new(None)),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current dashboard metrics
    pub async fn get_metrics(&self) -> DashboardResult<DashboardMetrics> {
        // Check cache first
        let cache = self.metrics_cache.read().await;
        if let Some(metrics) = cache.as_ref() {
            if Utc::now().signed_duration_since(metrics.last_updated).num_seconds()
                < self.config.refresh_interval_seconds as i64
            {
                return Ok(metrics.clone());
            }
        }
        drop(cache);

        // Refresh metrics
        let metrics = self.refresh_metrics().await?;

        // Update cache
        let mut cache = self.metrics_cache.write().await;
        *cache = Some(metrics.clone());

        Ok(metrics)
    }

    /// Refresh all dashboard metrics
    async fn refresh_metrics(&self) -> DashboardResult<DashboardMetrics> {
        // In a real implementation, these would query from databases
        let policy_metrics = self.collect_policy_metrics().await?;
        let audit_metrics = self.collect_audit_metrics().await?;
        let security_metrics = self.collect_security_metrics().await?;
        let gdpr_metrics = self.collect_gdpr_metrics().await?;
        let retention_metrics = self.collect_retention_metrics().await?;
        let alert_metrics = self.collect_alert_metrics().await?;

        // Calculate compliance score
        let compliance_score = ComplianceScore::calculate(
            policy_metrics.compliance_rate,
            audit_metrics.total_events_24h as f64 / 1000.0 * 100.0, // Normalize
            security_metrics.security_score,
            retention_metrics.compliance_rate,
        );

        // Collect trend data
        let trends = self.collect_trends().await?;

        Ok(DashboardMetrics {
            compliance_score,
            policy_metrics,
            audit_metrics,
            security_metrics,
            gdpr_metrics,
            retention_metrics,
            alert_metrics,
            trends,
            last_updated: Utc::now(),
        })
    }

    async fn collect_policy_metrics(&self) -> DashboardResult<PolicyMetric> {
        Ok(PolicyMetric {
            total_policies: 45,
            active_policies: 42,
            draft_policies: 2,
            deprecated_policies: 1,
            policies_needing_review: 3,
            policies_by_type: HashMap::from([
                ("retention".to_string(), 15),
                ("access".to_string(), 12),
                ("encryption".to_string(), 8),
                ("audit".to_string(), 10),
            ]),
            policy_violations_24h: 2,
            compliance_rate: 95.5,
        })
    }

    async fn collect_audit_metrics(&self) -> DashboardResult<AuditMetric> {
        Ok(AuditMetric {
            total_events_24h: 1250,
            total_events_7d: 8500,
            failed_auth_attempts_24h: 5,
            access_denials_24h: 2,
            high_severity_events_24h: 3,
            events_by_type: HashMap::from([
                ("auth_login".to_string(), 450),
                ("resource_read".to_string(), 320),
                ("resource_update".to_string(), 180),
            ]),
            unique_users_24h: 125,
            anomalies_detected: 1,
        })
    }

    async fn collect_security_metrics(&self) -> DashboardResult<SecurityMetric> {
        Ok(SecurityMetric {
            open_incidents: 2,
            incidents_24h: 1,
            incidents_7d: 8,
            critical_incidents: 0,
            mean_time_to_detect: 2.5,
            mean_time_to_resolve: 4.2,
            security_score: 92.0,
            vulnerabilities: vec![],
        })
    }

    async fn collect_gdpr_metrics(&self) -> DashboardResult<GdprMetric> {
        Ok(GdprMetric {
            open_requests: 8,
            requests_24h: 3,
            requests_7d: 22,
            overdue_requests: 1,
            average_response_time_hours: 18.5,
            requests_by_type: HashMap::from([
                ("access".to_string(), 10),
                ("deletion".to_string(), 8),
                ("rectification".to_string(), 4),
            ]),
            data_breach_count_30d: 0,
        })
    }

    async fn collect_retention_metrics(&self) -> DashboardResult<RetentionMetric> {
        let total = 1000;
        let compliant = 980;

        Ok(RetentionMetric {
            total_data_sets: total,
            compliant_data_sets: compliant,
            non_compliant_data_sets: total - compliant,
            data_sets_expiring_7d: 15,
            data_sets_expiring_30d: 45,
            legal_holds_active: 3,
            compliance_rate: (compliant as f64 / total as f64) * 100.0,
        })
    }

    async fn collect_alert_metrics(&self) -> DashboardResult<AlertMetric> {
        let alerts = self.alerts.read().await;
        let active_alerts: Vec<_> = alerts.iter().filter(|a| !a.acknowledged).cloned().collect();

        let critical = active_alerts.iter().filter(|a| a.severity == "critical").count();
        let warning = active_alerts.iter().filter(|a| a.severity == "warning").count();
        let info = active_alerts.iter().filter(|a| a.severity == "info").count();

        Ok(AlertMetric {
            active_alerts: active_alerts.len(),
            critical_alerts: critical,
            warning_alerts: warning,
            info_alerts: info,
            recent_alerts: active_alerts.into_iter().take(10).collect(),
        })
    }

    async fn collect_trends(&self) -> DashboardResult<HashMap<String, TrendData>> {
        let mut trends = HashMap::new();

        let now = Utc::now();
        let timestamps: Vec<_> = (0..7)
            .rev()
            .map(|i| now - chrono::Duration::days(i))
            .collect();

        trends.insert(
            "compliance_score".to_string(),
            TrendData::new(timestamps.clone(), vec![92.0, 93.5, 94.0, 95.0, 95.5, 96.0, 95.5]),
        );

        trends.insert(
            "audit_events".to_string(),
            TrendData::new(
                timestamps.clone(),
                vec![1100.0, 1150.0, 1200.0, 1250.0, 1300.0, 1280.0, 1250.0],
            ),
        );

        trends.insert(
            "security_incidents".to_string(),
            TrendData::new(timestamps, vec![2.0, 1.0, 3.0, 1.0, 2.0, 1.0, 1.0]),
        );

        Ok(trends)
    }

    /// Add a new alert
    pub async fn add_alert(&self, alert: Alert) {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);

        alerts.sort_by(|a, b| b.triggered_at.cmp(&a.triggered_at));
        alerts.truncate(100);
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid, acknowledged_by: String) -> DashboardResult<()> {
        let mut alerts = self.alerts.write().await;

        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(acknowledged_by);
            Ok(())
        } else {
            Err(DashboardError::DataNotAvailable(format!(
                "Alert {} not found",
                alert_id
            )))
        }
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.iter().filter(|a| !a.acknowledged).cloned().collect()
    }

    /// Clear metrics cache
    pub async fn clear_cache(&self) {
        let mut cache = self.metrics_cache.write().await;
        *cache = None;
    }

    /// Get dashboard configuration
    pub fn get_config(&self) -> &DashboardConfig {
        &self.config
    }
}

impl Default for ComplianceDashboard {
    fn default() -> Self {
        Self::new(DashboardConfig::default())
    }
}
