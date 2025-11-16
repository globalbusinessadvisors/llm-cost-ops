// GDPR Repository - Database operations for GDPR compliance

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::compliance::error::GdprResult;
use super::types::{BreachNotification, ConsentPurpose, ConsentRecord};

/// GDPR repository trait
#[async_trait]
pub trait GdprRepository: Send + Sync {
    // Consent operations
    async fn store_consent(&self, consent: ConsentRecord) -> GdprResult<()>;
    async fn get_consent(&self, user_id: &str, purpose: &ConsentPurpose) -> GdprResult<Option<ConsentRecord>>;
    async fn withdraw_consent(&self, user_id: &str, purpose: ConsentPurpose) -> GdprResult<()>;
    async fn get_consent_records_by_user(&self, user_id: &str) -> GdprResult<Vec<ConsentRecord>>;

    // Data export operations
    async fn get_usage_records_by_user(&self, user_id: &str) -> GdprResult<Vec<JsonValue>>;
    async fn get_cost_records_by_user(&self, user_id: &str) -> GdprResult<Vec<JsonValue>>;
    async fn get_api_keys_by_user(&self, user_id: &str) -> GdprResult<Vec<JsonValue>>;
    async fn get_audit_logs_by_user(&self, user_id: &str) -> GdprResult<Vec<JsonValue>>;

    // Data deletion operations
    async fn delete_usage_records(&self, user_id: &str) -> GdprResult<usize>;
    async fn delete_cost_records(&self, user_id: &str) -> GdprResult<usize>;
    async fn delete_api_keys(&self, user_id: &str) -> GdprResult<usize>;
    async fn delete_audit_logs(&self, user_id: &str) -> GdprResult<usize>;
    async fn delete_consent_records(&self, user_id: &str) -> GdprResult<usize>;

    // Anonymization operations
    async fn anonymize_usage_records(&self, user_id: &str) -> GdprResult<usize>;
    async fn anonymize_cost_records(&self, user_id: &str) -> GdprResult<usize>;
    async fn anonymize_audit_logs(&self, user_id: &str) -> GdprResult<usize>;

    // Legal hold check
    async fn check_legal_hold(&self, user_id: &str) -> GdprResult<bool>;

    // Breach operations
    async fn store_breach(&self, breach: BreachNotification) -> GdprResult<()>;
    async fn get_breach(&self, breach_id: &str) -> GdprResult<Option<BreachNotification>>;
    async fn update_breach(&self, breach: BreachNotification) -> GdprResult<()>;
}

/// In-memory GDPR repository (for testing)
#[derive(Clone)]
pub struct InMemoryGdprRepository {
    consents: Arc<RwLock<HashMap<String, ConsentRecord>>>,
    breaches: Arc<RwLock<HashMap<String, BreachNotification>>>,
}

impl InMemoryGdprRepository {
    pub fn new() -> Self {
        Self {
            consents: Arc::new(RwLock::new(HashMap::new())),
            breaches: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryGdprRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GdprRepository for InMemoryGdprRepository {
    async fn store_consent(&self, consent: ConsentRecord) -> GdprResult<()> {
        let mut consents = self.consents.write().await;
        let key = format!("{}:{}", consent.user_id, consent.purpose);
        consents.insert(key, consent);
        Ok(())
    }

    async fn get_consent(&self, user_id: &str, purpose: &ConsentPurpose) -> GdprResult<Option<ConsentRecord>> {
        let consents = self.consents.read().await;
        let purpose_str = serde_json::to_string(purpose)?;
        let key = format!("{}:{}", user_id, purpose_str);
        Ok(consents.get(&key).cloned())
    }

    async fn withdraw_consent(&self, user_id: &str, purpose: ConsentPurpose) -> GdprResult<()> {
        let mut consents = self.consents.write().await;
        let purpose_str = serde_json::to_string(&purpose)?;
        let key = format!("{}:{}", user_id, purpose_str);

        if let Some(consent) = consents.get_mut(&key) {
            consent.withdrawn_at = Some(chrono::Utc::now().to_rfc3339());
            consent.status = serde_json::to_string(&super::types::ConsentStatus::Withdrawn)?;
            consent.updated_at = chrono::Utc::now().to_rfc3339();
        }

        Ok(())
    }

    async fn get_consent_records_by_user(&self, user_id: &str) -> GdprResult<Vec<ConsentRecord>> {
        let consents = self.consents.read().await;
        Ok(consents
            .values()
            .filter(|c| c.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn get_usage_records_by_user(&self, _user_id: &str) -> GdprResult<Vec<JsonValue>> {
        Ok(vec![])
    }

    async fn get_cost_records_by_user(&self, _user_id: &str) -> GdprResult<Vec<JsonValue>> {
        Ok(vec![])
    }

    async fn get_api_keys_by_user(&self, _user_id: &str) -> GdprResult<Vec<JsonValue>> {
        Ok(vec![])
    }

    async fn get_audit_logs_by_user(&self, _user_id: &str) -> GdprResult<Vec<JsonValue>> {
        Ok(vec![])
    }

    async fn delete_usage_records(&self, _user_id: &str) -> GdprResult<usize> {
        Ok(0)
    }

    async fn delete_cost_records(&self, _user_id: &str) -> GdprResult<usize> {
        Ok(0)
    }

    async fn delete_api_keys(&self, _user_id: &str) -> GdprResult<usize> {
        Ok(0)
    }

    async fn delete_audit_logs(&self, _user_id: &str) -> GdprResult<usize> {
        Ok(0)
    }

    async fn delete_consent_records(&self, user_id: &str) -> GdprResult<usize> {
        let mut consents = self.consents.write().await;
        let keys_to_remove: Vec<String> = consents
            .iter()
            .filter(|(_, v)| v.user_id == user_id)
            .map(|(k, _)| k.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            consents.remove(&key);
        }

        Ok(count)
    }

    async fn anonymize_usage_records(&self, _user_id: &str) -> GdprResult<usize> {
        Ok(0)
    }

    async fn anonymize_cost_records(&self, _user_id: &str) -> GdprResult<usize> {
        Ok(0)
    }

    async fn anonymize_audit_logs(&self, _user_id: &str) -> GdprResult<usize> {
        Ok(0)
    }

    async fn check_legal_hold(&self, _user_id: &str) -> GdprResult<bool> {
        Ok(false)
    }

    async fn store_breach(&self, breach: BreachNotification) -> GdprResult<()> {
        let mut breaches = self.breaches.write().await;
        breaches.insert(breach.id.clone(), breach);
        Ok(())
    }

    async fn get_breach(&self, breach_id: &str) -> GdprResult<Option<BreachNotification>> {
        let breaches = self.breaches.read().await;
        Ok(breaches.get(breach_id).cloned())
    }

    async fn update_breach(&self, breach: BreachNotification) -> GdprResult<()> {
        let mut breaches = self.breaches.write().await;
        breaches.insert(breach.id.clone(), breach);
        Ok(())
    }
}
