// Example: Starting an ingestion webhook server for real-time usage data
//
// This example demonstrates how to set up the Observatory integration
// webhook server to receive usage data from external systems.
//
// Usage:
//   cargo run --example 02-ingestion-server

use llm_cost_ops::{
    ingestion::{start_webhook_server, DefaultIngestionHandler},
    storage::SqliteUsageRepository,
};
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability
    llm_cost_ops::init()?;

    println!("🚀 Starting LLM-CostOps Ingestion Server");
    println!("======================================");

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:cost-ops.db".to_string());

    println!("📊 Connecting to database: {}", database_url);

    let pool = SqlitePool::connect(&database_url).await?;

    // Run migrations
    println!("🔄 Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Create repository and handler
    let repository = SqliteUsageRepository::new(pool);
    let handler = DefaultIngestionHandler::new(repository);

    // Start webhook server
    let bind_addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    println!("🌐 Starting webhook server on {}", bind_addr);
    println!();
    println!("Available endpoints:");
    println!("  GET  /health              - Health check");
    println!("  POST /v1/usage            - Ingest single usage record");
    println!("  POST /v1/usage/batch      - Ingest batch of usage records");
    println!();
    println!("Example POST request:");
    println!(r#"
  curl -X POST http://localhost:8080/v1/usage \
    -H "Content-Type: application/json" \
    -d '{{
      "timestamp": "2025-11-15T10:30:00Z",
      "provider": "openai",
      "model": {{
        "name": "gpt-4",
        "context_window": 8192
      }},
      "organization_id": "org-123",
      "usage": {{
        "prompt_tokens": 100,
        "completion_tokens": 50,
        "total_tokens": 150
      }}
    }}'
"#);

    println!("Press Ctrl+C to stop the server");
    println!();

    start_webhook_server(&bind_addr, handler).await?;

    Ok(())
}
