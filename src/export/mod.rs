// Export and reporting module

pub mod formats;
pub mod reports;
pub mod config;
pub mod delivery;
pub mod scheduler;

pub use formats::{ExportFormat, ExportData, Exporter, create_exporter};
pub use reports::{
    ReportType, ReportRequest, ReportResponse, ReportGenerator,
    CostReport, UsageReport, ForecastReport, AuditReport,
    ReportFilters, ReportSummary, DateRange,
};
pub use config::{
    ExportConfig, EmailConfig, StorageConfig, ScheduledReportConfig,
    DeliveryTarget, ReportFiltersConfig,
};
pub use delivery::{
    DeliveryMethod, DeliveryRequest, DeliveryResponse, DeliveryStatus,
    ReportDelivery, EmailDelivery, StorageDelivery, WebhookDelivery,
    DeliveryCoordinator,
};
pub use scheduler::{
    ReportScheduler, CronScheduler, ScheduledReportStatus, ScheduledExecutionResult,
};

/// Export error types
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Export format error: {0}")]
    FormatError(String),

    #[error("Report generation error: {0}")]
    GenerationError(String),

    #[error("Delivery error: {0}")]
    DeliveryError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

pub type ExportResult<T> = Result<T, ExportError>;
