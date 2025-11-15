// API key storage and management

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::api_key::{ApiKeyHash, validate_api_key_format};
use super::{AuthError, AuthResult};

/// Trait for API key storage backends
#[async_trait]
pub trait ApiKeyStore: Send + Sync {
    /// Store a new API key hash
    async fn store(&self, key_hash: ApiKeyHash) -> AuthResult<()>;

    /// Retrieve an API key hash by the raw key
    async fn get_by_key(&self, raw_key: &str) -> AuthResult<Option<ApiKeyHash>>;

    /// Retrieve an API key hash by ID
    async fn get_by_id(&self, id: Uuid) -> AuthResult<Option<ApiKeyHash>>;

    /// List all API keys for an organization
    async fn list_by_organization(&self, org_id: &str) -> AuthResult<Vec<ApiKeyHash>>;

    /// Update an existing API key hash
    async fn update(&self, key_hash: ApiKeyHash) -> AuthResult<()>;

    /// Delete an API key
    async fn delete(&self, id: Uuid) -> AuthResult<()>;

    /// Revoke an API key
    async fn revoke(&self, id: Uuid) -> AuthResult<()>;

    /// Check if a raw API key is valid
    async fn verify(&self, raw_key: &str, expected_prefix: &str) -> AuthResult<ApiKeyHash>;
}

/// In-memory API key store (for development/testing)
#[derive(Debug, Clone)]
pub struct InMemoryApiKeyStore {
    keys: Arc<RwLock<HashMap<Uuid, ApiKeyHash>>>,
}

impl InMemoryApiKeyStore {
    /// Create a new in-memory store
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a store with pre-populated keys
    pub fn with_keys(keys: Vec<ApiKeyHash>) -> Self {
        let store = Self::new();
        let mut map = HashMap::new();
        for key in keys {
            map.insert(key.id, key);
        }
        *store.keys.blocking_write() = map;
        store
    }
}

impl Default for InMemoryApiKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ApiKeyStore for InMemoryApiKeyStore {
    async fn store(&self, key_hash: ApiKeyHash) -> AuthResult<()> {
        let mut keys = self.keys.write().await;
        keys.insert(key_hash.id, key_hash);
        Ok(())
    }

    async fn get_by_key(&self, raw_key: &str) -> AuthResult<Option<ApiKeyHash>> {
        let keys = self.keys.read().await;

        for key_hash in keys.values() {
            if key_hash.verify(raw_key) {
                return Ok(Some(key_hash.clone()));
            }
        }

        Ok(None)
    }

    async fn get_by_id(&self, id: Uuid) -> AuthResult<Option<ApiKeyHash>> {
        let keys = self.keys.read().await;
        Ok(keys.get(&id).cloned())
    }

    async fn list_by_organization(&self, org_id: &str) -> AuthResult<Vec<ApiKeyHash>> {
        let keys = self.keys.read().await;
        Ok(keys
            .values()
            .filter(|k| k.organization_id == org_id)
            .cloned()
            .collect())
    }

    async fn update(&self, key_hash: ApiKeyHash) -> AuthResult<()> {
        let mut keys = self.keys.write().await;

        if !keys.contains_key(&key_hash.id) {
            return Err(AuthError::InvalidApiKey);
        }

        keys.insert(key_hash.id, key_hash);
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> AuthResult<()> {
        let mut keys = self.keys.write().await;
        keys.remove(&id);
        Ok(())
    }

    async fn revoke(&self, id: Uuid) -> AuthResult<()> {
        let mut keys = self.keys.write().await;

        let key = keys
            .get_mut(&id)
            .ok_or(AuthError::InvalidApiKey)?;

        key.revoke();
        Ok(())
    }

    async fn verify(&self, raw_key: &str, expected_prefix: &str) -> AuthResult<ApiKeyHash> {
        // Validate format first
        validate_api_key_format(raw_key, expected_prefix)?;

        // Check if key exists and is valid
        let key_hash = self
            .get_by_key(raw_key)
            .await?
            .ok_or(AuthError::InvalidApiKey)?;

        // Verify the key (checks expiration, revocation, etc.)
        if !key_hash.verify(raw_key) {
            if key_hash.is_revoked {
                return Err(AuthError::ApiKeyRevoked);
            } else if key_hash.is_expired() {
                return Err(AuthError::TokenExpired);
            } else {
                return Err(AuthError::InvalidApiKey);
            }
        }

        Ok(key_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::api_key::ApiKey;

    #[tokio::test]
    async fn test_in_memory_store_creation() {
        let store = InMemoryApiKeyStore::new();
        let keys = store.keys.read().await;
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let store = InMemoryApiKeyStore::new();

        let api_key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec!["read".to_string()],
        );

        let key_hash = api_key.to_hash().unwrap();
        let key_id = key_hash.id;

        store.store(key_hash).await.unwrap();

        let retrieved = store.get_by_id(key_id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, key_id);
        assert_eq!(retrieved.organization_id, "org-123");
    }

    #[tokio::test]
    async fn test_get_by_key() {
        let store = InMemoryApiKeyStore::new();

        let api_key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        let raw_key = api_key.key.clone().unwrap();
        let key_hash = api_key.to_hash().unwrap();

        store.store(key_hash).await.unwrap();

        let retrieved = store.get_by_key(&raw_key).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.organization_id, "org-123");
    }

    #[tokio::test]
    async fn test_list_by_organization() {
        let store = InMemoryApiKeyStore::new();

        let key1 = ApiKey::generate(
            "org-123".to_string(),
            "Key 1".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        let key2 = ApiKey::generate(
            "org-123".to_string(),
            "Key 2".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        let key3 = ApiKey::generate(
            "org-456".to_string(),
            "Key 3".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        store.store(key1.to_hash().unwrap()).await.unwrap();
        store.store(key2.to_hash().unwrap()).await.unwrap();
        store.store(key3.to_hash().unwrap()).await.unwrap();

        let org123_keys = store.list_by_organization("org-123").await.unwrap();
        assert_eq!(org123_keys.len(), 2);

        let org456_keys = store.list_by_organization("org-456").await.unwrap();
        assert_eq!(org456_keys.len(), 1);
    }

    #[tokio::test]
    async fn test_update_key() {
        let store = InMemoryApiKeyStore::new();

        let api_key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        let mut key_hash = api_key.to_hash().unwrap();
        let key_id = key_hash.id;

        store.store(key_hash.clone()).await.unwrap();

        // Update the key
        key_hash.name = "Updated Name".to_string();
        store.update(key_hash).await.unwrap();

        let retrieved = store.get_by_id(key_id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
    }

    #[tokio::test]
    async fn test_delete_key() {
        let store = InMemoryApiKeyStore::new();

        let api_key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        let key_hash = api_key.to_hash().unwrap();
        let key_id = key_hash.id;

        store.store(key_hash).await.unwrap();
        assert!(store.get_by_id(key_id).await.unwrap().is_some());

        store.delete(key_id).await.unwrap();
        assert!(store.get_by_id(key_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let store = InMemoryApiKeyStore::new();

        let api_key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        let raw_key = api_key.key.clone().unwrap();
        let key_hash = api_key.to_hash().unwrap();
        let key_id = key_hash.id;

        store.store(key_hash).await.unwrap();

        // Verify key works before revocation
        let verified = store.verify(&raw_key, "llmco-").await;
        assert!(verified.is_ok());

        // Revoke the key
        store.revoke(key_id).await.unwrap();

        // Verify key fails after revocation
        let verified = store.verify(&raw_key, "llmco-").await;
        assert!(verified.is_err());
        assert!(matches!(verified.unwrap_err(), AuthError::ApiKeyRevoked));
    }

    #[tokio::test]
    async fn test_verify_invalid_format() {
        let store = InMemoryApiKeyStore::new();

        let result = store.verify("invalid-key", "llmco-").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidApiKey));
    }

    #[tokio::test]
    async fn test_verify_nonexistent_key() {
        let store = InMemoryApiKeyStore::new();

        let result = store.verify("llmco-nonexistentkey123456789012345678", "llmco-").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidApiKey));
    }
}
