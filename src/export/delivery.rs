// Report delivery system (email, storage, webhook)

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use lettre::message::{header, Attachment, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::config::{DeliveryTarget, EmailConfig, LocalStorageConfig, S3StorageConfig, StorageBackend, StorageConfig};
use super::formats::ExportFormat;
use super::reports::ReportResponse;
use super::{ExportError, ExportResult};

/// Delivery method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryMethod {
    Email,
    Storage,
    Webhook,
}

/// Delivery request
#[derive(Debug, Clone)]
pub struct DeliveryRequest {
    pub report: ReportResponse,
    pub format: ExportFormat,
    pub target: DeliveryTarget,
}

/// Delivery response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResponse {
    pub delivery_id: String,
    pub method: DeliveryMethod,
    pub status: DeliveryStatus,
    pub delivered_at: DateTime<Utc>,
    pub destination: String,
    pub size_bytes: usize,
}

/// Delivery status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Success,
    Failed,
    Partial,
}

/// Report delivery trait
#[async_trait]
pub trait ReportDelivery: Send + Sync {
    async fn deliver(&self, request: DeliveryRequest) -> ExportResult<DeliveryResponse>;
    fn supports(&self, target: &DeliveryTarget) -> bool;
}

/// Email delivery implementation
pub struct EmailDelivery {
    config: EmailConfig,
    templates: Handlebars<'static>,
}

impl EmailDelivery {
    pub fn new(config: EmailConfig) -> ExportResult<Self> {
        let mut templates = Handlebars::new();

        // Register default email template
        templates
            .register_template_string(
                "default",
                include_str!("templates/email_default.hbs"),
            )
            .map_err(|e| ExportError::TemplateError(format!("Failed to register template: {}", e)))?;

        Ok(Self { config, templates })
    }

    pub fn with_template(mut self, name: &str, template: &str) -> ExportResult<Self> {
        self.templates
            .register_template_string(name, template)
            .map_err(|e| ExportError::TemplateError(format!("Failed to register template: {}", e)))?;
        Ok(self)
    }

    async fn send_email(
        &self,
        recipients: &[String],
        subject: &str,
        body: &str,
        attachment_name: &str,
        attachment_data: Bytes,
        mime_type: &str,
    ) -> ExportResult<()> {
        // Build email message
        let mut message_builder = Message::builder()
            .from(
                format!("{} <{}>", self.config.from_name, self.config.from_email)
                    .parse()
                    .map_err(|e| ExportError::DeliveryError(format!("Invalid from address: {}", e)))?,
            )
            .subject(subject);

        // Add recipients
        for recipient in recipients {
            message_builder = message_builder.to(
                recipient
                    .parse()
                    .map_err(|e| ExportError::DeliveryError(format!("Invalid recipient: {}", e)))?,
            );
        }

        // Create multipart message with HTML body and attachment
        let attachment = Attachment::new(attachment_name.to_string())
            .body(attachment_data.to_vec(), mime_type.parse().unwrap());

        let multipart = MultiPart::mixed()
            .singlepart(
                SinglePart::builder()
                    .header(header::ContentType::TEXT_HTML)
                    .body(body.to_string()),
            )
            .singlepart(attachment);

        let email = message_builder
            .multipart(multipart)
            .map_err(|e| ExportError::DeliveryError(format!("Failed to build email: {}", e)))?;

        // Create SMTP transport
        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        let mut transport_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
            .map_err(|e| ExportError::DeliveryError(format!("Failed to create SMTP transport: {}", e)))?
            .credentials(creds)
            .timeout(Some(std::time::Duration::from_secs(self.config.timeout_secs)));

        if self.config.use_starttls {
            transport_builder = transport_builder.port(self.config.smtp_port);
        }

        let mailer = transport_builder.build();

        // Send email
        mailer
            .send(email)
            .await
            .map_err(|e| ExportError::DeliveryError(format!("Failed to send email: {}", e)))?;

        Ok(())
    }

    fn render_email_body(
        &self,
        template_name: &str,
        report: &ReportResponse,
    ) -> ExportResult<String> {
        let mut data = HashMap::new();
        data.insert("report_type", report.report_type.to_string());
        data.insert("report_id", report.id.clone());
        data.insert("generated_at", report.generated_at.to_rfc3339());
        data.insert("total_records", report.summary.total_records.to_string());
        data.insert(
            "date_range_start",
            report.summary.date_range.start.to_rfc3339(),
        );
        data.insert(
            "date_range_end",
            report.summary.date_range.end.to_rfc3339(),
        );

        self.templates
            .render(template_name, &data)
            .map_err(|e| ExportError::TemplateError(format!("Failed to render template: {}", e)))
    }
}

#[async_trait]
impl ReportDelivery for EmailDelivery {
    async fn deliver(&self, request: DeliveryRequest) -> ExportResult<DeliveryResponse> {
        let (recipients, subject, body_template) = match &request.target {
            DeliveryTarget::Email {
                recipients,
                subject,
                body_template,
            } => (recipients, subject, body_template),
            _ => {
                return Err(ExportError::DeliveryError(
                    "Invalid delivery target for email delivery".to_string(),
                ))
            }
        };

        // Export report data
        let exporter = super::formats::create_exporter(request.format);
        let exported_data = exporter.export(&request.report.data)?;
        let data_size = exported_data.len();

        // Render email body
        let template_name = body_template
            .as_deref()
            .unwrap_or(&self.config.template_name);
        let body = self.render_email_body(template_name, &request.report)?;

        // Generate attachment filename
        let attachment_name = format!(
            "report_{}_{}.{}",
            request.report.report_type,
            request.report.id,
            request.format.file_extension()
        );

        // Send email
        self.send_email(
            recipients,
            subject,
            &body,
            &attachment_name,
            exported_data,
            request.format.mime_type(),
        )
        .await?;

        Ok(DeliveryResponse {
            delivery_id: uuid::Uuid::new_v4().to_string(),
            method: DeliveryMethod::Email,
            status: DeliveryStatus::Success,
            delivered_at: Utc::now(),
            destination: recipients.join(", "),
            size_bytes: data_size,
        })
    }

    fn supports(&self, target: &DeliveryTarget) -> bool {
        matches!(target, DeliveryTarget::Email { .. })
    }
}

/// Storage delivery implementation
pub struct StorageDelivery {
    config: StorageConfig,
}

impl StorageDelivery {
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    async fn save_local(
        &self,
        local_config: &LocalStorageConfig,
        file_path: PathBuf,
        data: Bytes,
    ) -> ExportResult<PathBuf> {
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ExportError::DeliveryError(format!("Failed to create directory: {}", e)))?;
        }

        // Write file
        tokio::fs::write(&file_path, data)
            .await
            .map_err(|e| ExportError::DeliveryError(format!("Failed to write file: {}", e)))?;

        Ok(file_path)
    }

    async fn save_s3(
        &self,
        _s3_config: &S3StorageConfig,
        _key: String,
        _data: Bytes,
    ) -> ExportResult<String> {
        // S3 implementation would go here
        // For now, return a placeholder
        Err(ExportError::DeliveryError(
            "S3 delivery not yet implemented".to_string(),
        ))
    }

    fn generate_file_path(
        &self,
        local_config: &LocalStorageConfig,
        report: &ReportResponse,
        format: ExportFormat,
        custom_path: Option<&str>,
    ) -> PathBuf {
        if let Some(path) = custom_path {
            return PathBuf::from(path);
        }

        let mut base_path = local_config.base_dir.clone();

        // Add date subdirectories if enabled
        if local_config.use_date_subdirs {
            let date = report.generated_at.format("%Y/%m/%d").to_string();
            base_path = base_path.join(date);
        }

        // Generate filename from pattern
        let filename = local_config
            .file_pattern
            .replace("{report_type}", &report.report_type.to_string())
            .replace(
                "{date}",
                &report.generated_at.format("%Y%m%d_%H%M%S").to_string(),
            )
            .replace("{id}", &report.id)
            .replace("{ext}", format.file_extension());

        base_path.join(filename)
    }
}

#[async_trait]
impl ReportDelivery for StorageDelivery {
    async fn deliver(&self, request: DeliveryRequest) -> ExportResult<DeliveryResponse> {
        let custom_path = match &request.target {
            DeliveryTarget::Storage { path } => path.as_deref(),
            _ => {
                return Err(ExportError::DeliveryError(
                    "Invalid delivery target for storage delivery".to_string(),
                ))
            }
        };

        // Export report data
        let exporter = super::formats::create_exporter(request.format);
        let exported_data = exporter.export(&request.report.data)?;
        let data_size = exported_data.len();

        let destination = match self.config.backend {
            StorageBackend::Local => {
                let local_config = self.config.local.as_ref().ok_or_else(|| {
                    ExportError::DeliveryError("Local storage not configured".to_string())
                })?;

                let file_path = self.generate_file_path(
                    local_config,
                    &request.report,
                    request.format,
                    custom_path,
                );

                let saved_path = self
                    .save_local(local_config, file_path.clone(), exported_data)
                    .await?;

                saved_path.to_string_lossy().to_string()
            }
            StorageBackend::S3 => {
                let s3_config = self.config.s3.as_ref().ok_or_else(|| {
                    ExportError::DeliveryError("S3 storage not configured".to_string())
                })?;

                let key = custom_path.map(String::from).unwrap_or_else(|| {
                    format!(
                        "{}report_{}_{}_{}.{}",
                        s3_config.prefix,
                        request.report.report_type,
                        request.report.generated_at.format("%Y%m%d_%H%M%S"),
                        request.report.id,
                        request.format.file_extension()
                    )
                });

                self.save_s3(s3_config, key.clone(), exported_data)
                    .await?;

                format!("s3://{}/{}", s3_config.bucket, key)
            }
        };

        Ok(DeliveryResponse {
            delivery_id: uuid::Uuid::new_v4().to_string(),
            method: DeliveryMethod::Storage,
            status: DeliveryStatus::Success,
            delivered_at: Utc::now(),
            destination,
            size_bytes: data_size,
        })
    }

    fn supports(&self, target: &DeliveryTarget) -> bool {
        matches!(target, DeliveryTarget::Storage { .. })
    }
}

/// Webhook delivery implementation
pub struct WebhookDelivery {
    client: reqwest::Client,
}

impl WebhookDelivery {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for WebhookDelivery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReportDelivery for WebhookDelivery {
    async fn deliver(&self, request: DeliveryRequest) -> ExportResult<DeliveryResponse> {
        let (url, headers) = match &request.target {
            DeliveryTarget::Webhook { url, headers } => (url, headers),
            _ => {
                return Err(ExportError::DeliveryError(
                    "Invalid delivery target for webhook delivery".to_string(),
                ))
            }
        };

        // Export report data
        let exporter = super::formats::create_exporter(request.format);
        let exported_data = exporter.export(&request.report.data)?;
        let data_size = exported_data.len();

        // Build webhook request
        let mut req_builder = self
            .client
            .post(url)
            .header("Content-Type", request.format.mime_type())
            .header("X-Report-ID", &request.report.id)
            .header("X-Report-Type", request.report.report_type.to_string());

        // Add custom headers
        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }

        // Send request
        let response = req_builder
            .body(exported_data.to_vec())
            .send()
            .await
            .map_err(|e| ExportError::DeliveryError(format!("Webhook request failed: {}", e)))?;

        let status = if response.status().is_success() {
            DeliveryStatus::Success
        } else {
            DeliveryStatus::Failed
        };

        Ok(DeliveryResponse {
            delivery_id: uuid::Uuid::new_v4().to_string(),
            method: DeliveryMethod::Webhook,
            status,
            delivered_at: Utc::now(),
            destination: url.clone(),
            size_bytes: data_size,
        })
    }

    fn supports(&self, target: &DeliveryTarget) -> bool {
        matches!(target, DeliveryTarget::Webhook { .. })
    }
}

/// Multi-target delivery coordinator
pub struct DeliveryCoordinator {
    deliveries: Vec<Box<dyn ReportDelivery>>,
}

impl DeliveryCoordinator {
    pub fn new() -> Self {
        Self {
            deliveries: Vec::new(),
        }
    }

    pub fn add_delivery(&mut self, delivery: Box<dyn ReportDelivery>) {
        self.deliveries.push(delivery);
    }

    pub async fn deliver_to_targets(
        &self,
        report: ReportResponse,
        format: ExportFormat,
        targets: Vec<DeliveryTarget>,
    ) -> ExportResult<Vec<DeliveryResponse>> {
        let mut responses = Vec::new();

        for target in targets {
            // Find appropriate delivery method
            let delivery = self
                .deliveries
                .iter()
                .find(|d| d.supports(&target))
                .ok_or_else(|| {
                    ExportError::DeliveryError(format!("No delivery method found for target"))
                })?;

            let request = DeliveryRequest {
                report: report.clone(),
                format,
                target,
            };

            let response = delivery.deliver(request).await?;
            responses.push(response);
        }

        Ok(responses)
    }
}

impl Default for DeliveryCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::reports::{DateRange, ReportSummary};

    fn create_test_report() -> ReportResponse {
        let mut data = ExportData::new(vec!["Name".to_string(), "Value".to_string()]);
        data.add_row(vec![serde_json::json!("Test"), serde_json::json!(100)]);

        ReportResponse {
            id: "test-123".to_string(),
            report_type: ReportType::Cost,
            generated_at: Utc::now(),
            data,
            summary: ReportSummary {
                total_records: 1,
                date_range: DateRange {
                    start: Utc::now(),
                    end: Utc::now(),
                },
                aggregates: HashMap::new(),
            },
        }
    }

    #[tokio::test]
    async fn test_storage_delivery_local() {
        let config = StorageConfig {
            backend: StorageBackend::Local,
            local: Some(LocalStorageConfig {
                base_dir: PathBuf::from("/tmp/test-reports"),
                use_date_subdirs: false,
                file_pattern: "test_{id}.{ext}".to_string(),
            }),
            s3: None,
            retention_days: 30,
            auto_cleanup: false,
        };

        let delivery = StorageDelivery::new(config);
        let report = create_test_report();

        let request = DeliveryRequest {
            report,
            format: ExportFormat::Json,
            target: DeliveryTarget::Storage { path: None },
        };

        let result = delivery.deliver(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.method, DeliveryMethod::Storage);
        assert_eq!(response.status, DeliveryStatus::Success);
    }

    #[test]
    fn test_delivery_coordinator() {
        let mut coordinator = DeliveryCoordinator::new();

        let storage_config = StorageConfig::default();
        let storage_delivery = Box::new(StorageDelivery::new(storage_config));

        coordinator.add_delivery(storage_delivery);

        assert_eq!(coordinator.deliveries.len(), 1);
    }

    #[test]
    fn test_webhook_delivery_supports() {
        let delivery = WebhookDelivery::new();

        let webhook_target = DeliveryTarget::Webhook {
            url: "https://example.com/webhook".to_string(),
            headers: HashMap::new(),
        };

        assert!(delivery.supports(&webhook_target));

        let storage_target = DeliveryTarget::Storage { path: None };
        assert!(!delivery.supports(&storage_target));
    }
}
