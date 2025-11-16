// Data Deletion Service (GDPR Article 17 - Right to Erasure)

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use super::anonymization::DataAnonymizer;
use crate::compliance::error::GdprResult;
use super::repository::GdprRepository;
use super::types::{
    DeletedCounts, DeletionRequest, DeletionResponse, DeletionStatus, PersonalDataCategory,
    RetentionException,
};

/// Deletion result
#[derive(Debug, Clone)]
pub struct DeletionResult {
    pub deleted_counts: DeletedCounts,
    pub retention_exceptions: Vec<RetentionException>,
}

/// Data deleter trait
#[async_trait]
pub trait DataDeleter: Send + Sync {
    /// Delete user data with cascade
    async fn delete_user_data(&self, request: DeletionRequest) -> GdprResult<DeletionResponse>;

    /// Anonymize data instead of deleting (for legal retention)
    async fn anonymize_user_data(&self, user_id: &str) -> GdprResult<DeletionResult>;
}

/// Default data deleter implementation
pub struct DefaultDataDeleter<R: GdprRepository> {
    repository: Arc<R>,
    _anonymizer: Arc<DataAnonymizer>,
}

impl<R: GdprRepository> DefaultDataDeleter<R> {
    pub fn new(repository: Arc<R>, anonymizer: Arc<DataAnonymizer>) -> Self {
        Self {
            repository,
            _anonymizer: anonymizer,
        }
    }
}

#[async_trait]
impl<R: GdprRepository> DataDeleter for DefaultDataDeleter<R> {
    async fn delete_user_data(&self, request: DeletionRequest) -> GdprResult<DeletionResponse> {
        info!("Starting data deletion for user: {}", request.user_id);

        let mut deleted_counts = DeletedCounts::default();
        let mut retention_exceptions = Vec::new();

        // Check for legal holds or retention requirements
        let has_legal_hold = self
            .repository
            .check_legal_hold(&request.user_id)
            .await?;

        if has_legal_hold {
            warn!(
                "Legal hold detected for user {}, applying anonymization instead",
                request.user_id
            );

            // Anonymize instead of delete
            let result = self.anonymize_user_data(&request.user_id).await?;

            retention_exceptions.push(RetentionException {
                category: PersonalDataCategory::All,
                reason: "Legal hold active".to_string(),
                legal_basis: "Legal obligation under applicable law".to_string(),
                retention_until: Utc::now() + chrono::Duration::days(2555), // 7 years
            });

            return Ok(DeletionResponse {
                request_id: Uuid::new_v4(),
                user_id: request.user_id,
                organization_id: request.organization_id,
                status: DeletionStatus::Completed,
                deleted_counts: result.deleted_counts,
                retention_exceptions,
                completed_at: Some(Utc::now()),
            });
        }

        // Process each category
        for category in &request.categories {
            match category {
                PersonalDataCategory::UsageRecords => {
                    let count = self
                        .repository
                        .delete_usage_records(&request.user_id)
                        .await?;
                    deleted_counts.usage_records = count;
                    info!("Deleted {} usage records", count);
                }
                PersonalDataCategory::CostRecords => {
                    let count = self
                        .repository
                        .delete_cost_records(&request.user_id)
                        .await?;
                    deleted_counts.cost_records = count;
                    info!("Deleted {} cost records", count);
                }
                PersonalDataCategory::ApiKeys => {
                    let count = self
                        .repository
                        .delete_api_keys(&request.user_id)
                        .await?;
                    deleted_counts.api_keys = count;
                    info!("Deleted {} API keys", count);
                }
                PersonalDataCategory::AuditLogs => {
                    if request.retain_audit_log {
                        // Anonymize audit logs instead of deleting
                        let count = self
                            .repository
                            .anonymize_audit_logs(&request.user_id)
                            .await?;
                        deleted_counts.audit_logs = count;
                        info!("Anonymized {} audit logs", count);

                        retention_exceptions.push(RetentionException {
                            category: PersonalDataCategory::AuditLogs,
                            reason: "Security and compliance requirements".to_string(),
                            legal_basis: "Legitimate business interest".to_string(),
                            retention_until: Utc::now() + chrono::Duration::days(2555),
                        });
                    } else {
                        let count = self
                            .repository
                            .delete_audit_logs(&request.user_id)
                            .await?;
                        deleted_counts.audit_logs = count;
                        info!("Deleted {} audit logs", count);
                    }
                }
                PersonalDataCategory::ConsentRecords => {
                    let count = self
                        .repository
                        .delete_consent_records(&request.user_id)
                        .await?;
                    deleted_counts.consent_records = count;
                    info!("Deleted {} consent records", count);
                }
                PersonalDataCategory::All => {
                    // Delete all categories
                    deleted_counts.usage_records = self
                        .repository
                        .delete_usage_records(&request.user_id)
                        .await?;
                    deleted_counts.cost_records = self
                        .repository
                        .delete_cost_records(&request.user_id)
                        .await?;
                    deleted_counts.api_keys = self
                        .repository
                        .delete_api_keys(&request.user_id)
                        .await?;

                    if request.retain_audit_log {
                        deleted_counts.audit_logs = self
                            .repository
                            .anonymize_audit_logs(&request.user_id)
                            .await?;
                        retention_exceptions.push(RetentionException {
                            category: PersonalDataCategory::AuditLogs,
                            reason: "Security and compliance requirements".to_string(),
                            legal_basis: "Legitimate business interest".to_string(),
                            retention_until: Utc::now() + chrono::Duration::days(2555),
                        });
                    } else {
                        deleted_counts.audit_logs = self
                            .repository
                            .delete_audit_logs(&request.user_id)
                            .await?;
                    }

                    deleted_counts.consent_records = self
                        .repository
                        .delete_consent_records(&request.user_id)
                        .await?;

                    info!(
                        "Deleted all data: usage={}, cost={}, api_keys={}, audit={}, consent={}",
                        deleted_counts.usage_records,
                        deleted_counts.cost_records,
                        deleted_counts.api_keys,
                        deleted_counts.audit_logs,
                        deleted_counts.consent_records
                    );
                }
            }
        }

        Ok(DeletionResponse {
            request_id: Uuid::new_v4(),
            user_id: request.user_id,
            organization_id: request.organization_id,
            status: DeletionStatus::Completed,
            deleted_counts,
            retention_exceptions,
            completed_at: Some(Utc::now()),
        })
    }

    async fn anonymize_user_data(&self, user_id: &str) -> GdprResult<DeletionResult> {
        info!("Anonymizing data for user: {}", user_id);

        let mut deleted_counts = DeletedCounts::default();

        // Anonymize all personal data while retaining records
        deleted_counts.usage_records = self
            .repository
            .anonymize_usage_records(user_id)
            .await?;
        deleted_counts.cost_records = self
            .repository
            .anonymize_cost_records(user_id)
            .await?;
        deleted_counts.audit_logs = self
            .repository
            .anonymize_audit_logs(user_id)
            .await?;

        // API keys and consent records should be deleted
        deleted_counts.api_keys = self.repository.delete_api_keys(user_id).await?;
        deleted_counts.consent_records = self
            .repository
            .delete_consent_records(user_id)
            .await?;

        info!(
            "Anonymized data: usage={}, cost={}, audit={}; Deleted: api_keys={}, consent={}",
            deleted_counts.usage_records,
            deleted_counts.cost_records,
            deleted_counts.audit_logs,
            deleted_counts.api_keys,
            deleted_counts.consent_records
        );

        Ok(DeletionResult {
            deleted_counts,
            retention_exceptions: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::gdpr::repository::InMemoryGdprRepository;
    use chrono::Utc;

    #[tokio::test]
    async fn test_delete_user_data() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let anonymizer = Arc::new(DataAnonymizer::new());
        let deleter = DefaultDataDeleter::new(repo, anonymizer);

        let request = DeletionRequest {
            user_id: "user-123".to_string(),
            organization_id: "org-123".to_string(),
            categories: vec![PersonalDataCategory::All],
            reason: "User requested deletion".to_string(),
            requested_at: Utc::now(),
            requested_by: "user-123".to_string(),
            retain_audit_log: true,
        };

        let result = deleter.delete_user_data(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, DeletionStatus::Completed);
    }

    #[tokio::test]
    async fn test_anonymize_user_data() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let anonymizer = Arc::new(DataAnonymizer::new());
        let deleter = DefaultDataDeleter::new(repo, anonymizer);

        let result = deleter.anonymize_user_data("user-123").await;
        assert!(result.is_ok());
    }
}
