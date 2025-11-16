// Core ingestion handler implementation

use async_trait::async_trait;
use chrono::Utc;
use tracing::{error, info, warn};
use uuid::Uuid;
use validator::Validate;

use crate::domain::{Result, UsageRecord};
use crate::storage::UsageRepository;

use super::models::{
    IngestionError, IngestionResponse, IngestionStatus, UsageWebhookPayload,
};
use super::traits::{IngestionHandler, IngestionStorage, PayloadValidator};

/// Default ingestion handler that validates and stores usage records
#[derive(Clone)]
pub struct DefaultIngestionHandler<S: UsageRepository + Clone> {
    storage: S,
}

impl<S: UsageRepository + Clone> DefaultIngestionHandler<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Process and validate a payload
    fn process_payload(&self, payload: &UsageWebhookPayload) -> Result<UsageRecord> {
        let validator = DefaultPayloadValidator;
        // Validate using validator
        validator.validate(payload)?;

        // Convert to domain record
        Ok(payload.to_usage_record())
    }
}

#[async_trait]
impl<S: UsageRepository + Clone> IngestionHandler for DefaultIngestionHandler<S> {
    async fn handle_single(&self, payload: UsageWebhookPayload) -> Result<IngestionResponse> {
        let request_id = payload.request_id;

        info!(
            request_id = %request_id,
            organization_id = %payload.organization_id,
            provider = %payload.provider,
            "Processing single usage record"
        );

        // Process and validate
        match self.process_payload(&payload) {
            Ok(record) => {
                // Store in database
                match self.storage.create(&record).await {
                    Ok(_) => {
                        info!(
                            request_id = %request_id,
                            "Successfully ingested usage record"
                        );

                        Ok(IngestionResponse {
                            request_id,
                            status: IngestionStatus::Success,
                            accepted: 1,
                            rejected: 0,
                            errors: vec![],
                            processed_at: Utc::now(),
                        })
                    }
                    Err(e) => {
                        error!(
                            request_id = %request_id,
                            error = %e,
                            "Failed to store usage record"
                        );

                        Ok(IngestionResponse {
                            request_id,
                            status: IngestionStatus::Failed,
                            accepted: 0,
                            rejected: 1,
                            errors: vec![IngestionError {
                                index: None,
                                code: "STORAGE_ERROR".to_string(),
                                message: e.to_string(),
                                field: None,
                            }],
                            processed_at: Utc::now(),
                        })
                    }
                }
            }
            Err(e) => {
                warn!(
                    request_id = %request_id,
                    error = %e,
                    "Validation failed for usage record"
                );

                Ok(IngestionResponse {
                    request_id,
                    status: IngestionStatus::Failed,
                    accepted: 0,
                    rejected: 1,
                    errors: vec![IngestionError {
                        index: None,
                        code: "VALIDATION_ERROR".to_string(),
                        message: e.to_string(),
                        field: None,
                    }],
                    processed_at: Utc::now(),
                })
            }
        }
    }

    async fn handle_batch(
        &self,
        payloads: Vec<UsageWebhookPayload>,
    ) -> Result<IngestionResponse> {
        let batch_id = Uuid::new_v4();
        let batch_size = payloads.len();

        info!(
            batch_id = %batch_id,
            batch_size = batch_size,
            "Processing batch ingestion request"
        );

        let mut accepted = 0;
        let mut rejected = 0;
        let mut errors = Vec::new();

        for (index, payload) in payloads.into_iter().enumerate() {
            match self.process_payload(&payload) {
                Ok(record) => match self.storage.create(&record).await {
                    Ok(_) => {
                        accepted += 1;
                    }
                    Err(e) => {
                        rejected += 1;
                        errors.push(IngestionError {
                            index: Some(index),
                            code: "STORAGE_ERROR".to_string(),
                            message: e.to_string(),
                            field: None,
                        });
                    }
                },
                Err(e) => {
                    rejected += 1;
                    errors.push(IngestionError {
                        index: Some(index),
                        code: "VALIDATION_ERROR".to_string(),
                        message: e.to_string(),
                        field: None,
                    });
                }
            }
        }

        let status = if accepted == batch_size {
            IngestionStatus::Success
        } else if accepted > 0 {
            IngestionStatus::Partial
        } else {
            IngestionStatus::Failed
        };

        info!(
            batch_id = %batch_id,
            status = ?status,
            accepted = accepted,
            rejected = rejected,
            "Batch ingestion completed"
        );

        Ok(IngestionResponse {
            request_id: batch_id,
            status,
            accepted,
            rejected,
            errors,
            processed_at: Utc::now(),
        })
    }

    fn name(&self) -> &str {
        "default_ingestion_handler"
    }

    async fn health_check(&self) -> Result<bool> {
        // Could add more sophisticated health checks here
        Ok(true)
    }
}

/// Default payload validator using the validator crate
struct DefaultPayloadValidator;

impl PayloadValidator for DefaultPayloadValidator {
    fn validate(&self, payload: &UsageWebhookPayload) -> Result<()> {
        use crate::domain::CostOpsError;

        payload.validate().map_err(|e| {
            CostOpsError::Validation(format!("Validation failed: {}", e))
        })?;

        // Additional custom validations
        if payload.usage.total_tokens
            != payload.usage.prompt_tokens + payload.usage.completion_tokens
        {
            return Err(CostOpsError::TokenCountMismatch {
                calculated: payload.usage.prompt_tokens + payload.usage.completion_tokens,
                reported: payload.usage.total_tokens,
            });
        }

        if let Some(cached) = payload.usage.cached_tokens {
            if cached > payload.usage.prompt_tokens {
                return Err(CostOpsError::Validation(format!(
                    "Cached tokens ({}) cannot exceed prompt tokens ({})",
                    cached,
                    payload.usage.prompt_tokens
                )));
            }
        }

        Ok(())
    }

    fn validate_batch(&self, payloads: &[UsageWebhookPayload]) -> Vec<Result<()>> {
        payloads.iter().map(|p| self.validate(p)).collect()
    }
}

/// Ingestion storage adapter for UsageRepository
pub struct StorageAdapter<R: UsageRepository> {
    repository: R,
}

impl<R: UsageRepository> StorageAdapter<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: UsageRepository> IngestionStorage for StorageAdapter<R> {
    async fn store_usage(&self, record: UsageRecord) -> Result<()> {
        self.repository.create(&record).await
    }

    async fn store_batch(&self, records: Vec<UsageRecord>) -> Result<Vec<Result<()>>> {
        let mut results = Vec::new();
        for record in records {
            results.push(self.repository.create(&record).await);
        }
        Ok(results)
    }

    async fn health(&self) -> Result<bool> {
        // Simple health check - could be enhanced
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingestion::models::TokenUsageWebhook;
    use chrono::Utc;

    fn create_test_payload() -> UsageWebhookPayload {
        UsageWebhookPayload {
            request_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            provider: "openai".to_string(),
            model: super::super::models::ModelWebhook {
                name: "gpt-4".to_string(),
                version: None,
                context_window: Some(8192),
            },
            organization_id: "org-test".to_string(),
            project_id: Some("proj-test".to_string()),
            user_id: None,
            usage: TokenUsageWebhook {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                cached_tokens: None,
                reasoning_tokens: None,
            },
            performance: None,
            tags: vec![],
            metadata: Default::default(),
        }
    }

    #[test]
    fn test_payload_validation_success() {
        let validator = DefaultPayloadValidator;
        let payload = create_test_payload();

        assert!(validator.validate(&payload).is_ok());
    }

    #[test]
    fn test_payload_validation_token_mismatch() {
        let validator = DefaultPayloadValidator;
        let mut payload = create_test_payload();
        payload.usage.total_tokens = 999; // Mismatch

        assert!(validator.validate(&payload).is_err());
    }

    #[test]
    fn test_payload_validation_cached_tokens_exceed_prompt() {
        let validator = DefaultPayloadValidator;
        let mut payload = create_test_payload();
        payload.usage.cached_tokens = Some(200); // Exceeds prompt tokens

        assert!(validator.validate(&payload).is_err());
    }
}
