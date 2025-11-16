// Authentication and authorization module

pub mod api_key;
pub mod jwt;
pub mod middleware;
pub mod storage;
pub mod config;
pub mod rbac;
pub mod audit;
pub mod rbac_middleware;

pub use api_key::{ApiKey, ApiKeyHash};
pub use jwt::{JwtClaims, JwtManager, TokenPair};
pub use middleware::{
    auth_middleware,
    AuthContext,
    AuthMethod,
    AuthState,
    require_auth,
};
pub use storage::{ApiKeyStore, InMemoryApiKeyStore};
pub use config::AuthConfig;
pub use rbac::{
    Action, Permission, Resource, Role, RoleType, RbacError, RbacManager,
    UserRole,
};
pub use audit::{
    AuditError, AuditEvent, AuditEventType, AuditLogger, AuditQuery,
    AuditSeverity, AuditStatus, AuditStore, InMemoryAuditStore,
};
pub use rbac_middleware::{
    RbacState, require_permission, check_user_permission, check_user_scoped_permission,
};

/// Authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Invalid JWT token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Missing authentication credentials")]
    MissingCredentials,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("API key revoked")]
    ApiKeyRevoked,

    #[error("Authentication configuration error: {0}")]
    ConfigError(String),

    #[error("Internal authentication error: {0}")]
    InternalError(String),
}

impl From<AuthError> for llm_cost_ops::CostOpsError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidApiKey
            | AuthError::InvalidToken(_)
            | AuthError::TokenExpired
            | AuthError::MissingCredentials
            | AuthError::InsufficientPermissions
            | AuthError::ApiKeyRevoked => {
                llm_cost_ops::CostOpsError::Authorization(err.to_string())
            }
            AuthError::ConfigError(msg) | AuthError::InternalError(msg) => {
                llm_cost_ops::CostOpsError::Integration(msg)
            }
        }
    }
}

pub type AuthResult<T> = Result<T, AuthError>;
