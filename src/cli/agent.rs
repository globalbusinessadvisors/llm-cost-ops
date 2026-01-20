//! Agent CLI Commands
//!
//! CLI commands for invoking LLM-CostOps agents.
//!
//! # CLI Contract (per LLM-CostOps Constitution)
//! - CLI-invokable endpoints: analyze / forecast / inspect
//! - Machine-readable output (JSON)
//! - Deterministic behavior

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use rust_decimal::Decimal;

/// Agent commands
#[derive(Subcommand)]
pub enum AgentCommands {
    /// Run the Cost Forecasting Agent
    Forecast(ForecastArgs),

    /// Analyze historical cost data (alias for forecast)
    Analyze(ForecastArgs),

    /// Inspect agent capabilities and status
    Inspect(InspectArgs),

    /// List available agents
    List,
}

/// Arguments for forecast command
#[derive(Parser)]
pub struct ForecastArgs {
    /// Path to historical data file (JSON)
    #[arg(short, long)]
    pub input: Option<PathBuf>,

    /// Read historical data from stdin (JSON)
    #[arg(long)]
    pub stdin: bool,

    /// Forecast horizon in days (1-365)
    #[arg(short = 'H', long, default_value = "30")]
    pub horizon: u64,

    /// Forecast granularity (hourly, daily, weekly, monthly)
    #[arg(short, long, default_value = "daily")]
    pub granularity: String,

    /// Confidence level (0.0-1.0)
    #[arg(short, long, default_value = "0.95")]
    pub confidence: f64,

    /// Budget cap constraint
    #[arg(long)]
    pub budget_cap: Option<f64>,

    /// Maximum growth rate constraint (percentage)
    #[arg(long)]
    pub max_growth_rate: Option<f64>,

    /// Maximum cost per period constraint
    #[arg(long)]
    pub max_cost_per_period: Option<f64>,

    /// Organization ID
    #[arg(short, long)]
    pub organization: Option<String>,

    /// Project ID
    #[arg(short, long)]
    pub project: Option<String>,

    /// Execution reference (correlation ID)
    #[arg(long)]
    pub execution_ref: Option<String>,

    /// Output format (json, table, summary)
    #[arg(long, default_value = "json")]
    pub output: String,

    /// Preferred forecast model (linear_trend, moving_average, exponential_smoothing, auto)
    #[arg(long)]
    pub model: Option<String>,

    /// Query historical data from database instead of file
    #[arg(long)]
    pub from_db: bool,

    /// Time range for database query (e.g., "last-30-days")
    #[arg(long)]
    pub range: Option<String>,
}

/// Arguments for inspect command
#[derive(Parser)]
pub struct InspectArgs {
    /// Agent to inspect (default: cost-forecasting)
    #[arg(short, long, default_value = "cost-forecasting")]
    pub agent: String,

    /// Show detailed capabilities
    #[arg(long)]
    pub detailed: bool,

    /// Output format (json, table)
    #[arg(long, default_value = "json")]
    pub output: String,
}

/// Execute agent commands
pub async fn execute_agent_command(
    command: AgentCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        AgentCommands::Forecast(args) | AgentCommands::Analyze(args) => {
            execute_forecast(args).await
        }
        AgentCommands::Inspect(args) => {
            execute_inspect(args).await
        }
        AgentCommands::List => {
            execute_list().await
        }
    }
}

/// Execute forecast command
async fn execute_forecast(args: ForecastArgs) -> Result<(), Box<dyn std::error::Error>> {
    use crate::agents::{
        cost_forecasting::{
            CostForecastInput, ForecastConstraints, ForecastGranularity, ForecastMetadata,
            HistoricalDataPoint,
        },
        CostForecastingAgent,
    };

    // Load historical data
    let historical_data = if args.stdin {
        // Read from stdin
        let mut buffer = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer)?;
        serde_json::from_str::<Vec<HistoricalDataPoint>>(&buffer)?
    } else if let Some(input_path) = args.input {
        // Read from file
        let content = std::fs::read_to_string(input_path)?;
        serde_json::from_str::<Vec<HistoricalDataPoint>>(&content)?
    } else if args.from_db {
        // Query from database (placeholder for now)
        return Err("Database query not yet implemented. Use --input or --stdin".into());
    } else {
        return Err("Must specify --input, --stdin, or --from-db".into());
    };

    // Validate data
    if historical_data.len() < 7 {
        return Err(format!(
            "Insufficient data points: {} (minimum required: 7)",
            historical_data.len()
        ).into());
    }

    // Parse granularity
    let granularity = match args.granularity.to_lowercase().as_str() {
        "hourly" => ForecastGranularity::Hourly,
        "daily" => ForecastGranularity::Daily,
        "weekly" => ForecastGranularity::Weekly,
        "monthly" => ForecastGranularity::Monthly,
        _ => {
            return Err(format!("Invalid granularity: {}", args.granularity).into());
        }
    };

    // Build constraints
    let constraints = ForecastConstraints {
        budget_cap: args.budget_cap.map(Decimal::try_from).transpose()?,
        max_growth_rate: args.max_growth_rate,
        max_cost_per_period: args.max_cost_per_period.map(Decimal::try_from).transpose()?,
        roi_threshold: None,
        min_confidence: None,
    };

    // Build metadata
    let metadata = ForecastMetadata {
        organization_id: args.organization,
        project_id: args.project,
        execution_ref: args.execution_ref,
        source: Some("cli".to_string()),
        tags: vec![],
    };

    // Build input
    let input = CostForecastInput {
        historical_data,
        forecast_horizon_days: args.horizon,
        granularity,
        confidence_level: args.confidence,
        constraints,
        metadata,
        preferred_model: args.model,
    };

    // Create agent and run
    let agent = CostForecastingAgent::with_defaults()?;
    let output = agent.run(input).await?;

    // Format output
    match args.output.to_lowercase().as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "summary" => {
            print_summary(&output);
        }
        "table" => {
            print_table(&output);
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

/// Execute inspect command
async fn execute_inspect(args: InspectArgs) -> Result<(), Box<dyn std::error::Error>> {
    use crate::agents::{
        cost_forecasting::{AGENT_ID, AGENT_VERSION, MAX_FORECAST_DAYS, MIN_DATA_POINTS},
        AgentClassification,
        contracts::DecisionType,
    };

    let info = serde_json::json!({
        "agent_id": AGENT_ID,
        "agent_version": AGENT_VERSION,
        "classification": AgentClassification::Forecasting.to_string(),
        "decision_type": DecisionType::CostForecast.to_string(),
        "description": "Forecasts future LLM spend based on historical usage patterns and growth trends",
        "capabilities": {
            "forecast_horizons": ["hourly", "daily", "weekly", "monthly"],
            "max_forecast_days": MAX_FORECAST_DAYS,
            "min_data_points": MIN_DATA_POINTS,
            "supported_models": ["linear_trend", "moving_average", "exponential_smoothing", "auto"],
            "constraint_types": ["budget_cap", "roi_threshold", "max_cost_per_period", "max_growth_rate", "min_confidence"]
        },
        "compliance": {
            "persists_to": "ruvector-service",
            "emits_telemetry_to": "llm-observatory",
            "stateless": true,
            "deterministic": true
        },
        "endpoints": {
            "forecast": "POST /forecast",
            "analyze": "POST /analyze",
            "inspect": "GET /inspect"
        }
    });

    match args.output.to_lowercase().as_str() {
        "json" => {
            if args.detailed {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                // Simplified output
                let simple = serde_json::json!({
                    "agent_id": AGENT_ID,
                    "agent_version": AGENT_VERSION,
                    "classification": "FORECASTING",
                    "status": "ready"
                });
                println!("{}", serde_json::to_string_pretty(&simple)?);
            }
        }
        "table" => {
            println!("Agent: {}", AGENT_ID);
            println!("Version: {}", AGENT_VERSION);
            println!("Classification: FORECASTING");
            println!("Decision Type: cost_forecast");
            println!("Status: ready");
            if args.detailed {
                println!("\nCapabilities:");
                println!("  Max Forecast Days: {}", MAX_FORECAST_DAYS);
                println!("  Min Data Points: {}", MIN_DATA_POINTS);
                println!("  Supported Models: linear_trend, moving_average, exponential_smoothing, auto");
                println!("\nEndpoints:");
                println!("  POST /forecast  - Generate cost forecast");
                println!("  POST /analyze   - Analyze costs (alias)");
                println!("  GET  /inspect   - Inspect agent");
            }
        }
        _ => {
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
    }

    Ok(())
}

/// Execute list command
async fn execute_list() -> Result<(), Box<dyn std::error::Error>> {
    use crate::agents::cost_forecasting::{AGENT_ID, AGENT_VERSION};

    let agents = serde_json::json!({
        "agents": [
            {
                "id": AGENT_ID,
                "version": AGENT_VERSION,
                "classification": "FORECASTING",
                "decision_type": "cost_forecast",
                "status": "available"
            }
        ],
        "total": 1
    });

    println!("{}", serde_json::to_string_pretty(&agents)?);
    Ok(())
}

/// Print summary output
fn print_summary(output: &crate::agents::cost_forecasting::CostForecastOutput) {
    println!("=== Cost Forecast Summary ===\n");

    println!("Forecast Period: {} days", output.projections.len());
    println!("Model Used: {}", output.model_used);
    println!("Confidence: {:.1}%\n", output.confidence * 100.0);

    println!("Cost Projections:");
    println!("  Total Forecasted: ${}", output.total_forecasted_cost);
    println!("  Average Daily: ${}", output.average_daily_cost);
    println!("  Peak Daily: ${}", output.peak_daily_cost);
    println!();

    println!("Growth Analysis:");
    println!("  Pattern: {:?}", output.growth_pattern);
    println!("  Average Rate: {:.1}%\n", output.average_growth_rate);

    if !output.risk_indicators.is_empty() {
        println!("Risk Indicators:");
        for risk in &output.risk_indicators {
            println!("  [{:?}] {}: {}", risk.level, risk.risk_type, risk.description);
        }
        println!();
    }

    println!("Historical Data:");
    println!("  Data Points: {}", output.historical_summary.data_points);
    println!("  Period: {} to {}",
        output.historical_summary.period_start.format("%Y-%m-%d"),
        output.historical_summary.period_end.format("%Y-%m-%d")
    );
    println!("  Total Cost: ${}", output.historical_summary.total_cost);
    println!();

    println!("Generated At: {}", output.generated_at.format("%Y-%m-%d %H:%M:%S UTC"));
}

/// Print table output
fn print_table(output: &crate::agents::cost_forecasting::CostForecastOutput) {
    println!("{:<20} {:<15} {:<15} {:<15} {:<10}",
        "Date", "Projected", "Lower", "Upper", "Growth");
    println!("{}", "-".repeat(75));

    for projection in &output.projections {
        println!("{:<20} ${:<14} ${:<14} ${:<14} {:<10}",
            projection.timestamp.format("%Y-%m-%d"),
            format!("{:.2}", projection.projected_cost),
            format!("{:.2}", projection.lower_bound),
            format!("{:.2}", projection.upper_bound),
            projection.growth_rate
                .map(|r| format!("{:.1}%", r))
                .unwrap_or_else(|| "-".to_string())
        );
    }

    println!("\nTotal: ${}", output.total_forecasted_cost);
    println!("Confidence: {:.1}%", output.confidence * 100.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forecast_args_defaults() {
        use clap::Parser;

        // This would need actual CLI parsing to test
        // Just verify the module compiles
    }
}
