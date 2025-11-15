// RBAC enforcement middleware

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use std::sync::Arc;

use super::{
    audit::AuditLogger,
    rbac::{Action, Permission, RbacManager, Resource},
    AuthContext,
};

/// RBAC state for dependency injection
#[derive(Clone)]
pub struct RbacState {
    pub rbac_manager: Arc<RbacManager>,
    pub audit_logger: Option<Arc<AuditLogger>>,
}

impl RbacState {
    /// Create a new RBAC state
    pub fn new(rbac_manager: Arc<RbacManager>) -> Self {
        Self {
            rbac_manager,
            audit_logger: None,
        }
    }

    /// Create with audit logger
    pub fn with_audit(rbac_manager: Arc<RbacManager>, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            rbac_manager,
            audit_logger: Some(audit_logger),
        }
    }
}

/// Extract user ID from auth context
fn get_user_id(auth_context: &AuthContext) -> Option<String> {
    Some(auth_context.subject.clone())
}

/// Require specific permission middleware
pub async fn require_permission(
    Extension(rbac_state): Extension<RbacState>,
    Extension(auth_context): Extension<AuthContext>,
    required_permission: Permission,
    request: Request,
    next: Next,
) -> Response {
    let user_id = match get_user_id(&auth_context) {
        Some(id) => id,
        None => {
            return (StatusCode::UNAUTHORIZED, "User not authenticated").into_response();
        }
    };

    // Check permission
    let has_permission = rbac_state
        .rbac_manager
        .check_permission(&user_id, &required_permission)
        .await;

    if !has_permission {
        // Log access denied
        if let Some(ref logger) = rbac_state.audit_logger {
            let _ = logger
                .log_access_denied(
                    user_id.clone(),
                    required_permission.resource,
                    required_permission.scope.clone().unwrap_or_default(),
                    required_permission.action,
                )
                .await;
        }

        return (StatusCode::FORBIDDEN, "Insufficient permissions").into_response();
    }

    // Permission granted, continue
    next.run(request).await
}

/// Macro to create permission-checking middleware
#[macro_export]
macro_rules! require_perm {
    ($resource:expr, $action:expr) => {
        |Extension(rbac_state): Extension<$crate::auth::rbac_middleware::RbacState>,
         Extension(auth_context): Extension<$crate::auth::AuthContext>,
         request: axum::extract::Request,
         next: axum::middleware::Next| async move {
            $crate::auth::rbac_middleware::require_permission(
                Extension(rbac_state),
                Extension(auth_context),
                $crate::auth::rbac::Permission::new($resource, $action),
                request,
                next,
            )
            .await
        }
    };
}

/// Helper function to check if user has permission
pub async fn check_user_permission(
    rbac_manager: &RbacManager,
    user_id: &str,
    resource: Resource,
    action: Action,
) -> bool {
    let permission = Permission::new(resource, action);
    rbac_manager.check_permission(user_id, &permission).await
}

/// Helper function to check if user has scoped permission
pub async fn check_user_scoped_permission(
    rbac_manager: &RbacManager,
    user_id: &str,
    resource: Resource,
    action: Action,
    scope: String,
) -> bool {
    let permission = Permission::scoped(resource, action, scope);
    rbac_manager.check_permission(user_id, &permission).await
}

/// Create permission checking function for routes
pub fn permission_checker(
    resource: Resource,
    action: Action,
) -> impl Fn(Extension<RbacState>, Extension<AuthContext>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
    move |rbac_state: Extension<RbacState>,
          auth_context: Extension<AuthContext>,
          request: Request,
          next: Next| {
        let required_permission = Permission::new(resource, action);
        Box::pin(async move {
            require_permission(rbac_state, auth_context, required_permission, request, next).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{
        api_key::ApiKey,
        jwt::JwtClaims,
        rbac::{Role, RoleType},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "success"
    }

    #[tokio::test]
    async fn test_permission_middleware_granted() {
        let rbac_manager = Arc::new(RbacManager::new());

        // Assign super_admin role
        rbac_manager
            .assign_user_role("test_user", "super_admin")
            .await
            .unwrap();

        let rbac_state = RbacState::new(rbac_manager.clone());

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(Extension(rbac_state))
            .layer(middleware::from_fn(|req: Request, next: Next| async move {
                // Inject auth context
                let mut req = req;
                let claims = JwtClaims {
                    sub: "test_user".to_string(),
                    exp: 0,
                    iat: 0,
                    organization_id: Some("test_org".to_string()),
                };
                let auth_context = AuthContext {
                    organization_id: "test_org".to_string(),
                    subject: "test_user".to_string(),
                    auth_method: crate::auth::AuthMethod::Jwt,
                    permissions: vec![],
                    jwt_claims: Some(claims),
                    api_key: None,
                };
                req.extensions_mut().insert(auth_context);
                next.run(req).await
            }))
            .layer(middleware::from_fn(move |req: Request, next: Next| {
                let permission = Permission::new(Resource::Usage, Action::Read);
                async move {
                    let rbac_state = req.extensions().get::<RbacState>().unwrap().clone();
                    let auth_context = req.extensions().get::<AuthContext>().unwrap().clone();
                    require_permission(
                        Extension(rbac_state),
                        Extension(auth_context),
                        permission,
                        req,
                        next,
                    )
                    .await
                }
            }));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_permission_middleware_denied() {
        let rbac_manager = Arc::new(RbacManager::new());

        // User without any roles
        let rbac_state = RbacState::new(rbac_manager.clone());

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(Extension(rbac_state))
            .layer(middleware::from_fn(|req: Request, next: Next| async move {
                let mut req = req;
                let claims = JwtClaims {
                    sub: "test_user".to_string(),
                    exp: 0,
                    iat: 0,
                    organization_id: Some("test_org".to_string()),
                };
                let auth_context = AuthContext {
                    organization_id: "test_org".to_string(),
                    subject: "test_user".to_string(),
                    auth_method: crate::auth::AuthMethod::Jwt,
                    permissions: vec![],
                    jwt_claims: Some(claims),
                    api_key: None,
                };
                req.extensions_mut().insert(auth_context);
                next.run(req).await
            }))
            .layer(middleware::from_fn(move |req: Request, next: Next| {
                let permission = Permission::new(Resource::Usage, Action::Delete);
                async move {
                    let rbac_state = req.extensions().get::<RbacState>().unwrap().clone();
                    let auth_context = req.extensions().get::<AuthContext>().unwrap().clone();
                    require_permission(
                        Extension(rbac_state),
                        Extension(auth_context),
                        permission,
                        req,
                        next,
                    )
                    .await
                }
            }));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_check_user_permission() {
        let rbac_manager = RbacManager::new();

        rbac_manager
            .assign_user_role("user1", "super_admin")
            .await
            .unwrap();

        let has_perm = check_user_permission(
            &rbac_manager,
            "user1",
            Resource::Usage,
            Action::Read,
        )
        .await;

        assert!(has_perm);

        let no_perm = check_user_permission(
            &rbac_manager,
            "user2",
            Resource::Usage,
            Action::Read,
        )
        .await;

        assert!(!no_perm);
    }

    #[tokio::test]
    async fn test_check_user_scoped_permission() {
        let rbac_manager = RbacManager::new();

        // Create custom org-scoped role
        let mut role = Role::new(
            "custom_role".to_string(),
            "Custom Role".to_string(),
            "Test role".to_string(),
        );
        role.add_permission(Permission::scoped(
            Resource::Usage,
            Action::Read,
            "org1".to_string(),
        ));

        rbac_manager.create_role(role).await.unwrap();
        rbac_manager
            .assign_user_role("user1", "custom_role")
            .await
            .unwrap();

        // Should have scoped permission
        let has_perm = check_user_scoped_permission(
            &rbac_manager,
            "user1",
            Resource::Usage,
            Action::Read,
            "org1".to_string(),
        )
        .await;

        assert!(has_perm);

        // Should not have permission for different scope
        let no_perm = check_user_scoped_permission(
            &rbac_manager,
            "user1",
            Resource::Usage,
            Action::Read,
            "org2".to_string(),
        )
        .await;

        assert!(!no_perm);
    }
}
