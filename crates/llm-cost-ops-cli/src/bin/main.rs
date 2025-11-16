use anyhow::Result;
use chrono::Utc;
use llm_cost_ops::{
    config::Config,
    domain::{PricingStructure, PricingTable, Provider, UsageRecord},
    engine::{CostAggregator, CostCalculator},
    storage::{CostRepository, PricingRepository, UsageRepository},
};
use llm_cost_ops_cli::cli::{Cli, Commands, PricingCommands};
use rust_decimal::Decimal;
use sqlx::sqlite::SqlitePool;
use std::str::FromStr;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

    // Initialize logging
    llm_cost_ops::init()?;

    info!("LLM-CostOps v{}", llm_cost_ops::VERSION);

    // Load configuration
    let config = if let Some(config_path) = &cli.config {
        Config::from_file(config_path)?
    } else {
        Config::default_sqlite()
    };

    match &cli.command {
        Commands::Init { database_url } => {
            init_database(database_url.as_deref().unwrap_or(&config.database.url)).await?;
        }
        Commands::Ingest { file, provider } => {
            ingest_usage(&config, file, provider.as_deref()).await?;
        }
        Commands::Query {
            range,
            organization,
            group_by,
            output,
        } => {
            query_costs(&config, range, organization.as_deref(), group_by.as_deref(), output)
                .await?;
        }
        Commands::Summary {
            period,
            organization,
        } => {
            generate_summary(&config, period, organization.as_deref()).await?;
        }
        Commands::Export {
            output,
            format,
            period,
        } => {
            export_data(&config, output, format, period.as_deref()).await?;
        }
        Commands::Pricing { command } => {
            handle_pricing_command(&config, command).await?;
        }
    }

    Ok(())
}

async fn init_database(database_url: &str) -> Result<()> {
    info!("Initializing database: {}", database_url);

    let _pool = SqlitePool::connect(database_url).await?;

    // Run migrations
    // TODO: Update migration path for workspace structure
    // sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Database initialized (migrations skipped - needs configuration)");

    Ok(())
}

async fn ingest_usage(config: &Config, file: &std::path::Path, _provider: Option<&str>) -> Result<()> {
    info!("Ingesting usage from file: {:?}", file);

    let pool = SqlitePool::connect(&config.database.url).await?;
    let usage_repo = llm_cost_ops::storage::SqliteUsageRepository::new(pool.clone());
    let pricing_repo = llm_cost_ops::storage::SqlitePricingRepository::new(pool.clone());
    let cost_repo = llm_cost_ops::storage::SqliteCostRepository::new(pool);

    // Read usage records from file
    let contents = tokio::fs::read_to_string(file).await?;
    let records: Vec<UsageRecord> = serde_json::from_str(&contents)?;

    info!("Found {} usage records", records.len());

    let calculator = CostCalculator::new();

    for record in records {
        // Validate record
        record.validate()?;

        // Store usage record
        usage_repo.create(&record).await?;

        // Get pricing for this record
        let pricing = pricing_repo
            .get_active(&record.provider, &record.model.name, &record.timestamp)
            .await?;

        if let Some(pricing) = pricing {
            // Calculate cost
            let cost_record = calculator.calculate(&record, &pricing)?;

            // Store cost record
            cost_repo.create(&cost_record).await?;

            info!(
                "Processed record {} - Cost: {} {}",
                record.id, cost_record.total_cost, cost_record.currency.as_str()
            );
        } else {
            error!(
                "No pricing found for provider={} model={} date={}",
                record.provider, record.model.name, record.timestamp
            );
        }
    }

    info!("Ingestion complete");
    Ok(())
}

async fn query_costs(
    config: &Config,
    range: &str,
    organization: Option<&str>,
    _group_by: Option<&str>,
    output: &str,
) -> Result<()> {
    info!("Querying costs for range: {}", range);

    let pool = SqlitePool::connect(&config.database.url).await?;
    let cost_repo = llm_cost_ops::storage::SqliteCostRepository::new(pool);

    let (start, end) = parse_time_range(range)?;

    let org_id = organization.unwrap_or("default");
    let records = cost_repo.list_by_organization(org_id, start, end).await?;

    match output {
        "json" => {
            let json = serde_json::to_string_pretty(&records)?;
            println!("{}", json);
        }
        "table" => {
            println!("\n{:<37} {:<20} {:<15} {:<12}", "ID", "Provider", "Model", "Total Cost");
            println!("{}", "-".repeat(85));
            for record in &records {
                println!(
                    "{:<37} {:<20} {:<15} ${:<11.6}",
                    record.id.to_string(),
                    record.provider.to_string(),
                    record.model,
                    record.total_cost
                );
            }
            println!("\nTotal records: {}", records.len());
        }
        "csv" => {
            println!("id,provider,model,total_cost,currency,timestamp");
            for record in &records {
                println!(
                    "{},{},{},{},{},{}",
                    record.id,
                    record.provider,
                    record.model,
                    record.total_cost,
                    record.currency.as_str(),
                    record.timestamp
                );
            }
        }
        _ => {
            anyhow::bail!("Unsupported output format: {}", output);
        }
    }

    Ok(())
}

async fn generate_summary(config: &Config, period: &str, organization: Option<&str>) -> Result<()> {
    info!("Generating cost summary for period: {}", period);

    let pool = SqlitePool::connect(&config.database.url).await?;
    let cost_repo = llm_cost_ops::storage::SqliteCostRepository::new(pool);

    let (start, end) = parse_time_range(period)?;
    let org_id = organization.unwrap_or("default");

    let records = cost_repo.list_by_organization(org_id, start, end).await?;

    let aggregator = CostAggregator::new();
    let summary = aggregator.aggregate(&records, start, end)?;

    println!("\n=== Cost Summary ===");
    println!("Period: {} to {}", start, end);
    println!("Organization: {}", org_id);
    println!("\nTotal Cost: ${:.6}", summary.total_cost);
    println!("Total Requests: {}", summary.total_requests);
    println!("Avg Cost/Request: ${:.6}", summary.avg_cost_per_request);

    println!("\n--- By Provider ---");
    for (provider, cost) in &summary.by_provider {
        println!("{}: ${:.6}", provider, cost);
    }

    println!("\n--- By Model ---");
    for (model, cost) in &summary.by_model {
        println!("{}: ${:.6}", model, cost);
    }

    if !summary.by_project.is_empty() {
        println!("\n--- By Project ---");
        for (project, cost) in &summary.by_project {
            println!("{}: ${:.6}", project, cost);
        }
    }

    Ok(())
}

async fn export_data(
    config: &Config,
    output: &std::path::Path,
    format: &str,
    period: Option<&str>,
) -> Result<()> {
    info!("Exporting data to: {:?} (format: {})", output, format);

    let pool = SqlitePool::connect(&config.database.url).await?;
    let cost_repo = llm_cost_ops::storage::SqliteCostRepository::new(pool);

    let (start, end) = if let Some(period) = period {
        parse_time_range(period)?
    } else {
        (Utc::now() - chrono::Duration::days(30), Utc::now())
    };

    let records = cost_repo.list_by_organization("default", start, end).await?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&records)?;
            tokio::fs::write(output, json).await?;
        }
        "csv" => {
            let mut csv = String::new();
            csv.push_str("id,usage_id,provider,model,total_cost,currency,timestamp\n");
            for record in &records {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    record.id,
                    record.usage_id,
                    record.provider,
                    record.model,
                    record.total_cost,
                    record.currency.as_str(),
                    record.timestamp
                ));
            }
            tokio::fs::write(output, csv).await?;
        }
        _ => {
            anyhow::bail!("Unsupported export format: {}", format);
        }
    }

    info!("Exported {} records to {:?}", records.len(), output);
    Ok(())
}

async fn handle_pricing_command(config: &Config, command: &PricingCommands) -> Result<()> {
    let pool = SqlitePool::connect(&config.database.url).await?;
    let pricing_repo = llm_cost_ops::storage::SqlitePricingRepository::new(pool);

    match command {
        PricingCommands::List => {
            let tables = pricing_repo.list_all().await?;
            println!("\n{:<37} {:<15} {:<20} {:<20}", "ID", "Provider", "Model", "Effective Date");
            println!("{}", "-".repeat(95));
            for table in &tables {
                println!(
                    "{:<37} {:<15} {:<20} {}",
                    table.id.to_string(),
                    table.provider.to_string(),
                    table.model,
                    table.effective_date
                );
            }
            println!("\nTotal pricing tables: {}", tables.len());
        }
        PricingCommands::Add {
            provider,
            model,
            input_price,
            output_price,
        } => {
            let provider = Provider::from_str(provider);
            let pricing = PricingStructure::simple_per_token(
                Decimal::from_str(&input_price.to_string())?,
                Decimal::from_str(&output_price.to_string())?,
            );

            let table = PricingTable::new(provider.unwrap_or_else(|_| Provider::OpenAI), model.clone(), pricing);

            pricing_repo.create(&table).await?;

            info!(
                "Created pricing table: provider={} model={} input=${}/M output=${}/M",
                table.provider, table.model, input_price, output_price
            );
        }
        PricingCommands::Get { provider, model } => {
            let provider_result = Provider::from_str(provider);
            let provider_val = provider_result.unwrap_or_else(|_| Provider::OpenAI);
            let now = Utc::now();

            if let Some(table) = pricing_repo.get_active(&provider_val, model, &now).await? {
                println!("\nPricing Table");
                println!("=============");
                println!("Provider: {}", table.provider);
                println!("Model: {}", table.model);
                println!("Currency: {}", table.currency.as_str());
                println!("Effective: {}", table.effective_date);
                if let Some(end) = table.end_date {
                    println!("End Date: {}", end);
                }
                println!("\nPricing: {}", serde_json::to_string_pretty(&table.pricing)?);
            } else {
                println!("No active pricing found for provider={} model={}", provider_val, model);
            }
        }
    }

    Ok(())
}

fn parse_time_range(range: &str) -> Result<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)> {
    let end = Utc::now();
    let start = match range {
        "last-hour" => end - chrono::Duration::hours(1),
        "last-24-hours" | "last-day" => end - chrono::Duration::hours(24),
        "last-7-days" | "last-week" => end - chrono::Duration::days(7),
        "last-30-days" | "last-month" => end - chrono::Duration::days(30),
        "last-90-days" => end - chrono::Duration::days(90),
        _ => anyhow::bail!("Unsupported time range: {}", range),
    };

    Ok((start, end))
}
