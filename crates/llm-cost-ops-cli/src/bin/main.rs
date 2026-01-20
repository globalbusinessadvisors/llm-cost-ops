use anyhow::Result;
use chrono::{Duration, Utc};
use llm_cost_ops::{
    config::Config,
    domain::{PricingStructure, PricingTable, Provider, UsageRecord},
    engine::{CostAggregator, CostCalculator},
    storage::{CostRepository, PricingRepository, UsageRepository},
    agents::{
        AgentId, AgentVersion, AgentClassification,
        BudgetEnforcementAgent, BudgetEnforcementConfig,
        BudgetEvaluationRequest, BudgetConstraintSignal,
        RuvectorClient, RuvectorConfig,
        budget_enforcement::{BudgetDefinition, BudgetScope, SpendData},
        contracts::ExecutionRef,
    },
};
use llm_cost_ops_api::api::{ApiServer, ApiServerConfig};
use llm_cost_ops_cli::{
    cli::{Cli, Commands, PricingCommands, AgentCommands, BudgetEnforcementCommands},
    run_all_benchmarks,
};
use rust_decimal::Decimal;
use sqlx::sqlite::SqlitePool;
use std::str::FromStr;
use tracing::{info, error};
use uuid::Uuid;

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
        Commands::Run {
            output,
            no_summary,
            filter,
        } => {
            run_benchmarks(output, !no_summary, filter.as_deref()).await?;
        }
        Commands::Agent { command } => {
            handle_agent_command(command).await?;
        }
        Commands::Serve {
            host,
            port,
            request_timeout,
            enable_cors,
        } => {
            start_server(&host, port.unwrap_or(8080), *request_timeout, *enable_cors).await?;
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

async fn run_benchmarks(
    output: &std::path::Path,
    generate_summary: bool,
    filter: Option<&str>,
) -> Result<()> {
    info!("Running benchmarks");

    if let Some(filter) = filter {
        info!("Applying filter: {}", filter);
    }

    run_all_benchmarks(output, generate_summary, filter).await?;

    info!("Benchmark execution complete");
    Ok(())
}

/// Handle agent commands
async fn handle_agent_command(command: &AgentCommands) -> Result<()> {
    match command {
        AgentCommands::List => {
            list_agents();
        }
        AgentCommands::Info { agent_id } => {
            agent_info(agent_id);
        }
        AgentCommands::BudgetEnforcement { command } => {
            handle_budget_enforcement_command(command).await?;
        }
    }
    Ok(())
}

/// List available agents
fn list_agents() {
    println!("\n=== Available LLM-CostOps Agents ===\n");
    println!("{:<40} {:<25} {:<15}", "Agent ID", "Classification", "Version");
    println!("{}", "-".repeat(80));

    // Budget Enforcement Agent
    let agent = BudgetEnforcementAgent::with_defaults();
    println!(
        "{:<40} {:<25} {:<15}",
        agent.agent_id().to_string(),
        agent.classification().to_string(),
        agent.agent_version().to_string()
    );

    println!("\nTotal agents: 1");
    println!("\nUse 'cost-ops agent <agent-name> --help' for more information.");
}

/// Get agent information
fn agent_info(agent_id: &str) {
    match agent_id {
        "llm-costops.budget-enforcement" | "budget-enforcement" => {
            let agent = BudgetEnforcementAgent::with_defaults();
            println!("\n=== Budget Enforcement Agent ===\n");
            println!("Agent ID:       {}", agent.agent_id());
            println!("Version:        {}", agent.agent_version());
            println!("Classification: {}", agent.classification());
            println!("\nPurpose:");
            println!("  Evaluate budget thresholds and emit advisory or gating signals");
            println!("  when limits are approached or exceeded.");
            println!("\nDecision Type: budget_constraint_evaluation");
            println!("\nCapabilities:");
            println!("  - Analyze current and projected spend against budget limits");
            println!("  - Compute confidence scores based on data completeness");
            println!("  - Emit advisory signals (informational, warning, gating)");
            println!("  - Persist DecisionEvents to ruvector-service");
            println!("  - Emit telemetry compatible with LLM-Observatory");
            println!("\nLimitations (by design):");
            println!("  - Does NOT enforce budgets directly (advisory only)");
            println!("  - Does NOT intercept runtime execution");
            println!("  - Does NOT execute SQL directly");
            println!("\nCLI Usage:");
            println!("  cost-ops agent budget-enforcement analyze \\");
            println!("    --tenant-id <tenant> \\");
            println!("    --budget-id <budget> \\");
            println!("    --budget-limit <amount> \\");
            println!("    --current-spend <amount> \\");
            println!("    --execution-ref <execution-id>");
        }
        _ => {
            error!("Unknown agent: {}", agent_id);
            println!("Unknown agent: {}", agent_id);
            println!("\nUse 'cost-ops agent list' to see available agents.");
        }
    }
}

/// Handle Budget Enforcement Agent commands
async fn handle_budget_enforcement_command(command: &BudgetEnforcementCommands) -> Result<()> {
    match command {
        BudgetEnforcementCommands::Analyze {
            tenant_id,
            budget_id,
            budget_limit,
            currency,
            current_spend,
            execution_ref,
            include_forecast,
            warning_threshold,
            critical_threshold,
            output,
            ruvector_endpoint,
            dry_run,
        } => {
            analyze_budget(
                tenant_id,
                budget_id,
                *budget_limit,
                currency,
                *current_spend,
                execution_ref.as_deref(),
                *include_forecast,
                *warning_threshold,
                *critical_threshold,
                output,
                ruvector_endpoint.as_deref(),
                *dry_run,
            )
            .await?;
        }
        BudgetEnforcementCommands::Inspect => {
            inspect_budget_enforcement_agent();
        }
        BudgetEnforcementCommands::Health { check_ruvector } => {
            check_budget_enforcement_health(*check_ruvector).await?;
        }
    }
    Ok(())
}

/// Analyze budget using Budget Enforcement Agent
async fn analyze_budget(
    tenant_id: &str,
    budget_id: &str,
    budget_limit: f64,
    currency: &str,
    current_spend: f64,
    execution_ref: Option<&str>,
    include_forecast: bool,
    warning_threshold: f64,
    critical_threshold: f64,
    output_format: &str,
    ruvector_endpoint: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    info!(
        "Evaluating budget: tenant={} budget={} limit={} current={}",
        tenant_id, budget_id, budget_limit, current_spend
    );

    // Create agent configuration
    let mut config = BudgetEnforcementConfig::default();
    config.persist_events = !dry_run && ruvector_endpoint.is_some();
    config.emit_telemetry = !dry_run && ruvector_endpoint.is_some();

    // Create agent
    let mut agent = BudgetEnforcementAgent::new(config);

    // Set up RuVector client if endpoint provided and not dry run
    if let Some(endpoint) = ruvector_endpoint {
        if !dry_run {
            let ruvector_config = RuvectorConfig {
                endpoint: endpoint.to_string(),
                ..RuvectorConfig::default()
            };
            match RuvectorClient::new(ruvector_config) {
                Ok(client) => {
                    agent = agent.with_ruvector_client(client);
                    info!("RuVector client configured: {}", endpoint);
                }
                Err(e) => {
                    error!("Failed to create RuVector client: {}", e);
                }
            }
        }
    }

    // Create budget definition
    let budget = BudgetDefinition {
        budget_id: budget_id.to_string(),
        name: format!("Budget-{}", budget_id),
        limit: Decimal::from_str(&budget_limit.to_string())?,
        currency: currency.to_string(),
        period_start: Utc::now() - Duration::days(15),
        period_end: Utc::now() + Duration::days(15),
        warning_threshold,
        critical_threshold,
        gating_threshold: 1.0,
        enable_forecasting: include_forecast,
        is_soft_limit: true,
        scope: BudgetScope::Tenant {
            tenant_id: tenant_id.to_string(),
        },
    };

    // Create spend data
    let spend_data = SpendData {
        current_spend: Decimal::from_str(&current_spend.to_string())?,
        currency: currency.to_string(),
        daily_spend_history: vec![],
        data_completeness: 1.0,
        data_as_of: Utc::now(),
    };

    // Create execution reference
    let exec_ref = ExecutionRef::new(
        execution_ref
            .map(|s| Uuid::parse_str(s).unwrap_or_else(|_| Uuid::new_v4()))
            .unwrap_or_else(Uuid::new_v4),
        tenant_id,
    );

    // Create evaluation request
    let request = BudgetEvaluationRequest::new(budget, spend_data, exec_ref);

    // Evaluate (this produces a signal but doesn't persist without client)
    let signal = agent.evaluate(&request).await?;

    // Output result
    match output_format {
        "json" => {
            let json = serde_json::to_string_pretty(&signal)?;
            println!("{}", json);
        }
        "table" => {
            print_budget_signal_table(&signal);
        }
        _ => {
            anyhow::bail!("Unsupported output format: {}", output_format);
        }
    }

    if dry_run {
        info!("Dry run - no events persisted");
    }

    Ok(())
}

/// Print budget signal as a table
fn print_budget_signal_table(signal: &BudgetConstraintSignal) {
    println!("\n=== Budget Constraint Signal ===\n");
    println!("Signal ID:      {}", signal.signal_id);
    println!("Budget ID:      {}", signal.budget_id);
    println!("Signal Type:    {:?}", signal.signal_type);
    println!("Severity:       {:?}", signal.severity);
    println!("Violation Type: {:?}", signal.violation_type);
    println!("\nMessage: {}", signal.message);
    println!("\n--- Budget Status ---");
    println!("Current Spend:  {} (limit: {})", signal.current_spend, signal.budget_limit);
    println!("Remaining:      {}", signal.remaining_budget);
    println!("Utilization:    {:.2}%", signal.utilization_percent);
    println!("Days Remaining: {}", signal.days_remaining);
    println!("Daily Average:  {}", signal.daily_average);

    if let Some(projected) = signal.projected_spend {
        println!("\n--- Forecast ---");
        println!("Projected Spend:       {}", projected);
        println!("Projected Utilization: {:.2}%", signal.projected_utilization.unwrap_or(0.0));
    }

    println!("\n--- Recommended Action ---");
    println!("{:?}", signal.recommended_action);

    if !signal.alerts.is_empty() {
        println!("\n--- Alerts ({}) ---", signal.alerts.len());
        for alert in &signal.alerts {
            println!("  [{:?}] {}", alert.severity, alert.message);
        }
    }

    println!("\nTimestamp: {}", signal.timestamp);
}

/// Inspect Budget Enforcement Agent configuration
fn inspect_budget_enforcement_agent() {
    let agent = BudgetEnforcementAgent::with_defaults();
    let config = BudgetEnforcementConfig::default();

    println!("\n=== Budget Enforcement Agent Configuration ===\n");
    println!("Agent ID:       {}", agent.agent_id());
    println!("Version:        {}", agent.agent_version());
    println!("Classification: {}", agent.classification());
    println!("\n--- Default Configuration ---");
    println!("Persist Events: {}", config.persist_events);
    println!("Emit Telemetry: {}", config.emit_telemetry);
    println!("Min Data Completeness (High Confidence): {}", config.min_data_completeness_for_high_confidence);
    println!("Min Forecast Data Points: {}", config.min_forecast_data_points);
    println!("Forecast Confidence Factor: {}", config.forecast_confidence_factor);

    println!("\n--- Environment Variables ---");
    println!("RUVECTOR_ENDPOINT:     {}", std::env::var("RUVECTOR_ENDPOINT").unwrap_or_else(|_| "(not set)".to_string()));
    println!("RUVECTOR_API_KEY:      {}", if std::env::var("RUVECTOR_API_KEY").is_ok() { "(set)" } else { "(not set)" });
}

/// Check Budget Enforcement Agent health
async fn check_budget_enforcement_health(check_ruvector: bool) -> Result<()> {
    println!("\n=== Budget Enforcement Agent Health ===\n");

    // Agent health is always OK (stateless)
    println!("[OK] Agent runtime");

    if check_ruvector {
        match RuvectorConfig::from_env() {
            Ok(config) => {
                match RuvectorClient::new(config) {
                    Ok(client) => {
                        match client.health_check().await {
                            Ok(true) => println!("[OK] RuVector service"),
                            Ok(false) => println!("[WARN] RuVector service unhealthy"),
                            Err(e) => println!("[ERR] RuVector service: {}", e),
                        }
                    }
                    Err(e) => println!("[ERR] RuVector client: {}", e),
                }
            }
            Err(e) => println!("[SKIP] RuVector: {}", e),
        }
    } else {
        println!("[SKIP] RuVector service (use --check-ruvector to test)");
    }

    println!("\nOverall: OK");
    Ok(())
}

/// Start the HTTP server for Cloud Run deployment
async fn start_server(host: &str, port: u16, request_timeout: u64, enable_cors: bool) -> Result<()> {
    info!("Starting LLM-CostOps server on {}:{}", host, port);

    let config = ApiServerConfig {
        host: host.to_string(),
        port,
        request_timeout_secs: request_timeout,
        enable_cors,
        enable_logging: true,
    };

    let server = ApiServer::new(config);

    info!("Server configuration:");
    info!("  Host: {}", host);
    info!("  Port: {}", port);
    info!("  Request timeout: {}s", request_timeout);
    info!("  CORS enabled: {}", enable_cors);
    info!("");
    info!("Agent endpoints:");
    info!("  /api/v1/agents/budget-enforcement/*");
    info!("  /api/v1/agents/cost-forecasting/*");
    info!("  /api/v1/agents/roi-estimation/*");
    info!("  /api/v1/agents/cost-performance/*");
    info!("  /api/v1/agents/cost-attribution/*");
    info!("");
    info!("Health endpoints:");
    info!("  /health");
    info!("  /ready");
    info!("  /info");

    server.run().await.map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
