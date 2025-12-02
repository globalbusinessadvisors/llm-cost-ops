use clap::{Parser, Subcommand};
use std::path::PathBuf;

// CLI argument definitions for LLM Cost Operations

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
