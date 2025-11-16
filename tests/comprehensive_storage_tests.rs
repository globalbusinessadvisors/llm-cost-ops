use chrono::{Duration, Utc};
use llm_cost_ops::{
    domain::{
        CostCalculation, CostRecord, Currency, IngestionSource, ModelIdentifier, PricingStructure,
        PricingTable, Provider, UsageRecord,
    },
    storage::{
        CostRepository, DatabaseConfig, DatabasePool, DatabaseType, PricingRepository,
        SqliteCostRepository, SqlitePool, SqlitePricingRepository, SqliteUsageRepository,
        UsageRepository,
    },
};
use rust_decimal::Decimal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool as RawSqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use uuid::Uuid;

// ============================================================================
// Test Helpers and Setup
// ============================================================================

async fn setup_test_pool() -> RawSqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to create test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

fn create_test_usage_record() -> UsageRecord {
    UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: Some("gpt-4-0613".to_string()),
            context_window: Some(8192),
        },
        organization_id: "org-test".to_string(),
        project_id: Some("proj-test".to_string()),
        user_id: Some("user-test".to_string()),
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 1500,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: Some(2500),
        time_to_first_token_ms: Some(150),
        tags: vec!["production".to_string(), "api".to_string()],
        metadata: serde_json::json!({"request_id": "req-123"}),
        ingested_at: Utc::now(),
        source: IngestionSource::Api {
            endpoint: "https://api.example.com".to_string(),
        },
    }
}

fn create_test_pricing_table(provider: Provider, model: &str) -> PricingTable {
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );
    PricingTable::new(provider, model.to_string(), pricing)
}

fn create_test_cost_record(usage_id: Uuid, organization_id: &str) -> CostRecord {
    let calculation = CostCalculation::new(
        Decimal::from_str("0.01").unwrap(),
        Decimal::from_str("0.015").unwrap(),
        Currency::USD,
        Uuid::new_v4(),
    );

    CostRecord::new(
        usage_id,
        Provider::OpenAI,
        "gpt-4".to_string(),
        organization_id.to_string(),
        calculation,
        PricingStructure::simple_per_token(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
        ),
    )
}

// ============================================================================
// Database Configuration Tests
// ============================================================================

#[test]
fn test_database_config_default() {
    let config = DatabaseConfig::default();
    assert_eq!(config.database_type, DatabaseType::Sqlite);
    assert_eq!(config.url, "sqlite::memory:");
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.min_connections, 2);
    assert!(config.run_migrations);
}

#[test]
fn test_database_config_sqlite() {
    let config = DatabaseConfig::sqlite("test.db");
    assert_eq!(config.database_type, DatabaseType::Sqlite);
    assert_eq!(config.url, "sqlite://test.db");
}

#[test]
fn test_database_config_sqlite_memory() {
    let config = DatabaseConfig::sqlite_memory();
    assert_eq!(config.url, "sqlite::memory:");
}

#[test]
fn test_database_config_validation_empty_url() {
    let mut config = DatabaseConfig::default();
    config.url = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_database_config_validation_zero_max_connections() {
    let mut config = DatabaseConfig::default();
    config.max_connections = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_database_config_validation_min_exceeds_max() {
    let mut config = DatabaseConfig::default();
    config.min_connections = 20;
    config.max_connections = 10;
    assert!(config.validate().is_err());
}

#[test]
fn test_database_config_validation_valid() {
    let config = DatabaseConfig::default();
    assert!(config.validate().is_ok());
}

// ============================================================================
// Connection Pool Tests
// ============================================================================

#[tokio::test]
async fn test_sqlite_pool_creation() {
    let mut config = DatabaseConfig::sqlite_memory();
    config.run_migrations = false;

    let pool = SqlitePool::new(&config).await;
    assert!(pool.is_ok());
}

#[tokio::test]
async fn test_sqlite_pool_health_check() {
    let mut config = DatabaseConfig::sqlite_memory();
    config.run_migrations = false;

    let pool = SqlitePool::new(&config).await.unwrap();
    assert!(pool.health_check().await.is_ok());
}

#[tokio::test]
async fn test_sqlite_pool_stats() {
    let mut config = DatabaseConfig::sqlite_memory();
    config.run_migrations = false;

    let pool = SqlitePool::new(&config).await.unwrap();
    let stats = pool.stats();

    assert!(stats.connections > 0);
    assert!(stats.idle_connections >= 0);
}

#[tokio::test]
async fn test_sqlite_pool_close() {
    let mut config = DatabaseConfig::sqlite_memory();
    config.run_migrations = false;

    let pool = SqlitePool::new(&config).await.unwrap();
    pool.close().await;
    // Pool should be closed, health check should fail
    assert!(pool.health_check().await.is_err());
}

#[tokio::test]
async fn test_database_pool_enum_creation() {
    let config = DatabaseConfig::sqlite_memory();
    let pool = DatabasePool::new(&config).await;
    assert!(pool.is_ok());

    let pool = pool.unwrap();
    assert_eq!(pool.database_type(), DatabaseType::Sqlite);
}

#[tokio::test]
async fn test_database_pool_health_check() {
    let config = DatabaseConfig::sqlite_memory();
    let pool = DatabasePool::new(&config).await.unwrap();
    assert!(pool.health_check().await.is_ok());
}

#[tokio::test]
async fn test_database_pool_stats() {
    let config = DatabaseConfig::sqlite_memory();
    let pool = DatabasePool::new(&config).await.unwrap();
    let stats = pool.stats();

    assert!(stats.connections > 0);
}

// ============================================================================
// UsageRepository Tests
// ============================================================================

#[tokio::test]
async fn test_usage_repository_create() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let record = create_test_usage_record();
    let result = repo.create(&record).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_usage_repository_get_by_id() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let record = create_test_usage_record();
    let record_id = record.id;

    repo.create(&record).await.unwrap();

    let retrieved = repo.get_by_id(record_id).await.unwrap();
    assert!(retrieved.is_some());

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, record_id);
    assert_eq!(retrieved.provider, Provider::OpenAI);
    assert_eq!(retrieved.model.name, "gpt-4");
}

#[tokio::test]
async fn test_usage_repository_get_by_id_not_found() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let non_existent_id = Uuid::new_v4();
    let result = repo.get_by_id(non_existent_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_usage_repository_list_by_organization() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    // Create records for org-1
    let mut record1 = create_test_usage_record();
    record1.organization_id = "org-1".to_string();
    record1.timestamp = now - Duration::days(1);

    let mut record2 = create_test_usage_record();
    record2.organization_id = "org-1".to_string();
    record2.timestamp = now - Duration::days(2);

    // Create record for org-2
    let mut record3 = create_test_usage_record();
    record3.organization_id = "org-2".to_string();
    record3.timestamp = now - Duration::days(3);

    repo.create(&record1).await.unwrap();
    repo.create(&record2).await.unwrap();
    repo.create(&record3).await.unwrap();

    let org1_records = repo.list_by_organization("org-1", start, end).await.unwrap();
    assert_eq!(org1_records.len(), 2);
    assert!(org1_records.iter().all(|r| r.organization_id == "org-1"));
}

#[tokio::test]
async fn test_usage_repository_list_empty_organization() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    let records = repo.list_by_organization("non-existent", start, end).await.unwrap();
    assert_eq!(records.len(), 0);
}

#[tokio::test]
async fn test_usage_repository_create_with_optional_fields() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let mut record = create_test_usage_record();
    record.cached_tokens = Some(200);
    record.reasoning_tokens = Some(100);
    record.project_id = None;
    record.user_id = None;

    repo.create(&record).await.unwrap();

    let retrieved = repo.get_by_id(record.id).await.unwrap().unwrap();
    assert_eq!(retrieved.cached_tokens, Some(200));
    assert_eq!(retrieved.reasoning_tokens, Some(100));
}

#[tokio::test]
async fn test_usage_repository_multiple_providers() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    let mut openai_record = create_test_usage_record();
    openai_record.provider = Provider::OpenAI;
    openai_record.organization_id = "org-multi".to_string();

    let mut anthropic_record = create_test_usage_record();
    anthropic_record.provider = Provider::Anthropic;
    anthropic_record.organization_id = "org-multi".to_string();

    repo.create(&openai_record).await.unwrap();
    repo.create(&anthropic_record).await.unwrap();

    let records = repo.list_by_organization("org-multi", start, end).await.unwrap();
    assert_eq!(records.len(), 2);
}

#[tokio::test]
async fn test_usage_repository_date_range_filtering() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();

    let mut old_record = create_test_usage_record();
    old_record.timestamp = now - Duration::days(30);
    old_record.organization_id = "org-date".to_string();

    let mut new_record = create_test_usage_record();
    new_record.timestamp = now - Duration::days(1);
    new_record.organization_id = "org-date".to_string();

    repo.create(&old_record).await.unwrap();
    repo.create(&new_record).await.unwrap();

    // Query only last 7 days
    let start = now - Duration::days(7);
    let end = now;
    let records = repo.list_by_organization("org-date", start, end).await.unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, new_record.id);
}

// ============================================================================
// PricingRepository Tests
// ============================================================================

#[tokio::test]
async fn test_pricing_repository_create() {
    let pool = setup_test_pool().await;
    let repo = SqlitePricingRepository::new(pool);

    let table = create_test_pricing_table(Provider::OpenAI, "gpt-4");
    let result = repo.create(&table).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pricing_repository_get_active() {
    let pool = setup_test_pool().await;
    let repo = SqlitePricingRepository::new(pool);

    let mut table = create_test_pricing_table(Provider::OpenAI, "gpt-4-active");
    table.effective_date = Utc::now() - Duration::days(30);
    table.end_date = Some(Utc::now() + Duration::days(30));

    repo.create(&table).await.unwrap();

    let active = repo
        .get_active(&Provider::OpenAI, "gpt-4-active", &Utc::now())
        .await
        .unwrap();

    assert!(active.is_some());
    let active = active.unwrap();
    assert_eq!(active.id, table.id);
    assert_eq!(active.model, "gpt-4-active");
}

#[tokio::test]
async fn test_pricing_repository_get_active_not_yet_effective() {
    let pool = setup_test_pool().await;
    let repo = SqlitePricingRepository::new(pool);

    let mut table = create_test_pricing_table(Provider::OpenAI, "gpt-4-future");
    table.effective_date = Utc::now() + Duration::days(10);

    repo.create(&table).await.unwrap();

    let active = repo
        .get_active(&Provider::OpenAI, "gpt-4-future", &Utc::now())
        .await
        .unwrap();

    assert!(active.is_none());
}

#[tokio::test]
async fn test_pricing_repository_get_active_expired() {
    let pool = setup_test_pool().await;
    let repo = SqlitePricingRepository::new(pool);

    let mut table = create_test_pricing_table(Provider::OpenAI, "gpt-4-expired");
    table.effective_date = Utc::now() - Duration::days(60);
    table.end_date = Some(Utc::now() - Duration::days(30));

    repo.create(&table).await.unwrap();

    let active = repo
        .get_active(&Provider::OpenAI, "gpt-4-expired", &Utc::now())
        .await
        .unwrap();

    assert!(active.is_none());
}

#[tokio::test]
async fn test_pricing_repository_list_all() {
    let pool = setup_test_pool().await;
    let repo = SqlitePricingRepository::new(pool);

    let table1 = create_test_pricing_table(Provider::OpenAI, "gpt-4-list");
    let table2 = create_test_pricing_table(Provider::Anthropic, "claude-3-sonnet");

    repo.create(&table1).await.unwrap();
    repo.create(&table2).await.unwrap();

    let all_tables = repo.list_all().await.unwrap();

    // Should include our 2 tables plus any default tables from migrations
    assert!(all_tables.len() >= 2);
    assert!(all_tables.iter().any(|t| t.model == "gpt-4-list"));
    assert!(all_tables.iter().any(|t| t.model == "claude-3-sonnet"));
}

#[tokio::test]
async fn test_pricing_repository_multiple_versions() {
    let pool = setup_test_pool().await;
    let repo = SqlitePricingRepository::new(pool);

    // Old pricing
    let mut old_table = create_test_pricing_table(Provider::OpenAI, "gpt-4-versions");
    old_table.effective_date = Utc::now() - Duration::days(60);
    old_table.end_date = Some(Utc::now() - Duration::days(30));

    // Current pricing
    let mut current_table = create_test_pricing_table(Provider::OpenAI, "gpt-4-versions");
    current_table.effective_date = Utc::now() - Duration::days(30);

    repo.create(&old_table).await.unwrap();
    repo.create(&current_table).await.unwrap();

    let active = repo
        .get_active(&Provider::OpenAI, "gpt-4-versions", &Utc::now())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(active.id, current_table.id);
}

#[tokio::test]
async fn test_pricing_repository_different_regions() {
    let pool = setup_test_pool().await;
    let repo = SqlitePricingRepository::new(pool);

    let us_table = create_test_pricing_table(Provider::OpenAI, "gpt-4-regional")
        .with_region("us-east-1".to_string());

    let eu_table = create_test_pricing_table(Provider::OpenAI, "gpt-4-regional")
        .with_region("eu-west-1".to_string());

    repo.create(&us_table).await.unwrap();
    repo.create(&eu_table).await.unwrap();

    let all_tables = repo.list_all().await.unwrap();
    let regional_tables: Vec<_> = all_tables
        .iter()
        .filter(|t| t.model == "gpt-4-regional")
        .collect();

    assert_eq!(regional_tables.len(), 2);
}

// ============================================================================
// CostRepository Tests
// ============================================================================

#[tokio::test]
async fn test_cost_repository_create() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let usage_repo = SqliteUsageRepository::new(pool);

    let usage = create_test_usage_record();
    usage_repo.create(&usage).await.unwrap();

    let cost = create_test_cost_record(usage.id, "org-test");
    let result = cost_repo.create(&cost).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cost_repository_get_by_usage_id() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let usage_repo = SqliteUsageRepository::new(pool);

    let usage = create_test_usage_record();
    usage_repo.create(&usage).await.unwrap();

    let cost = create_test_cost_record(usage.id, "org-test");
    let cost_id = cost.id;
    cost_repo.create(&cost).await.unwrap();

    let retrieved = cost_repo.get_by_usage_id(usage.id).await.unwrap();
    assert!(retrieved.is_some());

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.id, cost_id);
    assert_eq!(retrieved.usage_id, usage.id);
}

#[tokio::test]
async fn test_cost_repository_get_by_usage_id_not_found() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool);

    let non_existent_id = Uuid::new_v4();
    let result = cost_repo.get_by_usage_id(non_existent_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_cost_repository_list_by_organization() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let usage_repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    // Create usage records
    let mut usage1 = create_test_usage_record();
    usage1.organization_id = "org-cost-1".to_string();
    usage1.timestamp = now - Duration::days(1);

    let mut usage2 = create_test_usage_record();
    usage2.organization_id = "org-cost-1".to_string();
    usage2.timestamp = now - Duration::days(2);

    usage_repo.create(&usage1).await.unwrap();
    usage_repo.create(&usage2).await.unwrap();

    // Create cost records
    let mut cost1 = create_test_cost_record(usage1.id, "org-cost-1");
    cost1.timestamp = usage1.timestamp;

    let mut cost2 = create_test_cost_record(usage2.id, "org-cost-1");
    cost2.timestamp = usage2.timestamp;

    cost_repo.create(&cost1).await.unwrap();
    cost_repo.create(&cost2).await.unwrap();

    let records = cost_repo
        .list_by_organization("org-cost-1", start, end)
        .await
        .unwrap();

    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|r| r.organization_id == "org-cost-1"));
}

#[tokio::test]
async fn test_cost_repository_list_empty_organization() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool);

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    let records = cost_repo
        .list_by_organization("non-existent", start, end)
        .await
        .unwrap();

    assert_eq!(records.len(), 0);
}

#[tokio::test]
async fn test_cost_repository_multiple_currencies() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let usage_repo = SqliteUsageRepository::new(pool);

    let usage1 = create_test_usage_record();
    let usage2 = create_test_usage_record();

    usage_repo.create(&usage1).await.unwrap();
    usage_repo.create(&usage2).await.unwrap();

    let mut cost_usd = create_test_cost_record(usage1.id, "org-multi-currency");
    cost_usd.currency = Currency::USD;

    let mut cost_eur = create_test_cost_record(usage2.id, "org-multi-currency");
    cost_eur.currency = Currency::EUR;

    cost_repo.create(&cost_usd).await.unwrap();
    cost_repo.create(&cost_eur).await.unwrap();

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    let records = cost_repo
        .list_by_organization("org-multi-currency", start, end)
        .await
        .unwrap();

    assert_eq!(records.len(), 2);
}

#[tokio::test]
async fn test_cost_repository_date_range_filtering() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let usage_repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();

    let mut old_usage = create_test_usage_record();
    old_usage.timestamp = now - Duration::days(30);
    old_usage.organization_id = "org-cost-date".to_string();

    let mut new_usage = create_test_usage_record();
    new_usage.timestamp = now - Duration::days(1);
    new_usage.organization_id = "org-cost-date".to_string();

    usage_repo.create(&old_usage).await.unwrap();
    usage_repo.create(&new_usage).await.unwrap();

    let mut old_cost = create_test_cost_record(old_usage.id, "org-cost-date");
    old_cost.timestamp = old_usage.timestamp;

    let mut new_cost = create_test_cost_record(new_usage.id, "org-cost-date");
    new_cost.timestamp = new_usage.timestamp;

    cost_repo.create(&old_cost).await.unwrap();
    cost_repo.create(&new_cost).await.unwrap();

    // Query only last 7 days
    let start = now - Duration::days(7);
    let end = now;
    let records = cost_repo
        .list_by_organization("org-cost-date", start, end)
        .await
        .unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, new_cost.id);
}

// ============================================================================
// Migration Tests
// ============================================================================

#[tokio::test]
async fn test_migrations_run_successfully() {
    let mut config = DatabaseConfig::sqlite_memory();
    config.run_migrations = true;

    let pool = SqlitePool::new(&config).await;
    assert!(pool.is_ok());
}

#[tokio::test]
async fn test_migrations_create_all_tables() {
    let pool = setup_test_pool().await;

    // Verify tables exist by attempting to query them
    let usage_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usage_records")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(usage_count, 0);

    let cost_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cost_records")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(cost_count, 0);

    let pricing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pricing_tables")
        .fetch_one(&pool)
        .await
        .unwrap();
    // Should have default pricing entries from migrations
    assert!(pricing_count > 0);
}

#[tokio::test]
async fn test_migrations_create_indexes() {
    let pool = setup_test_pool().await;

    // Check if indexes exist (SQLite specific)
    let index_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Should have multiple indexes created
    assert!(index_count > 0);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_cost_repository_foreign_key_constraint() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool);

    // Try to create cost record with non-existent usage_id
    let non_existent_usage_id = Uuid::new_v4();
    let cost = create_test_cost_record(non_existent_usage_id, "org-test");

    let result = cost_repo.create(&cost).await;
    // Should fail due to foreign key constraint
    assert!(result.is_err());
}

#[tokio::test]
async fn test_duplicate_usage_record_id() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let record = create_test_usage_record();
    repo.create(&record).await.unwrap();

    // Try to create the same record again
    let result = repo.create(&record).await;
    // Should fail due to primary key constraint
    assert!(result.is_err());
}

#[tokio::test]
async fn test_database_pool_with_invalid_config() {
    let mut config = DatabaseConfig::default();
    config.url = String::new(); // Invalid empty URL

    let result = DatabasePool::new(&config).await;
    assert!(result.is_err());
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_usage_record_creation() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let mut handles = vec![];

    for i in 0..10 {
        let repo_clone = repo.clone();
        let handle = tokio::spawn(async move {
            let mut record = create_test_usage_record();
            record.organization_id = format!("org-concurrent-{}", i);
            repo_clone.create(&record).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_reads() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    // Create a record
    let record = create_test_usage_record();
    repo.create(&record).await.unwrap();

    let mut handles = vec![];

    for _ in 0..20 {
        let repo_clone = repo.clone();
        let record_id = record.id;
        let handle = tokio::spawn(async move {
            repo_clone.get_by_id(record_id).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}

// ============================================================================
// Large Dataset Tests
// ============================================================================

#[tokio::test]
async fn test_bulk_usage_record_insertion() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    for i in 0..100 {
        let mut record = create_test_usage_record();
        record.organization_id = format!("org-bulk-{}", i % 10);
        repo.create(&record).await.unwrap();
    }

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    let records = repo.list_by_organization("org-bulk-0", start, end).await.unwrap();
    assert_eq!(records.len(), 10);
}

#[tokio::test]
async fn test_query_performance_with_indexes() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    // Insert 1000 records
    for i in 0..1000 {
        let mut record = create_test_usage_record();
        record.organization_id = "org-performance".to_string();
        record.timestamp = Utc::now() - Duration::seconds(i);
        repo.create(&record).await.unwrap();
    }

    let now = Utc::now();
    let start = now - Duration::days(1);
    let end = now;

    let start_time = std::time::Instant::now();
    let records = repo.list_by_organization("org-performance", start, end).await.unwrap();
    let elapsed = start_time.elapsed();

    assert!(records.len() > 0);
    // Query should complete in reasonable time (< 100ms)
    assert!(elapsed.as_millis() < 100);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_usage_record_with_zero_tokens() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let mut record = create_test_usage_record();
    record.prompt_tokens = 0;
    record.completion_tokens = 0;
    record.total_tokens = 0;

    // Should still be able to store it
    let result = repo.create(&record).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_usage_record_with_very_large_tokens() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let mut record = create_test_usage_record();
    record.prompt_tokens = u64::MAX / 2;
    record.completion_tokens = u64::MAX / 2;
    record.total_tokens = u64::MAX;

    // Should handle large values
    let result = repo.create(&record).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pricing_table_with_null_end_date() {
    let pool = setup_test_pool().await;
    let repo = SqlitePricingRepository::new(pool);

    let mut table = create_test_pricing_table(Provider::OpenAI, "gpt-4-no-end");
    table.end_date = None; // No expiration

    repo.create(&table).await.unwrap();

    let active = repo
        .get_active(&Provider::OpenAI, "gpt-4-no-end", &Utc::now())
        .await
        .unwrap();

    assert!(active.is_some());
}

#[tokio::test]
async fn test_cost_record_with_zero_cost() {
    let pool = setup_test_pool().await;
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let usage_repo = SqliteUsageRepository::new(pool);

    let usage = create_test_usage_record();
    usage_repo.create(&usage).await.unwrap();

    let calculation = CostCalculation::new(
        Decimal::ZERO,
        Decimal::ZERO,
        Currency::USD,
        Uuid::new_v4(),
    );

    let cost = CostRecord::new(
        usage.id,
        Provider::OpenAI,
        "free-model".to_string(),
        "org-test".to_string(),
        calculation,
        PricingStructure::simple_per_token(Decimal::ZERO, Decimal::ZERO),
    );

    let result = cost_repo.create(&cost).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_usage_record_with_empty_tags() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let mut record = create_test_usage_record();
    record.tags = vec![];

    repo.create(&record).await.unwrap();

    let retrieved = repo.get_by_id(record.id).await.unwrap().unwrap();
    assert_eq!(retrieved.tags.len(), 0);
}

#[tokio::test]
async fn test_usage_record_with_null_metadata() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let mut record = create_test_usage_record();
    record.metadata = serde_json::Value::Null;

    repo.create(&record).await.unwrap();

    let retrieved = repo.get_by_id(record.id).await.unwrap().unwrap();
    assert!(retrieved.metadata.is_null());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_cost_calculation_pipeline() {
    let pool = setup_test_pool().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool.clone());
    let cost_repo = SqliteCostRepository::new(pool);

    // 1. Create pricing table
    let pricing_table = create_test_pricing_table(Provider::OpenAI, "gpt-4");
    pricing_repo.create(&pricing_table).await.unwrap();

    // 2. Create usage record
    let usage = create_test_usage_record();
    usage_repo.create(&usage).await.unwrap();

    // 3. Create cost record
    let cost = create_test_cost_record(usage.id, &usage.organization_id);
    cost_repo.create(&cost).await.unwrap();

    // 4. Verify the complete pipeline
    let retrieved_usage = usage_repo.get_by_id(usage.id).await.unwrap().unwrap();
    let retrieved_cost = cost_repo.get_by_usage_id(usage.id).await.unwrap().unwrap();

    assert_eq!(retrieved_usage.id, usage.id);
    assert_eq!(retrieved_cost.usage_id, usage.id);
    assert_eq!(retrieved_cost.organization_id, usage.organization_id);
}

#[tokio::test]
async fn test_multi_organization_isolation() {
    let pool = setup_test_pool().await;
    let usage_repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    // Create records for different organizations
    for org_num in 1..=5 {
        for _ in 0..10 {
            let mut record = create_test_usage_record();
            record.organization_id = format!("org-{}", org_num);
            usage_repo.create(&record).await.unwrap();
        }
    }

    // Verify each organization only sees their own records
    for org_num in 1..=5 {
        let records = usage_repo
            .list_by_organization(&format!("org-{}", org_num), start, end)
            .await
            .unwrap();

        assert_eq!(records.len(), 10);
        assert!(records.iter().all(|r| r.organization_id == format!("org-{}", org_num)));
    }
}

#[tokio::test]
async fn test_ordering_by_timestamp() {
    let pool = setup_test_pool().await;
    let repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();
    let start = now - Duration::days(7);
    let end = now;

    // Create records with different timestamps
    for i in 0..5 {
        let mut record = create_test_usage_record();
        record.organization_id = "org-ordering".to_string();
        record.timestamp = now - Duration::days(i);
        repo.create(&record).await.unwrap();
    }

    let records = repo.list_by_organization("org-ordering", start, end).await.unwrap();

    // Should be ordered by timestamp descending
    for i in 1..records.len() {
        assert!(records[i-1].timestamp >= records[i].timestamp);
    }
}
