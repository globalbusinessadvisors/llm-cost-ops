//! Compliance reporting and monitoring system
//!
//! This module provides comprehensive compliance reporting, policy management,
//! and automated compliance checks for regulatory requirements including:
//! - SOC 2
//! - GDPR
//! - HIPAA
//! - PCI DSS
//! - Custom compliance frameworks

pub mod error;
pub mod policies;
pub mod reports;
pub mod dashboard;
pub mod checks;
pub mod scheduler;

// Audit logging system
pub mod audit;
pub mod audit_repository;
pub mod audit_middleware;

// GDPR compliance system
pub mod gdpr;

pub use error::{GdprError, GdprResult};

pub use policies::{
    CompliancePolicy, PolicyType, PolicyConfig, PolicyRule, PolicyRuleType,
    PolicyVersion, PolicyStatus, RetentionPolicy, AccessPolicy, DataClassification,
    PolicyManager, PolicyError, PolicyResult, RetentionPeriod,
};

pub use reports::{
    ComplianceReport, ReportGenerator, ReportType, ReportFormat, ReportFilter,
    AuditLogSummary, AccessControlReport, RetentionComplianceReport,
    SecurityIncidentReport, Soc2EvidenceReport, GdprRequestReport,
    EncryptionStatusReport, ReportMetadata, ReportSection, ReportError,
    ReportResult,
};

pub use dashboard::{
    ComplianceDashboard, DashboardMetrics, DashboardConfig, PolicyMetric,
    AuditMetric, SecurityMetric, GdprMetric, RetentionMetric, AlertMetric,
    ComplianceStatus, ComplianceScore, TrendData, DashboardError, DashboardResult,
};

pub use checks::{
    ComplianceCheck, CheckResult, CheckSeverity, CheckStatus, ViolationResult,
    ComplianceCheckEngine, CheckType, RetentionCheck, AccessCheck, EncryptionCheck,
    AuditLogCheck, GdprCheck, PolicyViolation, RemediationAction, CheckError,
    CheckResultData,
};

pub use scheduler::{
    ComplianceScheduler, ScheduledTask, TaskSchedule, TaskResult, TaskStatus,
    SchedulerConfig, SchedulerError, SchedulerResult, TaskExecution, TaskHistory,
};

// Audit logging exports
pub use audit::{
    AuditLog, AuditEventType, AuditOutcome, Actor, ActorType,
    ResourceInfo, ActionType, AuditMetadata, AuditChanges,
    HttpRequestInfo, GeoLocation,
};

pub use audit_repository::{
    AuditRepository, PostgresAuditRepository, AuditFilter,
    AuditExportFormat, RetentionPolicy as AuditRetentionPolicy,
};

pub use audit_middleware::{
    AuditMiddleware, AuditState, create_audit_layer,
};

// GDPR exports
pub use gdpr::{
    // Types
    ConsentRecord, ConsentPurpose, ConsentStatus, DataExportFormat,
    DataExportRequest, DataExportResponse, DeletionRequest, DeletionResponse,
    DeletionStatus, BreachNotification, BreachSeverity, BreachStatus,
    ProcessingRestriction, RestrictionReason, PersonalDataCategory,
    GdprRetentionPolicy, AnonymizationMethod,
    // Services
    DataExporter, DefaultDataExporter, DataDeleter, DefaultDataDeleter,
    DeletionResult, ConsentManager, DefaultConsentManager, ConsentValidator,
    BreachNotifier, DefaultBreachNotifier, BreachDetector,
    DataAnonymizer, anonymize_record,
    // Repository
    GdprRepository, InMemoryGdprRepository,
    // Main Service
    GdprService,
    // Router
    create_gdpr_router,
};

/// Compliance error types
#[derive(Debug, thiserror::Error)]
pub enum ComplianceError {
    #[error("Policy error: {0}")]
    Policy(String),

    #[error("Report generation error: {0}")]
    Report(String),

    #[error("Dashboard error: {0}")]
    Dashboard(String),

    #[error("Compliance check error: {0}")]
    Check(String),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Audit error: {0}")]
    Audit(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("GDPR error: {0}")]
    Gdpr(#[from] GdprError),
}

impl From<serde_json::Error> for ComplianceError {
    fn from(err: serde_json::Error) -> Self {
        ComplianceError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for ComplianceError {
    fn from(err: std::io::Error) -> Self {
        ComplianceError::Storage(err.to_string())
    }
}

pub type ComplianceResult<T> = Result<T, ComplianceError>;
