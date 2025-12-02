//! LLM-CostOps CLI - Command-line interface
//!
//! This crate provides the command-line interface for LLM Cost Operations,
//! including database initialization, data ingestion, querying, and reporting.

pub mod cli;
pub mod benchmarks;

// Re-export CLI types
pub use cli::{Cli, Commands, PricingCommands};
pub use benchmarks::run_all_benchmarks;
