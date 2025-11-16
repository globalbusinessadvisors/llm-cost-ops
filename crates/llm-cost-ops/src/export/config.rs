// Export and reporting configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::formats::ExportFormat;
use super::reports::ReportType;

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Default export format
    pub default_format: ExportFormat,

    /// Output directory for file exports
    pub output_dir: PathBuf,

    /// Maximum export size in bytes
    pub max_export_size: usize,

    /// Enable compression for exports
    pub enable_compression: bool,

    /// Email delivery configuration
    pub email: Option<EmailConfig>,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Scheduled reports configuration
    pub scheduled_reports: Vec<ScheduledReportConfig>,

    /// Report templates directory
    pub templates_dir: PathBuf,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            default_format: ExportFormat::Csv,
            output_dir: PathBuf::from("./exports"),
            max_export_size: 100 * 1024 * 1024, // 100MB
            enable_compression: true,
            email: None,
            storage: StorageConfig::default(),
            scheduled_reports: Vec::new(),
            templates_dir: PathBuf::from("./templates"),
        }
    }
}

/// Email delivery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server host
    pub smtp_host: String,

    /// SMTP server port
    pub smtp_port: u16,

    /// SMTP username
    pub smtp_username: String,

    /// SMTP password
    pub smtp_password: String,

    /// Use TLS
    pub use_tls: bool,

    /// Use STARTTLS
    pub use_starttls: bool,

    /// From email address
    pub from_email: String,

    /// From name
    pub from_name: String,

    /// Default recipients for reports
    pub default_recipients: Vec<String>,

    /// Email template for reports
    pub template_name: String,

    /// Connection timeout in seconds
    pub timeout_secs: u64,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            use_tls: false,
            use_starttls: true,
            from_email: "reports@llm-cost-ops.local".to_string(),
            from_name: "LLM Cost Ops".to_string(),
            default_recipients: Vec::new(),
            template_name: "default".to_string(),
            timeout_secs: 30,
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,

    /// Local filesystem configuration
    pub local: Option<LocalStorageConfig>,

    /// S3 configuration
    pub s3: Option<S3StorageConfig>,

    /// Retention policy in days
    pub retention_days: u32,

    /// Enable automatic cleanup
    pub auto_cleanup: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Local,
            local: Some(LocalStorageConfig::default()),
            s3: None,
            retention_days: 90,
            auto_cleanup: true,
        }
    }
}

/// Storage backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Local,
    S3,
}

/// Local storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    /// Base directory for storing reports
    pub base_dir: PathBuf,

    /// Use subdirectories by date
    pub use_date_subdirs: bool,

    /// File naming pattern
    pub file_pattern: String,
}

impl Default for LocalStorageConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("./reports"),
            use_date_subdirs: true,
            file_pattern: "{report_type}_{date}_{id}.{ext}".to_string(),
        }
    }
}

/// S3 storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3StorageConfig {
    /// S3 bucket name
    pub bucket: String,

    /// S3 region
    pub region: String,

    /// S3 access key
    pub access_key: String,

    /// S3 secret key
    pub secret_key: String,

    /// S3 endpoint (for S3-compatible services)
    pub endpoint: Option<String>,

    /// Prefix for report objects
    pub prefix: String,

    /// Storage class
    pub storage_class: String,
}

impl Default for S3StorageConfig {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            region: "us-east-1".to_string(),
            access_key: String::new(),
            secret_key: String::new(),
            endpoint: None,
            prefix: "reports/".to_string(),
            storage_class: "STANDARD".to_string(),
        }
    }
}

/// Scheduled report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledReportConfig {
    /// Unique identifier for the scheduled report
    pub id: String,

    /// Report type
    pub report_type: ReportType,

    /// Schedule in cron format
    pub schedule: String,

    /// Export format
    pub format: ExportFormat,

    /// Delivery methods
    pub delivery: Vec<DeliveryTarget>,

    /// Report filters
    pub filters: ReportFiltersConfig,

    /// Enabled flag
    pub enabled: bool,

    /// Timezone for scheduling
    pub timezone: String,
}

/// Delivery target
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DeliveryTarget {
    Email {
        recipients: Vec<String>,
        subject: String,
        body_template: Option<String>,
    },
    Storage {
        path: Option<String>,
    },
    Webhook {
        url: String,
        headers: std::collections::HashMap<String, String>,
    },
}

/// Report filters configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportFiltersConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub user_id: Option<String>,
    pub resource_type: Option<String>,
    pub organization_id: Option<String>,
    pub tags: std::collections::HashMap<String, String>,
}

/// Report template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    /// Template name
    pub name: String,

    /// Template file path
    pub file_path: PathBuf,

    /// Template format
    pub format: TemplateFormat,

    /// Default variables
    pub default_vars: std::collections::HashMap<String, String>,
}

/// Template format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateFormat {
    Html,
    Text,
    Markdown,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,

    /// Compression level (1-9)
    pub level: u32,

    /// Minimum size for compression (bytes)
    pub min_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Gzip,
            level: 6,
            min_size: 1024, // 1KB
        }
    }
}

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    Gzip,
    Brotli,
    Zstd,
}

impl ExportConfig {
    /// Load configuration from file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), Box<dyn std::error::Error>> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Get email configuration
    pub fn email_config(&self) -> Option<&EmailConfig> {
        self.email.as_ref()
    }

    /// Get storage configuration
    pub fn storage_config(&self) -> &StorageConfig {
        &self.storage
    }

    /// Get scheduled reports
    pub fn scheduled_reports(&self) -> &[ScheduledReportConfig] {
        &self.scheduled_reports
    }

    /// Add scheduled report
    pub fn add_scheduled_report(&mut self, config: ScheduledReportConfig) {
        self.scheduled_reports.push(config);
    }

    /// Remove scheduled report
    pub fn remove_scheduled_report(&mut self, id: &str) -> Option<ScheduledReportConfig> {
        if let Some(pos) = self.scheduled_reports.iter().position(|r| r.id == id) {
            Some(self.scheduled_reports.remove(pos))
        } else {
            None
        }
    }

    /// Get scheduled report by ID
    pub fn get_scheduled_report(&self, id: &str) -> Option<&ScheduledReportConfig> {
        self.scheduled_reports.iter().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ExportConfig::default();
        assert_eq!(config.default_format, ExportFormat::Csv);
        assert_eq!(config.max_export_size, 100 * 1024 * 1024);
        assert!(config.enable_compression);
    }

    #[test]
    fn test_email_config_default() {
        let config = EmailConfig::default();
        assert_eq!(config.smtp_port, 587);
        assert!(config.use_starttls);
        assert!(!config.use_tls);
    }

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.backend, StorageBackend::Local);
        assert_eq!(config.retention_days, 90);
        assert!(config.auto_cleanup);
    }

    #[test]
    fn test_add_remove_scheduled_report() {
        let mut config = ExportConfig::default();

        let scheduled = ScheduledReportConfig {
            id: "test-report".to_string(),
            report_type: ReportType::Cost,
            schedule: "0 0 * * *".to_string(),
            format: ExportFormat::Csv,
            delivery: vec![],
            filters: ReportFiltersConfig::default(),
            enabled: true,
            timezone: "UTC".to_string(),
        };

        config.add_scheduled_report(scheduled.clone());
        assert_eq!(config.scheduled_reports().len(), 1);

        let removed = config.remove_scheduled_report("test-report");
        assert!(removed.is_some());
        assert_eq!(config.scheduled_reports().len(), 0);
    }

    #[test]
    fn test_compression_config() {
        let config = CompressionConfig::default();
        assert_eq!(config.algorithm, CompressionAlgorithm::Gzip);
        assert_eq!(config.level, 6);
        assert_eq!(config.min_size, 1024);
    }
}
