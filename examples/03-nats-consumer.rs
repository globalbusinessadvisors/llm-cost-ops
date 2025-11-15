// Example: NATS stream consumer for real-time usage ingestion
//
// This example demonstrates how to consume usage data from a NATS stream
// and store it in the database.
//
// Prerequisites:
//   - NATS server running (e.g., `docker run -p 4222:4222 nats:latest`)
//
// Usage:
//   NATS_URL=nats://localhost:4222 cargo run --example 03-nats-consumer

use llm_cost_ops::{
    ingestion::{DefaultIngestionHandler, NatsConsumer},
    storage::SqliteUsageRepository,
};
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability
    llm_cost_ops::init()?;

    println!("🚀 Starting LLM-CostOps NATS Consumer");
    println!("====================================");

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

    // Configure NATS
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    let subject = std::env::var("NATS_SUBJECT")
        .unwrap_or_else(|_| "llm.usage".to_string());

    println!("📡 Connecting to NATS: {}", nats_url);
    println!("📬 Subscribing to subject: {}", subject);

    // Create NATS consumer
    let mut consumer = NatsConsumer::new(
        &[nats_url],
        subject,
        handler,
    ).await?;

    println!("✅ NATS consumer ready");
    println!("🔄 Waiting for messages...");
    println!();
    println!("Press Ctrl+C to stop the consumer");
    println!();

    // Start consuming
    consumer.start().await?;

    Ok(())
}
