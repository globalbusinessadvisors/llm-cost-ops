use chrono::Utc;
use llm_cost_ops::{
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

async fn setup_integration_db() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .expect("Failed to create integration test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_end_to_end_openai_workflow() {
    let pool = setup_integration_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool.clone());
    let cost_repo = SqliteCostRepository::new(pool);

    // Step 1: Setup pricing (should already exist from migrations, but we'll add a custom one)
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4-integration".to_string(), pricing);
    pricing_repo
        .create(&pricing_table)
        .await
        .expect("Failed to create pricing");

    // Step 2: Create usage record (simulating API call)
    let usage = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4-integration".to_string(),
            version: Some("gpt-4-0613".to_string()),
            context_window: Some(8192),
        },
        organization_id: "org-integration".to_string(),
        project_id: Some("proj-e2e".to_string()),
        user_id: Some("user-test".to_string()),
        prompt_tokens: 1500,
        completion_tokens: 800,
        total_tokens: 2300,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: Some(3200),
        tags: vec!["production".to_string(), "e2e-test".to_string()],
        metadata: serde_json::json!({
            "request_id": "req-e2e-001",
            "endpoint": "/v1/chat/completions"
        }),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "api".to_string(),
            endpoint: Some("https://api.openai.com".to_string()),
        },
    };

    // Step 3: Validate and store usage
    usage.validate().expect("Usage validation failed");
    usage_repo
        .create(&usage)
        .await
        .expect("Failed to store usage");

    // Step 4: Get active pricing
    let active_pricing = pricing_repo
        .get_active(&Provider::OpenAI, "gpt-4-integration", &usage.timestamp)
        .await
        .expect("Failed to get pricing")
        .expect("No active pricing found");

    // Step 5: Calculate cost
    let calculator = CostCalculator::new();
    let cost = calculator
        .calculate(&usage, &active_pricing)
        .expect("Cost calculation failed");

    // Step 6: Store cost record
    cost_repo
        .create(&cost)
        .await
        .expect("Failed to store cost");

    // Step 7: Verify the complete pipeline
    let retrieved_usage = usage_repo
        .get_by_id(usage.id)
        .await
        .expect("Failed to retrieve usage")
        .expect("Usage not found");

    let retrieved_cost = cost_repo
        .get_by_usage_id(usage.id)
        .await
        .expect("Failed to retrieve cost")
        .expect("Cost not found");

    // Assertions
    assert_eq!(retrieved_usage.id, usage.id);
    assert_eq!(retrieved_usage.prompt_tokens, 1500);
    assert_eq!(retrieved_usage.completion_tokens, 800);

    assert_eq!(retrieved_cost.usage_id, usage.id);
    // Expected: (1500 * 10 / 1M) + (800 * 30 / 1M) = 0.015 + 0.024 = 0.039
    assert_eq!(
        retrieved_cost.total_cost,
        Decimal::from_str("0.0390000000").unwrap()
    );
    assert_eq!(retrieved_cost.currency.as_str(), "USD");
}

#[tokio::test]
async fn test_end_to_end_anthropic_with_caching() {
    let pool = setup_integration_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool.clone());
    let cost_repo = SqliteCostRepository::new(pool);

    // Setup pricing with cache discount
    let pricing = PricingStructure::with_cache_discount(
        Decimal::from_str("3.0").unwrap(),
        Decimal::from_str("15.0").unwrap(),
        Decimal::from_str("0.9").unwrap(), // 90% discount
    );

    let pricing_table = PricingTable::new(
        Provider::Anthropic,
        "claude-3-sonnet-integration".to_string(),
        pricing,
    );
    pricing_repo
        .create(&pricing_table)
        .await
        .expect("Failed to create pricing");

    // Create usage with cached tokens
    let usage = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::Anthropic,
        model: ModelIdentifier {
            name: "claude-3-sonnet-integration".to_string(),
            version: Some("20240229".to_string()),
            context_window: Some(200000),
        },
        organization_id: "org-integration".to_string(),
        project_id: Some("proj-e2e".to_string()),
        user_id: None,
        prompt_tokens: 5000,
        completion_tokens: 2000,
        total_tokens: 7000,
        cached_tokens: Some(2000), // 2000 tokens from cache
        reasoning_tokens: None,
        latency_ms: Some(4500),
        tags: vec!["production".to_string()],
        metadata: serde_json::json!({}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "api".to_string(),
            endpoint: Some("https://api.anthropic.com".to_string()),
        },
    };

    usage.validate().expect("Usage validation failed");
    usage_repo
        .create(&usage)
        .await
        .expect("Failed to store usage");

    let active_pricing = pricing_repo
        .get_active(
            &Provider::Anthropic,
            "claude-3-sonnet-integration",
            &usage.timestamp,
        )
        .await
        .expect("Failed to get pricing")
        .expect("No active pricing found");

    let calculator = CostCalculator::new();
    let cost = calculator
        .calculate(&usage, &active_pricing)
        .expect("Cost calculation failed");

    cost_repo
        .create(&cost)
        .await
        .expect("Failed to store cost");

    let retrieved_cost = cost_repo
        .get_by_usage_id(usage.id)
        .await
        .expect("Failed to retrieve cost")
        .expect("Cost not found");

    // Input: 5000 * 3 / 1M = 0.015
    // Cached discount: 2000 * 3 / 1M * 0.9 = 0.0054
    // Net input: 0.015 - 0.0054 = 0.0096
    // Output: 2000 * 15 / 1M = 0.03
    // Total: 0.0096 + 0.03 = 0.0396
    assert_eq!(
        retrieved_cost.total_cost,
        Decimal::from_str("0.0396000000").unwrap()
    );
}

#[tokio::test]
async fn test_multi_provider_aggregation() {
    let pool = setup_integration_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool.clone());
    let cost_repo = SqliteCostRepository::new(pool);

    let now = Utc::now();

    // Setup multiple providers
    let openai_pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );
    let openai_table = PricingTable::new(Provider::OpenAI, "gpt-4-multi".to_string(), openai_pricing);

    let anthropic_pricing = PricingStructure::simple_per_token(
        Decimal::from_str("3.0").unwrap(),
        Decimal::from_str("15.0").unwrap(),
    );
    let anthropic_table = PricingTable::new(
        Provider::Anthropic,
        "claude-3-sonnet-multi".to_string(),
        anthropic_pricing,
    );

    pricing_repo.create(&openai_table).await.unwrap();
    pricing_repo.create(&anthropic_table).await.unwrap();

    // Create usage records for different providers
    let calculator = CostCalculator::new();

    for i in 0..5 {
        let provider = if i % 2 == 0 {
            Provider::OpenAI
        } else {
            Provider::Anthropic
        };
        let model_name = if i % 2 == 0 {
            "gpt-4-multi"
        } else {
            "claude-3-sonnet-multi"
        };

        let usage = UsageRecord {
            id: Uuid::new_v4(),
            timestamp: now - chrono::Duration::hours(i),
            provider: provider.clone(),
            model: ModelIdentifier {
                name: model_name.to_string(),
                version: None,
                context_window: None,
            },
            organization_id: "org-multi".to_string(),
            project_id: Some(format!("proj-{}", i % 2)),
            user_id: None,
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
            cached_tokens: None,
            reasoning_tokens: None,
            latency_ms: None,
            tags: vec![],
            metadata: serde_json::json!({}),
            ingested_at: now,
            source: llm_cost_ops::domain::IngestionSource {
                source_type: "test".to_string(),
                endpoint: None,
            },
        };

        usage_repo.create(&usage).await.unwrap();

        let pricing = pricing_repo
            .get_active(&provider, model_name, &usage.timestamp)
            .await
            .unwrap()
            .unwrap();

        let cost = calculator.calculate(&usage, &pricing).unwrap();
        cost_repo.create(&cost).await.unwrap();
    }

    // Aggregate costs
    let start = now - chrono::Duration::days(1);
    let end = now + chrono::Duration::hours(1);

    let all_costs = cost_repo
        .list_by_organization("org-multi", start, end)
        .await
        .unwrap();

    assert_eq!(all_costs.len(), 5);

    let aggregator = llm_cost_ops::engine::CostAggregator::new();
    let summary = aggregator.aggregate(&all_costs, start, end).unwrap();

    assert_eq!(summary.total_requests, 5);
    assert!(summary.total_cost > Decimal::ZERO);

    // Verify by-provider breakdown
    assert!(summary.by_provider.contains_key("openai"));
    assert!(summary.by_provider.contains_key("anthropic"));

    // OpenAI should be more expensive (3 requests * higher rate)
    assert!(summary.by_provider["openai"] > summary.by_provider["anthropic"]);
}

#[tokio::test]
async fn test_time_range_filtering() {
    let pool = setup_integration_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let cost_repo = SqliteCostRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool);

    let now = Utc::now();

    // Setup pricing
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );
    let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4-time".to_string(), pricing);
    pricing_repo.create(&pricing_table).await.unwrap();

    let calculator = CostCalculator::new();

    // Create records at different times
    let timestamps = vec![
        now - chrono::Duration::days(10), // Outside range
        now - chrono::Duration::days(5),  // Inside range
        now - chrono::Duration::days(3),  // Inside range
        now - chrono::Duration::days(1),  // Inside range
        now,                               // Inside range
    ];

    for (i, timestamp) in timestamps.iter().enumerate() {
        let usage = UsageRecord {
            id: Uuid::new_v4(),
            timestamp: *timestamp,
            provider: Provider::OpenAI,
            model: ModelIdentifier {
                name: "gpt-4-time".to_string(),
                version: None,
                context_window: None,
            },
            organization_id: "org-time".to_string(),
            project_id: Some(format!("proj-{}", i)),
            user_id: None,
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
            cached_tokens: None,
            reasoning_tokens: None,
            latency_ms: None,
            tags: vec![],
            metadata: serde_json::json!({}),
            ingested_at: now,
            source: llm_cost_ops::domain::IngestionSource {
                source_type: "test".to_string(),
                endpoint: None,
            },
        };

        usage_repo.create(&usage).await.unwrap();

        let cost = calculator.calculate(&usage, &pricing_table).unwrap();
        cost_repo.create(&cost).await.unwrap();
    }

    // Query last 7 days
    let start = now - chrono::Duration::days(7);
    let end = now + chrono::Duration::hours(1);

    let recent_costs = cost_repo
        .list_by_organization("org-time", start, end)
        .await
        .unwrap();

    // Should only get 4 records (excluding the one from 10 days ago)
    assert_eq!(recent_costs.len(), 4);

    // All records should be within the time range
    for cost in recent_costs {
        assert!(cost.timestamp >= start);
        assert!(cost.timestamp <= end);
    }
}

#[tokio::test]
async fn test_error_handling_missing_pricing() {
    let pool = setup_integration_db().await;
    let pricing_repo = SqlitePricingRepository::new(pool);

    // Try to get pricing that doesn't exist
    let result = pricing_repo
        .get_active(
            &Provider::Custom("nonexistent".to_string()),
            "nonexistent-model",
            &Utc::now(),
        )
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_concurrent_ingestion() {
    let pool = setup_integration_db().await;
    let usage_repo = SqliteUsageRepository::new(pool.clone());
    let pricing_repo = SqlitePricingRepository::new(pool.clone());
    let cost_repo = SqliteCostRepository::new(pool);

    // Setup pricing
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );
    let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4-concurrent".to_string(), pricing);
    pricing_repo.create(&pricing_table).await.unwrap();

    let calculator = CostCalculator::new();
    let now = Utc::now();

    // Create multiple records concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let usage_repo = usage_repo.clone();
        let cost_repo = cost_repo.clone();
        let calculator = calculator.clone();
        let pricing_table = pricing_table.clone();

        let handle = tokio::spawn(async move {
            let usage = UsageRecord {
                id: Uuid::new_v4(),
                timestamp: now,
                provider: Provider::OpenAI,
                model: ModelIdentifier {
                    name: "gpt-4-concurrent".to_string(),
                    version: None,
                    context_window: None,
                },
                organization_id: "org-concurrent".to_string(),
                project_id: Some(format!("proj-{}", i)),
                user_id: None,
                prompt_tokens: 1000,
                completion_tokens: 500,
                total_tokens: 1500,
                cached_tokens: None,
                reasoning_tokens: None,
                latency_ms: None,
                tags: vec![],
                metadata: serde_json::json!({"index": i}),
                ingested_at: now,
                source: llm_cost_ops::domain::IngestionSource {
                    source_type: "test".to_string(),
                    endpoint: None,
                },
            };

            usage_repo.create(&usage).await.unwrap();

            let cost = calculator.calculate(&usage, &pricing_table).unwrap();
            cost_repo.create(&cost).await.unwrap();
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all records were created
    let start = now - chrono::Duration::hours(1);
    let end = now + chrono::Duration::hours(1);

    let all_costs = cost_repo
        .list_by_organization("org-concurrent", start, end)
        .await
        .unwrap();

    assert_eq!(all_costs.len(), 10);
}
