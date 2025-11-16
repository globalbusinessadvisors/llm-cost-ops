/// Comprehensive authentication tests
///
/// Tests JWT, API keys, RBAC, and authentication middleware

use chrono::{Duration, Utc};
use llm_cost_ops::auth::*;
use uuid::Uuid;

#[test]
fn test_jwt_manager_creation() {
    let config = AuthConfig::default_for_test();
    let manager = JwtManager::new(config);
    assert!(manager.is_ok());
}

#[test]
fn test_generate_token_pair() {
    let config = AuthConfig::default_for_test();
    let manager = JwtManager::new(config).unwrap();

    let result = manager.generate_token_pair(
        "user-123".to_string(),
        "org-456".to_string(),
        vec!["read:usage".to_string(), "write:usage".to_string()],
    );

    assert!(result.is_ok());
    let token_pair = result.unwrap();
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert_eq!(token_pair.token_type, "Bearer");
    assert!(token_pair.expires_in > 0);
}

#[test]
fn test_validate_access_token() {
    let config = AuthConfig::default_for_test();
    let manager = JwtManager::new(config).unwrap();

    let token_pair = manager
        .generate_token_pair(
            "user-123".to_string(),
            "org-456".to_string(),
            vec!["read:usage".to_string()],
        )
        .unwrap();

    let claims = manager.validate_token(&token_pair.access_token);
    assert!(claims.is_ok());

    let claims = claims.unwrap();
    assert_eq!(claims.sub, "user-123");
    assert_eq!(claims.org, "org-456");
    assert_eq!(claims.token_type, TokenType::Access);
    assert!(claims.permissions.contains(&"read:usage".to_string()));
}

#[test]
fn test_validate_expired_token() {
    let mut config = AuthConfig::default_for_test();
    config.jwt.access_token_expiration = Duration::milliseconds(1); // Very short expiration

    let manager = JwtManager::new(config).unwrap();

    let token_pair = manager
        .generate_token_pair(
            "user-123".to_string(),
            "org-456".to_string(),
            vec![],
        )
        .unwrap();

    // Wait for token to expire
    std::thread::sleep(std::time::Duration::from_millis(10));

    let result = manager.validate_token(&token_pair.access_token);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthError::TokenExpired));
}

#[test]
fn test_validate_invalid_token() {
    let config = AuthConfig::default_for_test();
    let manager = JwtManager::new(config).unwrap();

    let result = manager.validate_token("invalid.token.here");
    assert!(result.is_err());
}

#[test]
fn test_refresh_token() {
    let config = AuthConfig::default_for_test();
    let manager = JwtManager::new(config).unwrap();

    let token_pair = manager
        .generate_token_pair(
            "user-123".to_string(),
            "org-456".to_string(),
            vec!["read:usage".to_string()],
        )
        .unwrap();

    let new_token_pair = manager.refresh_access_token(&token_pair.refresh_token);
    assert!(new_token_pair.is_ok());

    let new_pair = new_token_pair.unwrap();
    assert_ne!(new_pair.access_token, token_pair.access_token);
}

#[test]
fn test_api_key_generation() {
    let key = ApiKey::generate("org-123".to_string(), "Test Key".to_string());

    assert!(!key.key_id.is_empty());
    assert!(!key.key_hash.is_empty());
    assert_eq!(key.organization_id, "org-123");
    assert_eq!(key.name, "Test Key");
    assert!(!key.revoked);
}

#[test]
fn test_api_key_validation() {
    let (key, secret) = ApiKey::generate_with_secret("org-123".to_string(), "Test Key".to_string());

    let is_valid = key.validate(&secret);
    assert!(is_valid);

    let is_invalid = key.validate("wrong_secret");
    assert!(!is_invalid);
}

#[test]
fn test_api_key_revocation() {
    let mut key = ApiKey::generate("org-123".to_string(), "Test Key".to_string());

    assert!(!key.revoked);
    key.revoke();
    assert!(key.revoked);
}

#[test]
fn test_api_key_expiration() {
    let mut key = ApiKey::generate("org-123".to_string(), "Test Key".to_string());
    key.expires_at = Some(Utc::now() - Duration::hours(1)); // Expired

    assert!(key.is_expired());
}

#[test]
fn test_rbac_role_creation() {
    let role = Role::new(
        "admin".to_string(),
        "Administrator".to_string(),
        vec![
            Permission::ReadUsage,
            Permission::WriteUsage,
            Permission::AdminAccess,
        ],
    );

    assert_eq!(role.name, "admin");
    assert_eq!(role.permissions.len(), 3);
}

#[test]
fn test_rbac_role_has_permission() {
    let role = Role::new(
        "viewer".to_string(),
        "Viewer".to_string(),
        vec![Permission::ReadUsage],
    );

    assert!(role.has_permission(&Permission::ReadUsage));
    assert!(!role.has_permission(&Permission::WriteUsage));
}

#[test]
fn test_rbac_role_assignment() {
    let user_id = Uuid::new_v4();
    let role_id = Uuid::new_v4();

    let assignment = RoleAssignment::new(user_id, role_id, "org-123".to_string());

    assert_eq!(assignment.user_id, user_id);
    assert_eq!(assignment.role_id, role_id);
    assert_eq!(assignment.organization_id, "org-123");
}

#[test]
fn test_permission_check() {
    let role = Role::new(
        "editor".to_string(),
        "Editor".to_string(),
        vec![Permission::ReadUsage, Permission::WriteUsage],
    );

    let checker = PermissionChecker::new(vec![role]);

    assert!(checker.has_permission("editor", &Permission::ReadUsage));
    assert!(checker.has_permission("editor", &Permission::WriteUsage));
    assert!(!checker.has_permission("editor", &Permission::AdminAccess));
}

#[test]
fn test_permission_serialization() {
    let permissions = vec![
        Permission::ReadUsage,
        Permission::WriteUsage,
        Permission::AdminAccess,
    ];

    for perm in permissions {
        let json = serde_json::to_string(&perm).unwrap();
        let deserialized: Permission = serde_json::from_str(&json).unwrap();
        assert_eq!(perm, deserialized);
    }
}

// === Security Tests ===

#[test]
fn test_token_signature_verification() {
    let config = AuthConfig::default_for_test();
    let manager = JwtManager::new(config).unwrap();

    let token_pair = manager
        .generate_token_pair("user-123".to_string(), "org-456".to_string(), vec![])
        .unwrap();

    // Tamper with the token
    let parts: Vec<&str> = token_pair.access_token.split('.').collect();
    let tampered = format!("{}.{}.tampered", parts[0], parts[1]);

    let result = manager.validate_token(&tampered);
    assert!(result.is_err());
}

#[test]
fn test_api_key_constant_time_comparison() {
    let (key, secret) = ApiKey::generate_with_secret("org-123".to_string(), "Test".to_string());

    // Both validations should take similar time regardless of correctness
    let start1 = std::time::Instant::now();
    let _ = key.validate(&secret);
    let elapsed1 = start1.elapsed();

    let start2 = std::time::Instant::now();
    let _ = key.validate("wrong_secret_with_same_length");
    let elapsed2 = start2.elapsed();

    // Allow some variance but they should be in the same ballpark
    let ratio = elapsed1.as_nanos() as f64 / elapsed2.as_nanos().max(1) as f64;
    assert!(ratio > 0.5 && ratio < 2.0, "Timing attack vulnerability detected");
}

#[test]
fn test_multiple_permissions() {
    let role = Role::new(
        "power_user".to_string(),
        "Power User".to_string(),
        vec![
            Permission::ReadUsage,
            Permission::WriteUsage,
            Permission::ReadCosts,
            Permission::WriteCosts,
        ],
    );

    assert_eq!(role.permissions.len(), 4);
    assert!(role.has_permission(&Permission::ReadUsage));
    assert!(role.has_permission(&Permission::WriteCosts));
}

#[test]
fn test_role_inheritance() {
    let viewer_role = Role::new(
        "viewer".to_string(),
        "Viewer".to_string(),
        vec![Permission::ReadUsage, Permission::ReadCosts],
    );

    let editor_role = Role::new(
        "editor".to_string(),
        "Editor".to_string(),
        vec![
            Permission::ReadUsage,
            Permission::ReadCosts,
            Permission::WriteUsage,
            Permission::WriteCosts,
        ],
    );

    // Editor should have all viewer permissions plus more
    for perm in &viewer_role.permissions {
        assert!(editor_role.has_permission(perm));
    }
}

// === Performance Tests ===

#[test]
fn test_token_validation_performance() {
    let config = AuthConfig::default_for_test();
    let manager = JwtManager::new(config).unwrap();

    let token_pair = manager
        .generate_token_pair("user-123".to_string(), "org-456".to_string(), vec![])
        .unwrap();

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = manager.validate_token(&token_pair.access_token);
    }
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100, "Token validation too slow: {:?}", elapsed);
}

#[test]
fn test_bulk_api_key_generation() {
    let start = std::time::Instant::now();

    let keys: Vec<_> = (0..100)
        .map(|i| ApiKey::generate(format!("org-{}", i), format!("Key {}", i)))
        .collect();

    let elapsed = start.elapsed();
    assert_eq!(keys.len(), 100);
    assert!(elapsed.as_millis() < 500, "Bulk key generation too slow: {:?}", elapsed);
}
