//! LLM-CostOps - Cost operations platform for LLM deployments
//!
//! This library provides comprehensive cost tracking, forecasting, and optimization
//! for Large Language Model deployments across multiple providers.

pub mod domain;
pub mod engine;
pub mod storage;
pub mod config;
pub mod cli;
pub mod integrations;
pub mod observability;
pub mod ingestion;
pub mod metrics;
pub mod auth;
pub mod dlq;
pub mod compression;
pub mod api;
pub mod forecasting;
pub mod export;
pub mod agents;
pub mod governance;

// Re-export commonly used types
pub use domain::{
    CostOpsError, Result, Provider, UsageRecord, CostRecord,
    PricingTable, PricingStructure, Currency,
};

pub use engine::{CostCalculator, TokenNormalizer, CostAggregator};

pub use storage::{
    CostRepository, PricingRepository, UsageRepository,
};

pub use ingestion::{
    IngestionHandler, DefaultIngestionHandler, UsageWebhookPayload,
    IngestionResponse, IngestionStatus, start_webhook_server,
};

pub use metrics::{init_metrics, MetricsConfig};

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

pub use compression::{
    CompressionAlgorithm, CompressionLevel, CompressionConfig,
    Compressor, compress, decompress, CompressionLayer, compression_layer,
};

pub use api::{
    ApiServer, ApiServerConfig, create_api_router,
    ApiError as ApiErrorType, ApiResult, PaginatedResponse, PaginationParams,
};

pub use forecasting::{
    DataPoint, TimeSeriesData, ForecastConfig, ForecastHorizon, TrendDirection,
    SeasonalityPattern, ForecastModel, LinearTrendModel, MovingAverageModel,
    ExponentialSmoothingModel, ForecastEngine, ForecastRequest, ForecastMetrics,
    AnomalyDetector, AnomalyResult, AnomalyMethod, BudgetForecaster,
    BudgetForecast, BudgetAlert, AlertSeverity,
};

pub use observability::{
    // Configuration
    ObservabilityConfig,
    ObservabilityMetricsConfig,
    TracingConfig, TracingFormat,
    LoggingConfig, LoggingFormat, HealthConfig, OtlpConfig,
    // Metrics
    MetricsRegistry, MetricsError, Timer, start_timer,
    // Tracing
    CorrelationId, RequestId, TraceContext,
    init_tracing_with_config, create_span_with_context,
    info_span_with_context, debug_span_with_context, trace_span_with_context,
    warn_span_with_context, error_span_with_context,
    extract_trace_context_from_headers, inject_trace_context_into_headers,
    TraceContextLayer,
    // Logging
    LogLevel, LogEntry, StructuredLogger, PerformanceLogger,
    // Health
    HealthStatus, ComponentHealth, SystemHealth,
    HealthCheck, HealthChecker,
    DatabaseHealthCheck, CacheHealthCheck, ExternalServiceHealthCheck,
    FunctionHealthCheck,
    // Stack
    init_observability, ObservabilityStack,
};

pub use export::{
    // Formats
    ExportFormat, ExportData, Exporter, create_exporter,
    // Reports
    ReportType, ReportRequest, ReportResponse, ReportGenerator,
    CostReport, UsageReport, ForecastReport, AuditReport,
    ReportFilters, ReportSummary, DateRange,
    // Configuration
    ExportConfig, EmailConfig, StorageConfig, ScheduledReportConfig,
    DeliveryTarget, ReportFiltersConfig,
    // Delivery
    DeliveryMethod, DeliveryRequest, DeliveryResponse, DeliveryStatus,
    ReportDelivery, EmailDelivery, StorageDelivery, WebhookDelivery,
    DeliveryCoordinator,
    // Scheduler
    ReportScheduler, CronScheduler, ScheduledReportStatus, ScheduledExecutionResult,
    // Errors
    ExportError, ExportResult,
};

pub use agents::{
    // Agent framework
    Agent, AgentClassification, AgentError,
    // Contracts
    AgentId, AgentVersion, DecisionEvent, DecisionType,
    ConstraintApplied, ValidationError,
    // Cost Forecasting Agent
    CostForecastingAgent, CostForecastInput, CostForecastOutput,
    ForecastProjection, RiskIndicator, RiskLevel,
    // Infrastructure
    RuVectorClient, RuVectorConfig, RuVectorError,
    AgentTelemetry, TelemetryEvent, TelemetryEmitter,
    // Edge Function
    create_router as create_agent_router, create_app as create_agent_app,
    EdgeFunctionState, ForecastRequest as AgentForecastRequest,
    ForecastResponse as AgentForecastResponse,
    // Registry
    AgentRegistry, AgentRegistryEntry, global_registry,
};

pub use governance::{
    // Phase 4 Layer 1 - Governance & FinOps
    // Constants
    AGENT_PHASE, AGENT_LAYER, MAX_TOKENS, MAX_LATENCY_MS,
    // Configuration
    GovernanceConfig,
    // Signal types
    GovernanceDecisionEvent, GovernanceDecisionType,
    CostRiskSignal, BudgetThresholdSignal, PolicyViolationSignal, ApprovalRequiredSignal,
    GovernanceRiskLevel, ApprovalType, ViolationType,
    // Signal emitters
    CostSignalEmitter, PolicySignalEmitter, ApprovalSignalEmitter,
    emit_cost_risk_signal, emit_budget_threshold_signal,
    emit_policy_violation_signal, emit_approval_required_signal,
    // Policy evaluation (analysis only - NO ENFORCEMENT)
    PolicyEvaluator, PolicyRule, PolicyResult, PolicySeverity,
    // Performance budgets
    PerformanceBudget, PerformanceGuard, BudgetExceeded,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the library with default configuration
pub fn init() -> Result<()> {
    // Initialize observability
    observability::init_tracing();
    Ok(())
}
