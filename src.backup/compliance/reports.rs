//! Compliance reporting system
//!
//! Generates various compliance reports in multiple formats for regulatory requirements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::{AuditEventType, AuditSeverity};
use super::policies::{DataClassification, PolicyType};

/// Report error types
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("Report generation failed: {0}")]
    GenerationFailed(String),

    #[error("Invalid report type: {0}")]
    InvalidType(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Data not found: {0}")]
    DataNotFound(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type ReportResult<T> = Result<T, ReportError>;

/// Report output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Json,
    Csv,
    Pdf,
    Html,
}

/// Report types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    AuditLogSummary,
    AccessControl,
    RetentionCompliance,
    SecurityIncident,
    Soc2Evidence,
    GdprRequest,
    EncryptionStatus,
    PolicyCompliance,
    UserActivity,
    DataBreaches,
}

/// Report filtering options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ReportFilter {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
    pub resource_type: Option<String>,
    pub severity: Option<AuditSeverity>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub policy_type: Option<PolicyType>,
    pub classification: Option<DataClassification>,
    pub tags: Option<Vec<String>>,
}


/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub id: Uuid,
    pub report_type: ReportType,
    pub title: String,
    pub description: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_records: usize,
    pub format: ReportFormat,
    pub tags: Vec<String>,
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub summary: String,
    pub data: serde_json::Value,
    pub charts: Vec<ChartData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_type: String,
    pub title: String,
    pub labels: Vec<String>,
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub label: String,
    pub data: Vec<f64>,
}

/// Main compliance report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub metadata: ReportMetadata,
    pub executive_summary: String,
    pub sections: Vec<ReportSection>,
    pub findings: Vec<Finding>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: String,
    pub category: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub affected_resources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: String,
    pub title: String,
    pub description: String,
    pub implementation_steps: Vec<String>,
    pub estimated_effort: String,
}

/// Audit log summary report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogSummary {
    pub total_events: usize,
    pub events_by_type: HashMap<String, usize>,
    pub events_by_severity: HashMap<String, usize>,
    pub failed_auth_attempts: usize,
    pub successful_auth: usize,
    pub resource_changes: usize,
    pub access_denials: usize,
    pub top_users: Vec<UserActivity>,
    pub top_resources: Vec<ResourceAccess>,
    pub anomalies: Vec<AnomalyReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: String,
    pub event_count: usize,
    pub last_activity: DateTime<Utc>,
    pub risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAccess {
    pub resource: String,
    pub access_count: usize,
    pub unique_users: usize,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub detected_at: DateTime<Utc>,
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub details: serde_json::Value,
}

/// Access control report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlReport {
    pub total_policies: usize,
    pub active_policies: usize,
    pub policy_violations: Vec<PolicyViolationDetail>,
    pub role_assignments: HashMap<String, usize>,
    pub permission_usage: HashMap<String, usize>,
    pub excessive_permissions: Vec<ExcessivePermission>,
    pub inactive_users: Vec<InactiveUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationDetail {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub violation_type: String,
    pub occurred_at: DateTime<Utc>,
    pub user_id: String,
    pub resource: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcessivePermission {
    pub user_id: String,
    pub role: String,
    pub unused_permissions: Vec<String>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InactiveUser {
    pub user_id: String,
    pub last_login: DateTime<Utc>,
    pub days_inactive: i64,
    pub assigned_roles: Vec<String>,
}

/// Retention compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionComplianceReport {
    pub total_data_sets: usize,
    pub compliant_data_sets: usize,
    pub non_compliant_data_sets: usize,
    pub retention_violations: Vec<RetentionViolation>,
    pub upcoming_expirations: Vec<DataExpiration>,
    pub legal_holds: Vec<LegalHold>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionViolation {
    pub data_id: Uuid,
    pub data_type: String,
    pub classification: DataClassification,
    pub created_at: DateTime<Utc>,
    pub expected_deletion: DateTime<Utc>,
    pub days_overdue: i64,
    pub policy_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExpiration {
    pub data_id: Uuid,
    pub data_type: String,
    pub expires_at: DateTime<Utc>,
    pub days_until_expiration: i64,
    pub auto_delete_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalHold {
    pub hold_id: Uuid,
    pub data_ids: Vec<Uuid>,
    pub reason: String,
    pub initiated_by: String,
    pub initiated_at: DateTime<Utc>,
    pub expected_release: Option<DateTime<Utc>>,
}

/// Security incident report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncidentReport {
    pub total_incidents: usize,
    pub incidents_by_severity: HashMap<String, usize>,
    pub incidents: Vec<SecurityIncident>,
    pub mean_time_to_detect: f64,
    pub mean_time_to_respond: f64,
    pub open_incidents: usize,
    pub closed_incidents: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub incident_id: Uuid,
    pub incident_type: String,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub affected_systems: Vec<String>,
    pub affected_users: Vec<String>,
    pub root_cause: Option<String>,
    pub remediation: Option<String>,
}

/// SOC 2 evidence collection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2EvidenceReport {
    pub control_objectives: Vec<ControlObjective>,
    pub evidence_items: Vec<EvidenceItem>,
    pub compliance_score: f64,
    pub gaps: Vec<ComplianceGap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlObjective {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub evidence_count: usize,
    pub last_tested: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: Uuid,
    pub control_id: String,
    pub evidence_type: String,
    pub description: String,
    pub collected_at: DateTime<Utc>,
    pub collected_by: String,
    pub verification_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceGap {
    pub control_id: String,
    pub gap_type: String,
    pub description: String,
    pub severity: String,
    pub remediation_plan: Option<String>,
}

/// GDPR request report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprRequestReport {
    pub total_requests: usize,
    pub requests_by_type: HashMap<String, usize>,
    pub requests: Vec<GdprRequest>,
    pub average_response_time: f64,
    pub overdue_requests: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprRequest {
    pub request_id: Uuid,
    pub request_type: String,
    pub user_id: String,
    pub submitted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub response_time_hours: Option<f64>,
}

/// Encryption status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStatusReport {
    pub total_data_stores: usize,
    pub encrypted_stores: usize,
    pub unencrypted_stores: usize,
    pub encryption_coverage: f64,
    pub key_rotations: Vec<KeyRotation>,
    pub non_compliant_stores: Vec<NonCompliantStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotation {
    pub key_id: String,
    pub last_rotation: DateTime<Utc>,
    pub next_rotation: DateTime<Utc>,
    pub rotation_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonCompliantStore {
    pub store_id: String,
    pub store_type: String,
    pub classification: DataClassification,
    pub encryption_required: bool,
    pub current_encryption: Option<String>,
    pub compliance_issue: String,
}

/// Report generator
pub struct ReportGenerator {
    // In a real implementation, this would have database connections, etc.
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate a compliance report
    pub async fn generate_report(
        &self,
        report_type: ReportType,
        filter: ReportFilter,
        format: ReportFormat,
        generated_by: String,
    ) -> ReportResult<ComplianceReport> {
        let metadata = ReportMetadata {
            id: Uuid::new_v4(),
            report_type,
            title: self.get_report_title(report_type),
            description: self.get_report_description(report_type),
            generated_at: Utc::now(),
            generated_by,
            period_start: filter.start_date.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30)),
            period_end: filter.end_date.unwrap_or_else(Utc::now),
            total_records: 0,
            format,
            tags: filter.tags.clone().unwrap_or_default(),
        };

        let report = match report_type {
            ReportType::AuditLogSummary => self.generate_audit_log_summary(metadata, filter).await?,
            ReportType::AccessControl => self.generate_access_control_report(metadata, filter).await?,
            ReportType::RetentionCompliance => self.generate_retention_report(metadata, filter).await?,
            ReportType::SecurityIncident => self.generate_security_incident_report(metadata, filter).await?,
            ReportType::Soc2Evidence => self.generate_soc2_report(metadata, filter).await?,
            ReportType::GdprRequest => self.generate_gdpr_report(metadata, filter).await?,
            ReportType::EncryptionStatus => self.generate_encryption_report(metadata, filter).await?,
            _ => {
                return Err(ReportError::InvalidType(format!(
                    "Report type {:?} not implemented",
                    report_type
                )))
            }
        };

        Ok(report)
    }

    async fn generate_audit_log_summary(
        &self,
        mut metadata: ReportMetadata,
        _filter: ReportFilter,
    ) -> ReportResult<ComplianceReport> {
        // In real implementation, query audit logs from database
        let summary = AuditLogSummary {
            total_events: 1250,
            events_by_type: HashMap::from([
                ("auth_login".to_string(), 450),
                ("resource_read".to_string(), 320),
                ("resource_update".to_string(), 180),
            ]),
            events_by_severity: HashMap::from([
                ("info".to_string(), 1000),
                ("warning".to_string(), 200),
                ("error".to_string(), 50),
            ]),
            failed_auth_attempts: 15,
            successful_auth: 435,
            resource_changes: 280,
            access_denials: 8,
            top_users: vec![],
            top_resources: vec![],
            anomalies: vec![],
        };

        metadata.total_records = summary.total_events;

        Ok(ComplianceReport {
            metadata,
            executive_summary: "Audit log analysis for the reporting period shows normal activity patterns with no significant security concerns.".to_string(),
            sections: vec![
                ReportSection {
                    title: "Event Overview".to_string(),
                    summary: format!("Total of {} events recorded", summary.total_events),
                    data: serde_json::to_value(&summary)?,
                    charts: vec![],
                },
            ],
            findings: vec![],
            recommendations: vec![],
        })
    }

    async fn generate_access_control_report(
        &self,
        metadata: ReportMetadata,
        _filter: ReportFilter,
    ) -> ReportResult<ComplianceReport> {
        let report_data = AccessControlReport {
            total_policies: 45,
            active_policies: 42,
            policy_violations: vec![],
            role_assignments: HashMap::new(),
            permission_usage: HashMap::new(),
            excessive_permissions: vec![],
            inactive_users: vec![],
        };

        Ok(ComplianceReport {
            metadata,
            executive_summary: "Access control review shows good policy compliance with minimal violations.".to_string(),
            sections: vec![
                ReportSection {
                    title: "Policy Status".to_string(),
                    summary: format!("{} active policies out of {} total", report_data.active_policies, report_data.total_policies),
                    data: serde_json::to_value(&report_data)?,
                    charts: vec![],
                },
            ],
            findings: vec![],
            recommendations: vec![],
        })
    }

    async fn generate_retention_report(
        &self,
        metadata: ReportMetadata,
        _filter: ReportFilter,
    ) -> ReportResult<ComplianceReport> {
        let report_data = RetentionComplianceReport {
            total_data_sets: 1000,
            compliant_data_sets: 980,
            non_compliant_data_sets: 20,
            retention_violations: vec![],
            upcoming_expirations: vec![],
            legal_holds: vec![],
        };

        Ok(ComplianceReport {
            metadata,
            executive_summary: "Data retention compliance is at 98% with 20 data sets requiring attention.".to_string(),
            sections: vec![
                ReportSection {
                    title: "Retention Overview".to_string(),
                    summary: format!("{:.1}% compliance rate", (report_data.compliant_data_sets as f64 / report_data.total_data_sets as f64) * 100.0),
                    data: serde_json::to_value(&report_data)?,
                    charts: vec![],
                },
            ],
            findings: vec![],
            recommendations: vec![],
        })
    }

    async fn generate_security_incident_report(
        &self,
        metadata: ReportMetadata,
        _filter: ReportFilter,
    ) -> ReportResult<ComplianceReport> {
        let report_data = SecurityIncidentReport {
            total_incidents: 12,
            incidents_by_severity: HashMap::from([
                ("low".to_string(), 8),
                ("medium".to_string(), 3),
                ("high".to_string(), 1),
            ]),
            incidents: vec![],
            mean_time_to_detect: 2.5,
            mean_time_to_respond: 4.2,
            open_incidents: 2,
            closed_incidents: 10,
        };

        Ok(ComplianceReport {
            metadata,
            executive_summary: "12 security incidents recorded with average detection time of 2.5 hours.".to_string(),
            sections: vec![
                ReportSection {
                    title: "Incident Summary".to_string(),
                    summary: format!("{} total incidents, {} currently open", report_data.total_incidents, report_data.open_incidents),
                    data: serde_json::to_value(&report_data)?,
                    charts: vec![],
                },
            ],
            findings: vec![],
            recommendations: vec![],
        })
    }

    async fn generate_soc2_report(
        &self,
        metadata: ReportMetadata,
        _filter: ReportFilter,
    ) -> ReportResult<ComplianceReport> {
        let report_data = Soc2EvidenceReport {
            control_objectives: vec![],
            evidence_items: vec![],
            compliance_score: 95.5,
            gaps: vec![],
        };

        Ok(ComplianceReport {
            metadata,
            executive_summary: "SOC 2 compliance score is 95.5% with minimal control gaps identified.".to_string(),
            sections: vec![
                ReportSection {
                    title: "Control Objectives".to_string(),
                    summary: format!("Compliance score: {:.1}%", report_data.compliance_score),
                    data: serde_json::to_value(&report_data)?,
                    charts: vec![],
                },
            ],
            findings: vec![],
            recommendations: vec![],
        })
    }

    async fn generate_gdpr_report(
        &self,
        metadata: ReportMetadata,
        _filter: ReportFilter,
    ) -> ReportResult<ComplianceReport> {
        let report_data = GdprRequestReport {
            total_requests: 45,
            requests_by_type: HashMap::from([
                ("access".to_string(), 20),
                ("deletion".to_string(), 15),
                ("rectification".to_string(), 10),
            ]),
            requests: vec![],
            average_response_time: 18.5,
            overdue_requests: 2,
        };

        Ok(ComplianceReport {
            metadata,
            executive_summary: "45 GDPR requests processed with average response time of 18.5 hours.".to_string(),
            sections: vec![
                ReportSection {
                    title: "GDPR Requests".to_string(),
                    summary: format!("{} total requests, {} overdue", report_data.total_requests, report_data.overdue_requests),
                    data: serde_json::to_value(&report_data)?,
                    charts: vec![],
                },
            ],
            findings: vec![],
            recommendations: vec![],
        })
    }

    async fn generate_encryption_report(
        &self,
        metadata: ReportMetadata,
        _filter: ReportFilter,
    ) -> ReportResult<ComplianceReport> {
        let report_data = EncryptionStatusReport {
            total_data_stores: 25,
            encrypted_stores: 24,
            unencrypted_stores: 1,
            encryption_coverage: 96.0,
            key_rotations: vec![],
            non_compliant_stores: vec![],
        };

        Ok(ComplianceReport {
            metadata,
            executive_summary: "Encryption coverage is at 96% with 1 data store requiring attention.".to_string(),
            sections: vec![
                ReportSection {
                    title: "Encryption Status".to_string(),
                    summary: format!("{:.1}% of data stores encrypted", report_data.encryption_coverage),
                    data: serde_json::to_value(&report_data)?,
                    charts: vec![],
                },
            ],
            findings: vec![],
            recommendations: vec![],
        })
    }

    fn get_report_title(&self, report_type: ReportType) -> String {
        match report_type {
            ReportType::AuditLogSummary => "Audit Log Summary Report",
            ReportType::AccessControl => "Access Control Report",
            ReportType::RetentionCompliance => "Data Retention Compliance Report",
            ReportType::SecurityIncident => "Security Incident Report",
            ReportType::Soc2Evidence => "SOC 2 Evidence Collection Report",
            ReportType::GdprRequest => "GDPR Request Report",
            ReportType::EncryptionStatus => "Encryption Status Report",
            ReportType::PolicyCompliance => "Policy Compliance Report",
            ReportType::UserActivity => "User Activity Report",
            ReportType::DataBreaches => "Data Breach Report",
        }
        .to_string()
    }

    fn get_report_description(&self, report_type: ReportType) -> String {
        match report_type {
            ReportType::AuditLogSummary => "Summary of audit log events and activity",
            ReportType::AccessControl => "Analysis of access control policies and violations",
            ReportType::RetentionCompliance => "Data retention policy compliance status",
            ReportType::SecurityIncident => "Security incidents and response metrics",
            ReportType::Soc2Evidence => "SOC 2 control evidence and compliance status",
            ReportType::GdprRequest => "GDPR data subject request tracking",
            ReportType::EncryptionStatus => "Encryption coverage and key rotation status",
            ReportType::PolicyCompliance => "Overall policy compliance metrics",
            ReportType::UserActivity => "User activity and behavior analysis",
            ReportType::DataBreaches => "Data breach incidents and notifications",
        }
        .to_string()
    }

    /// Export report to specified format
    pub fn export_report(
        &self,
        report: &ComplianceReport,
        format: ReportFormat,
    ) -> ReportResult<Vec<u8>> {
        match format {
            ReportFormat::Json => self.export_json(report),
            ReportFormat::Csv => self.export_csv(report),
            ReportFormat::Html => self.export_html(report),
            ReportFormat::Pdf => Err(ReportError::InvalidFormat(
                "PDF export not yet implemented".to_string(),
            )),
        }
    }

    fn export_json(&self, report: &ComplianceReport) -> ReportResult<Vec<u8>> {
        let json = serde_json::to_vec_pretty(report)?;
        Ok(json)
    }

    fn export_csv(&self, report: &ComplianceReport) -> ReportResult<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write headers
        wtr.write_record([
            "Report ID",
            "Type",
            "Title",
            "Generated At",
            "Generated By",
            "Total Records",
        ])?;

        // Write data
        wtr.write_record(&[
            report.metadata.id.to_string(),
            format!("{:?}", report.metadata.report_type),
            report.metadata.title.clone(),
            report.metadata.generated_at.to_rfc3339(),
            report.metadata.generated_by.clone(),
            report.metadata.total_records.to_string(),
        ])?;

        wtr.flush()?;
        wtr.into_inner().map_err(|e| ReportError::ExportFailed(format!("Failed to finalize CSV writer: {}", e)))
    }

    fn export_html(&self, report: &ComplianceReport) -> ReportResult<Vec<u8>> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .metadata {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
        .section {{ margin: 20px 0; padding: 15px; border-left: 4px solid #007bff; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="metadata">
        <p><strong>Report ID:</strong> {}</p>
        <p><strong>Generated:</strong> {}</p>
        <p><strong>Generated By:</strong> {}</p>
        <p><strong>Period:</strong> {} to {}</p>
        <p><strong>Total Records:</strong> {}</p>
    </div>
    <h2>Executive Summary</h2>
    <p>{}</p>
    <h2>Sections</h2>
    {}
</body>
</html>"#,
            report.metadata.title,
            report.metadata.title,
            report.metadata.id,
            report.metadata.generated_at.to_rfc3339(),
            report.metadata.generated_by,
            report.metadata.period_start.to_rfc3339(),
            report.metadata.period_end.to_rfc3339(),
            report.metadata.total_records,
            report.executive_summary,
            report
                .sections
                .iter()
                .map(|s| format!(
                    r#"<div class="section"><h3>{}</h3><p>{}</p></div>"#,
                    s.title, s.summary
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(html.into_bytes())
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_audit_log_report() {
        let generator = ReportGenerator::new();
        let filter = ReportFilter::default();

        let report = generator
            .generate_report(
                ReportType::AuditLogSummary,
                filter,
                ReportFormat::Json,
                "test@example.com".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(report.metadata.report_type, ReportType::AuditLogSummary);
        assert!(!report.sections.is_empty());
    }

    #[tokio::test]
    async fn test_export_json() {
        let generator = ReportGenerator::new();
        let filter = ReportFilter::default();

        let report = generator
            .generate_report(
                ReportType::AccessControl,
                filter,
                ReportFormat::Json,
                "test@example.com".to_string(),
            )
            .await
            .unwrap();

        let json = generator
            .export_report(&report, ReportFormat::Json)
            .unwrap();

        assert!(!json.is_empty());
    }

    #[tokio::test]
    async fn test_export_html() {
        let generator = ReportGenerator::new();
        let filter = ReportFilter::default();

        let report = generator
            .generate_report(
                ReportType::SecurityIncident,
                filter,
                ReportFormat::Html,
                "test@example.com".to_string(),
            )
            .await
            .unwrap();

        let html = generator
            .export_report(&report, ReportFormat::Html)
            .unwrap();

        let html_string = String::from_utf8(html).unwrap();
        assert!(html_string.contains("<!DOCTYPE html>"));
        assert!(html_string.contains(&report.metadata.title));
    }
}
