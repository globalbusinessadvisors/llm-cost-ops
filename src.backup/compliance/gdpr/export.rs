// Data Export Service (GDPR Article 15 - Right to Access)

use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::compliance::error::{GdprError, GdprResult};
use super::types::{
    DataExportFormat, DataExportRequest, DataExportResponse, ExportMetadata,
    PersonalDataCategory,
};
use super::repository::GdprRepository;

/// Data exporter trait
#[async_trait]
pub trait DataExporter: Send + Sync {
    /// Export user data in requested format
    async fn export_data(&self, request: DataExportRequest) -> GdprResult<DataExportResponse>;
}

/// Default data exporter implementation
pub struct DefaultDataExporter<R: GdprRepository> {
    repository: Arc<R>,
}

impl<R: GdprRepository> DefaultDataExporter<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: GdprRepository> DataExporter for DefaultDataExporter<R> {
    async fn export_data(&self, request: DataExportRequest) -> GdprResult<DataExportResponse> {
        // Collect all requested data
        let mut all_data = json!({
            "user_id": request.user_id,
            "organization_id": request.organization_id,
            "export_date": Utc::now().to_rfc3339(),
            "data": {}
        });

        let mut total_records = 0;

        for category in &request.categories {
            match category {
                PersonalDataCategory::UsageRecords => {
                    let records = self
                        .repository
                        .get_usage_records_by_user(&request.user_id)
                        .await?;
                    total_records += records.len();
                    all_data["data"]["usage_records"] = json!(records);
                }
                PersonalDataCategory::CostRecords => {
                    let records = self
                        .repository
                        .get_cost_records_by_user(&request.user_id)
                        .await?;
                    total_records += records.len();
                    all_data["data"]["cost_records"] = json!(records);
                }
                PersonalDataCategory::ApiKeys => {
                    let records = self
                        .repository
                        .get_api_keys_by_user(&request.user_id)
                        .await?;
                    total_records += records.len();
                    all_data["data"]["api_keys"] = json!(records);
                }
                PersonalDataCategory::AuditLogs => {
                    let records = self
                        .repository
                        .get_audit_logs_by_user(&request.user_id)
                        .await?;
                    total_records += records.len();
                    all_data["data"]["audit_logs"] = json!(records);
                }
                PersonalDataCategory::ConsentRecords => {
                    let records = self
                        .repository
                        .get_consent_records_by_user(&request.user_id)
                        .await?;
                    total_records += records.len();
                    all_data["data"]["consent_records"] = json!(records);
                }
                PersonalDataCategory::All => {
                    // Export all categories
                    let usage = self
                        .repository
                        .get_usage_records_by_user(&request.user_id)
                        .await?;
                    let cost = self
                        .repository
                        .get_cost_records_by_user(&request.user_id)
                        .await?;
                    let api_keys = self
                        .repository
                        .get_api_keys_by_user(&request.user_id)
                        .await?;
                    let audit = self
                        .repository
                        .get_audit_logs_by_user(&request.user_id)
                        .await?;
                    let consent = self
                        .repository
                        .get_consent_records_by_user(&request.user_id)
                        .await?;

                    total_records += usage.len() + cost.len() + api_keys.len() + audit.len() + consent.len();

                    all_data["data"]["usage_records"] = json!(usage);
                    all_data["data"]["cost_records"] = json!(cost);
                    all_data["data"]["api_keys"] = json!(api_keys);
                    all_data["data"]["audit_logs"] = json!(audit);
                    all_data["data"]["consent_records"] = json!(consent);
                }
            }
        }

        // Convert to requested format
        let data = match request.format {
            DataExportFormat::Json => {
                serde_json::to_vec_pretty(&all_data)
                    .map_err(|e| GdprError::ExportFailed(e.to_string()))?
            }
            DataExportFormat::Csv => {
                self.to_csv(&all_data)
                    .map_err(|e| GdprError::ExportFailed(e.to_string()))?
            }
            DataExportFormat::Xml => {
                self.to_xml(&all_data)
                    .map_err(|e| GdprError::ExportFailed(e.to_string()))?
            }
        };

        // Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let checksum = format!("{:x}", hasher.finalize());

        let metadata = ExportMetadata {
            total_records,
            categories_included: request.categories.clone(),
            export_size_bytes: data.len(),
            checksum,
        };

        Ok(DataExportResponse {
            request_id: Uuid::new_v4(),
            user_id: request.user_id,
            organization_id: request.organization_id,
            format: request.format,
            data,
            metadata,
            completed_at: Utc::now(),
        })
    }
}

impl<R: GdprRepository> DefaultDataExporter<R> {
    fn to_csv(&self, data: &serde_json::Value) -> GdprResult<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write header
        wtr.write_record(["category", "record_id", "timestamp", "data"])?;

        // Write each category
        if let Some(obj) = data["data"].as_object() {
            for (category, records) in obj {
                if let Some(arr) = records.as_array() {
                    for record in arr {
                        let record_id = record["id"].as_str().unwrap_or("");
                        let timestamp = record
                            .get("timestamp")
                            .or_else(|| record.get("created_at"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let data_str = serde_json::to_string(record)?;

                        wtr.write_record([category, record_id, timestamp, &data_str])?;
                    }
                }
            }
        }

        wtr.into_inner().map_err(|e| GdprError::ExportFailed(format!("Failed to finalize CSV writer: {}", e)))
    }

    fn to_xml(&self, data: &serde_json::Value) -> GdprResult<Vec<u8>> {
        // Simple XML conversion
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<export>\n");

        if let Some(user_id) = data["user_id"].as_str() {
            xml.push_str(&format!("  <user_id>{}</user_id>\n", user_id));
        }
        if let Some(org_id) = data["organization_id"].as_str() {
            xml.push_str(&format!("  <organization_id>{}</organization_id>\n", org_id));
        }
        if let Some(date) = data["export_date"].as_str() {
            xml.push_str(&format!("  <export_date>{}</export_date>\n", date));
        }

        xml.push_str("  <data>\n");

        if let Some(obj) = data["data"].as_object() {
            for (category, records) in obj {
                xml.push_str(&format!("    <{}>\n", category));

                if let Some(arr) = records.as_array() {
                    for record in arr {
                        xml.push_str("      <record>\n");
                        if let Some(rec_obj) = record.as_object() {
                            for (key, value) in rec_obj {
                                let val_str = match value {
                                    serde_json::Value::String(s) => s.clone(),
                                    v => v.to_string(),
                                };
                                xml.push_str(&format!(
                                    "        <{}>{}<!/{}>\\n",
                                    key,
                                    escape_xml(&val_str),
                                    key
                                ));
                            }
                        }
                        xml.push_str("      </record>\n");
                    }
                }

                xml.push_str(&format!("    </{}>\n", category));
            }
        }

        xml.push_str("  </data>\n");
        xml.push_str("</export>\n");

        Ok(xml.into_bytes())
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Create a data exporter instance
pub fn create_data_exporter<R: GdprRepository + 'static>(repository: Arc<R>) -> Arc<dyn DataExporter> {
    Arc::new(DefaultDataExporter::new(repository))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::gdpr::repository::InMemoryGdprRepository;

    #[tokio::test]
    async fn test_data_export_json() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let exporter = DefaultDataExporter::new(repo);

        let request = DataExportRequest {
            user_id: "user-123".to_string(),
            organization_id: "org-123".to_string(),
            format: DataExportFormat::Json,
            categories: vec![PersonalDataCategory::All],
            requested_at: Utc::now(),
            requested_by: "admin".to_string(),
        };

        let result = exporter.export_data(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.user_id, "user-123");
        assert_eq!(response.format, DataExportFormat::Json);
    }

    #[tokio::test]
    async fn test_data_export_csv() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let exporter = DefaultDataExporter::new(repo);

        let request = DataExportRequest {
            user_id: "user-123".to_string(),
            organization_id: "org-123".to_string(),
            format: DataExportFormat::Csv,
            categories: vec![PersonalDataCategory::UsageRecords],
            requested_at: Utc::now(),
            requested_by: "admin".to_string(),
        };

        let result = exporter.export_data(request).await;
        assert!(result.is_ok());
    }
}
