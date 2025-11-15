// Authentication configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,

    /// JWT configuration
    pub jwt: JwtConfig,

    /// API key configuration
    pub api_key: ApiKeyConfig,

    /// Allowed authentication methods
    pub allowed_methods: Vec<AuthMethodConfig>,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key for signing tokens
    pub secret: String,

    /// JWT issuer
    pub issuer: String,

    /// JWT audience
    pub audience: String,

    /// Access token expiration in seconds
    pub access_token_exp_secs: u64,

    /// Refresh token expiration in seconds
    pub refresh_token_exp_secs: u64,

    /// Algorithm to use for signing (HS256, HS384, HS512, RS256, etc.)
    pub algorithm: String,
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Prefix for API keys (e.g., "sk-" or "llm-")
    pub prefix: String,

    /// Length of the random portion of API keys
    pub key_length: usize,

    /// Enable API key rotation
    pub rotation_enabled: bool,

    /// API key rotation period in days
    pub rotation_period_days: u64,
}

/// Allowed authentication methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethodConfig {
    /// JWT bearer token
    Jwt,
    /// API key (header or query param)
    ApiKey,
    /// Both JWT and API key
    Both,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt: JwtConfig::default(),
            api_key: ApiKeyConfig::default(),
            allowed_methods: vec![AuthMethodConfig::Both],
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".to_string()),
            issuer: "llm-cost-ops".to_string(),
            audience: "llm-cost-ops-api".to_string(),
            access_token_exp_secs: 3600,      // 1 hour
            refresh_token_exp_secs: 604800,   // 7 days
            algorithm: "HS256".to_string(),
        }
    }
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            prefix: "llmco-".to_string(),
            key_length: 32,
            rotation_enabled: false,
            rotation_period_days: 90,
        }
    }
}

impl AuthConfig {
    /// Create a production configuration
    pub fn production(jwt_secret: String) -> Self {
        Self {
            enabled: true,
            jwt: JwtConfig {
                secret: jwt_secret,
                issuer: "llm-cost-ops".to_string(),
                audience: "llm-cost-ops-api".to_string(),
                access_token_exp_secs: 3600,
                refresh_token_exp_secs: 604800,
                algorithm: "HS256".to_string(),
            },
            api_key: ApiKeyConfig {
                prefix: "llmco-".to_string(),
                key_length: 32,
                rotation_enabled: true,
                rotation_period_days: 90,
            },
            allowed_methods: vec![AuthMethodConfig::Both],
        }
    }

    /// Create a development configuration (less secure, for testing)
    pub fn development() -> Self {
        Self {
            enabled: true,
            jwt: JwtConfig {
                secret: "dev-secret-key".to_string(),
                issuer: "llm-cost-ops-dev".to_string(),
                audience: "llm-cost-ops-api-dev".to_string(),
                access_token_exp_secs: 86400,    // 24 hours
                refresh_token_exp_secs: 2592000, // 30 days
                algorithm: "HS256".to_string(),
            },
            api_key: ApiKeyConfig {
                prefix: "llmco-dev-".to_string(),
                key_length: 24,
                rotation_enabled: false,
                rotation_period_days: 0,
            },
            allowed_methods: vec![AuthMethodConfig::Both],
        }
    }

    /// Create a disabled configuration (no authentication)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            jwt: JwtConfig::default(),
            api_key: ApiKeyConfig::default(),
            allowed_methods: vec![],
        }
    }

    /// Get access token expiration as Duration
    pub fn access_token_duration(&self) -> Duration {
        Duration::from_secs(self.jwt.access_token_exp_secs)
    }

    /// Get refresh token expiration as Duration
    pub fn refresh_token_duration(&self) -> Duration {
        Duration::from_secs(self.jwt.refresh_token_exp_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AuthConfig::default();
        assert!(config.enabled);
        assert_eq!(config.jwt.issuer, "llm-cost-ops");
        assert_eq!(config.api_key.prefix, "llmco-");
    }

    #[test]
    fn test_production_config() {
        let config = AuthConfig::production("secret123".to_string());
        assert!(config.enabled);
        assert_eq!(config.jwt.secret, "secret123");
        assert!(config.api_key.rotation_enabled);
    }

    #[test]
    fn test_development_config() {
        let config = AuthConfig::development();
        assert!(config.enabled);
        assert!(!config.api_key.rotation_enabled);
    }

    #[test]
    fn test_disabled_config() {
        let config = AuthConfig::disabled();
        assert!(!config.enabled);
    }
}
