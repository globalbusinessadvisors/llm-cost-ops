use chrono::Utc;
use llm_cost_ops::{
    domain::{
        CostRecord, Currency, ModelIdentifier, PricingStructure, PricingTable, Provider,
        UsageRecord,
    },
    storage::{CostRepository, PricingRepository, SqliteCostRepository, SqlitePricingRepository, SqliteUsageRepository, UsageRepository},
};
use rust_decimal::Decimal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use uuid::Uuid;

async fn setup_test_db() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
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
        tags: vec!["production".to_string(), "api".to_string()],
        metadata: serde_json::json!({"request_id": "req-123"}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "api".to_string(),
            endpoint: Some("https://api.example.com".to_string()),
        },
    }
}

#[tokio::test]
async fn test_usage_repository_create_and_get() {
    let pool = setup_test_db().await;
    let repo = SqliteUsageRepository::new(pool);

    let record = create_test_usage_record();
    let record_id = record.id;

    // Create
    repo.create(&record).await.expect("Failed to create record");

    // Get by ID
    let retrieved = repo
        .get_by_id(record_id)
        .await
        .expect("Failed to get record")
        .expect("Record not found");

    assert_eq!(retrieved.id, record_id);
    assert_eq!(retrieved.provider, Provider::OpenAI);
    assert_eq!(retrieved.model.name, "gpt-4");
    assert_eq!(retrieved.organization_id, "org-test");
    assert_eq!(retrieved.prompt_tokens, 1000);
    assert_eq!(retrieved.completion_tokens, 500);
}

#[tokio::test]
async fn test_usage_repository_list_by_organization() {
    let pool = setup_test_db().await;
    let repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();
    let start = now - chrono::Duration::days(7);
    let end = now;

    // Create multiple records
    let mut record1 = create_test_usage_record();
    record1.organization_id = "org-1".to_string();
    record1.timestamp = now - chrono::Duration::days(1);

    let mut record2 = create_test_usage_record();
    record2.organization_id = "org-1".to_string();
    record2.timestamp = now - chrono::Duration::days(2);

    let mut record3 = create_test_usage_record();
    record3.organization_id = "org-2".to_string();
    record3.timestamp = now - chrono::Duration::days(3);

    repo.create(&record1).await.expect("Failed to create record1");
    repo.create(&record2).await.expect("Failed to create record2");
    repo.create(&record3).await.expect("Failed to create record3");

    // List by organization
    let org1_records = repo
        .list_by_organization("org-1", start, end)
        .await
        .expect("Failed to list records");

    assert_eq!(org1_records.len(), 2);
    assert!(org1_records
        .iter()
        .all(|r| r.organization_id == "org-1"));

    let org2_records = repo
        .list_by_organization("org-2", start, end)
        .await
        .expect("Failed to list records");

    assert_eq!(org2_records.len(), 1);
    assert_eq!(org2_records[0].organization_id, "org-2");
}

#[tokio::test]
async fn test_pricing_repository_create_and_get() {
    let pool = setup_test_db().await;
    let repo = SqlitePricingRepository::new(pool);

    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    let table = PricingTable::new(Provider::OpenAI, "gpt-4-test".to_string(), pricing);
    let table_id = table.id;

    // Create
    repo.create(&table).await.expect("Failed to create pricing table");

    // Get by ID
    let retrieved = repo
        .get_by_id(table_id)
        .await
        .expect("Failed to get pricing table")
        .expect("Pricing table not found");

    assert_eq!(retrieved.id, table_id);
    assert_eq!(retrieved.provider, Provider::OpenAI);
    assert_eq!(retrieved.model, "gpt-4-test");
    assert_eq!(retrieved.currency.as_str(), "USD");
}

#[tokio::test]
async fn test_pricing_repository_get_active() {
    let pool = setup_test_db().await;
    let repo = SqlitePricingRepository::new(pool);

    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    let mut table = PricingTable::new(Provider::OpenAI, "gpt-4-active".to_string(), pricing);
    table.effective_date = Utc::now() - chrono::Duration::days(30);
    table.end_date = Some(Utc::now() + chrono::Duration::days(30));

    repo.create(&table).await.expect("Failed to create pricing table");

    // Get active pricing
    let active = repo
        .get_active(&Provider::OpenAI, "gpt-4-active", &Utc::now())
        .await
        .expect("Failed to get active pricing")
        .expect("Active pricing not found");

    assert_eq!(active.id, table.id);
    assert_eq!(active.provider, Provider::OpenAI);
    assert_eq!(active.model, "gpt-4-active");
}

#[tokio::test]
async fn test_pricing_repository_list_all() {
    let pool = setup_test_db().await;
    let repo = SqlitePricingRepository::new(pool);

    let pricing1 = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    let pricing2 = PricingStructure::simple_per_token(
        Decimal::from_str("3.0").unwrap(),
        Decimal::from_str("15.0").unwrap(),
    );

    let table1 = PricingTable::new(Provider::OpenAI, "gpt-4-list".to_string(), pricing1);
    let table2 = PricingTable::new(Provider::Anthropic, "claude-3-sonnet".to_string(), pricing2);

    repo.create(&table1).await.expect("Failed to create table1");
    repo.create(&table2).await.expect("Failed to create table2");

    // List all (should include defaults from migrations + our 2)
    let all_tables = repo.list_all().await.expect("Failed to list all tables");

    // Should have at least our 2 tables (migrations may add more defaults)
    assert!(all_tables.len() >= 2);
    assert!(all_tables.iter().any(|t| t.model == "gpt-4-list"));
    assert!(all_tables.iter().any(|t| t.model == "claude-3-sonnet"));
}

#[tokio::test]
async fn test_cost_repository_create_and_get() {
    let pool = setup_test_db().await;
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let usage_repo = SqliteUsageRepository::new(pool);

    // Create usage record first
    let usage = create_test_usage_record();
    usage_repo
        .create(&usage)
        .await
        .expect("Failed to create usage record");

    // Create cost record
    let cost = CostRecord {
        id: Uuid::new_v4(),
        usage_id: usage.id,
        provider: Provider::OpenAI,
        model: "gpt-4".to_string(),
        input_cost: Decimal::from_str("0.01").unwrap(),
        output_cost: Decimal::from_str("0.015").unwrap(),
        total_cost: Decimal::from_str("0.025").unwrap(),
        currency: Currency::USD,
        timestamp: Utc::now(),
        organization_id: Some("org-test".to_string()),
        project_id: Some("proj-test".to_string()),
    };

    let cost_id = cost.id;

    cost_repo.create(&cost).await.expect("Failed to create cost record");

    // Get by usage ID
    let retrieved = cost_repo
        .get_by_usage_id(usage.id)
        .await
        .expect("Failed to get cost record")
        .expect("Cost record not found");

    assert_eq!(retrieved.id, cost_id);
    assert_eq!(retrieved.usage_id, usage.id);
    assert_eq!(retrieved.total_cost, Decimal::from_str("0.025").unwrap());
}

#[tokio::test]
async fn test_cost_repository_list_by_organization() {
    let pool = setup_test_db().await;
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let usage_repo = SqliteUsageRepository::new(pool);

    let now = Utc::now();
    let start = now - chrono::Duration::days(7);
    let end = now;

    // Create usage records
    let mut usage1 = create_test_usage_record();
    usage1.organization_id = "org-cost-1".to_string();
    usage1.timestamp = now - chrono::Duration::days(1);

    let mut usage2 = create_test_usage_record();
    usage2.organization_id = "org-cost-1".to_string();
    usage2.timestamp = now - chrono::Duration::days(2);

    usage_repo.create(&usage1).await.expect("Failed to create usage1");
    usage_repo.create(&usage2).await.expect("Failed to create usage2");

    // Create cost records
    let cost1 = CostRecord {
        id: Uuid::new_v4(),
        usage_id: usage1.id,
        provider: Provider::OpenAI,
        model: "gpt-4".to_string(),
        input_cost: Decimal::from_str("0.01").unwrap(),
        output_cost: Decimal::from_str("0.02").unwrap(),
        total_cost: Decimal::from_str("0.03").unwrap(),
        currency: Currency::USD,
        timestamp: usage1.timestamp,
        organization_id: Some("org-cost-1".to_string()),
        project_id: Some("proj-test".to_string()),
    };

    let cost2 = CostRecord {
        id: Uuid::new_v4(),
        usage_id: usage2.id,
        provider: Provider::Anthropic,
        model: "claude-3-sonnet".to_string(),
        input_cost: Decimal::from_str("0.005").unwrap(),
        output_cost: Decimal::from_str("0.015").unwrap(),
        total_cost: Decimal::from_str("0.02").unwrap(),
        currency: Currency::USD,
        timestamp: usage2.timestamp,
        organization_id: Some("org-cost-1".to_string()),
        project_id: Some("proj-test".to_string()),
    };

    cost_repo.create(&cost1).await.expect("Failed to create cost1");
    cost_repo.create(&cost2).await.expect("Failed to create cost2");

    // List by organization
    let records = cost_repo
        .list_by_organization("org-cost-1", start, end)
        .await
        .expect("Failed to list cost records");

    assert_eq!(records.len(), 2);
    assert!(records
        .iter()
        .all(|r| r.organization_id == Some("org-cost-1".to_string())));
}

#[tokio::test]
async fn test_full_ingestion_pipeline() {
    let pool = setup_test_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool.clone());
    let cost_repo = SqliteCostRepository::new(pool);

    // Create pricing table
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);
    pricing_repo
        .create(&pricing_table)
        .await
        .expect("Failed to create pricing table");

    // Create usage record
    let usage = create_test_usage_record();
    usage_repo
        .create(&usage)
        .await
        .expect("Failed to create usage record");

    // Calculate cost
    let calculator = llm_cost_ops::engine::CostCalculator::new();
    let cost = calculator
        .calculate(&usage, &pricing_table)
        .expect("Failed to calculate cost");

    // Store cost record
    cost_repo
        .create(&cost)
        .await
        .expect("Failed to create cost record");

    // Verify the full pipeline
    let retrieved_usage = usage_repo
        .get_by_id(usage.id)
        .await
        .expect("Failed to get usage")
        .expect("Usage not found");

    let retrieved_cost = cost_repo
        .get_by_usage_id(usage.id)
        .await
        .expect("Failed to get cost")
        .expect("Cost not found");

    assert_eq!(retrieved_usage.id, usage.id);
    assert_eq!(retrieved_cost.usage_id, usage.id);
    assert_eq!(
        retrieved_cost.total_cost,
        Decimal::from_str("0.025").unwrap()
    );
}
