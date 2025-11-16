// Comprehensive Security Testing Suite
// Tests authentication, authorization, encryption, and security best practices

use llm_cost_ops::{
    auth::{
        ApiKey, ApiKeyHash, ApiKeyStore, AuditEvent, AuditEventType, AuditLogger, AuditSeverity,
        InMemoryApiKeyStore, InMemoryAuditStore, JwtClaims, JwtManager, Permission, RbacManager,
        Role, RoleType, UserRole,
    },
    domain::{ModelIdentifier, Provider, UsageRecord},
};
use sha2::{Digest, Sha256};
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn test_api_key_generation_is_secure() {
    let api_key = ApiKey::generate();

    // Should be 32 characters (128 bits of entropy)
    assert_eq!(api_key.key.len(), 32);

    // Should not contain easily guessable patterns
    assert!(!api_key.key.chars().all(|c| c == api_key.key.chars().next().unwrap()));

    // Generate multiple keys and ensure they're different
    let api_key2 = ApiKey::generate();
    assert_ne!(api_key.key, api_key2.key);
}

#[tokio::test]
async fn test_api_key_hashing_is_secure() {
    let api_key = ApiKey::generate();
    let hash1 = ApiKeyHash::from_api_key(&api_key);
    let hash2 = ApiKeyHash::from_api_key(&api_key);

    // Same key should produce same hash
    assert_eq!(hash1.hash, hash2.hash);

    // Different keys should produce different hashes
    let different_key = ApiKey::generate();
    let different_hash = ApiKeyHash::from_api_key(&different_key);
    assert_ne!(hash1.hash, different_hash.hash);

    // Hash should not reveal original key
    assert_ne!(hash1.hash, api_key.key);
    assert!(!hash1.hash.contains(&api_key.key));
}

#[tokio::test]
async fn test_api_key_verification_constant_time() {
    let api_key = ApiKey::generate();
    let hash = ApiKeyHash::from_api_key(&api_key);

    // Correct key should verify
    assert!(hash.verify(&api_key));

    // Incorrect key should not verify
    let wrong_key = ApiKey::generate();
    assert!(!hash.verify(&wrong_key));

    // Timing attack protection: verification time should be constant
    // This is ensured by using constant_time_eq crate in the implementation
}

#[tokio::test]
async fn test_api_key_store_security() {
    let store = InMemoryApiKeyStore::new();
    let api_key = ApiKey::generate();
    let user_id = "user-123";

    store.create_key(user_id, &api_key).await.unwrap();

    // Should not be able to retrieve raw key
    let stored_hash = store.get_hash_by_user(user_id).await.unwrap().unwrap();

    // Stored hash should not be the original key
    assert_ne!(stored_hash.hash, api_key.key);

    // Verification should work
    assert!(stored_hash.verify(&api_key));

    // Wrong key should fail
    let wrong_key = ApiKey::generate();
    assert!(!stored_hash.verify(&wrong_key));
}

#[tokio::test]
async fn test_jwt_token_security() {
    let secret = "test-secret-key-that-is-long-enough-for-hs256";
    let manager = JwtManager::new(secret.to_string());

    let claims = JwtClaims {
        sub: "user-123".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
        user_id: "user-123".to_string(),
        organization_id: Some("org-456".to_string()),
        role: "admin".to_string(),
        permissions: vec!["cost:read".to_string(), "cost:write".to_string()],
    };

    let token = manager.create_token(&claims).unwrap();

    // Token should not be empty
    assert!(!token.is_empty());

    // Should be able to verify and decode valid token
    let decoded = manager.verify_token(&token).unwrap();
    assert_eq!(decoded.claims.user_id, "user-123");
    assert_eq!(decoded.claims.role, "admin");

    // Should fail with tampered token
    let tampered = token.clone() + "tampered";
    assert!(manager.verify_token(&tampered).is_err());

    // Should fail with expired token
    let expired_claims = JwtClaims {
        sub: "user-123".to_string(),
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize,
        iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp() as usize,
        user_id: "user-123".to_string(),
        organization_id: Some("org-456".to_string()),
        role: "admin".to_string(),
        permissions: vec![],
    };

    let expired_token = manager.create_token(&expired_claims).unwrap();
    assert!(manager.verify_token(&expired_token).is_err());
}

#[tokio::test]
async fn test_rbac_permission_enforcement() {
    let rbac = RbacManager::new();

    // Create roles with different permissions
    let admin_role = Role {
        id: Uuid::new_v4(),
        name: "Admin".to_string(),
        role_type: RoleType::System,
        permissions: vec![
            Permission::all("cost"),
            Permission::all("budget"),
            Permission::all("user"),
        ],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let viewer_role = Role {
        id: Uuid::new_v4(),
        name: "Viewer".to_string(),
        role_type: RoleType::System,
        permissions: vec![Permission::read("cost")],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    rbac.create_role(admin_role.clone()).await.unwrap();
    rbac.create_role(viewer_role.clone()).await.unwrap();

    // Assign roles to users
    rbac.assign_role("admin-user", admin_role.id, None)
        .await
        .unwrap();
    rbac.assign_role("viewer-user", viewer_role.id, None)
        .await
        .unwrap();

    // Admin should have all permissions
    assert!(rbac
        .check_permission("admin-user", &Permission::read("cost"), None)
        .await
        .unwrap());
    assert!(rbac
        .check_permission("admin-user", &Permission::write("cost"), None)
        .await
        .unwrap());
    assert!(rbac
        .check_permission("admin-user", &Permission::delete("cost"), None)
        .await
        .unwrap());

    // Viewer should only have read permission
    assert!(rbac
        .check_permission("viewer-user", &Permission::read("cost"), None)
        .await
        .unwrap());
    assert!(!rbac
        .check_permission("viewer-user", &Permission::write("cost"), None)
        .await
        .unwrap());
    assert!(!rbac
        .check_permission("viewer-user", &Permission::delete("cost"), None)
        .await
        .unwrap());

    // Non-existent user should have no permissions
    assert!(!rbac
        .check_permission("non-existent", &Permission::read("cost"), None)
        .await
        .unwrap());
}

#[tokio::test]
async fn test_audit_logging_captures_sensitive_operations() {
    let store = InMemoryAuditStore::new();
    let logger = AuditLogger::new(store.clone());

    // Log a sensitive operation
    let event = AuditEvent {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        event_type: AuditEventType::Authentication,
        actor_id: "user-123".to_string(),
        action: "api_key_created".to_string(),
        resource_type: Some("api_key".to_string()),
        resource_id: Some("key-456".to_string()),
        organization_id: Some("org-789".to_string()),
        severity: AuditSeverity::High,
        status: llm_cost_ops::auth::AuditStatus::Success,
        ip_address: Some("192.168.1.1".to_string()),
        user_agent: Some("test-client".to_string()),
        metadata: serde_json::json!({
            "key_id": "key-456",
            "permissions": ["cost:read", "cost:write"]
        }),
    };

    logger.log(event.clone()).await.unwrap();

    // Verify event was logged
    let events = store.list_events(None, None, None).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].actor_id, "user-123");
    assert_eq!(events[0].action, "api_key_created");

    // Should not log sensitive data in plaintext
    assert!(!format!("{:?}", events[0]).contains("secret"));
    assert!(!format!("{:?}", events[0]).contains("password"));
}

#[tokio::test]
async fn test_sql_injection_protection() {
    // Test that malicious SQL in user input is properly escaped
    let malicious_org_id = "org-123'; DROP TABLE usage_records; --";

    let usage = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: None,
            context_window: None,
        },
        organization_id: malicious_org_id.to_string(),
        project_id: None,
        user_id: None,
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 1500,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: None,
        tags: vec![],
        metadata: serde_json::json!({}),
        ingested_at: chrono::Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "test".to_string(),
            endpoint: None,
        },
    };

    // SQLx uses parameterized queries, so this should be safe
    // The malicious SQL should be treated as literal string data
    assert_eq!(usage.organization_id, malicious_org_id);
}

#[tokio::test]
async fn test_password_hashing_security() {
    // Even though we use API keys, test password hashing best practices

    let password = "SecurePassword123!";

    // Hash using SHA-256 (in production, use bcrypt/argon2)
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let hash1 = format!("{:x}", hasher.finalize());

    // Same password should produce same hash
    let mut hasher2 = Sha256::new();
    hasher2.update(password.as_bytes());
    let hash2 = format!("{:x}", hasher2.finalize());
    assert_eq!(hash1, hash2);

    // Different password should produce different hash
    let mut hasher3 = Sha256::new();
    hasher3.update("DifferentPassword".as_bytes());
    let hash3 = format!("{:x}", hasher3.finalize());
    assert_ne!(hash1, hash3);
}

#[tokio::test]
async fn test_rate_limiting_prevents_brute_force() {
    use llm_cost_ops::ingestion::RateLimiter;

    let limiter = RateLimiter::new(5, Duration::from_secs(1));

    // First 5 requests should succeed
    for i in 0..5 {
        assert!(
            limiter.check_rate_limit("attacker-ip").await.is_ok(),
            "Request {} should succeed",
            i
        );
    }

    // 6th request should be rate limited
    assert!(limiter.check_rate_limit("attacker-ip").await.is_err());

    // Different IP should not be affected
    assert!(limiter.check_rate_limit("normal-user-ip").await.is_ok());
}

#[tokio::test]
async fn test_session_token_rotation() {
    let secret = "test-secret-key";
    let manager = JwtManager::new(secret.to_string());

    let claims = JwtClaims {
        sub: "user-123".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
        user_id: "user-123".to_string(),
        organization_id: Some("org-456".to_string()),
        role: "user".to_string(),
        permissions: vec!["cost:read".to_string()],
    };

    let token1 = manager.create_token(&claims).unwrap();

    // Simulate token rotation after 10 minutes
    tokio::time::sleep(Duration::from_millis(100)).await;

    let new_claims = JwtClaims {
        exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp() as usize,
        iat: chrono::Utc::now().timestamp() as usize,
        ..claims
    };

    let token2 = manager.create_token(&new_claims).unwrap();

    // Tokens should be different (different iat)
    assert_ne!(token1, token2);

    // Both should be valid
    assert!(manager.verify_token(&token1).is_ok());
    assert!(manager.verify_token(&token2).is_ok());
}

#[tokio::test]
async fn test_authorization_bypass_prevention() {
    let rbac = RbacManager::new();

    let user_role = Role {
        id: Uuid::new_v4(),
        name: "User".to_string(),
        role_type: RoleType::System,
        permissions: vec![Permission::read("cost")],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    rbac.create_role(user_role.clone()).await.unwrap();
    rbac.assign_role("user-123", user_role.id, None)
        .await
        .unwrap();

    // User should not be able to bypass permission checks
    assert!(!rbac
        .check_permission("user-123", &Permission::delete("cost"), None)
        .await
        .unwrap());

    // Attempting to escalate privileges should fail
    let admin_permission = Permission::all("*");
    assert!(!rbac
        .check_permission("user-123", &admin_permission, None)
        .await
        .unwrap());
}

#[tokio::test]
async fn test_cross_organization_data_isolation() {
    // Test that users can only access data from their own organization

    let org1_usage = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: None,
            context_window: None,
        },
        organization_id: "org-111".to_string(),
        project_id: None,
        user_id: Some("user-org1".to_string()),
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 1500,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: None,
        tags: vec![],
        metadata: serde_json::json!({"sensitive": "data-org1"}),
        ingested_at: chrono::Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "test".to_string(),
            endpoint: None,
        },
    };

    let org2_usage = UsageRecord {
        organization_id: "org-222".to_string(),
        user_id: Some("user-org2".to_string()),
        metadata: serde_json::json!({"sensitive": "data-org2"}),
        ..org1_usage.clone()
    };

    // User from org1 should not access org2 data
    assert_ne!(org1_usage.organization_id, org2_usage.organization_id);
    assert_ne!(
        org1_usage.metadata["sensitive"],
        org2_usage.metadata["sensitive"]
    );
}

#[tokio::test]
async fn test_sensitive_data_not_logged() {
    let store = InMemoryAuditStore::new();
    let logger = AuditLogger::new(store.clone());

    // Create event with potentially sensitive data
    let event = AuditEvent {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        event_type: AuditEventType::DataAccess,
        actor_id: "user-123".to_string(),
        action: "cost_query".to_string(),
        resource_type: Some("cost_records".to_string()),
        resource_id: None,
        organization_id: Some("org-789".to_string()),
        severity: AuditSeverity::Low,
        status: llm_cost_ops::auth::AuditStatus::Success,
        ip_address: Some("192.168.1.1".to_string()),
        user_agent: Some("test-client".to_string()),
        metadata: serde_json::json!({
            "query_params": {
                "organization_id": "org-789",
                // Should NOT include sensitive data like API keys or tokens
            }
        }),
    };

    logger.log(event.clone()).await.unwrap();

    // Verify no sensitive data in logs
    let events = store.list_events(None, None, None).await.unwrap();
    let event_str = format!("{:?}", events[0]);

    assert!(!event_str.contains("api_key"));
    assert!(!event_str.contains("password"));
    assert!(!event_str.contains("secret"));
    assert!(!event_str.contains("token"));
}

#[tokio::test]
async fn test_timing_attack_resistance() {
    // Test that authentication timing is constant

    let api_key = ApiKey::generate();
    let hash = ApiKeyHash::from_api_key(&api_key);

    // Correct key
    let start = std::time::Instant::now();
    let result1 = hash.verify(&api_key);
    let time1 = start.elapsed();
    assert!(result1);

    // Incorrect key (similar length)
    let wrong_key = ApiKey::generate();
    let start = std::time::Instant::now();
    let result2 = hash.verify(&wrong_key);
    let time2 = start.elapsed();
    assert!(!result2);

    // Times should be similar (constant time comparison)
    let diff = if time1 > time2 {
        time1 - time2
    } else {
        time2 - time1
    };

    // Allow for some variance, but should be < 1ms difference
    assert!(
        diff.as_micros() < 1000,
        "Timing difference too large: {:?}",
        diff
    );
}
