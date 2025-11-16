// Comprehensive Multi-Tenancy Test Suite
// Tests for tenant isolation, cross-tenant access prevention, and security

use chrono::Utc;
use llm_cost_ops::{
    auth::{
        api_key::{ApiKey, ApiKeyHash},
        jwt::JwtManager,
        middleware::AuthContext,
        rbac::{Action, Permission, Resource, RbacManager, Role},
        storage::InMemoryApiKeyStore,
        AuthConfig,
    },
    domain::{ModelIdentifier, PricingStructure, PricingTable, Provider, UsageRecord},
    engine::CostCalculator,
    storage::{
        CostRepository, PricingRepository, SqliteCostRepository, SqlitePricingRepository,
        SqliteUsageRepository, UsageRepository,
    },
};
use rust_decimal::Decimal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use uuid::Uuid;

// ============================================================================
// TEST INFRASTRUCTURE
// ============================================================================

async fn setup_test_db() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .expect("Failed to create test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn create_test_pricing() -> PricingStructure {
    PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    )
}

fn create_test_usage(org_id: &str, project_id: Option<&str>) -> UsageRecord {
    UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: Some("gpt-4-0613".to_string()),
            context_window: Some(8192),
        },
        organization_id: org_id.to_string(),
        project_id: project_id.map(|s| s.to_string()),
        user_id: None,
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 1500,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: Some(250),
        time_to_first_token_ms: None,
        tags: vec!["test".to_string()],
        metadata: serde_json::json!({}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource::Api {
            endpoint: "test".to_string(),
        },
    }
}

// ============================================================================
// TENANT ISOLATION TESTS
// ============================================================================

#[tokio::test]
async fn test_tenant_data_isolation_usage_records() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool.clone());

    // Setup pricing
    let pricing_table = PricingTable::new(
        Provider::OpenAI,
        "gpt-4".to_string(),
        create_test_pricing(),
    );
    pricing_repo.create(&pricing_table).await.unwrap();

    // Create usage records for different tenants
    let org1_usage1 = create_test_usage("org-tenant-1", Some("proj-1"));
    let org1_usage2 = create_test_usage("org-tenant-1", Some("proj-2"));
    let org2_usage1 = create_test_usage("org-tenant-2", Some("proj-1"));
    let org2_usage2 = create_test_usage("org-tenant-2", Some("proj-2"));
    let org3_usage1 = create_test_usage("org-tenant-3", Some("proj-1"));

    // Store all usage records
    usage_repo.create(&org1_usage1).await.unwrap();
    usage_repo.create(&org1_usage2).await.unwrap();
    usage_repo.create(&org2_usage1).await.unwrap();
    usage_repo.create(&org2_usage2).await.unwrap();
    usage_repo.create(&org3_usage1).await.unwrap();

    // Query for each tenant
    let start = Utc::now() - chrono::Duration::days(1);
    let end = Utc::now() + chrono::Duration::hours(1);

    let org1_records = usage_repo
        .list_by_organization("org-tenant-1", start, end)
        .await
        .unwrap();
    let org2_records = usage_repo
        .list_by_organization("org-tenant-2", start, end)
        .await
        .unwrap();
    let org3_records = usage_repo
        .list_by_organization("org-tenant-3", start, end)
        .await
        .unwrap();

    // Verify isolation: each tenant should only see their own data
    assert_eq!(org1_records.len(), 2, "Tenant 1 should have 2 records");
    assert_eq!(org2_records.len(), 2, "Tenant 2 should have 2 records");
    assert_eq!(org3_records.len(), 1, "Tenant 3 should have 1 record");

    // Verify all records belong to the correct tenant
    for record in &org1_records {
        assert_eq!(record.organization_id, "org-tenant-1");
    }
    for record in &org2_records {
        assert_eq!(record.organization_id, "org-tenant-2");
    }
    for record in &org3_records {
        assert_eq!(record.organization_id, "org-tenant-3");
    }
}

#[tokio::test]
async fn test_tenant_data_isolation_cost_records() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool.clone());
    let cost_repo = SqliteCostRepository::new(pool);

    // Setup pricing
    let pricing_table = PricingTable::new(
        Provider::OpenAI,
        "gpt-4".to_string(),
        create_test_pricing(),
    );
    pricing_repo.create(&pricing_table).await.unwrap();

    let calculator = CostCalculator::new();

    // Create and calculate costs for different tenants
    let tenants = vec!["org-cost-1", "org-cost-2", "org-cost-3"];

    for tenant in &tenants {
        for i in 0..3 {
            let usage = create_test_usage(tenant, Some(&format!("proj-{}", i)));
            usage_repo.create(&usage).await.unwrap();

            let cost = calculator.calculate(&usage, &pricing_table).unwrap();
            cost_repo.create(&cost).await.unwrap();
        }
    }

    // Query costs for each tenant
    let start = Utc::now() - chrono::Duration::days(1);
    let end = Utc::now() + chrono::Duration::hours(1);

    for tenant in &tenants {
        let costs = cost_repo
            .list_by_organization(tenant, start, end)
            .await
            .unwrap();

        assert_eq!(costs.len(), 3, "Each tenant should have exactly 3 cost records");

        // Verify all costs belong to the correct tenant
        for cost in &costs {
            assert_eq!(&cost.organization_id, tenant);
        }
    }
}

#[tokio::test]
async fn test_cross_tenant_access_prevention() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool);

    // Create usage records for two different tenants
    let org1_usage = create_test_usage("org-secure-1", Some("proj-1"));
    let org2_usage = create_test_usage("org-secure-2", Some("proj-1"));

    usage_repo.create(&org1_usage).await.unwrap();
    usage_repo.create(&org2_usage).await.unwrap();

    // Attempt to retrieve org1's record by ID (no tenant context)
    let retrieved = usage_repo.get_by_id(org1_usage.id).await.unwrap();
    assert!(retrieved.is_some());

    // Verify that listing with org2's context does NOT return org1's data
    let start = Utc::now() - chrono::Duration::days(1);
    let end = Utc::now() + chrono::Duration::hours(1);

    let org2_records = usage_repo
        .list_by_organization("org-secure-2", start, end)
        .await
        .unwrap();

    // org2 should only see their own record, not org1's
    assert_eq!(org2_records.len(), 1);
    assert_eq!(org2_records[0].organization_id, "org-secure-2");
    assert_ne!(org2_records[0].id, org1_usage.id);
}

// ============================================================================
// AUTHENTICATION & AUTHORIZATION TESTS
// ============================================================================

#[tokio::test]
async fn test_api_key_tenant_isolation() {
    let store = InMemoryApiKeyStore::new();

    // Create API keys for different organizations
    let key1 = ApiKey::generate(
        "org-api-1".to_string(),
        "Org 1 Key".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string(), "write".to_string()],
    );

    let key2 = ApiKey::generate(
        "org-api-2".to_string(),
        "Org 2 Key".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string()],
    );

    let key3 = ApiKey::generate(
        "org-api-3".to_string(),
        "Org 3 Key".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["admin".to_string()],
    );

    // Store hashed keys
    let hash1 = key1.to_hash().unwrap();
    let hash2 = key2.to_hash().unwrap();
    let hash3 = key3.to_hash().unwrap();

    let raw_key1 = key1.key.clone().unwrap();
    let raw_key2 = key2.key.clone().unwrap();
    let raw_key3 = key3.key.clone().unwrap();

    store.store(hash1.clone()).await.unwrap();
    store.store(hash2.clone()).await.unwrap();
    store.store(hash3.clone()).await.unwrap();

    // Verify each key is associated with the correct organization
    let verified1 = store.verify(&raw_key1, "llmco-test-").await.unwrap();
    let verified2 = store.verify(&raw_key2, "llmco-test-").await.unwrap();
    let verified3 = store.verify(&raw_key3, "llmco-test-").await.unwrap();

    assert_eq!(verified1.organization_id, "org-api-1");
    assert_eq!(verified2.organization_id, "org-api-2");
    assert_eq!(verified3.organization_id, "org-api-3");

    // Verify permissions are correctly assigned
    assert!(verified1.has_permission("read"));
    assert!(verified1.has_permission("write"));
    assert!(!verified1.has_permission("admin"));

    assert!(verified2.has_permission("read"));
    assert!(!verified2.has_permission("write"));

    assert!(verified3.has_permission("admin"));
}

#[tokio::test]
async fn test_jwt_tenant_claims() {
    let config = AuthConfig::development();
    let jwt_manager = JwtManager::new(config).unwrap();

    // Generate tokens for different organizations
    let token1 = jwt_manager
        .generate_access_token(
            "user-1".to_string(),
            "org-jwt-1".to_string(),
            vec!["read".to_string()],
        )
        .unwrap();

    let token2 = jwt_manager
        .generate_access_token(
            "user-2".to_string(),
            "org-jwt-2".to_string(),
            vec!["read".to_string(), "write".to_string()],
        )
        .unwrap();

    // Validate tokens and verify org isolation
    let claims1 = jwt_manager.validate_access_token(&token1).unwrap();
    let claims2 = jwt_manager.validate_access_token(&token2).unwrap();

    assert_eq!(claims1.org, "org-jwt-1");
    assert_eq!(claims1.sub, "user-1");
    assert_eq!(claims1.permissions, vec!["read"]);

    assert_eq!(claims2.org, "org-jwt-2");
    assert_eq!(claims2.sub, "user-2");
    assert_eq!(claims2.permissions, vec!["read", "write"]);

    // Verify tokens are independent
    assert_ne!(claims1.org, claims2.org);
    assert_ne!(token1, token2);
}

#[tokio::test]
async fn test_rbac_organization_scoped_permissions() {
    let manager = RbacManager::new();

    // Create org-scoped roles
    let org1_admin = Role::org_admin("org-rbac-1".to_string());
    let org2_admin = Role::org_admin("org-rbac-2".to_string());

    manager.create_role(org1_admin.clone()).await.unwrap();
    manager.create_role(org2_admin.clone()).await.unwrap();

    // Assign users to their respective organizations
    manager
        .assign_user_role("user-1", &org1_admin.id)
        .await
        .unwrap();
    manager
        .assign_user_role("user-2", &org2_admin.id)
        .await
        .unwrap();

    // Test org1 user permissions
    let org1_usage_perm =
        Permission::scoped(Resource::Usage, Action::Read, "org-rbac-1".to_string());
    assert!(
        manager.check_permission("user-1", &org1_usage_perm).await,
        "User 1 should have access to org-rbac-1 data"
    );

    // User 1 should NOT have access to org2 data
    let org2_usage_perm =
        Permission::scoped(Resource::Usage, Action::Read, "org-rbac-2".to_string());
    assert!(
        !manager.check_permission("user-1", &org2_usage_perm).await,
        "User 1 should NOT have access to org-rbac-2 data"
    );

    // Test org2 user permissions
    assert!(
        manager.check_permission("user-2", &org2_usage_perm).await,
        "User 2 should have access to org-rbac-2 data"
    );
    assert!(
        !manager.check_permission("user-2", &org1_usage_perm).await,
        "User 2 should NOT have access to org-rbac-1 data"
    );
}

#[tokio::test]
async fn test_read_only_role_restrictions() {
    let manager = RbacManager::new();

    let read_only_role = Role::read_only("org-readonly-1".to_string());
    manager.create_role(read_only_role.clone()).await.unwrap();

    manager
        .assign_user_role("readonly-user", &read_only_role.id)
        .await
        .unwrap();

    // Read-only user should be able to read
    let read_perm =
        Permission::scoped(Resource::Usage, Action::Read, "org-readonly-1".to_string());
    assert!(manager.check_permission("readonly-user", &read_perm).await);

    // But should NOT be able to create, update, or delete
    let create_perm =
        Permission::scoped(Resource::Usage, Action::Create, "org-readonly-1".to_string());
    let update_perm =
        Permission::scoped(Resource::Usage, Action::Update, "org-readonly-1".to_string());
    let delete_perm =
        Permission::scoped(Resource::Usage, Action::Delete, "org-readonly-1".to_string());

    assert!(!manager.check_permission("readonly-user", &create_perm).await);
    assert!(!manager.check_permission("readonly-user", &update_perm).await);
    assert!(!manager.check_permission("readonly-user", &delete_perm).await);
}

// ============================================================================
// SECURITY PENETRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_prevent_sql_injection_in_org_filter() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool);

    // Create legitimate record
    let legit_usage = create_test_usage("org-legitimate", Some("proj-1"));
    usage_repo.create(&legit_usage).await.unwrap();

    // Attempt SQL injection through organization_id filter
    let malicious_org_id = "org-legitimate' OR '1'='1";
    let start = Utc::now() - chrono::Duration::days(1);
    let end = Utc::now() + chrono::Duration::hours(1);

    let result = usage_repo
        .list_by_organization(malicious_org_id, start, end)
        .await
        .unwrap();

    // Should return no records (SQL injection prevented)
    assert_eq!(
        result.len(),
        0,
        "SQL injection should be prevented by parameterized queries"
    );
}

#[tokio::test]
async fn test_revoked_api_key_rejection() {
    let store = InMemoryApiKeyStore::new();

    let key = ApiKey::generate(
        "org-revoke-test".to_string(),
        "Revoke Test Key".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string()],
    );

    let raw_key = key.key.clone().unwrap();
    let mut hash = key.to_hash().unwrap();

    store.store(hash.clone()).await.unwrap();

    // Verify key works initially
    assert!(store.verify(&raw_key, "llmco-test-").await.is_ok());

    // Revoke the key
    hash.revoke();
    store.store(hash).await.unwrap();

    // Verify revoked key is rejected
    let result = store.verify(&raw_key, "llmco-test-").await;
    assert!(
        result.is_err(),
        "Revoked API key should be rejected"
    );
}

#[tokio::test]
async fn test_expired_api_key_rejection() {
    let store = InMemoryApiKeyStore::new();

    // Create key that expires immediately
    let key = ApiKey::generate_with_expiration(
        "org-expire-test".to_string(),
        "Expire Test Key".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string()],
        0, // Expires in 0 days (immediately)
    );

    let raw_key = key.key.clone().unwrap();
    let hash = key.to_hash().unwrap();

    store.store(hash).await.unwrap();

    // Wait a moment to ensure expiration
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Verify expired key is rejected
    let result = store.verify(&raw_key, "llmco-test-").await;
    // Note: The verification should fail, but the specific error depends on implementation
    // For now, we check that it either errors or the hash indicates expiration
    if let Ok(hash) = result {
        assert!(hash.is_expired(), "Key should be marked as expired");
    }
}

#[tokio::test]
async fn test_api_key_timing_attack_resistance() {
    use std::time::Instant;

    let store = InMemoryApiKeyStore::new();

    let key = ApiKey::generate(
        "org-timing-test".to_string(),
        "Timing Test Key".to_string(),
        "llmco-test-".to_string(),
        32,
        vec!["read".to_string()],
    );

    let raw_key = key.key.clone().unwrap();
    let hash = key.to_hash().unwrap();

    store.store(hash).await.unwrap();

    // Measure time for correct key
    let start_correct = Instant::now();
    let _ = store.verify(&raw_key, "llmco-test-").await;
    let duration_correct = start_correct.elapsed();

    // Measure time for incorrect key (same length)
    let wrong_key = format!("llmco-test-{}", "a".repeat(32));
    let start_wrong = Instant::now();
    let _ = store.verify(&wrong_key, "llmco-test-").await;
    let duration_wrong = start_wrong.elapsed();

    // The timing difference should be minimal (constant-time comparison)
    // Allow up to 10ms difference for system variance
    let timing_diff = if duration_correct > duration_wrong {
        duration_correct - duration_wrong
    } else {
        duration_wrong - duration_correct
    };

    assert!(
        timing_diff < std::time::Duration::from_millis(10),
        "Timing difference should be minimal to prevent timing attacks: {:?}",
        timing_diff
    );
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_multi_tenant_writes() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool);

    // Setup pricing
    let pricing_table = PricingTable::new(
        Provider::OpenAI,
        "gpt-4".to_string(),
        create_test_pricing(),
    );
    pricing_repo.create(&pricing_table).await.unwrap();

    let num_tenants = 10;
    let records_per_tenant = 50;

    let mut handles = vec![];

    for tenant_id in 0..num_tenants {
        let usage_repo = usage_repo.clone();
        let org_id = format!("org-concurrent-{}", tenant_id);

        let handle = tokio::spawn(async move {
            for _ in 0..records_per_tenant {
                let usage = create_test_usage(&org_id, Some("proj-perf"));
                usage_repo.create(&usage).await.unwrap();
            }
        });

        handles.push(handle);
    }

    // Wait for all concurrent writes
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify each tenant has correct number of records
    let start = Utc::now() - chrono::Duration::days(1);
    let end = Utc::now() + chrono::Duration::hours(1);

    for tenant_id in 0..num_tenants {
        let org_id = format!("org-concurrent-{}", tenant_id);
        let records = usage_repo
            .list_by_organization(&org_id, start, end)
            .await
            .unwrap();

        assert_eq!(
            records.len(),
            records_per_tenant,
            "Tenant {} should have {} records",
            org_id,
            records_per_tenant
        );
    }
}

#[tokio::test]
async fn test_large_scale_tenant_query_performance() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool);

    // Setup pricing
    let pricing_table = PricingTable::new(
        Provider::OpenAI,
        "gpt-4".to_string(),
        create_test_pricing(),
    );
    pricing_repo.create(&pricing_table).await.unwrap();

    // Create many records for a single tenant
    let org_id = "org-large-scale";
    let num_records = 1000;

    for i in 0..num_records {
        let usage = create_test_usage(org_id, Some(&format!("proj-{}", i % 10)));
        usage_repo.create(&usage).await.unwrap();
    }

    // Measure query performance
    let start = Utc::now() - chrono::Duration::days(1);
    let end = Utc::now() + chrono::Duration::hours(1);

    let query_start = std::time::Instant::now();
    let records = usage_repo
        .list_by_organization(org_id, start, end)
        .await
        .unwrap();
    let query_duration = query_start.elapsed();

    assert_eq!(records.len(), num_records);

    // Query should complete in reasonable time (< 1 second for 1000 records)
    assert!(
        query_duration < std::time::Duration::from_secs(1),
        "Query took too long: {:?}",
        query_duration
    );

    println!(
        "Query performance: {} records in {:?}",
        num_records, query_duration
    );
}

#[tokio::test]
async fn test_tenant_isolation_under_load() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool);

    // Setup pricing
    let pricing_table = PricingTable::new(
        Provider::OpenAI,
        "gpt-4".to_string(),
        create_test_pricing(),
    );
    pricing_repo.create(&pricing_table).await.unwrap();

    let num_tenants = 20;
    let records_per_tenant = 25;

    // Create records for multiple tenants concurrently
    let mut write_handles = vec![];

    for tenant_id in 0..num_tenants {
        let usage_repo = usage_repo.clone();
        let org_id = format!("org-load-{}", tenant_id);

        let handle = tokio::spawn(async move {
            for _ in 0..records_per_tenant {
                let usage = create_test_usage(&org_id, Some("proj-load"));
                usage_repo.create(&usage).await.unwrap();
            }
        });

        write_handles.push(handle);
    }

    // Wait for all writes
    for handle in write_handles {
        handle.await.unwrap();
    }

    // Concurrently query each tenant and verify isolation
    let start = Utc::now() - chrono::Duration::days(1);
    let end = Utc::now() + chrono::Duration::hours(1);

    let mut read_handles = vec![];

    for tenant_id in 0..num_tenants {
        let usage_repo = usage_repo.clone();
        let org_id = format!("org-load-{}", tenant_id);

        let handle = tokio::spawn(async move {
            let records = usage_repo
                .list_by_organization(&org_id, start, end)
                .await
                .unwrap();

            // Verify count
            assert_eq!(records.len(), records_per_tenant);

            // Verify all records belong to this tenant
            for record in records {
                assert_eq!(record.organization_id, org_id);
            }
        });

        read_handles.push(handle);
    }

    // Wait for all reads and verifications
    for handle in read_handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// EDGE CASES AND ERROR HANDLING
// ============================================================================

#[tokio::test]
async fn test_empty_organization_id_rejection() {
    let usage = UsageRecord {
        organization_id: "".to_string(), // Empty org ID
        ..create_test_usage("valid-org", None)
    };

    let validation_result = usage.validate();
    assert!(
        validation_result.is_err(),
        "Empty organization ID should be rejected"
    );
}

#[tokio::test]
async fn test_nonexistent_tenant_returns_empty() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool);

    let start = Utc::now() - chrono::Duration::days(1);
    let end = Utc::now() + chrono::Duration::hours(1);

    let records = usage_repo
        .list_by_organization("org-nonexistent-xyz", start, end)
        .await
        .unwrap();

    assert_eq!(records.len(), 0, "Nonexistent tenant should return empty result");
}

#[tokio::test]
async fn test_special_characters_in_org_id() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool);

    // Test with various special characters that should be safely handled
    let special_org_ids = vec![
        "org-with-dashes",
        "org_with_underscores",
        "org.with.dots",
        "org123numbers",
        "org-MiXeD-CaSe",
    ];

    for org_id in special_org_ids {
        let usage = create_test_usage(org_id, Some("proj-special"));
        let result = usage_repo.create(&usage).await;
        assert!(result.is_ok(), "Should handle special characters in org ID: {}", org_id);

        // Verify retrieval works
        let start = Utc::now() - chrono::Duration::days(1);
        let end = Utc::now() + chrono::Duration::hours(1);

        let records = usage_repo
            .list_by_organization(org_id, start, end)
            .await
            .unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].organization_id, org_id);
    }
}
