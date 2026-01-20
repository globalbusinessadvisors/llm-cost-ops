//! LLM-CostOps - Core library for cost operations
//!
//! This is the core library providing cost tracking, forecasting, and optimization
//! for Large Language Model deployments.

pub mod domain;
pub mod engine;
pub mod storage;
pub mod config;
pub mod observability;
pub mod metrics;
pub mod compression;
pub mod forecasting;
pub mod export;
pub mod benchmarks;
pub mod adapters;
pub mod integrations;
pub mod agents;

// Re-export commonly used types
pub use domain::{
    CostOpsError, Result, Provider, UsageRecord, CostRecord,
    PricingTable, PricingStructure, Currency,
};

pub use engine::{CostCalculator, TokenNormalizer, CostAggregator};

pub use storage::{
    CostRepository, PricingRepository, UsageRepository,
};

pub use metrics::{init_metrics, MetricsConfig};

pub use compression::{
    CompressionAlgorithm, CompressionLevel, CompressionConfig,
    Compressor, compress, decompress, CompressionLayer, compression_layer,
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

pub use forecasting::{
    DataPoint, TimeSeriesData, ForecastConfig, ForecastHorizon, TrendDirection,
    SeasonalityPattern, ForecastModel, LinearTrendModel, MovingAverageModel,
    ExponentialSmoothingModel, ForecastEngine, ForecastRequest, ForecastMetrics,
    AnomalyDetector, AnomalyResult, AnomalyMethod, BudgetForecaster,
    BudgetForecast, BudgetAlert, AlertSeverity,
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

pub use benchmarks::{
    // Core functionality
    run_all_benchmarks, run_category_benchmarks, run_quick_benchmark,
    available_categories, targets_in_category,
    // Types
    BenchmarkResult, BenchmarkSummary,
    BenchmarkIo, BenchmarkIoError,
    MarkdownGenerator,
};

pub use adapters::{
    BenchTarget, BenchmarkRegistry,
};

pub use agents::{
    // Contracts
    AgentId, AgentVersion, DecisionType, DecisionEvent,
    AgentClassification, ConstraintType, SignalType,
    // RuVector Client
    RuvectorClient, RuvectorConfig, RuvectorError,
    // Budget Enforcement Agent
    BudgetEnforcementAgent, BudgetEvaluationRequest, BudgetConstraintSignal,
    BudgetEnforcementConfig, BudgetSignalSeverity, BudgetViolationType,
    // Registry
    AgentRegistry, AgentMetadata,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the library with default configuration
pub fn init() -> Result<()> {
    // Initialize observability
    observability::init_tracing();
    Ok(())
}
