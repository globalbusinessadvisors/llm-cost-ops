# SPARC Phase 3: Architecture

## Table of Contents
1. [System Architecture Overview](#system-architecture-overview)
2. [Component Diagram](#component-diagram)
3. [Rust Crate Recommendations](#rust-crate-recommendations)
4. [Data Models](#data-models)
5. [Deployment Architecture](#deployment-architecture)
6. [Scalability & Performance](#scalability--performance)
7. [Integration Patterns](#integration-patterns)
8. [Security Architecture](#security-architecture)

---

## System Architecture Overview

LLM-CostOps follows a modular, event-driven architecture designed for high-performance cost analysis and forecasting. The system is built around five core subsystems that communicate through well-defined interfaces.

### Core Architectural Principles

1. **Modularity**: Each component operates independently with clear boundaries
2. **Performance**: Leverage Rust's zero-cost abstractions and async runtime
3. **Extensibility**: Plugin-based provider system for easy integration
4. **Observability**: Built-in metrics, tracing, and logging at every layer
5. **Deployment Flexibility**: Support CLI, daemon, and API modes

---

## Component Diagram

```mermaid
graph TB
    subgraph "Ingestion Layer"
        A[API Connectors] --> B[Stream Processor]
        C[File Importers] --> B
        D[Webhook Receivers] --> B
        B --> E[Normalization Engine]
    end

    subgraph "Processing Layer"
        E --> F[Usage Aggregator]
        E --> G[Cost Calculator]
        F --> H[Performance Correlator]
        G --> H
        H --> I[Metrics Collector]
    end

    subgraph "Storage Layer"
        I --> J[(TimescaleDB)]
        I --> K[(RocksDB Cache)]
        I --> L[(Parquet Archive)]
    end

    subgraph "Analysis Layer"
        J --> M[Statistical Engine]
        K --> M
        M --> N[Forecasting Service]
        M --> O[Anomaly Detector]
        M --> P[ROI Calculator]
    end

    subgraph "API Layer"
        N --> Q[REST API / Axum]
        O --> Q
        P --> Q
        N --> R[gRPC Service / Tonic]
        Q --> S[CLI Interface]
        R --> T[Dashboard Client]
    end

    subgraph "Observability"
        U[Prometheus Metrics] -.-> B
        U -.-> F
        U -.-> M
        V[OpenTelemetry Traces] -.-> B
        V -.-> M
        W[Structured Logging] -.-> B
        W -.-> F
        W -.-> M
    end
```

### Component Responsibilities

#### 1. Ingestion Layer
- **API Connectors**: Poll provider APIs (OpenAI, Anthropic, Google, etc.)
- **Stream Processor**: Handle real-time streaming data with backpressure
- **File Importers**: Parse CSV, JSON, and custom formats
- **Webhook Receivers**: Accept push-based notifications
- **Normalization Engine**: Convert provider-specific formats to internal schema

#### 2. Processing Layer
- **Usage Aggregator**: Group and aggregate usage by dimensions (model, user, project)
- **Cost Calculator**: Apply pricing models and compute costs
- **Performance Correlator**: Link usage with performance metrics (latency, quality)
- **Metrics Collector**: Gather system and business metrics

#### 3. Storage Layer
- **TimescaleDB**: Primary time-series storage for historical data
- **RocksDB Cache**: Hot data cache for recent lookups
- **Parquet Archive**: Cold storage for historical analysis

#### 4. Analysis Layer
- **Statistical Engine**: Core statistical operations (aggregations, distributions)
- **Forecasting Service**: Time-series forecasting using multiple algorithms
- **Anomaly Detector**: Identify cost spikes and unusual patterns
- **ROI Calculator**: Compute cost efficiency and return metrics

#### 5. API Layer
- **REST API**: Standard HTTP endpoints for queries and configuration
- **gRPC Service**: High-performance RPC for internal services
- **CLI Interface**: Command-line tools for operations
- **Dashboard Integration**: Data feeds for visualization

---

## Rust Crate Recommendations

### Core Runtime & Async
```toml
[dependencies]
# Async runtime
tokio = { version = "1.37", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec", "time"] }
async-stream = "0.3"
futures = "0.3"

# Async utilities
async-trait = "0.1"
pin-project = "1.1"
```

### Data Ingestion & Streaming
```toml
# HTTP clients for API polling
reqwest = { version = "0.12", features = ["json", "stream"] }
hyper = { version = "1.3", features = ["full"] }

# Kafka/Pulsar for event streaming
rdkafka = { version = "0.36", features = ["ssl", "sasl"] }
# Alternative: pulsar = "6.1"

# Rate limiting and backoff
governor = "0.6"
backoff = "0.4"

# Webhook server
axum = { version = "0.7", features = ["ws", "macros"] }
tower = { version = "0.4", features = ["limit", "buffer"] }
tower-http = { version = "0.5", features = ["trace", "cors"] }
```

### Serialization & Data Formats
```toml
# JSON/serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Binary serialization
bincode = "1.3"
rmp-serde = "1.1"  # MessagePack

# Protocol Buffers
prost = "0.12"
prost-types = "0.12"
tonic = { version = "0.11", features = ["gzip", "tls"] }
tonic-build = "0.11"

# CSV parsing
csv = "1.3"
csv-async = { version = "1.3", features = ["tokio"] }

# Parquet for archival
parquet = "51.0"
arrow = "51.0"
```

### Storage & Databases
```toml
# SQL database (TimescaleDB/PostgreSQL)
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "chrono", "uuid", "json"] }

# Alternative ORM
# diesel = { version = "2.1", features = ["postgres", "chrono", "uuid"] }
# diesel-async = { version = "0.4", features = ["postgres", "deadpool"] }

# Embedded KV store (cache)
rocksdb = "0.22"

# Redis for distributed cache
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }

# Connection pooling
deadpool-postgres = "0.13"
bb8 = "0.8"
```

### Statistical Analysis & ML
```toml
# Numerical computing
ndarray = { version = "0.15", features = ["rayon", "serde"] }
ndarray-stats = "0.5"
statrs = "0.16"

# DataFrames (like pandas)
polars = { version = "0.38", features = ["lazy", "temporal", "performant", "parquet", "csv"] }

# Machine Learning
linfa = "0.7"
linfa-linear = "0.7"
linfa-clustering = "0.7"

# Time-series analysis
# Note: Rust doesn't have direct Prophet equivalent, but we can use:
# - augurs for forecasting: augurs = "0.2"
# - or build custom ARIMA/ETS using statrs + ndarray
augurs = { version = "0.2", features = ["forecasting"] }

# Statistical distributions
rand = "0.8"
rand_distr = "0.4"
```

### Time & Date Handling
```toml
# Date/time handling
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.9"

# Human-readable durations
humantime = "2.1"
humantime-serde = "1.1"
```

### HTTP APIs & Services
```toml
# REST API framework
axum = { version = "0.7", features = ["macros", "multipart", "ws"] }
axum-extra = { version = "0.9", features = ["typed-header"] }

# Alternative: actix-web = { version = "4.5", features = ["macros"] }

# gRPC
tonic = { version = "0.11", features = ["gzip", "tls"] }
tonic-reflection = "0.11"

# API validation
validator = { version = "0.18", features = ["derive"] }

# OpenAPI documentation
utoipa = { version = "4.2", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "6.0", features = ["axum"] }
```

### Metrics & Observability
```toml
# Prometheus metrics
prometheus = "0.13"
prometheus-client = "0.22"

# OpenTelemetry
opentelemetry = { version = "0.22", features = ["trace", "metrics"] }
opentelemetry-otlp = { version = "0.15", features = ["grpc-tonic"] }
opentelemetry-prometheus = "0.15"

# Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.23"
tracing-appender = "0.2"

# Error handling with context
anyhow = "1.0"
thiserror = "1.0"
```

### Reporting & Visualization
```toml
# Chart generation
plotters = { version = "0.3", features = ["all_series", "all_elements"] }
plotters-svg = "0.3"

# Alternative: resvg for SVG rendering
# resvg = "0.41"

# PDF generation
printpdf = "0.7"

# HTML reports
tera = "1.19"  # Template engine
markup = "0.13"  # Type-safe HTML
```

### CLI & Configuration
```toml
# CLI framework
clap = { version = "4.5", features = ["derive", "env", "cargo"] }
clap_complete = "4.5"

# Configuration management
config = "0.14"
figment = { version = "0.10", features = ["toml", "yaml", "env"] }

# Terminal UI
indicatif = "0.17"  # Progress bars
console = "0.15"
colored = "2.1"

# Interactive prompts
dialoguer = "0.11"
```

### Utilities
```toml
# UUID generation
uuid = { version = "1.8", features = ["v4", "serde"] }

# Lazy static
once_cell = "1.19"
lazy_static = "1.4"

# Parallel processing
rayon = "1.10"

# Cron scheduling
cron = "0.12"
tokio-cron-scheduler = "0.10"
```

---

## Data Models

### Core Domain Models

```rust
// File: src/domain/usage.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Primary usage record from LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub provider: Provider,
    pub model: ModelIdentifier,
    pub organization_id: String,
    pub project_id: Option<String>,
    pub user_id: Option<String>,

    // Token usage
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,

    // Additional usage metrics
    pub request_count: u32,
    pub cached_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,  // For models with reasoning

    // Performance correlation
    pub latency_ms: Option<u64>,
    pub time_to_first_token_ms: Option<u64>,

    // Metadata
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,

    // Ingestion tracking
    pub ingested_at: DateTime<Utc>,
    pub source: IngestionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Provider {
    OpenAI,
    Anthropic,
    GoogleVertexAI,
    AzureOpenAI,
    Cohere,
    Mistral,
    TogetherAI,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIdentifier {
    pub name: String,
    pub version: Option<String>,
    pub context_window: u64,
    pub capabilities: Vec<ModelCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCapability {
    TextGeneration,
    Vision,
    FunctionCalling,
    Reasoning,
    Streaming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngestionSource {
    Api { endpoint: String },
    File { path: String },
    Webhook { source: String },
    Stream { topic: String },
}
```

```rust
// File: src/domain/cost.rs

use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Cost model per provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModel {
    pub provider: Provider,
    pub model: String,
    pub effective_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub pricing: PricingStructure,
    pub currency: Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingStructure {
    /// Per-token pricing (most common)
    PerToken {
        input_price_per_million: Decimal,
        output_price_per_million: Decimal,
        cached_input_discount: Option<Decimal>,
    },

    /// Per-request pricing
    PerRequest {
        price_per_request: Decimal,
        included_tokens: u64,
        overage_price_per_million: Decimal,
    },

    /// Tiered pricing
    Tiered {
        tiers: Vec<PricingTier>,
    },

    /// Custom pricing logic
    Custom {
        formula: String,
        parameters: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub min_tokens: u64,
    pub max_tokens: Option<u64>,
    pub input_price_per_million: Decimal,
    pub output_price_per_million: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    Custom(String),
}

/// Computed cost record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    pub usage_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub provider: Provider,
    pub model: String,

    // Cost breakdown
    pub input_cost: Decimal,
    pub output_cost: Decimal,
    pub total_cost: Decimal,
    pub currency: Currency,

    // Cost model reference
    pub cost_model_id: Uuid,
    pub pricing_structure: PricingStructure,

    // Aggregation dimensions
    pub organization_id: String,
    pub project_id: Option<String>,
    pub tags: Vec<String>,

    pub calculated_at: DateTime<Utc>,
}
```

```rust
// File: src/domain/performance.rs

/// Performance metrics correlated with usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub id: Uuid,
    pub usage_id: Uuid,
    pub timestamp: DateTime<Utc>,

    // Latency metrics
    pub total_latency_ms: u64,
    pub time_to_first_token_ms: Option<u64>,
    pub tokens_per_second: Option<f64>,

    // Quality metrics (if available)
    pub quality_score: Option<f64>,
    pub user_rating: Option<u8>,
    pub task_success: Option<bool>,

    // Error tracking
    pub error_occurred: bool,
    pub error_type: Option<String>,
    pub retry_count: u8,

    // Resource utilization
    pub cache_hit: bool,
    pub request_size_bytes: u64,
    pub response_size_bytes: u64,
}

/// Performance-cost correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEfficiency {
    pub metric_id: Uuid,
    pub cost_per_token: Decimal,
    pub cost_per_second: Decimal,
    pub quality_adjusted_cost: Option<Decimal>,
    pub efficiency_score: f64,  // 0-100 scale
}
```

```rust
// File: src/domain/forecast.rs

use ndarray::Array1;

/// Forecast result from time-series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    pub forecast_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub model: ForecastModel,
    pub confidence_level: f64,

    // Forecast horizon
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub granularity: ForecastGranularity,

    // Predictions
    pub predictions: Vec<ForecastPoint>,

    // Model metrics
    pub mape: Option<f64>,  // Mean Absolute Percentage Error
    pub rmse: Option<f64>,  // Root Mean Square Error
    pub r_squared: Option<f64>,

    // Dimensions
    pub dimensions: ForecastDimensions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub timestamp: DateTime<Utc>,
    pub predicted_value: Decimal,
    pub lower_bound: Decimal,
    pub upper_bound: Decimal,
    pub confidence_interval: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForecastModel {
    Linear,
    ARIMA { p: usize, d: usize, q: usize },
    ExponentialSmoothing,
    Prophet,
    Ensemble { models: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForecastGranularity {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastDimensions {
    pub organization_id: Option<String>,
    pub project_id: Option<String>,
    pub provider: Option<Provider>,
    pub model: Option<String>,
    pub tags: Vec<String>,
}
```

```rust
// File: src/domain/roi.rs

/// ROI calculation and optimization insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROICalculation {
    pub calculation_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,

    // Cost metrics
    pub total_cost: Decimal,
    pub cost_breakdown: CostBreakdown,

    // Value metrics
    pub total_requests: u64,
    pub successful_requests: u64,
    pub average_quality_score: Option<f64>,

    // ROI metrics
    pub cost_per_request: Decimal,
    pub cost_per_successful_request: Decimal,
    pub efficiency_ratio: f64,

    // Optimization opportunities
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub potential_savings: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub by_provider: HashMap<Provider, Decimal>,
    pub by_model: HashMap<String, Decimal>,
    pub by_project: HashMap<String, Decimal>,
    pub by_tag: HashMap<String, Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: OptimizationType,
    pub current_cost: Decimal,
    pub optimized_cost: Decimal,
    pub potential_savings: Decimal,
    pub confidence: f64,
    pub description: String,
    pub action_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    ModelDowngrade { from: String, to: String },
    PromptOptimization { estimated_token_reduction: f64 },
    CachingOpportunity { cache_hit_rate_potential: f64 },
    ProviderSwitch { from: Provider, to: Provider },
    BatchingOpportunity { batch_size: usize },
    RateLimitAdjustment,
}
```

### Storage Schema (TimescaleDB)

```sql
-- File: migrations/001_initial_schema.sql

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Usage records (hypertable)
CREATE TABLE usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    model_version TEXT,
    organization_id TEXT NOT NULL,
    project_id TEXT,
    user_id TEXT,

    -- Token counts
    prompt_tokens BIGINT NOT NULL,
    completion_tokens BIGINT NOT NULL,
    total_tokens BIGINT NOT NULL,
    cached_tokens BIGINT,
    reasoning_tokens BIGINT,

    -- Request metrics
    request_count INTEGER NOT NULL DEFAULT 1,
    latency_ms BIGINT,
    time_to_first_token_ms BIGINT,

    -- Metadata
    tags TEXT[],
    metadata JSONB,

    -- Ingestion tracking
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_type TEXT NOT NULL,
    source_metadata JSONB,

    -- Indexes for common queries
    INDEX idx_timestamp ON usage_records (timestamp DESC),
    INDEX idx_provider_model ON usage_records (provider, model_name),
    INDEX idx_organization ON usage_records (organization_id, timestamp DESC),
    INDEX idx_project ON usage_records (project_id, timestamp DESC) WHERE project_id IS NOT NULL,
    INDEX idx_tags ON usage_records USING GIN (tags)
);

-- Convert to hypertable (partitioned by time)
SELECT create_hypertable('usage_records', 'timestamp');

-- Cost models
CREATE TABLE cost_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    effective_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ,
    pricing_structure JSONB NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (provider, model_name, effective_date)
);

-- Cost records (hypertable)
CREATE TABLE cost_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    usage_id UUID REFERENCES usage_records(id),
    timestamp TIMESTAMPTZ NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,

    input_cost DECIMAL(20, 10) NOT NULL,
    output_cost DECIMAL(20, 10) NOT NULL,
    total_cost DECIMAL(20, 10) NOT NULL,
    currency TEXT NOT NULL,

    cost_model_id UUID REFERENCES cost_models(id),
    organization_id TEXT NOT NULL,
    project_id TEXT,
    tags TEXT[],

    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    INDEX idx_cost_timestamp ON cost_records (timestamp DESC),
    INDEX idx_cost_organization ON cost_records (organization_id, timestamp DESC)
);

SELECT create_hypertable('cost_records', 'timestamp');

-- Continuous aggregates for common queries
CREATE MATERIALIZED VIEW hourly_cost_summary
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', timestamp) AS bucket,
    provider,
    model_name,
    organization_id,
    project_id,
    SUM(total_cost) AS total_cost,
    SUM(prompt_tokens + completion_tokens) AS total_tokens,
    COUNT(*) AS request_count,
    AVG(latency_ms) AS avg_latency_ms
FROM usage_records ur
JOIN cost_records cr ON ur.id = cr.usage_id
GROUP BY bucket, provider, model_name, organization_id, project_id;

-- Forecasts
CREATE TABLE forecasts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    model_type TEXT NOT NULL,
    confidence_level FLOAT NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL,
    granularity TEXT NOT NULL,
    mape FLOAT,
    rmse FLOAT,
    r_squared FLOAT,
    dimensions JSONB NOT NULL,

    INDEX idx_forecast_dates ON forecasts (start_date, end_date)
);

CREATE TABLE forecast_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    forecast_id UUID REFERENCES forecasts(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL,
    predicted_value DECIMAL(20, 10) NOT NULL,
    lower_bound DECIMAL(20, 10) NOT NULL,
    upper_bound DECIMAL(20, 10) NOT NULL,
    confidence_interval FLOAT NOT NULL,

    INDEX idx_forecast_points ON forecast_points (forecast_id, timestamp)
);
```

---

## Deployment Architecture

### Deployment Modes

LLM-CostOps supports multiple deployment modes to fit different operational needs:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Deployment Modes                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Standalone Daemon Mode                                     │
│     ┌──────────────────────────────────────────┐              │
│     │  llm-costops daemon                      │              │
│     │  ├─ Ingestion Workers                    │              │
│     │  ├─ Processing Pipeline                  │              │
│     │  ├─ Storage Management                   │              │
│     │  ├─ Analysis Engine                      │              │
│     │  └─ Local API (optional)                 │              │
│     └──────────────────────────────────────────┘              │
│                                                                 │
│  2. CLI Utility Mode                                           │
│     $ llm-costops import --file usage.csv                      │
│     $ llm-costops report --period last-30-days                 │
│     $ llm-costops forecast --model arima --horizon 90          │
│                                                                 │
│  3. API Microservice Mode                                      │
│     ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│     │  Ingestion   │  │  Processing  │  │   Analysis   │     │
│     │  Service     │  │  Service     │  │   Service    │     │
│     └──────────────┘  └──────────────┘  └──────────────┘     │
│            │                  │                  │             │
│            └──────────────────┴──────────────────┘             │
│                          │                                     │
│                   ┌──────────────┐                             │
│                   │  API Gateway │                             │
│                   └──────────────┘                             │
│                                                                 │
│  4. Hybrid Mode                                                │
│     - Daemon for continuous ingestion                          │
│     - API for external queries                                 │
│     - CLI for administrative tasks                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Mode 1: Standalone Daemon

**Use Case**: Single-tenant deployments, development, small teams

```yaml
# config/daemon.yaml
daemon:
  mode: standalone
  workers:
    ingestion: 4
    processing: 8
    analysis: 2

  ingestion:
    sources:
      - type: api_poll
        provider: openai
        interval: 5m
      - type: api_poll
        provider: anthropic
        interval: 5m
      - type: webhook
        port: 8080
        path: /webhooks/usage

  storage:
    timescaledb:
      url: postgresql://localhost/llm_costops
      connection_pool: 20
    rocksdb:
      path: /var/lib/llm-costops/cache
      cache_size_mb: 1024

  api:
    enabled: true
    bind: 127.0.0.1:3000
    auth:
      type: api_key
```

**Binary Execution**:
```bash
# Start daemon
llm-costops daemon --config config/daemon.yaml

# Run in background
llm-costops daemon --config config/daemon.yaml --daemonize

# With systemd
systemctl start llm-costops
```

### Mode 2: CLI Utility

**Use Case**: Ad-hoc analysis, batch processing, scripting

```bash
# Import historical data
llm-costops import \
  --provider openai \
  --format csv \
  --file usage-2024-01.csv

# Generate reports
llm-costops report \
  --type cost-summary \
  --period 2024-01-01..2024-01-31 \
  --group-by model,project \
  --output report.html

# Run forecasts
llm-costops forecast \
  --model arima \
  --horizon 90 \
  --dimensions organization=acme,provider=openai \
  --output forecast.json

# Calculate ROI
llm-costops roi \
  --period last-30-days \
  --compare-with previous-30-days \
  --suggestions

# Export data
llm-costops export \
  --format parquet \
  --period 2024-Q1 \
  --output archive/2024-q1.parquet
```

### Mode 3: API Microservice

**Use Case**: Multi-tenant SaaS, enterprise deployments, high scale

```
┌─────────────────────────────────────────────────────────────────┐
│                    Microservice Architecture                    │
└─────────────────────────────────────────────────────────────────┘

                          ┌──────────────┐
                          │ Load Balancer│
                          └──────┬───────┘
                                 │
                    ┌────────────┼────────────┐
                    │                         │
          ┌─────────▼────────┐     ┌─────────▼────────┐
          │  API Gateway     │     │  API Gateway     │
          │  (Axum + gRPC)   │     │  (Axum + gRPC)   │
          └─────────┬────────┘     └─────────┬────────┘
                    │                         │
        ┌───────────┼─────────────────────────┼──────────┐
        │           │                         │          │
┌───────▼──────┐ ┌──▼────────┐ ┌────────────▼───┐ ┌────▼───────┐
│ Ingestion    │ │Processing │ │  Analysis      │ │  Export    │
│ Service      │ │Service    │ │  Service       │ │  Service   │
│              │ │           │ │                │ │            │
│ - API polls  │ │- Normalize│ │ - Forecasting  │ │ - Parquet  │
│ - Webhooks   │ │- Aggregate│ │ - Anomaly Det. │ │ - CSV      │
│ - Streaming  │ │- Cost calc│ │ - ROI calc     │ │ - Reports  │
└───────┬──────┘ └──┬────────┘ └────────────┬───┘ └────┬───────┘
        │           │                       │          │
        └───────────┼───────────────────────┼──────────┘
                    │                       │
           ┌────────▼────────┐     ┌────────▼────────┐
           │  TimescaleDB    │     │  RocksDB/Redis  │
           │  (Primary)      │     │  (Cache)        │
           └─────────────────┘     └─────────────────┘
```

**Service Definitions**:

```toml
# services/ingestion/Cargo.toml
[package]
name = "llm-costops-ingestion"
version = "0.1.0"

[dependencies]
# Service framework
tokio = { version = "1.37", features = ["full"] }
tonic = "0.11"
axum = "0.7"

# Shared domain
llm-costops-core = { path = "../../core" }
llm-costops-domain = { path = "../../domain" }

# Ingestion-specific
rdkafka = "0.36"
reqwest = "0.12"
```

**Docker Compose Deployment**:

```yaml
# docker-compose.yml
version: '3.8'

services:
  timescaledb:
    image: timescale/timescaledb:latest-pg15
    environment:
      POSTGRES_DB: llm_costops
      POSTGRES_USER: costops
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - timescale_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  ingestion:
    build:
      context: .
      dockerfile: services/ingestion/Dockerfile
    environment:
      DATABASE_URL: postgresql://costops:${DB_PASSWORD}@timescaledb/llm_costops
      REDIS_URL: redis://redis:6379
    depends_on:
      - timescaledb
      - redis
    deploy:
      replicas: 3

  processing:
    build:
      context: .
      dockerfile: services/processing/Dockerfile
    environment:
      DATABASE_URL: postgresql://costops:${DB_PASSWORD}@timescaledb/llm_costops
      REDIS_URL: redis://redis:6379
    depends_on:
      - timescaledb
      - redis
    deploy:
      replicas: 4

  analysis:
    build:
      context: .
      dockerfile: services/analysis/Dockerfile
    environment:
      DATABASE_URL: postgresql://costops:${DB_PASSWORD}@timescaledb/llm_costops
      REDIS_URL: redis://redis:6379
    depends_on:
      - timescaledb
      - redis
    deploy:
      replicas: 2

  api-gateway:
    build:
      context: .
      dockerfile: services/api/Dockerfile
    ports:
      - "3000:3000"
      - "50051:50051"  # gRPC
    environment:
      DATABASE_URL: postgresql://costops:${DB_PASSWORD}@timescaledb/llm_costops
      REDIS_URL: redis://redis:6379
    depends_on:
      - ingestion
      - processing
      - analysis

volumes:
  timescale_data:
```

### Mode 4: Hybrid Deployment

**Use Case**: Flexibility for varied workloads

```yaml
# Daemon for continuous ingestion
daemon:
  mode: hybrid
  components:
    ingestion: enabled
    processing: enabled
    analysis: disabled  # Run on-demand via CLI
    api: enabled

# API for external queries
api:
  bind: 0.0.0.0:3000
  grpc_bind: 0.0.0.0:50051

# CLI for administrative tasks and analysis
cli:
  default_output: json
  pager: less
```

---

## Scalability & Performance

### Data Partitioning Strategies

#### Time-based Partitioning (TimescaleDB Hypertables)

```sql
-- Automatic partitioning by time
SELECT create_hypertable('usage_records', 'timestamp',
    chunk_time_interval => INTERVAL '1 day');

-- Compression for old data
ALTER TABLE usage_records SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'provider,model_name,organization_id',
    timescaledb.compress_orderby = 'timestamp DESC'
);

-- Auto-compress chunks older than 7 days
SELECT add_compression_policy('usage_records', INTERVAL '7 days');

-- Retention policy: drop data older than 2 years
SELECT add_retention_policy('usage_records', INTERVAL '2 years');
```

#### Sharding Strategy for Multi-tenant

```rust
// Organization-based sharding
pub enum ShardKey {
    Organization(String),
    Provider(Provider),
    Composite { org: String, provider: Provider },
}

pub struct ShardRouter {
    shards: Vec<DatabasePool>,
    strategy: ShardStrategy,
}

impl ShardRouter {
    pub fn route(&self, key: &ShardKey) -> &DatabasePool {
        let shard_id = match &self.strategy {
            ShardStrategy::Hash => {
                let hash = hash_key(key);
                hash % self.shards.len()
            },
            ShardStrategy::Range => {
                // Range-based routing for ordered access
                self.find_range_shard(key)
            },
        };
        &self.shards[shard_id]
    }
}
```

### Caching Layers

```rust
// Multi-level cache architecture
pub struct CacheLayer {
    // L1: In-memory cache (fast, small)
    l1: Arc<RwLock<LruCache<String, CachedValue>>>,

    // L2: Redis (distributed, medium)
    l2: RedisPool,

    // L3: RocksDB (local persistent, large)
    l3: Arc<rocksdb::DB>,
}

impl CacheLayer {
    pub async fn get(&self, key: &str) -> Option<CachedValue> {
        // Try L1 first
        if let Some(value) = self.l1.read().await.get(key) {
            return Some(value.clone());
        }

        // Try L2
        if let Ok(Some(value)) = self.l2.get(key).await {
            self.l1.write().await.put(key.to_string(), value.clone());
            return Some(value);
        }

        // Try L3
        if let Ok(Some(bytes)) = self.l3.get(key) {
            let value = deserialize(&bytes)?;
            self.l2.set(key, &value).await.ok();
            self.l1.write().await.put(key.to_string(), value.clone());
            return Some(value);
        }

        None
    }
}

// Cache warming for common queries
pub struct CacheWarmer {
    cache: Arc<CacheLayer>,
    db: DatabasePool,
}

impl CacheWarmer {
    pub async fn warm_recent_costs(&self) {
        // Pre-populate cache with last 24h aggregates
        let results = self.db
            .query("SELECT * FROM hourly_cost_summary
                    WHERE bucket > NOW() - INTERVAL '24 hours'")
            .await?;

        for row in results {
            let key = format!("cost:hourly:{}", row.bucket);
            self.cache.set(&key, &row).await;
        }
    }
}
```

### Stream Processing vs Batch

```rust
// Stream processing for real-time ingestion
pub struct StreamProcessor {
    kafka_consumer: StreamConsumer,
    processing_pipeline: Pipeline,
    batch_size: usize,
    flush_interval: Duration,
}

impl StreamProcessor {
    pub async fn run(&mut self) {
        let mut batch = Vec::with_capacity(self.batch_size);
        let mut last_flush = Instant::now();

        loop {
            tokio::select! {
                // Receive messages
                message = self.kafka_consumer.recv() => {
                    if let Ok(msg) = message {
                        batch.push(msg);

                        // Flush on batch size
                        if batch.len() >= self.batch_size {
                            self.flush_batch(&mut batch).await;
                            last_flush = Instant::now();
                        }
                    }
                }

                // Flush on interval
                _ = tokio::time::sleep_until(last_flush + self.flush_interval) => {
                    if !batch.is_empty() {
                        self.flush_batch(&mut batch).await;
                        last_flush = Instant::now();
                    }
                }
            }
        }
    }

    async fn flush_batch(&self, batch: &mut Vec<UsageRecord>) {
        // Bulk insert for performance
        self.processing_pipeline.process_batch(batch).await;
        batch.clear();
    }
}

// Batch processing for historical analysis
pub struct BatchProcessor {
    db: DatabasePool,
    chunk_size: usize,
}

impl BatchProcessor {
    pub async fn process_historical_data(&self, start: DateTime<Utc>, end: DateTime<Utc>) {
        // Process in parallel chunks
        let chunks = self.partition_time_range(start, end, self.chunk_size);

        let tasks: Vec<_> = chunks.into_iter()
            .map(|(chunk_start, chunk_end)| {
                let db = self.db.clone();
                tokio::spawn(async move {
                    process_time_range(db, chunk_start, chunk_end).await
                })
            })
            .collect();

        // Wait for all chunks
        futures::future::join_all(tasks).await;
    }
}
```

### Horizontal Scaling

```
┌─────────────────────────────────────────────────────────────────┐
│                    Horizontal Scaling Strategy                  │
└─────────────────────────────────────────────────────────────────┘

Ingestion Tier (Stateless, Scale by load)
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│Ingestion │ │Ingestion │ │Ingestion │ │Ingestion │
│Worker 1  │ │Worker 2  │ │Worker 3  │ │Worker N  │
└────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘
     │            │            │            │
     └────────────┴────────────┴────────────┘
                  │
          ┌───────▼────────┐
          │ Message Queue  │
          │ (Kafka/Redis)  │
          └───────┬────────┘
                  │
Processing Tier (Stateless, Scale by throughput)
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│Processing│ │Processing│ │Processing│ │Processing│
│Worker 1  │ │Worker 2  │ │Worker 3  │ │Worker N  │
└────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘
     │            │            │            │
     └────────────┴────────────┴────────────┘
                  │
Storage Tier (Sharded, Scale by data volume)
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│  Shard 1 │ │  Shard 2 │ │  Shard 3 │ │  Shard N │
│(Org A-F) │ │(Org G-M) │ │(Org N-S) │ │(Org T-Z) │
└──────────┘ └──────────┘ └──────────┘ └──────────┘

Analysis Tier (Compute-intensive, Scale by workload)
┌──────────┐ ┌──────────┐ ┌──────────┐
│Forecast  │ │ Anomaly  │ │   ROI    │
│Service   │ │ Detector │ │ Calculator│
└──────────┘ └──────────┘ └──────────┘
```

**Auto-scaling Configuration** (Kubernetes):

```yaml
# k8s/ingestion-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ingestion-worker
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ingestion-worker
  minReplicas: 2
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Pods
    pods:
      metric:
        name: kafka_consumer_lag
      target:
        type: AverageValue
        averageValue: "1000"
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Pods
        value: 1
        periodSeconds: 60
```

### Performance Benchmarks & Targets

| Operation | Target | Strategy |
|-----------|--------|----------|
| Ingest rate | 100K records/sec | Stream processing, batching |
| Query latency (recent) | <100ms | L1/L2 cache, indexed queries |
| Query latency (historical) | <2s | Continuous aggregates, compression |
| Forecast generation | <5s | Pre-computed models, incremental updates |
| API response time (p95) | <200ms | Caching, connection pooling |
| Storage efficiency | 10:1 compression | TimescaleDB compression, Parquet |

---

## Integration Patterns

### Async Message Processing

```rust
// Event-driven architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostOpsEvent {
    UsageRecorded {
        record: UsageRecord,
        timestamp: DateTime<Utc>,
    },
    CostCalculated {
        cost: CostRecord,
        timestamp: DateTime<Utc>,
    },
    AnomalyDetected {
        anomaly: Anomaly,
        severity: Severity,
        timestamp: DateTime<Utc>,
    },
    ForecastUpdated {
        forecast_id: Uuid,
        timestamp: DateTime<Utc>,
    },
}

pub struct EventBus {
    producer: KafkaProducer,
}

impl EventBus {
    pub async fn publish(&self, event: CostOpsEvent) -> Result<()> {
        let topic = match &event {
            CostOpsEvent::UsageRecorded { .. } => "usage-events",
            CostOpsEvent::CostCalculated { .. } => "cost-events",
            CostOpsEvent::AnomalyDetected { .. } => "anomaly-events",
            CostOpsEvent::ForecastUpdated { .. } => "forecast-events",
        };

        let payload = serde_json::to_vec(&event)?;
        self.producer.send(topic, &payload).await?;
        Ok(())
    }
}

// Event handlers
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: CostOpsEvent) -> Result<()>;
}

pub struct AnomalyAlertHandler {
    alerting_service: Arc<AlertingService>,
}

#[async_trait]
impl EventHandler for AnomalyAlertHandler {
    async fn handle(&self, event: CostOpsEvent) -> Result<()> {
        if let CostOpsEvent::AnomalyDetected { anomaly, severity, .. } = event {
            if severity >= Severity::High {
                self.alerting_service.send_alert(anomaly).await?;
            }
        }
        Ok(())
    }
}
```

### REST API Design

```rust
// File: src/api/rest/mod.rs

use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Query, State},
    Json,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health & metrics
        .route("/health", get(health_check))
        .route("/metrics", get(prometheus_metrics))

        // Usage endpoints
        .route("/api/v1/usage", post(ingest_usage))
        .route("/api/v1/usage/:id", get(get_usage))
        .route("/api/v1/usage/search", post(search_usage))

        // Cost endpoints
        .route("/api/v1/costs/summary", get(cost_summary))
        .route("/api/v1/costs/breakdown", get(cost_breakdown))
        .route("/api/v1/costs/trends", get(cost_trends))

        // Forecast endpoints
        .route("/api/v1/forecasts", post(create_forecast))
        .route("/api/v1/forecasts/:id", get(get_forecast))
        .route("/api/v1/forecasts/latest", get(latest_forecast))

        // ROI endpoints
        .route("/api/v1/roi/calculate", post(calculate_roi))
        .route("/api/v1/roi/optimizations", get(get_optimizations))

        .with_state(state)
}

// Example handler
async fn cost_summary(
    Query(params): Query<CostSummaryParams>,
    State(state): State<AppState>,
) -> Result<Json<CostSummaryResponse>, ApiError> {
    let summary = state.cost_service
        .get_summary(params.period, params.dimensions)
        .await?;

    Ok(Json(summary))
}
```

### gRPC Service Definition

```protobuf
// proto/costops.proto

syntax = "proto3";
package costops.v1;

service CostOpsService {
  // Ingestion
  rpc IngestUsage(UsageRecordRequest) returns (IngestResponse);
  rpc IngestUsageBatch(stream UsageRecordRequest) returns (BatchIngestResponse);

  // Queries
  rpc GetCostSummary(CostSummaryRequest) returns (CostSummaryResponse);
  rpc GetForecast(ForecastRequest) returns (ForecastResponse);
  rpc GetOptimizations(OptimizationRequest) returns (OptimizationResponse);

  // Streaming
  rpc StreamCostUpdates(StreamRequest) returns (stream CostUpdate);
  rpc StreamAnomalies(StreamRequest) returns (stream AnomalyAlert);
}

message UsageRecordRequest {
  string provider = 1;
  string model = 2;
  int64 prompt_tokens = 3;
  int64 completion_tokens = 4;
  string organization_id = 5;
  map<string, string> metadata = 6;
}

message CostSummaryRequest {
  string period_start = 1;
  string period_end = 2;
  repeated string dimensions = 3;
  string organization_id = 4;
}
```

---

## Security Architecture

### Authentication & Authorization

```rust
// API key authentication
pub struct ApiKeyAuth {
    keys: Arc<RwLock<HashMap<String, ApiKeyMetadata>>>,
}

pub struct ApiKeyMetadata {
    pub organization_id: String,
    pub scopes: Vec<Scope>,
    pub rate_limit: RateLimit,
    pub expires_at: Option<DateTime<Utc>>,
}

// Middleware
pub async fn auth_middleware(
    State(auth): State<Arc<ApiKeyAuth>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let api_key = extract_api_key(&request)?;
    let metadata = auth.validate_key(&api_key).await?;

    // Add to request extensions
    request.extensions_mut().insert(metadata);

    Ok(next.run(request).await)
}

// Row-level security (RLS) in queries
pub fn apply_rls(
    query: QueryBuilder,
    user: &ApiKeyMetadata,
) -> QueryBuilder {
    query.and_where("organization_id = ?", &user.organization_id)
}
```

### Data Encryption

```rust
// Encryption at rest (sensitive fields)
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>> {
        // Encrypt sensitive data before storage
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_bytes())?;
        Ok(ciphertext)
    }
}

// TLS for data in transit
pub fn configure_tls() -> TlsConfig {
    TlsConfig::builder()
        .cert_path("certs/server.crt")
        .key_path("certs/server.key")
        .client_ca_path("certs/ca.crt")
        .build()
}
```

---

## Summary

This architecture provides:

1. **Modular Design**: Clean separation of concerns with well-defined boundaries
2. **High Performance**: Leveraging Rust's efficiency, async I/O, and TimescaleDB
3. **Scalability**: Horizontal scaling at every tier with proven patterns
4. **Flexibility**: Multiple deployment modes for different use cases
5. **Observability**: Built-in metrics, tracing, and logging
6. **Extensibility**: Plugin architecture for new providers and features

The system is designed to handle:
- **Scale**: 100K+ records/second ingestion
- **Data Volume**: Petabytes with compression and archival
- **Query Performance**: Sub-second for recent data, seconds for historical
- **Real-time**: Streaming processing with <1s latency
- **Forecasting**: Multiple time-series algorithms with confidence intervals

Next steps: Proceed to **SPARC Phase 4: Refinement** for implementation details, testing strategies, and optimization plans.
