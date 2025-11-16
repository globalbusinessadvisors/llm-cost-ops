// GDPR Compliance Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Data export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataExportFormat {
    Json,
    Csv,
    Xml,
}

/// Personal data category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonalDataCategory {
    UsageRecords,
    CostRecords,
    ApiKeys,
    AuditLogs,
    ConsentRecords,
    All,
}

/// Data export request (Article 15)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportRequest {
    pub user_id: String,
    pub organization_id: String,
    pub format: DataExportFormat,
    pub categories: Vec<PersonalDataCategory>,
    pub requested_at: DateTime<Utc>,
    pub requested_by: String,
}

/// Data export response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportResponse {
    pub request_id: Uuid,
    pub user_id: String,
    pub organization_id: String,
    pub format: DataExportFormat,
    pub data: Vec<u8>,
    pub metadata: ExportMetadata,
    pub completed_at: DateTime<Utc>,
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub total_records: usize,
    pub categories_included: Vec<PersonalDataCategory>,
    pub export_size_bytes: usize,
    pub checksum: String,
}

/// Deletion status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeletionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    PartiallyCompleted,
}

/// Data deletion request (Article 17)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRequest {
    pub user_id: String,
    pub organization_id: String,
    pub categories: Vec<PersonalDataCategory>,
    pub reason: String,
    pub requested_at: DateTime<Utc>,
    pub requested_by: String,
    pub retain_audit_log: bool,
}

/// Data deletion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionResponse {
    pub request_id: Uuid,
    pub user_id: String,
    pub organization_id: String,
    pub status: DeletionStatus,
    pub deleted_counts: DeletedCounts,
    pub retention_exceptions: Vec<RetentionException>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Counts of deleted records
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeletedCounts {
    pub usage_records: usize,
    pub cost_records: usize,
    pub api_keys: usize,
    pub audit_logs: usize,
    pub consent_records: usize,
}

/// Retention exception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionException {
    pub category: PersonalDataCategory,
    pub reason: String,
    pub legal_basis: String,
    pub retention_until: DateTime<Utc>,
}

/// Consent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsentStatus {
    Given,
    Withdrawn,
    Expired,
}

/// Consent purpose
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentPurpose {
    DataProcessing,
    Marketing,
    Analytics,
    ThirdPartySharing,
    Custom(String),
}

/// Consent record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConsentRecord {
    pub id: String,
    pub user_id: String,
    pub organization_id: String,
    pub purpose: String, // JSON serialized ConsentPurpose
    pub status: String,  // JSON serialized ConsentStatus
    pub given_at: String,
    pub withdrawn_at: Option<String>,
    pub expires_at: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub consent_text: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Processing restriction (Article 18)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProcessingRestriction {
    pub id: String,
    pub user_id: String,
    pub organization_id: String,
    pub reason: String, // JSON serialized RestrictionReason
    pub restricted_at: String,
    pub lifted_at: Option<String>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Restriction reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionReason {
    DataAccuracyDispute,
    UnlawfulProcessing,
    LegalClaim,
    UserRequest,
}

/// Breach severity (Articles 33-34)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BreachSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Breach status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BreachStatus {
    Detected,
    Investigating,
    Contained,
    NotificationSent,
    Resolved,
}

/// Data breach notification
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BreachNotification {
    pub id: String,
    pub breach_type: String,
    pub severity: String, // JSON serialized BreachSeverity
    pub status: String,   // JSON serialized BreachStatus
    pub detected_at: String,
    pub contained_at: Option<String>,
    pub resolved_at: Option<String>,
    pub affected_users: i64,
    pub affected_records: i64,
    pub description: String,
    pub impact_assessment: String,
    pub mitigation_measures: String,
    pub notification_sent_at: Option<String>,
    pub authority_notified_at: Option<String>,
    pub users_notified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub category: PersonalDataCategory,
    pub retention_days: i64,
    pub legal_basis: String,
    pub auto_delete: bool,
}

/// Anonymization method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnonymizationMethod {
    Hashing,
    Masking,
    Generalization,
    Suppression,
}

/// Anonymized record marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedRecord {
    pub original_id: String,
    pub anonymized_at: DateTime<Utc>,
    pub method: AnonymizationMethod,
    pub reason: String,
}

impl ConsentRecord {
    pub fn new(
        user_id: String,
        organization_id: String,
        purpose: ConsentPurpose,
        consent_text: String,
        version: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            organization_id,
            purpose: serde_json::to_string(&purpose).unwrap_or_default(),
            status: serde_json::to_string(&ConsentStatus::Given).unwrap_or_default(),
            given_at: now.clone(),
            withdrawn_at: None,
            expires_at: None,
            ip_address,
            user_agent,
            consent_text,
            version,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn get_status(&self) -> ConsentStatus {
        serde_json::from_str(&self.status).unwrap_or(ConsentStatus::Withdrawn)
    }

    pub fn get_purpose(&self) -> Option<ConsentPurpose> {
        serde_json::from_str(&self.purpose).ok()
    }
}

impl ProcessingRestriction {
    pub fn new(
        user_id: String,
        organization_id: String,
        reason: RestrictionReason,
        notes: Option<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            organization_id,
            reason: serde_json::to_string(&reason).unwrap_or_default(),
            restricted_at: now.clone(),
            lifted_at: None,
            is_active: true,
            notes,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn get_reason(&self) -> Option<RestrictionReason> {
        serde_json::from_str(&self.reason).ok()
    }
}

impl BreachNotification {
    pub fn new(
        breach_type: String,
        severity: BreachSeverity,
        affected_users: i64,
        affected_records: i64,
        description: String,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            breach_type,
            severity: serde_json::to_string(&severity).unwrap_or_default(),
            status: serde_json::to_string(&BreachStatus::Detected).unwrap_or_default(),
            detected_at: now.clone(),
            contained_at: None,
            resolved_at: None,
            affected_users,
            affected_records,
            description,
            impact_assessment: String::new(),
            mitigation_measures: String::new(),
            notification_sent_at: None,
            authority_notified_at: None,
            users_notified_at: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn get_severity(&self) -> BreachSeverity {
        serde_json::from_str(&self.severity).unwrap_or(BreachSeverity::Low)
    }

    pub fn get_status(&self) -> BreachStatus {
        serde_json::from_str(&self.status).unwrap_or(BreachStatus::Detected)
    }
}
