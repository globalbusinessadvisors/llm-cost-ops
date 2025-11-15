// JWT token generation and validation

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::{AuthConfig, AuthError, AuthResult};

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user/organization ID)
    pub sub: String,

    /// Organization ID
    pub org: String,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Issued at (Unix timestamp)
    pub iat: i64,

    /// Not before (Unix timestamp)
    pub nbf: i64,

    /// JWT ID (unique identifier for this token)
    pub jti: String,

    /// Token type ("access" or "refresh")
    pub token_type: TokenType,

    /// Permissions/scopes
    pub permissions: Vec<String>,
}

/// Token type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

/// Token pair (access + refresh)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token (short-lived)
    pub access_token: String,

    /// Refresh token (long-lived)
    pub refresh_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Expiration time in seconds
    pub expires_in: i64,
}

/// JWT manager for token operations
pub struct JwtManager {
    config: AuthConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
}

impl JwtManager {
    /// Create a new JWT manager with the given configuration
    pub fn new(config: AuthConfig) -> AuthResult<Self> {
        let algorithm = parse_algorithm(&config.jwt.algorithm)?;

        let encoding_key = EncodingKey::from_secret(config.jwt.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_bytes());

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            algorithm,
        })
    }

    /// Generate a token pair (access + refresh)
    pub fn generate_token_pair(
        &self,
        subject: String,
        organization_id: String,
        permissions: Vec<String>,
    ) -> AuthResult<TokenPair> {
        let access_token = self.generate_access_token(
            subject.clone(),
            organization_id.clone(),
            permissions.clone(),
        )?;

        let refresh_token = self.generate_refresh_token(
            subject,
            organization_id,
            permissions,
        )?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.jwt.access_token_exp_secs as i64,
        })
    }

    /// Generate an access token
    pub fn generate_access_token(
        &self,
        subject: String,
        organization_id: String,
        permissions: Vec<String>,
    ) -> AuthResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.jwt.access_token_exp_secs as i64);

        let claims = JwtClaims {
            sub: subject,
            org: organization_id,
            iss: self.config.jwt.issuer.clone(),
            aud: self.config.jwt.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
            permissions,
        };

        encode(&Header::new(self.algorithm), &claims, &self.encoding_key)
            .map_err(|e| AuthError::InternalError(format!("Failed to encode JWT: {}", e)))
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(
        &self,
        subject: String,
        organization_id: String,
        permissions: Vec<String>,
    ) -> AuthResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.jwt.refresh_token_exp_secs as i64);

        let claims = JwtClaims {
            sub: subject,
            org: organization_id,
            iss: self.config.jwt.issuer.clone(),
            aud: self.config.jwt.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
            permissions,
        };

        encode(&Header::new(self.algorithm), &claims, &self.encoding_key)
            .map_err(|e| AuthError::InternalError(format!("Failed to encode JWT: {}", e)))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> AuthResult<JwtClaims> {
        let mut validation = Validation::new(self.algorithm);
        validation.set_issuer(&[self.config.jwt.issuer.clone()]);
        validation.set_audience(&[self.config.jwt.audience.clone()]);

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(format!("Token validation failed: {}", e)),
            })?;

        Ok(token_data.claims)
    }

    /// Validate an access token specifically
    pub fn validate_access_token(&self, token: &str) -> AuthResult<JwtClaims> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Access {
            return Err(AuthError::InvalidToken(
                "Expected access token, got refresh token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Validate a refresh token specifically
    pub fn validate_refresh_token(&self, token: &str) -> AuthResult<JwtClaims> {
        let claims = self.validate_token(token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(AuthError::InvalidToken(
                "Expected refresh token, got access token".to_string(),
            ));
        }

        Ok(claims)
    }

    /// Refresh an access token using a refresh token
    pub fn refresh_access_token(&self, refresh_token: &str) -> AuthResult<TokenPair> {
        let claims = self.validate_refresh_token(refresh_token)?;

        self.generate_token_pair(claims.sub, claims.org, claims.permissions)
    }
}

impl JwtClaims {
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        self.exp < now
    }

    /// Check if the token is not yet valid
    pub fn is_not_yet_valid(&self) -> bool {
        let now = Utc::now().timestamp();
        self.nbf > now
    }

    /// Get expiration as DateTime
    pub fn expiration(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or_else(Utc::now)
    }

    /// Check if token has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission || p == "*")
    }
}

/// Parse algorithm string to Algorithm enum
fn parse_algorithm(alg: &str) -> AuthResult<Algorithm> {
    match alg.to_uppercase().as_str() {
        "HS256" => Ok(Algorithm::HS256),
        "HS384" => Ok(Algorithm::HS384),
        "HS512" => Ok(Algorithm::HS512),
        "RS256" => Ok(Algorithm::RS256),
        "RS384" => Ok(Algorithm::RS384),
        "RS512" => Ok(Algorithm::RS512),
        "ES256" => Ok(Algorithm::ES256),
        "ES384" => Ok(Algorithm::ES384),
        _ => Err(AuthError::ConfigError(format!(
            "Unsupported JWT algorithm: {}",
            alg
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AuthConfig {
        AuthConfig::development()
    }

    #[test]
    fn test_jwt_manager_creation() {
        let config = test_config();
        let manager = JwtManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_generate_access_token() {
        let config = test_config();
        let manager = JwtManager::new(config).unwrap();

        let token = manager.generate_access_token(
            "user-123".to_string(),
            "org-456".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );

        assert!(token.is_ok());
        let token_str = token.unwrap();
        assert!(!token_str.is_empty());
    }

    #[test]
    fn test_generate_token_pair() {
        let config = test_config();
        let manager = JwtManager::new(config).unwrap();

        let pair = manager.generate_token_pair(
            "user-123".to_string(),
            "org-456".to_string(),
            vec!["read".to_string()],
        );

        assert!(pair.is_ok());
        let pair = pair.unwrap();
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
    }

    #[test]
    fn test_validate_access_token() {
        let config = test_config();
        let manager = JwtManager::new(config).unwrap();

        let token = manager
            .generate_access_token(
                "user-123".to_string(),
                "org-456".to_string(),
                vec!["read".to_string()],
            )
            .unwrap();

        let claims = manager.validate_access_token(&token);
        assert!(claims.is_ok());

        let claims = claims.unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.org, "org-456");
        assert_eq!(claims.token_type, TokenType::Access);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_validate_refresh_token() {
        let config = test_config();
        let manager = JwtManager::new(config).unwrap();

        let token = manager
            .generate_refresh_token(
                "user-123".to_string(),
                "org-456".to_string(),
                vec![],
            )
            .unwrap();

        let claims = manager.validate_refresh_token(&token);
        assert!(claims.is_ok());

        let claims = claims.unwrap();
        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_refresh_access_token() {
        let config = test_config();
        let manager = JwtManager::new(config).unwrap();

        let initial_pair = manager
            .generate_token_pair(
                "user-123".to_string(),
                "org-456".to_string(),
                vec!["read".to_string()],
            )
            .unwrap();

        let new_pair = manager.refresh_access_token(&initial_pair.refresh_token);
        assert!(new_pair.is_ok());

        let new_pair = new_pair.unwrap();
        assert!(!new_pair.access_token.is_empty());
        assert_ne!(new_pair.access_token, initial_pair.access_token);
    }

    #[test]
    fn test_invalid_token() {
        let config = test_config();
        let manager = JwtManager::new(config).unwrap();

        let result = manager.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_token_type() {
        let config = test_config();
        let manager = JwtManager::new(config).unwrap();

        let access_token = manager
            .generate_access_token("user-123".to_string(), "org-456".to_string(), vec![])
            .unwrap();

        // Try to validate access token as refresh token
        let result = manager.validate_refresh_token(&access_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_has_permission() {
        let claims = JwtClaims {
            sub: "user-123".to_string(),
            org: "org-456".to_string(),
            iss: "test".to_string(),
            aud: "test".to_string(),
            exp: Utc::now().timestamp() + 3600,
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        assert!(claims.has_permission("read"));
        assert!(claims.has_permission("write"));
        assert!(!claims.has_permission("admin"));
    }

    #[test]
    fn test_parse_algorithm() {
        assert!(matches!(parse_algorithm("HS256"), Ok(Algorithm::HS256)));
        assert!(matches!(parse_algorithm("hs256"), Ok(Algorithm::HS256)));
        assert!(matches!(parse_algorithm("RS256"), Ok(Algorithm::RS256)));
        assert!(parse_algorithm("INVALID").is_err());
    }
}
