use clap::{Parser, Subcommand};
use std::path::PathBuf;

// CLI argument definitions for LLM Cost Operations
// Export command types for main.rs

#[derive(Parser)]
#[command(name = "cost-ops")]
#[command(about = "LLM Cost Operations Platform", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize database and create schema
    Init {
        /// Database URL (e.g., sqlite:cost-ops.db)
        #[arg(long)]
        database_url: Option<String>,
    },

    /// Ingest usage metrics
    Ingest {
        /// Input file path (JSON)
        #[arg(short, long)]
        file: PathBuf,

        /// Provider name
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// Query cost records
    Query {
        /// Time range (e.g., "last-24-hours", "last-7-days")
        #[arg(long, default_value = "last-24-hours")]
        range: String,

        /// Organization ID filter
        #[arg(long)]
        organization: Option<String>,

        /// Group by dimension (provider, model, project)
        #[arg(long)]
        group_by: Option<String>,

        /// Output format (json, table, csv)
        #[arg(long, default_value = "table")]
        output: String,
    },

    /// Generate cost summary
    Summary {
        /// Time period (e.g., "last-30-days")
        #[arg(long, default_value = "last-30-days")]
        period: String,

        /// Organization ID filter
        #[arg(long)]
        organization: Option<String>,
    },

    /// Export cost data
    Export {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Format (json, csv, parquet)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Time period
        #[arg(long)]
        period: Option<String>,
    },

    /// Manage pricing tables
    Pricing {
        #[command(subcommand)]
        command: PricingCommands,
    },

    /// Run benchmarks and generate performance reports
    Run {
        /// Output directory for benchmark results
        #[arg(short, long, default_value = "benchmarks/output")]
        output: PathBuf,

        /// Skip generating summary markdown report
        #[arg(long)]
        no_summary: bool,

        /// Benchmark filter (e.g., "cost_calculation", "aggregation")
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Agent operations (financial governance, cost analysis, forecasting)
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },

    /// Start the HTTP server (for Cloud Run deployment)
    Serve {
        /// Host address to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on (default: PORT env var or 8080)
        #[arg(long, env = "PORT")]
        port: Option<u16>,

        /// Request timeout in seconds
        #[arg(long, default_value = "30")]
        request_timeout: u64,

        /// Enable CORS
        #[arg(long, default_value = "true")]
        enable_cors: bool,
    },
}

/// Agent subcommands
#[derive(Subcommand)]
pub enum AgentCommands {
    /// Budget Enforcement Agent - Evaluate budget thresholds and emit advisory signals
    #[command(name = "budget-enforcement")]
    BudgetEnforcement {
        #[command(subcommand)]
        command: BudgetEnforcementCommands,
    },

    /// List available agents
    List,

    /// Get agent information
    Info {
        /// Agent ID
        #[arg(long)]
        agent_id: String,
    },
}

/// Budget Enforcement Agent subcommands
#[derive(Subcommand)]
pub enum BudgetEnforcementCommands {
    /// Analyze budget against current spend (CLI-invokable endpoint)
    Analyze {
        /// Tenant/organization ID
        #[arg(long)]
        tenant_id: String,

        /// Budget ID
        #[arg(long)]
        budget_id: String,

        /// Budget limit
        #[arg(long)]
        budget_limit: f64,

        /// Budget currency
        #[arg(long, default_value = "USD")]
        currency: String,

        /// Current spend
        #[arg(long)]
        current_spend: f64,

        /// Execution reference ID (what triggered this evaluation)
        #[arg(long)]
        execution_ref: Option<String>,

        /// Include forecast in evaluation
        #[arg(long, default_value = "false")]
        include_forecast: bool,

        /// Warning threshold (0.0-1.0, default 0.80)
        #[arg(long, default_value = "0.80")]
        warning_threshold: f64,

        /// Critical threshold (0.0-1.0, default 0.95)
        #[arg(long, default_value = "0.95")]
        critical_threshold: f64,

        /// Output format (json, table)
        #[arg(long, default_value = "json")]
        output: String,

        /// RuVector service endpoint (for persistence)
        #[arg(long)]
        ruvector_endpoint: Option<String>,

        /// Dry run (don't persist)
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },

    /// Inspect agent configuration
    Inspect,

    /// Get agent health status
    Health {
        /// Check RuVector connectivity
        #[arg(long, default_value = "false")]
        check_ruvector: bool,
    },
}

#[derive(Subcommand)]
pub enum PricingCommands {
    /// List all pricing tables
    List,

    /// Add a new pricing table
    Add {
        /// Provider name
        #[arg(long)]
        provider: String,

        /// Model name
        #[arg(long)]
        model: String,

        /// Input price per million tokens
        #[arg(long)]
        input_price: f64,

        /// Output price per million tokens
        #[arg(long)]
        output_price: f64,
    },

    /// Get active pricing for a model
    Get {
        /// Provider name
        #[arg(long)]
        provider: String,

        /// Model name
        #[arg(long)]
        model: String,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
