//! Compliance module for LLM Cost Ops
//!
//! Provides GDPR compliance, audit logging, RBAC, and DLQ functionality.

pub mod compliance;
pub mod auth;
pub mod dlq;

// Re-export commonly used types
pub use auth::{
    AuthConfig, AuthContext, AuthMethod, ApiKey, ApiKeyHash,
    JwtClaims, JwtManager, TokenPair, ApiKeyStore, InMemoryApiKeyStore,
    auth_middleware, AuthState,
    // RBAC exports
    Action, Permission, Resource, Role, RoleType, RbacError, RbacManager, UserRole,
    // Audit exports
    AuditError, AuditEvent, AuditEventType, AuditLogger, AuditQuery,
    AuditSeverity, AuditStatus, AuditStore, InMemoryAuditStore,
    // RBAC middleware
    RbacState, require_permission, check_user_permission, check_user_scoped_permission,
};

pub use dlq::{
    DlqConfig, DlqItem, DlqItemStatus, FailureReason, DlqMetadata,
    DlqStore, InMemoryDlqStore,
    DlqProcessor, ProcessingResult, DlqItemHandler,
};

pub use compliance::{
    // Errors
    ComplianceError, ComplianceResult, GdprError, GdprResult,
    // Policies
    CompliancePolicy, PolicyType, PolicyConfig, PolicyRule, PolicyRuleType,
    PolicyVersion, PolicyStatus, RetentionPolicy, AccessPolicy, DataClassification,
    PolicyManager, PolicyError, PolicyResult, RetentionPeriod,
    // Reports
    ComplianceReport, ReportGenerator as ComplianceReportGenerator,
    ReportType as ComplianceReportType, ReportFormat as ComplianceReportFormat,
    ReportFilter as ComplianceReportFilter, AuditLogSummary, AccessControlReport,
    RetentionComplianceReport, SecurityIncidentReport, Soc2EvidenceReport,
    GdprRequestReport, EncryptionStatusReport, ReportMetadata as ComplianceReportMetadata,
    ReportSection, ReportError, ReportResult,
    // Dashboard
    ComplianceDashboard, DashboardMetrics, DashboardConfig as ComplianceDashboardConfig,
    PolicyMetric, AuditMetric, SecurityMetric, GdprMetric, RetentionMetric,
    AlertMetric, ComplianceStatus, ComplianceScore, TrendData, DashboardError,
    DashboardResult,
    // Checks
    ComplianceCheck, CheckResult as ComplianceCheckResult, CheckSeverity, CheckStatus,
    ViolationResult, ComplianceCheckEngine, CheckType, RetentionCheck, AccessCheck,
    EncryptionCheck, AuditLogCheck, GdprCheck, PolicyViolation, RemediationAction,
    CheckError, CheckResultData,
    // Scheduler
    ComplianceScheduler, ScheduledTask, TaskSchedule, TaskResult as ComplianceTaskResult,
    TaskStatus as ComplianceTaskStatus, SchedulerConfig as ComplianceSchedulerConfig,
    SchedulerError, SchedulerResult, TaskExecution, TaskHistory,
};
