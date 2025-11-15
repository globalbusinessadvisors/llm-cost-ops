// Authentication middleware for Axum

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{
    api_key::ApiKeyHash,
    jwt::{JwtClaims, JwtManager},
    storage::ApiKeyStore,
    AuthConfig, AuthError,
};

/// Authentication context extracted from the request
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Organization ID
    pub organization_id: String,

    /// Subject (user ID or service account ID)
    pub subject: String,

    /// Authentication method used
    pub auth_method: AuthMethod,

    /// Permissions/scopes
    pub permissions: Vec<String>,

    /// JWT claims (if authenticated via JWT)
    pub jwt_claims: Option<JwtClaims>,

    /// API key info (if authenticated via API key)
    pub api_key: Option<ApiKeyHash>,
}

/// Authentication method used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMethod {
    /// JWT bearer token
    Jwt,
    /// API key
    ApiKey,
}

/// Shared authentication state
#[derive(Clone)]
pub struct AuthState<S: ApiKeyStore> {
    pub config: AuthConfig,
    pub jwt_manager: Arc<JwtManager>,
    pub api_key_store: Arc<S>,
}

impl<S: ApiKeyStore> AuthState<S> {
    pub fn new(config: AuthConfig, api_key_store: S) -> Result<Self, AuthError> {
        let jwt_manager = Arc::new(JwtManager::new(config.clone())?);
        Ok(Self {
            config,
            jwt_manager,
            api_key_store: Arc::new(api_key_store),
        })
    }
}

/// Authentication middleware
pub async fn auth_middleware<S: ApiKeyStore + 'static>(
    State(auth_state): State<AuthState<S>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthResponse> {
    // If authentication is disabled, pass through
    if !auth_state.config.enabled {
        return Ok(next.run(request).await);
    }

    // Extract authentication context
    let auth_context = extract_auth_context(&headers, &auth_state).await?;

    // Insert auth context into request extensions
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

/// Extract authentication context from request headers
async fn extract_auth_context<S: ApiKeyStore>(
    headers: &HeaderMap,
    auth_state: &AuthState<S>,
) -> Result<AuthContext, AuthResponse> {
    // Try JWT authentication first
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                return authenticate_jwt(token, auth_state).await;
            }
        }
    }

    // Try API key authentication
    // Check X-API-Key header
    if let Some(api_key_header) = headers.get("x-api-key") {
        if let Ok(api_key) = api_key_header.to_str() {
            return authenticate_api_key(api_key, auth_state).await;
        }
    }

    // Check Authorization header with "ApiKey" scheme
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("ApiKey ") {
                let api_key = &auth_str[7..];
                return authenticate_api_key(api_key, auth_state).await;
            }
        }
    }

    Err(AuthResponse::MissingCredentials)
}

/// Authenticate using JWT
async fn authenticate_jwt<S: ApiKeyStore>(
    token: &str,
    auth_state: &AuthState<S>,
) -> Result<AuthContext, AuthResponse> {
    let claims = auth_state
        .jwt_manager
        .validate_access_token(token)
        .map_err(|e| AuthResponse::from_auth_error(e))?;

    Ok(AuthContext {
        organization_id: claims.org.clone(),
        subject: claims.sub.clone(),
        auth_method: AuthMethod::Jwt,
        permissions: claims.permissions.clone(),
        jwt_claims: Some(claims),
        api_key: None,
    })
}

/// Authenticate using API key
async fn authenticate_api_key<S: ApiKeyStore>(
    api_key: &str,
    auth_state: &AuthState<S>,
) -> Result<AuthContext, AuthResponse> {
    let key_hash = auth_state
        .api_key_store
        .verify(api_key, &auth_state.config.api_key.prefix)
        .await
        .map_err(|e| AuthResponse::from_auth_error(e))?;

    Ok(AuthContext {
        organization_id: key_hash.organization_id.clone(),
        subject: key_hash.id.to_string(),
        auth_method: AuthMethod::ApiKey,
        permissions: key_hash.permissions.clone(),
        jwt_claims: None,
        api_key: Some(key_hash),
    })
}

/// Helper function to require authentication (for use in extractors)
pub fn require_auth(request: &Request) -> Result<AuthContext, AuthResponse> {
    request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .ok_or(AuthResponse::MissingCredentials)
}

/// Authentication response for errors
#[derive(Debug)]
pub enum AuthResponse {
    MissingCredentials,
    InvalidToken,
    TokenExpired,
    InvalidApiKey,
    ApiKeyRevoked,
    InsufficientPermissions,
    InternalError(String),
}

impl AuthResponse {
    fn from_auth_error(err: AuthError) -> Self {
        match err {
            AuthError::InvalidApiKey => Self::InvalidApiKey,
            AuthError::InvalidToken(_) => Self::InvalidToken,
            AuthError::TokenExpired => Self::TokenExpired,
            AuthError::MissingCredentials => Self::MissingCredentials,
            AuthError::InsufficientPermissions => Self::InsufficientPermissions,
            AuthError::ApiKeyRevoked => Self::ApiKeyRevoked,
            AuthError::ConfigError(msg) | AuthError::InternalError(msg) => {
                Self::InternalError(msg)
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingCredentials => StatusCode::UNAUTHORIZED,
            Self::InvalidToken | Self::TokenExpired | Self::InvalidApiKey | Self::ApiKeyRevoked => {
                StatusCode::UNAUTHORIZED
            }
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_message(&self) -> &str {
        match self {
            Self::MissingCredentials => "Missing authentication credentials",
            Self::InvalidToken => "Invalid authentication token",
            Self::TokenExpired => "Authentication token expired",
            Self::InvalidApiKey => "Invalid API key",
            Self::ApiKeyRevoked => "API key has been revoked",
            Self::InsufficientPermissions => "Insufficient permissions",
            Self::InternalError(_) => "Internal authentication error",
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AuthResponse {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.error_message().to_string();

        let body = Json(ErrorResponse {
            error: format!("{:?}", self),
            message,
        });

        (status, body).into_response()
    }
}

impl AuthContext {
    /// Check if the authenticated entity has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission || p == "*")
    }

    /// Require a specific permission (returns error if not present)
    pub fn require_permission(&self, permission: &str) -> Result<(), AuthResponse> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(AuthResponse::InsufficientPermissions)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{api_key::ApiKey, storage::InMemoryApiKeyStore};

    fn test_auth_state() -> AuthState<InMemoryApiKeyStore> {
        let config = AuthConfig::development();
        let store = InMemoryApiKeyStore::new();
        AuthState::new(config, store).unwrap()
    }

    #[tokio::test]
    async fn test_auth_state_creation() {
        let state = test_auth_state();
        assert!(state.config.enabled);
    }

    #[tokio::test]
    async fn test_authenticate_jwt() {
        let state = test_auth_state();

        let token = state
            .jwt_manager
            .generate_access_token(
                "user-123".to_string(),
                "org-456".to_string(),
                vec!["read".to_string()],
            )
            .unwrap();

        let context = authenticate_jwt(&token, &state).await;
        assert!(context.is_ok());

        let context = context.unwrap();
        assert_eq!(context.organization_id, "org-456");
        assert_eq!(context.subject, "user-123");
        assert_eq!(context.auth_method, AuthMethod::Jwt);
        assert!(context.has_permission("read"));
    }

    #[tokio::test]
    async fn test_authenticate_invalid_jwt() {
        let state = test_auth_state();
        let result = authenticate_jwt("invalid.token.here", &state).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authenticate_api_key() {
        let state = test_auth_state();

        let api_key = ApiKey::generate(
            "org-123".to_string(),
            "Test Key".to_string(),
            "llmco-dev-".to_string(),
            24,
            vec!["write".to_string()],
        );

        let raw_key = api_key.key.clone().unwrap();
        let key_hash = api_key.to_hash().unwrap();

        state.api_key_store.store(key_hash).await.unwrap();

        let context = authenticate_api_key(&raw_key, &state).await;
        assert!(context.is_ok());

        let context = context.unwrap();
        assert_eq!(context.organization_id, "org-123");
        assert_eq!(context.auth_method, AuthMethod::ApiKey);
        assert!(context.has_permission("write"));
    }

    #[tokio::test]
    async fn test_authenticate_invalid_api_key() {
        let state = test_auth_state();
        let result = authenticate_api_key("llmco-dev-invalidkey123456789012", &state).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_has_permission() {
        let context = AuthContext {
            organization_id: "org-123".to_string(),
            subject: "user-456".to_string(),
            auth_method: AuthMethod::Jwt,
            permissions: vec!["read".to_string(), "write".to_string()],
            jwt_claims: None,
            api_key: None,
        };

        assert!(context.has_permission("read"));
        assert!(context.has_permission("write"));
        assert!(!context.has_permission("admin"));
    }

    #[test]
    fn test_wildcard_permission() {
        let context = AuthContext {
            organization_id: "org-123".to_string(),
            subject: "user-456".to_string(),
            auth_method: AuthMethod::Jwt,
            permissions: vec!["*".to_string()],
            jwt_claims: None,
            api_key: None,
        };

        assert!(context.has_permission("anything"));
        assert!(context.has_permission("read"));
        assert!(context.has_permission("admin"));
    }

    #[test]
    fn test_require_permission() {
        let context = AuthContext {
            organization_id: "org-123".to_string(),
            subject: "user-456".to_string(),
            auth_method: AuthMethod::Jwt,
            permissions: vec!["read".to_string()],
            jwt_claims: None,
            api_key: None,
        };

        assert!(context.require_permission("read").is_ok());
        assert!(context.require_permission("write").is_err());
    }
}
