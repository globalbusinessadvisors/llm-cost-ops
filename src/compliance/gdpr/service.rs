// GDPR Service - Main service orchestrator

use std::sync::Arc;

use super::anonymization::DataAnonymizer;
use super::breach::{BreachNotifier, DefaultBreachNotifier};
use super::consent::{ConsentManager, DefaultConsentManager};
use super::deletion::{DataDeleter, DefaultDataDeleter};
use crate::compliance::error::GdprResult;
use super::export::{DataExporter, DefaultDataExporter};
use super::repository::GdprRepository;
use super::types::{
    BreachNotification, ConsentPurpose, ConsentRecord, DataExportRequest, DataExportResponse,
    DeletionRequest, DeletionResponse,
};

/// Main GDPR service
pub struct GdprService<R: GdprRepository> {
    _repository: Arc<R>,
    exporter: Arc<dyn DataExporter>,
    deleter: Arc<dyn DataDeleter>,
    consent_manager: Arc<dyn ConsentManager>,
    breach_notifier: Arc<dyn BreachNotifier>,
}

impl<R: GdprRepository + 'static> GdprService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        let anonymizer = Arc::new(DataAnonymizer::new());

        let exporter = Arc::new(DefaultDataExporter::new(repository.clone()));
        let deleter = Arc::new(DefaultDataDeleter::new(
            repository.clone(),
            anonymizer.clone(),
        ));
        let consent_manager = Arc::new(DefaultConsentManager::new(repository.clone()));
        let breach_notifier = Arc::new(DefaultBreachNotifier::new(repository.clone()));

        Self {
            _repository: repository,
            exporter,
            deleter,
            consent_manager,
            breach_notifier,
        }
    }

    // === Data Export (Article 15) ===

    /// Export user data
    pub async fn export_user_data(&self, request: DataExportRequest) -> GdprResult<DataExportResponse> {
        self.exporter.export_data(request).await
    }

    // === Data Deletion (Article 17) ===

    /// Delete user data
    pub async fn delete_user_data(&self, request: DeletionRequest) -> GdprResult<DeletionResponse> {
        self.deleter.delete_user_data(request).await
    }

    /// Anonymize user data
    pub async fn anonymize_user_data(&self, user_id: &str) -> GdprResult<()> {
        self.deleter.anonymize_user_data(user_id).await?;
        Ok(())
    }

    // === Consent Management ===

    /// Record user consent
    pub async fn record_consent(
        &self,
        user_id: String,
        organization_id: String,
        purpose: ConsentPurpose,
        consent_text: String,
        version: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> GdprResult<ConsentRecord> {
        self.consent_manager
            .record_consent(
                user_id,
                organization_id,
                purpose,
                consent_text,
                version,
                ip_address,
                user_agent,
            )
            .await
    }

    /// Withdraw consent
    pub async fn withdraw_consent(&self, user_id: &str, purpose: ConsentPurpose) -> GdprResult<()> {
        self.consent_manager.withdraw_consent(user_id, purpose).await
    }

    /// Check if user has given consent
    pub async fn has_consent(&self, user_id: &str, purpose: &ConsentPurpose) -> GdprResult<bool> {
        self.consent_manager.has_consent(user_id, purpose).await
    }

    /// Get all user consents
    pub async fn get_user_consents(&self, user_id: &str) -> GdprResult<Vec<ConsentRecord>> {
        self.consent_manager.get_user_consents(user_id).await
    }

    // === Breach Notification (Articles 33-34) ===

    /// Report a data breach
    pub async fn report_breach(&self, breach: BreachNotification) -> GdprResult<()> {
        self.breach_notifier.report_breach(breach).await
    }

    /// Notify supervisory authority
    pub async fn notify_authority(&self, breach_id: &str) -> GdprResult<()> {
        self.breach_notifier.notify_authority(breach_id).await
    }

    /// Notify affected users
    pub async fn notify_users(&self, breach_id: &str) -> GdprResult<()> {
        self.breach_notifier.notify_users(breach_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::gdpr::repository::InMemoryGdprRepository;
    use crate::compliance::gdpr::types::{DataExportFormat, PersonalDataCategory};
    use chrono::Utc;

    #[tokio::test]
    async fn test_gdpr_service_export() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let service = GdprService::new(repo);

        let request = DataExportRequest {
            user_id: "user-123".to_string(),
            organization_id: "org-123".to_string(),
            format: DataExportFormat::Json,
            categories: vec![PersonalDataCategory::All],
            requested_at: Utc::now(),
            requested_by: "admin".to_string(),
        };

        let result = service.export_user_data(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gdpr_service_consent() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let service = GdprService::new(repo);

        let result = service
            .record_consent(
                "user-123".to_string(),
                "org-123".to_string(),
                ConsentPurpose::DataProcessing,
                "I consent to data processing".to_string(),
                "1.0".to_string(),
                None,
                None,
            )
            .await;

        assert!(result.is_ok());

        let has_consent = service
            .has_consent("user-123", &ConsentPurpose::DataProcessing)
            .await;
        assert!(has_consent.is_ok());
        assert!(has_consent.unwrap());
    }
}
