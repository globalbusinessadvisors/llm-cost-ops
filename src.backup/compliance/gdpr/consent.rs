// Consent Management Service

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

use crate::compliance::error::{GdprError, GdprResult};
use super::repository::GdprRepository;
use super::types::{ConsentPurpose, ConsentRecord, ConsentStatus};

/// Consent manager trait
#[async_trait]
pub trait ConsentManager: Send + Sync {
    /// Record user consent
    async fn record_consent(
        &self,
        user_id: String,
        organization_id: String,
        purpose: ConsentPurpose,
        consent_text: String,
        version: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> GdprResult<ConsentRecord>;

    /// Withdraw consent
    async fn withdraw_consent(&self, user_id: &str, purpose: ConsentPurpose) -> GdprResult<()>;

    /// Check if user has given consent for a purpose
    async fn has_consent(&self, user_id: &str, purpose: &ConsentPurpose) -> GdprResult<bool>;

    /// Get all consent records for a user
    async fn get_user_consents(&self, user_id: &str) -> GdprResult<Vec<ConsentRecord>>;
}

/// Consent validator
pub struct ConsentValidator;

impl ConsentValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate consent record
    pub fn validate(&self, consent: &ConsentRecord) -> GdprResult<()> {
        if consent.user_id.is_empty() {
            return Err(GdprError::validation("User ID cannot be empty"));
        }

        if consent.organization_id.is_empty() {
            return Err(GdprError::validation("Organization ID cannot be empty"));
        }

        if consent.consent_text.is_empty() {
            return Err(GdprError::validation("Consent text cannot be empty"));
        }

        if consent.version.is_empty() {
            return Err(GdprError::validation("Consent version cannot be empty"));
        }

        Ok(())
    }

    /// Check if consent is still valid
    pub fn is_valid(&self, consent: &ConsentRecord) -> bool {
        // Check status
        if consent.get_status() != ConsentStatus::Given {
            return false;
        }

        // Check expiration
        if let Some(expires_at) = &consent.expires_at {
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                if expiry.with_timezone(&Utc) < Utc::now() {
                    return false;
                }
            }
        }

        true
    }
}

impl Default for ConsentValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Default consent manager implementation
pub struct DefaultConsentManager<R: GdprRepository> {
    repository: Arc<R>,
    validator: ConsentValidator,
}

impl<R: GdprRepository> DefaultConsentManager<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            validator: ConsentValidator::new(),
        }
    }
}

#[async_trait]
impl<R: GdprRepository> ConsentManager for DefaultConsentManager<R> {
    async fn record_consent(
        &self,
        user_id: String,
        organization_id: String,
        purpose: ConsentPurpose,
        consent_text: String,
        version: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> GdprResult<ConsentRecord> {
        let consent = ConsentRecord::new(
            user_id,
            organization_id,
            purpose,
            consent_text,
            version,
            ip_address,
            user_agent,
        );

        self.validator.validate(&consent)?;
        self.repository.store_consent(consent.clone()).await?;

        Ok(consent)
    }

    async fn withdraw_consent(&self, user_id: &str, purpose: ConsentPurpose) -> GdprResult<()> {
        self.repository.withdraw_consent(user_id, purpose).await
    }

    async fn has_consent(&self, user_id: &str, purpose: &ConsentPurpose) -> GdprResult<bool> {
        let consent = self.repository.get_consent(user_id, purpose).await?;

        if let Some(c) = consent {
            Ok(self.validator.is_valid(&c))
        } else {
            Ok(false)
        }
    }

    async fn get_user_consents(&self, user_id: &str) -> GdprResult<Vec<ConsentRecord>> {
        self.repository.get_consent_records_by_user(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::gdpr::repository::InMemoryGdprRepository;

    #[test]
    fn test_consent_validator() {
        let validator = ConsentValidator::new();
        let consent = ConsentRecord::new(
            "user-123".to_string(),
            "org-123".to_string(),
            ConsentPurpose::DataProcessing,
            "I agree to data processing".to_string(),
            "1.0".to_string(),
            None,
            None,
        );

        assert!(validator.validate(&consent).is_ok());
        assert!(validator.is_valid(&consent));
    }

    #[tokio::test]
    async fn test_record_consent() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let manager = DefaultConsentManager::new(repo);

        let result = manager
            .record_consent(
                "user-123".to_string(),
                "org-123".to_string(),
                ConsentPurpose::DataProcessing,
                "I agree".to_string(),
                "1.0".to_string(),
                Some("192.168.1.1".to_string()),
                Some("Mozilla/5.0".to_string()),
            )
            .await;

        assert!(result.is_ok());
    }
}
