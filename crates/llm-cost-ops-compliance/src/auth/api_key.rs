// API key generation, validation, and management

use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{AuthError, AuthResult};

/// API key representation (raw key, should be shown only once to user)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Unique identifier for this API key
    pub id: Uuid,

    /// Organization ID this key belongs to
    pub organization_id: String,

    /// The raw API key (only available at creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Human-readable name/description for this key
    pub name: String,

    /// Key prefix (e.g., "llmco-")
    pub prefix: String,

    /// When the key was created
    pub created_at: DateTime<Utc>,

    /// When the key expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,

    /// When the key was last used
    pub last_used_at: Option<DateTime<Utc>>,

    /// Whether the key is currently active
    pub is_active: bool,

    /// Permissions associated with this key
    pub permissions: Vec<String>,
}

/// Hashed API key (stored in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyHash {
    /// Unique identifier for this API key
    pub id: Uuid,

    /// Organization ID this key belongs to
    pub organization_id: String,

    /// SHA-256 hash of the API key
    pub key_hash: String,

    /// Key prefix (for identification)
    pub prefix: String,

    /// Human-readable name/description for this key
    pub name: String,

    /// When the key was created
    pub created_at: DateTime<Utc>,

    /// When the key expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,

    /// When the key was last used
    pub last_used_at: Option<DateTime<Utc>>,

    /// Whether the key is currently active
    pub is_active: bool,

    /// Whether the key is revoked
    pub is_revoked: bool,

    /// Permissions associated with this key
    pub permissions: Vec<String>,
}

impl ApiKey {
    /// Generate a new API key
    pub fn generate(
        organization_id: String,
        name: String,
        prefix: String,
        key_length: usize,
        permissions: Vec<String>,
    ) -> Self {
        let random_bytes: Vec<u8> = (0..key_length)
            .map(|_| rand::random::<u8>())
            .collect();

        let random_part = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(&random_bytes);

        let key = format!("{}{}", prefix, random_part);

        Self {
            id: Uuid::new_v4(),
            organization_id,
            key: Some(key),
            name,
            prefix,
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            is_active: true,
            permissions,
        }
    }

    /// Generate a new API key with expiration
    pub fn generate_with_expiration(
        organization_id: String,
        name: String,
        prefix: String,
        key_length: usize,
        permissions: Vec<String>,
        expires_in_days: u64,
    ) -> Self {
        let mut key = Self::generate(
            organization_id,
            name,
            prefix,
            key_length,
            permissions,
        );
        key.expires_at = Some(Utc::now() + chrono::Duration::days(expires_in_days as i64));
        key
    }

    /// Convert to hashed representation for storage
    pub fn to_hash(&self) -> AuthResult<ApiKeyHash> {
        let key = self
            .key
            .as_ref()
            .ok_or(AuthError::InternalError("API key not available".to_string()))?;

        let key_hash = hash_api_key(key);

        Ok(ApiKeyHash {
            id: self.id,
            organization_id: self.organization_id.clone(),
            key_hash,
            prefix: self.prefix.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            expires_at: self.expires_at,
            last_used_at: self.last_used_at,
            is_active: self.is_active,
            is_revoked: false,
            permissions: self.permissions.clone(),
        })
    }

    /// Check if the key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
}

impl ApiKeyHash {
    /// Verify a raw API key against this hash
    pub fn verify(&self, raw_key: &str) -> bool {
        if !self.is_active || self.is_revoked {
            return false;
        }

        if self.is_expired() {
            return false;
        }

        let candidate_hash = hash_api_key(raw_key);

        // Use constant-time comparison to prevent timing attacks
        constant_time_eq::constant_time_eq(
            self.key_hash.as_bytes(),
            candidate_hash.as_bytes(),
        )
    }

    /// Check if the key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Update last used timestamp
    pub fn update_last_used(&mut self) {
        self.last_used_at = Some(Utc::now());
    }

    /// Revoke the key
    pub fn revoke(&mut self) {
        self.is_revoked = true;
        self.is_active = false;
    }

    /// Check if key has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission || p == "*")
    }
}

/// Hash an API key using SHA-256
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Validate API key format
pub fn validate_api_key_format(key: &str, expected_prefix: &str) -> AuthResult<()> {
    if !key.starts_with(expected_prefix) {
        return Err(AuthError::InvalidApiKey);
    }

    // Check minimum length
    if key.len() < expected_prefix.len() + 16 {
        return Err(AuthError::InvalidApiKey);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec!["read".to_string(), "write".to_string()],
        );

        assert!(key.key.is_some());
        assert!(key.key.as_ref().unwrap().starts_with("llmco-"));
        assert_eq!(key.organization_id, "org-123");
        assert_eq!(key.name, "Test Key");
        assert!(key.is_active);
        assert!(!key.is_expired());
    }

    #[test]
    fn test_generate_api_key_with_expiration() {
        let key = ApiKey::generate_with_expiration(
            "org-123".to_string(),
            "Expiring Key".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
            30,
        );

        assert!(key.expires_at.is_some());
        assert!(!key.is_expired());
    }

    #[test]
    fn test_hash_api_key() {
        let hash1 = hash_api_key("test-key-12345");
        let hash2 = hash_api_key("test-key-12345");
        let hash3 = hash_api_key("different-key");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_api_key_to_hash() {
        let key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec!["read".to_string()],
        );

        let hash = key.to_hash().unwrap();
        assert_eq!(hash.organization_id, key.organization_id);
        assert_eq!(hash.name, key.name);
        assert!(!hash.is_revoked);
    }

    #[test]
    fn test_verify_api_key() {
        let key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        let raw_key = key.key.clone().unwrap();
        let hash = key.to_hash().unwrap();

        assert!(hash.verify(&raw_key));
        assert!(!hash.verify("wrong-key"));
    }

    #[test]
    fn test_revoked_key_fails_verification() {
        let key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-".to_string(),
            32,
            vec![],
        );

        let raw_key = key.key.clone().unwrap();
        let mut hash = key.to_hash().unwrap();

        assert!(hash.verify(&raw_key));

        hash.revoke();
        assert!(!hash.verify(&raw_key));
    }

    #[test]
    fn test_has_permission() {
        let mut hash = ApiKeyHash {
            id: Uuid::new_v4(),
            organization_id: "org-123".to_string(),
            key_hash: "hash".to_string(),
            prefix: "llmco-".to_string(),
            name: "Test".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            is_active: true,
            is_revoked: false,
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        assert!(hash.has_permission("read"));
        assert!(hash.has_permission("write"));
        assert!(!hash.has_permission("admin"));

        // Test wildcard permission
        hash.permissions = vec!["*".to_string()];
        assert!(hash.has_permission("anything"));
    }

    #[test]
    fn test_validate_api_key_format() {
        assert!(validate_api_key_format("llmco-abcdef12345678901234567890", "llmco-").is_ok());
        assert!(validate_api_key_format("wrong-prefix", "llmco-").is_err());
        assert!(validate_api_key_format("llmco-short", "llmco-").is_err());
    }
}
