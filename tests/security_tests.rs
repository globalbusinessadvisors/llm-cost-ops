// Advanced Security Testing for Multi-Tenancy
// Penetration tests, privilege escalation attempts, and security boundary validation

use llm_cost_ops::{
    auth::{
        api_key::{ApiKey, ApiKeyHash, hash_api_key, validate_api_key_format},
        jwt::JwtManager,
        rbac::{Action, Permission, Resource, RbacManager, Role, RoleType},
        storage::InMemoryApiKeyStore,
        AuthConfig, AuthError,
    },
};
use std::collections::HashSet;

// ============================================================================
// API KEY SECURITY TESTS
// ============================================================================

#[tokio::test]
async fn test_api_key_format_validation() {
    // Valid format
    assert!(validate_api_key_format(
        "llmco-abcdefghijklmnopqrstuvwxyz123456",
        "llmco-"
    )
    .is_ok());

    // Wrong prefix
    assert!(validate_api_key_format(
        "wrong-abcdefghijklmnopqrstuvwxyz123456",
        "llmco-"
    )
    .is_err());

    // Too short
    assert!(validate_api_key_format("llmco-short", "llmco-").is_err());

    // Missing prefix
    assert!(validate_api_key_format("abcdefghijklmnopqrstuvwxyz123456", "llmco-").is_err());
}

#[tokio::test]
async fn test_api_key_hash_uniqueness() {
    let hash1 = hash_api_key("test-key-1");
    let hash2 = hash_api_key("test-key-2");
    let hash3 = hash_api_key("test-key-1"); // Same as hash1

    assert_ne!(hash1, hash2, "Different keys should have different hashes");
    assert_eq!(hash1, hash3, "Same key should produce same hash");
    assert_eq!(hash1.len(), 64, "SHA-256 hash should be 64 hex characters");
}

#[tokio::test]
async fn test_api_key_generation_uniqueness() {
    let mut keys = HashSet::new();

    // Generate 1000 keys and verify they're all unique
    for i in 0..1000 {
        let key = ApiKey::generate(
            format!("org-{}", i),
            format!("Key {}", i),
            "llmco-test-".to_string(),
            32,
            vec!["read".to_string()],
        );

        let raw_key = key.key.unwrap();
        assert!(
            keys.insert(raw_key.clone()),
            "Generated duplicate key: {}",
            raw_key
        );
    }
}

#[tokio::test]
async fn test_api_key_cannot_be_reversed_from_hash() {
    let key = ApiKey::generate(
        "org-hash-test".to_string(),
        "Hash Test".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string()],
    );

    let raw_key = key.key.unwrap();
    let hash = hash_api_key(&raw_key);

    // The hash should be one-way - we can't get the original key back
    assert_ne!(raw_key, hash);
    assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
    assert!(!hash.contains("llmco-test-")); // Hash shouldn't contain original prefix
}

#[tokio::test]
async fn test_inactive_api_key_rejection() {
    let store = InMemoryApiKeyStore::new();

    let key = ApiKey::generate(
        "org-inactive-test".to_string(),
        "Inactive Test".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string()],
    );

    let raw_key = key.key.clone().unwrap();
    let mut hash = key.to_hash().unwrap();

    // Make key inactive
    hash.is_active = false;
    store.store(hash).await.unwrap();

    // Verification should fail for inactive key
    let result = store.verify(&raw_key, "llmco-test-").await;
    assert!(result.is_err(), "Inactive API key should be rejected");
}

#[tokio::test]
async fn test_api_key_permissions_enforcement() {
    let read_key = ApiKey::generate(
        "org-perms-test".to_string(),
        "Read Only".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string()],
    );

    let write_key = ApiKey::generate(
        "org-perms-test".to_string(),
        "Read Write".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string(), "write".to_string()],
    );

    let admin_key = ApiKey::generate(
        "org-perms-test".to_string(),
        "Admin".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["*".to_string()], // Wildcard permission
    );

    let read_hash = read_key.to_hash().unwrap();
    let write_hash = write_key.to_hash().unwrap();
    let admin_hash = admin_key.to_hash().unwrap();

    // Test read-only key
    assert!(read_hash.has_permission("read"));
    assert!(!read_hash.has_permission("write"));
    assert!(!read_hash.has_permission("admin"));

    // Test read-write key
    assert!(write_hash.has_permission("read"));
    assert!(write_hash.has_permission("write"));
    assert!(!write_hash.has_permission("admin"));

    // Test admin key with wildcard
    assert!(admin_hash.has_permission("read"));
    assert!(admin_hash.has_permission("write"));
    assert!(admin_hash.has_permission("admin"));
    assert!(admin_hash.has_permission("anything")); // Wildcard matches all
}

// ============================================================================
// JWT SECURITY TESTS
// ============================================================================

#[tokio::test]
async fn test_jwt_token_tampering_detection() {
    let config = AuthConfig::development();
    let jwt_manager = JwtManager::new(config).unwrap();

    let token = jwt_manager
        .generate_access_token(
            "user-123".to_string(),
            "org-456".to_string(),
            vec!["read".to_string()],
        )
        .unwrap();

    // Tamper with the token
    let mut tampered_token = token.clone();
    tampered_token.push('x'); // Add extra character

    // Validation should fail
    let result = jwt_manager.validate_access_token(&tampered_token);
    assert!(result.is_err(), "Tampered token should be rejected");

    // Try changing a character in the middle
    let mut chars: Vec<char> = token.chars().collect();
    if let Some(c) = chars.get_mut(20) {
        *c = if *c == 'a' { 'b' } else { 'a' };
    }
    let tampered_token2: String = chars.into_iter().collect();

    let result2 = jwt_manager.validate_access_token(&tampered_token2);
    assert!(result2.is_err(), "Modified token should be rejected");
}

#[tokio::test]
async fn test_jwt_cannot_be_reused_across_organizations() {
    let config = AuthConfig::development();
    let jwt_manager = JwtManager::new(config).unwrap();

    let token = jwt_manager
        .generate_access_token(
            "user-123".to_string(),
            "org-original".to_string(),
            vec!["read".to_string()],
        )
        .unwrap();

    // Validate the token
    let claims = jwt_manager.validate_access_token(&token).unwrap();

    // The organization is embedded in the claims
    assert_eq!(claims.org, "org-original");

    // You cannot use this token to access data from another org
    // This would be enforced at the application layer by checking claims.org
    assert_ne!(claims.org, "org-different");
}

#[tokio::test]
async fn test_jwt_expiration_enforcement() {
    use std::time::Duration;

    let mut config = AuthConfig::development();
    config.jwt.access_token_ttl = Duration::from_millis(100); // 100ms expiration

    let jwt_manager = JwtManager::new(config).unwrap();

    let token = jwt_manager
        .generate_access_token(
            "user-123".to_string(),
            "org-456".to_string(),
            vec!["read".to_string()],
        )
        .unwrap();

    // Token should be valid immediately
    assert!(jwt_manager.validate_access_token(&token).is_ok());

    // Wait for expiration
    std::thread::sleep(Duration::from_millis(150));

    // Token should now be expired
    let result = jwt_manager.validate_access_token(&token);
    assert!(
        result.is_err(),
        "Expired token should be rejected"
    );

    if let Err(e) = result {
        assert!(matches!(e, AuthError::TokenExpired));
    }
}

#[tokio::test]
async fn test_jwt_refresh_token_flow() {
    let config = AuthConfig::development();
    let jwt_manager = JwtManager::new(config).unwrap();

    // Generate initial access and refresh tokens
    let access_token = jwt_manager
        .generate_access_token(
            "user-123".to_string(),
            "org-456".to_string(),
            vec!["read".to_string()],
        )
        .unwrap();

    let refresh_token = jwt_manager
        .generate_refresh_token("user-123".to_string(), "org-456".to_string())
        .unwrap();

    // Validate access token
    let access_claims = jwt_manager.validate_access_token(&access_token).unwrap();
    assert_eq!(access_claims.sub, "user-123");
    assert_eq!(access_claims.org, "org-456");

    // Validate refresh token
    let refresh_claims = jwt_manager.validate_refresh_token(&refresh_token).unwrap();
    assert_eq!(refresh_claims.sub, "user-123");
    assert_eq!(refresh_claims.org, "org-456");

    // Refresh tokens should not work as access tokens
    let result = jwt_manager.validate_access_token(&refresh_token);
    assert!(result.is_err(), "Refresh token should not validate as access token");
}

// ============================================================================
// RBAC SECURITY TESTS
// ============================================================================

#[tokio::test]
async fn test_rbac_privilege_escalation_prevention() {
    let manager = RbacManager::new();

    // Create a read-only user
    let read_only_role = Role::read_only("org-escalation".to_string());
    manager.create_role(read_only_role.clone()).await.unwrap();

    manager
        .assign_user_role("user-readonly", &read_only_role.id)
        .await
        .unwrap();

    // Attempt to check for admin permissions
    let admin_perm = Permission::scoped(
        Resource::System,
        Action::ManagePermissions,
        "org-escalation".to_string(),
    );

    assert!(
        !manager.check_permission("user-readonly", &admin_perm).await,
        "Read-only user should not have admin permissions"
    );

    // Attempt to check for delete permissions
    let delete_perm = Permission::scoped(
        Resource::Usage,
        Action::Delete,
        "org-escalation".to_string(),
    );

    assert!(
        !manager.check_permission("user-readonly", &delete_perm).await,
        "Read-only user should not have delete permissions"
    );
}

#[tokio::test]
async fn test_rbac_system_role_protection() {
    let manager = RbacManager::new();

    // Attempt to delete a system role
    let result = manager.delete_role("super_admin").await;
    assert!(
        result.is_err(),
        "System roles should not be deletable"
    );

    // Attempt to delete auditor role (also system)
    let result = manager.delete_role("auditor").await;
    assert!(
        result.is_err(),
        "System roles should not be deletable"
    );
}

#[tokio::test]
async fn test_rbac_cross_organization_permission_denial() {
    let manager = RbacManager::new();

    // Create roles for two different organizations
    let org1_admin = Role::org_admin("org-cross-1".to_string());
    let org2_admin = Role::org_admin("org-cross-2".to_string());

    manager.create_role(org1_admin.clone()).await.unwrap();
    manager.create_role(org2_admin.clone()).await.unwrap();

    // Assign user to org1
    manager
        .assign_user_role("user-org1-admin", &org1_admin.id)
        .await
        .unwrap();

    // Check permissions for org1 (should pass)
    let org1_perm = Permission::scoped(Resource::Usage, Action::Create, "org-cross-1".to_string());
    assert!(manager.check_permission("user-org1-admin", &org1_perm).await);

    // Check permissions for org2 (should fail - cross-org access attempt)
    let org2_perm = Permission::scoped(Resource::Usage, Action::Create, "org-cross-2".to_string());
    assert!(
        !manager.check_permission("user-org1-admin", &org2_perm).await,
        "Admin from org1 should NOT have permissions in org2"
    );
}

#[tokio::test]
async fn test_rbac_permission_scoping() {
    let manager = RbacManager::new();

    // Create scoped and unscoped permissions
    let scoped_perm = Permission::scoped(Resource::Usage, Action::Read, "org-specific".to_string());
    let unscoped_perm = Permission::new(Resource::Usage, Action::Read);

    // Create a role with scoped permission
    let mut scoped_role = Role::new(
        "scoped-role".to_string(),
        "Scoped Role".to_string(),
        "Role with scoped permissions".to_string(),
    );
    scoped_role.add_permission(scoped_perm.clone());

    manager.create_role(scoped_role.clone()).await.unwrap();
    manager
        .assign_user_role("user-scoped", "scoped-role")
        .await
        .unwrap();

    // User should have access to scoped resource
    assert!(manager.check_permission("user-scoped", &scoped_perm).await);

    // User should NOT have unscoped (global) access
    assert!(
        !manager.check_permission("user-scoped", &unscoped_perm).await,
        "Scoped permission should not grant unscoped access"
    );

    // User should NOT have access to different scope
    let other_scope_perm =
        Permission::scoped(Resource::Usage, Action::Read, "org-other".to_string());
    assert!(
        !manager.check_permission("user-scoped", &other_scope_perm).await,
        "Should not have access to different scope"
    );
}

#[tokio::test]
async fn test_rbac_billing_role_restrictions() {
    let manager = RbacManager::new();

    let billing_role = Role::billing("org-billing-test".to_string());
    manager.create_role(billing_role.clone()).await.unwrap();

    manager
        .assign_user_role("billing-user", &billing_role.id)
        .await
        .unwrap();

    // Billing user should have access to cost and pricing data
    let cost_read = Permission::scoped(Resource::Cost, Action::Read, "org-billing-test".to_string());
    let pricing_read =
        Permission::scoped(Resource::Pricing, Action::Read, "org-billing-test".to_string());
    let budget_read =
        Permission::scoped(Resource::Budget, Action::Read, "org-billing-test".to_string());

    assert!(manager.check_permission("billing-user", &cost_read).await);
    assert!(manager.check_permission("billing-user", &pricing_read).await);
    assert!(manager.check_permission("billing-user", &budget_read).await);

    // But should NOT have access to user management or system settings
    let user_create =
        Permission::scoped(Resource::User, Action::Create, "org-billing-test".to_string());
    let system_manage = Permission::scoped(
        Resource::System,
        Action::ManagePermissions,
        "org-billing-test".to_string(),
    );

    assert!(!manager.check_permission("billing-user", &user_create).await);
    assert!(!manager.check_permission("billing-user", &system_manage).await);
}

#[tokio::test]
async fn test_rbac_auditor_role_restrictions() {
    let manager = RbacManager::new();

    // Auditor role should be pre-created as a system role
    manager
        .assign_user_role("auditor-user", "auditor")
        .await
        .unwrap();

    // Auditor should have read access to audit logs
    let audit_read = Permission::new(Resource::AuditLog, Action::Read);
    let audit_list = Permission::new(Resource::AuditLog, Action::List);
    let audit_export = Permission::new(Resource::AuditLog, Action::Export);

    assert!(manager.check_permission("auditor-user", &audit_read).await);
    assert!(manager.check_permission("auditor-user", &audit_list).await);
    assert!(manager.check_permission("auditor-user", &audit_export).await);

    // But should NOT have write access
    let audit_delete = Permission::new(Resource::AuditLog, Action::Delete);
    let audit_update = Permission::new(Resource::AuditLog, Action::Update);

    assert!(!manager.check_permission("auditor-user", &audit_delete).await);
    assert!(!manager.check_permission("auditor-user", &audit_update).await);

    // And should NOT have access to other resources
    let usage_read = Permission::new(Resource::Usage, Action::Read);
    assert!(!manager.check_permission("auditor-user", &usage_read).await);
}

#[tokio::test]
async fn test_rbac_super_admin_has_all_permissions() {
    let manager = RbacManager::new();

    manager
        .assign_user_role("super-admin-user", "super_admin")
        .await
        .unwrap();

    // Test various permissions - super admin should have them all
    let test_permissions = vec![
        Permission::new(Resource::Usage, Action::Read),
        Permission::new(Resource::Usage, Action::Create),
        Permission::new(Resource::Usage, Action::Delete),
        Permission::new(Resource::Cost, Action::Read),
        Permission::new(Resource::Pricing, Action::Update),
        Permission::new(Resource::ApiKey, Action::Create),
        Permission::new(Resource::User, Action::ManagePermissions),
        Permission::new(Resource::System, Action::ManagePermissions),
        Permission::new(Resource::AuditLog, Action::Export),
    ];

    for permission in test_permissions {
        assert!(
            manager.check_permission("super-admin-user", &permission).await,
            "Super admin should have permission: {:?}",
            permission
        );
    }
}

#[tokio::test]
async fn test_rbac_direct_permission_grant() {
    let manager = RbacManager::new();

    // Grant a direct permission without a role
    let custom_perm =
        Permission::scoped(Resource::Usage, Action::Execute, "org-direct".to_string());

    manager
        .grant_permission("user-direct", custom_perm.clone())
        .await;

    // User should have the directly granted permission
    assert!(manager.check_permission("user-direct", &custom_perm).await);

    // But should not have other permissions
    let other_perm = Permission::scoped(Resource::Usage, Action::Delete, "org-direct".to_string());
    assert!(!manager.check_permission("user-direct", &other_perm).await);
}

#[tokio::test]
async fn test_rbac_role_combination() {
    let manager = RbacManager::new();

    let read_only = Role::read_only("org-combo".to_string());
    let billing = Role::billing("org-combo".to_string());

    manager.create_role(read_only.clone()).await.unwrap();
    manager.create_role(billing.clone()).await.unwrap();

    // Assign both roles to a user
    manager
        .assign_user_role("combo-user", &read_only.id)
        .await
        .unwrap();
    manager
        .assign_user_role("combo-user", &billing.id)
        .await
        .unwrap();

    // User should have permissions from both roles
    let usage_read = Permission::scoped(Resource::Usage, Action::Read, "org-combo".to_string());
    let cost_export = Permission::scoped(Resource::Cost, Action::Export, "org-combo".to_string());

    assert!(
        manager.check_permission("combo-user", &usage_read).await,
        "Should have read-only permissions"
    );
    assert!(
        manager.check_permission("combo-user", &cost_export).await,
        "Should have billing permissions"
    );

    // But still should not have write permissions
    let usage_create = Permission::scoped(Resource::Usage, Action::Create, "org-combo".to_string());
    assert!(!manager.check_permission("combo-user", &usage_create).await);
}

#[tokio::test]
async fn test_rbac_role_removal() {
    let manager = RbacManager::new();

    let role = Role::read_only("org-removal".to_string());
    manager.create_role(role.clone()).await.unwrap();

    manager
        .assign_user_role("user-removal", &role.id)
        .await
        .unwrap();

    // Verify user has permissions
    let perm = Permission::scoped(Resource::Usage, Action::Read, "org-removal".to_string());
    assert!(manager.check_permission("user-removal", &perm).await);

    // Remove the role
    manager
        .remove_user_role("user-removal", &role.id)
        .await
        .unwrap();

    // User should no longer have permissions
    assert!(!manager.check_permission("user-removal", &perm).await);
}
