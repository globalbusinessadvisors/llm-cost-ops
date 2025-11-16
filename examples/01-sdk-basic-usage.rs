//! Basic SDK usage example
//!
//! This example demonstrates:
//! - Creating an SDK client with the builder pattern
//! - Submitting usage data
//! - Querying cost data
//! - Requesting forecasts
//! - Error handling

use chrono::{Duration, Utc};
use llm_cost_ops::sdk::{
    CostOpsClient, CostRequest, ForecastRequest as SdkForecastRequest, UsageRequest,
};
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create SDK client using builder pattern
    let client = CostOpsClient::builder()
        .base_url("http://localhost:8080")?
        .api_key("your-api-key-here")
        .timeout(StdDuration::from_secs(30))
        .build()?;

    println!("SDK client initialized successfully");

    // Example 1: Submit usage data
    let org_id = Uuid::new_v4();
    let usage_request = UsageRequest {
        organization_id: org_id,
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        prompt_tokens: 500,
        completion_tokens: 200,
        total_tokens: 700,
        timestamp: Utc::now(),
        request_id: Some(format!("req-{}", Uuid::new_v4())),
        metadata: HashMap::new(),
    };

    match client.submit_usage(usage_request).await {
        Ok(response) => {
            println!("Usage submitted successfully!");
            println!("Record ID: {}", response.id);
            println!("Status: {}", response.status);
            if let Some(cost) = response.cost {
                println!("Total cost: {} {}", cost.total_cost, cost.currency);
            }
        }
        Err(e) => {
            eprintln!("Failed to submit usage: {}", e);
        }
    }

    // Example 2: Query cost data
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(7);

    let cost_request = CostRequest {
        organization_id: org_id,
        start_date,
        end_date,
        provider: Some("openai".to_string()),
        model: None,
        aggregation: None,
    };

    match client.get_costs(cost_request).await {
        Ok(response) => {
            println!("\nCost data retrieved successfully!");
            println!("Total cost: {} {}", response.total_cost, response.currency);
            println!("Number of periods: {}", response.breakdown.len());

            for period in response.breakdown.iter().take(3) {
                println!(
                    "  Period: {} - {}: {} {}",
                    period.period_start, period.period_end, period.cost, response.currency
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to get costs: {}", e);
        }
    }

    // Example 3: Get forecast
    let forecast_request = SdkForecastRequest {
        organization_id: org_id,
        horizon_days: 30,
        lookback_days: 90,
        confidence_level: Some(0.95),
        include_seasonality: true,
    };

    match client.get_forecast(forecast_request).await {
        Ok(response) => {
            println!("\nForecast retrieved successfully!");
            println!(
                "Total forecast (30 days): {} {}",
                response.total_forecast, response.currency
            );
            println!("Model accuracy (MAPE): {:.2}%", response.metrics.mape);

            if let Some(first) = response.forecasts.first() {
                println!(
                    "First day forecast: {} {} (range: {} - {})",
                    first.cost, response.currency, first.lower_bound, first.upper_bound
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to get forecast: {}", e);
        }
    }

    // Example 4: Check API health
    match client.health().await {
        Ok(response) => {
            println!("\nHealth check successful!");
            println!("Status: {}", response.status);
            println!("Version: {}", response.version);
            println!("Uptime: {} seconds", response.uptime_seconds);
        }
        Err(e) => {
            eprintln!("Health check failed: {}", e);
        }
    }

    Ok(())
}
