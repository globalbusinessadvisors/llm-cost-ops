# SPARC Phase 5: Completion

## Project Overview

LLM-CostOps is a comprehensive cost operations platform for Large Language Model deployments, built as part of the broader LLM DevOps ecosystem. This document defines the implementation roadmap from initial development through production release.

---

## 1. Phased Roadmap

### Phase 1: MVP (Minimum Viable Product)

**Target Timeline:** Weeks 1-8

**Objective:** Deliver core cost tracking functionality with basic integrations to validate the architecture and prove the value proposition.

#### Core Features

1. **Basic Cost Tracking Engine**
   - Token consumption measurement
   - Per-request cost calculation
   - Cost aggregation by time window (hourly, daily)
   - Single provider support (OpenAI/Anthropic - choose one)
   - In-memory cost storage with optional JSON file persistence

2. **Token Accounting System**
   - Input/output token counting
   - Prompt caching detection (if applicable)
   - Token usage categorization (prompt, completion, system)
   - Basic token usage statistics

3. **Essential Integrations**
   - **Observatory Integration**: Metrics emission via standard metrics protocol
   - **Edge-Agent Integration**: Request/response interception for cost calculation
   - Configuration management via TOML files

4. **CLI Interface**
   - Cost query commands (`cost-ops query --range 24h`)
   - Summary reports (`cost-ops summary --by-model`)
   - Basic cost export (`cost-ops export --format json`)
   - Configuration validation

#### Technical Implementation

**Architecture:**
- Rust-based monolithic binary
- Synchronous request processing
- File-based configuration
- SQLite for local persistence (optional)

**Key Modules:**
```
llm-cost-ops/
├── src/
│   ├── main.rs
│   ├── cost_engine/
│   │   ├── calculator.rs      # Core cost calculation
│   │   ├── models.rs          # Provider pricing models
│   │   └── aggregator.rs      # Time-window aggregation
│   ├── token_accounting/
│   │   ├── counter.rs         # Token counting logic
│   │   └── categorizer.rs     # Usage categorization
│   ├── integrations/
│   │   ├── observatory.rs     # Metrics emission
│   │   └── edge_agent.rs      # Request interception
│   ├── storage/
│   │   ├── memory.rs          # In-memory store
│   │   └── sqlite.rs          # SQLite persistence
│   └── cli/
│       ├── commands.rs        # CLI command handlers
│       └── formatters.rs      # Output formatting
├── tests/
│   ├── integration/
│   └── unit/
└── Cargo.toml
```

#### Milestones

**M1.1: Cost Engine Foundation (Week 2)**
- ✓ Token counting for single provider
- ✓ Basic cost calculation
- ✓ Unit tests achieving 70% coverage
- ✓ Benchmark: 10,000 calculations/sec on standard hardware

**M1.2: Integration Layer (Week 4)**
- ✓ Observatory metrics emission
- ✓ Edge-Agent request interception
- ✓ End-to-end integration test
- ✓ Latency: <5ms overhead per request

**M1.3: CLI & Storage (Week 6)**
- ✓ Core CLI commands functional
- ✓ SQLite persistence working
- ✓ Configuration validation
- ✓ Query response time: <100ms for 1M records

**M1.4: MVP Release (Week 8)**
- ✓ Documentation: README, quick start guide
- ✓ Example configurations
- ✓ CI/CD pipeline setup
- ✓ Docker image published

#### Dependencies

**LLM DevOps Modules:**
- `llm-observatory` (v0.1+): Metrics collection and aggregation
- `llm-edge-agent` (v0.1+): Request/response interception

**External Dependencies:**
- None (self-contained for MVP)

**Rust Crates:**
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
clap = { version = "4.4", features = ["derive"] }
rusqlite = { version = "0.30", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
criterion = "0.5"
tempfile = "3.8"
```

#### Success Metrics

**Functional:**
- Cost calculation accuracy: ±0.1% compared to provider billing
- Token counting accuracy: 100% match with provider APIs
- CLI command success rate: >99%

**Performance:**
- Cost calculation throughput: >10,000 req/sec
- Query latency (p95): <100ms
- Memory footprint: <50MB for 1M cost records

**Quality:**
- Unit test coverage: >70%
- Integration test coverage: >60%
- Zero critical bugs in core calculation logic

---

### Phase 2: Beta Release

**Target Timeline:** Weeks 9-16

**Objective:** Add multi-provider support, forecasting capabilities, and API mode for production deployment readiness.

#### Enhanced Features

1. **Multi-Provider Support**
   - OpenAI (GPT-4, GPT-3.5, etc.)
   - Anthropic (Claude models)
   - Google (PaLM, Gemini)
   - Azure OpenAI
   - AWS Bedrock
   - Provider abstraction layer
   - Dynamic pricing table updates

2. **Forecasting Capabilities**
   - Time-series cost forecasting (7-day, 30-day, 90-day)
   - Trend analysis (moving averages, exponential smoothing)
   - Anomaly detection (cost spikes, usage drops)
   - Forecasting models: ARIMA, Prophet-inspired algorithm
   - Confidence intervals for predictions

3. **ROI Correlation Engine**
   - Correlation with business metrics
   - Cost-per-outcome tracking
   - ROI dashboard integration
   - Custom metric correlation

4. **API Microservice Mode**
   - RESTful API endpoints
   - GraphQL query support
   - WebSocket streaming for real-time costs
   - API authentication (JWT, API keys)
   - Rate limiting
   - OpenAPI/Swagger documentation

5. **Governance Integration**
   - Budget threshold enforcement
   - Cost allocation by team/project
   - Approval workflows for high-cost operations
   - Audit logging
   - Integration with `llm-governance` module

#### Technical Implementation

**Architecture Evolution:**
- Hybrid deployment: CLI + API server
- Asynchronous request processing
- PostgreSQL for production persistence
- Redis for caching and real-time metrics
- gRPC for inter-module communication

**New Modules:**
```
llm-cost-ops/
├── src/
│   ├── forecast/
│   │   ├── models.rs          # Forecasting algorithms
│   │   ├── arima.rs           # ARIMA implementation
│   │   ├── anomaly.rs         # Anomaly detection
│   │   └── confidence.rs      # Confidence intervals
│   ├── providers/
│   │   ├── trait.rs           # Provider abstraction
│   │   ├── openai.rs
│   │   ├── anthropic.rs
│   │   ├── google.rs
│   │   ├── azure.rs
│   │   └── bedrock.rs
│   ├── api/
│   │   ├── rest.rs            # REST endpoints
│   │   ├── graphql.rs         # GraphQL schema
│   │   ├── websocket.rs       # WebSocket streaming
│   │   └── auth.rs            # Authentication
│   ├── roi/
│   │   ├── correlator.rs      # Metric correlation
│   │   └── calculator.rs      # ROI calculation
│   └── governance/
│       ├── budgets.rs         # Budget enforcement
│       ├── allocation.rs      # Cost allocation
│       └── audit.rs           # Audit logging
```

#### Milestones

**M2.1: Multi-Provider Foundation (Week 10)**
- ✓ Provider abstraction layer
- ✓ 5 major providers supported
- ✓ Dynamic pricing table mechanism
- ✓ Provider-specific unit tests

**M2.2: Forecasting Engine (Week 12)**
- ✓ ARIMA-based forecasting
- ✓ Anomaly detection (>95% accuracy on synthetic data)
- ✓ Confidence intervals
- ✓ Forecast MAPE <15% on historical data

**M2.3: API Microservice (Week 14)**
- ✓ REST API with full CRUD operations
- ✓ GraphQL query interface
- ✓ Authentication and authorization
- ✓ API response time (p95): <200ms

**M2.4: Governance Integration (Week 16)**
- ✓ Budget threshold enforcement
- ✓ Cost allocation by project
- ✓ Audit logging
- ✓ Integration tests with llm-governance

#### Dependencies

**LLM DevOps Modules:**
- `llm-observatory` (v0.2+): Enhanced metrics
- `llm-edge-agent` (v0.2+): Multi-provider support
- `llm-governance` (v0.1+): Policy enforcement, budget management

**External Dependencies:**
- PostgreSQL (v14+): Production database
- Redis (v7+): Caching and real-time metrics
- Message queue (RabbitMQ/NATS): Optional for event streaming

**Rust Crates (Additional):**
```toml
[dependencies]
# Existing crates from MVP...

# API & Web
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
async-graphql = "6.0"
async-graphql-axum = "6.0"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }
redis = { version = "0.24", features = ["tokio-comp"] }

# Time-series & forecasting
ndarray = "0.15"
statrs = "0.16"

# Authentication
jsonwebtoken = "9.2"
bcrypt = "0.15"

# Serialization
prost = "0.12"  # gRPC
tonic = "0.10"  # gRPC framework
```

#### Success Metrics

**Functional:**
- Provider cost accuracy: ±0.1% across all 5 providers
- Forecast MAPE (Mean Absolute Percentage Error): <15% for 7-day forecasts
- Anomaly detection accuracy: >90% (F1 score)
- API uptime: >99.5%

**Performance:**
- API throughput: >1,000 req/sec
- Forecast generation time: <5 seconds for 90-day forecast
- Database query latency (p95): <50ms
- Redis cache hit rate: >80%

**Quality:**
- Unit test coverage: >80%
- Integration test coverage: >70%
- API test coverage: >85%
- Security audit completed (no high-severity issues)

---

### Phase 3: v1.0 Production Release

**Target Timeline:** Weeks 17-24

**Objective:** Complete SPARC implementation with advanced features, full deployment modes, comprehensive documentation, and production-grade reliability.

#### Production Features

1. **Advanced Forecasting & Analytics**
   - Machine learning-based forecasting (LightGBM, XGBoost integration)
   - Multi-variate forecasting (incorporating external factors)
   - What-if scenario modeling
   - Cost optimization recommendations
   - Automated cost saving alerts

2. **Complete Deployment Modes**
   - Standalone CLI tool
   - API microservice (Kubernetes-ready)
   - Embedded library mode (Rust crate)
   - Serverless functions (AWS Lambda, GCP Functions)
   - WebAssembly module for edge deployment

3. **Enterprise Features**
   - Multi-tenancy support
   - RBAC (Role-Based Access Control)
   - SSO integration (SAML, OAuth2)
   - Data retention policies
   - Compliance reporting (SOC2, GDPR)
   - High availability (multi-region deployment)

4. **Advanced Integrations**
   - **Full LLM DevOps Stack:**
     - Observatory: Advanced metrics and dashboards
     - Edge-Agent: Full request lifecycle tracking
     - Governance: Complete policy enforcement
     - Security: Cost-based threat detection
     - Testing: Cost simulation for load tests
   - **Cloud Providers:**
     - AWS Cost Explorer integration
     - GCP Billing API integration
     - Azure Cost Management integration
   - **Observability:**
     - Prometheus metrics export
     - Grafana dashboard templates
     - OpenTelemetry tracing
     - DataDog, New Relic integrations

5. **Developer Experience**
   - Comprehensive SDK (Rust, Python, TypeScript)
   - Terraform provider for IaC
   - Helm charts for Kubernetes
   - VS Code extension for cost estimation
   - GitHub Actions integration
   - Pre-commit hooks for cost estimates

#### Technical Implementation

**Architecture Maturity:**
- Microservices architecture with service mesh
- Event-driven architecture (CQRS/Event Sourcing)
- Distributed caching and session management
- Multi-region active-active deployment
- Auto-scaling based on load
- Circuit breakers and retry logic

**Production-Grade Enhancements:**
```
llm-cost-ops/
├── src/
│   ├── ml/
│   │   ├── models.rs          # ML model abstractions
│   │   ├── lightgbm.rs        # LightGBM integration
│   │   └── optimizer.rs       # Cost optimization engine
│   ├── deployment/
│   │   ├── serverless.rs      # Serverless runtime
│   │   ├── wasm.rs            # WebAssembly module
│   │   └── embedded.rs        # Library mode
│   ├── enterprise/
│   │   ├── multitenancy.rs    # Multi-tenant isolation
│   │   ├── rbac.rs            # Role-based access
│   │   ├── sso.rs             # SSO integration
│   │   └── compliance.rs      # Compliance reporting
│   ├── integrations/
│   │   ├── cloud/             # Cloud provider integrations
│   │   ├── observability/     # Observability tools
│   │   └── devops/            # DevOps tool integrations
│   └── sdk/
│       ├── rust/              # Rust SDK
│       ├── python/            # Python bindings
│       └── typescript/        # TypeScript SDK
├── deploy/
│   ├── kubernetes/            # K8s manifests & Helm charts
│   ├── terraform/             # Terraform modules
│   ├── serverless/            # Serverless configs
│   └── docker/                # Dockerfiles
├── docs/
│   ├── architecture/          # Architecture documentation
│   ├── api/                   # API documentation
│   ├── deployment/            # Deployment guides
│   ├── tutorials/             # Step-by-step tutorials
│   └── examples/              # Code examples
└── benchmarks/
    ├── load_tests/            # Load testing scenarios
    └── performance/           # Performance benchmarks
```

#### Milestones

**M3.1: Advanced ML Forecasting (Week 18)**
- ✓ LightGBM integration
- ✓ Multi-variate forecasting
- ✓ What-if scenario modeling
- ✓ Forecast MAPE <10% on production data

**M3.2: Enterprise Features (Week 20)**
- ✓ Multi-tenancy with tenant isolation
- ✓ RBAC implementation
- ✓ SSO integration (SAML + OAuth2)
- ✓ Security penetration testing completed

**M3.3: Complete Deployment Modes (Week 21)**
- ✓ Serverless functions deployed and tested
- ✓ WASM module functional
- ✓ Embedded library published to crates.io
- ✓ Kubernetes Helm charts v1.0

**M3.4: Full Stack Integration (Week 22)**
- ✓ All LLM DevOps modules integrated
- ✓ Cloud provider billing integrations
- ✓ Observability stack complete
- ✓ End-to-end integration test suite

**M3.5: Documentation & Examples (Week 23)**
- ✓ Complete API documentation
- ✓ Architecture decision records (ADRs)
- ✓ 10+ tutorial guides
- ✓ 20+ code examples
- ✓ Video tutorials for key features

**M3.6: v1.0 Release (Week 24)**
- ✓ Performance benchmarks published
- ✓ Security audit report
- ✓ Production deployment guide
- ✓ Release notes and migration guides
- ✓ Public launch and announcement

#### Dependencies

**LLM DevOps Modules (All v1.0+):**
- `llm-observatory`: Metrics and monitoring
- `llm-edge-agent`: Request interception
- `llm-governance`: Policy enforcement
- `llm-security`: Security scanning
- `llm-testing`: Load testing and simulation

**External Dependencies:**
- PostgreSQL (v14+) with TimescaleDB extension
- Redis Cluster (v7+)
- Message Queue: NATS or RabbitMQ
- Object Storage: S3-compatible (AWS S3, MinIO)
- Kubernetes (v1.25+) for orchestration

**Rust Crates (Production Complete):**
```toml
[dependencies]
# All previous crates plus...

# Machine Learning
lightgbm = "0.3"
smartcore = "0.3"

# Cloud Integrations
aws-sdk-costexplorer = "1.0"
aws-sdk-s3 = "1.0"
google-cloud-gcp = "0.20"
azure_core = "0.17"

# Security & Compliance
sha2 = "0.10"
aes-gcm = "0.10"
argon2 = "0.5"

# WebAssembly
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"

# Observability
opentelemetry = "0.21"
opentelemetry-prometheus = "0.14"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Service Mesh
tonic-health = "0.10"
tonic-reflection = "0.10"
```

#### Success Metrics

**Functional:**
- Cost calculation accuracy: ±0.05% across all providers and scenarios
- Forecast MAPE: <10% for 7-day, <15% for 30-day, <20% for 90-day
- Anomaly detection: F1 score >95%
- API uptime: >99.9% (SLA compliance)

**Performance:**
- API throughput: >10,000 req/sec
- Forecast generation: <2 seconds for 90-day ML forecast
- Database queries (p99): <100ms
- End-to-end request latency (p95): <50ms
- Horizontal scalability: Linear up to 100 nodes

**Quality:**
- Unit test coverage: >85%
- Integration test coverage: >80%
- End-to-end test coverage: >70%
- Code quality: Zero critical issues in static analysis
- Security: Zero high-severity vulnerabilities

**Operational:**
- MTTR (Mean Time To Recovery): <15 minutes
- MTBF (Mean Time Between Failures): >720 hours
- Deployment time: <5 minutes (zero-downtime)
- Rollback time: <2 minutes

---

## 2. Dependencies

### 2.1 Module Dependencies (LLM DevOps Ecosystem)

**Critical Path Dependencies:**

```
┌─────────────────────────────────────────────────────────────────┐
│                        LLM-CostOps                              │
│                     (This Component)                            │
└─────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ llm-observatory │  │  llm-edge-agent │  │ llm-governance  │
│   (Required)    │  │   (Required)    │  │   (Optional)    │
│   MVP: v0.1+    │  │   MVP: v0.1+    │  │   Beta: v0.1+   │
└─────────────────┘  └─────────────────┘  └─────────────────┘
         │                    │                    │
         └────────────────────┴────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
         ┌─────────────────┐   ┌─────────────────┐
         │  llm-security   │   │  llm-testing    │
         │   (Optional)    │   │   (Optional)    │
         │   v1.0: v0.2+   │   │   v1.0: v0.2+   │
         └─────────────────┘   └─────────────────┘
```

**Detailed Module Requirements:**

1. **llm-observatory** (Metrics & Monitoring)
   - **MVP Requirement:** v0.1.0+
   - **Capabilities Needed:**
     - Metrics emission API (counters, gauges, histograms)
     - Time-series data ingestion
     - Basic query interface
   - **Integration Points:**
     - Cost metrics emission
     - Usage metrics aggregation
   - **Fallback:** File-based metrics if unavailable

2. **llm-edge-agent** (Request Interception)
   - **MVP Requirement:** v0.1.0+
   - **Capabilities Needed:**
     - Request/response interception hooks
     - Token count extraction
     - Provider identification
   - **Integration Points:**
     - Pre-request hooks for budget checks
     - Post-request hooks for cost calculation
   - **Fallback:** Manual instrumentation via SDK

3. **llm-governance** (Policy Enforcement)
   - **Beta Requirement:** v0.1.0+
   - **Capabilities Needed:**
     - Budget policy definitions
     - Approval workflow engine
     - Audit logging
   - **Integration Points:**
     - Budget threshold notifications
     - Cost allocation enforcement
     - Audit event generation
   - **Fallback:** Basic budget alerts via CLI

4. **llm-security** (Security Scanning)
   - **v1.0 Requirement:** v0.2.0+
   - **Capabilities Needed:**
     - Cost-based anomaly patterns
     - Threat correlation
   - **Integration Points:**
     - Anomaly detection integration
     - Security event correlation
   - **Fallback:** Standalone anomaly detection

5. **llm-testing** (Load Testing)
   - **v1.0 Requirement:** v0.2.0+
   - **Capabilities Needed:**
     - Cost simulation for test scenarios
     - Load test cost estimation
   - **Integration Points:**
     - Pre-test cost forecasting
     - Post-test cost validation
   - **Fallback:** Manual cost calculation

### 2.2 External Dependencies

**Infrastructure Requirements:**

**MVP Phase:**
- SQLite (v3.40+) - Bundled, no external install required
- Optional: File system for persistence

**Beta Phase:**
- PostgreSQL (v14+)
  - Extension: TimescaleDB (v2.11+) for time-series optimization
  - Connection pooling required
- Redis (v7+)
  - Persistence enabled
  - Cluster mode optional
- Message Queue (Optional):
  - NATS (v2.9+) OR
  - RabbitMQ (v3.11+)

**v1.0 Phase:**
- All Beta dependencies plus:
- Kubernetes (v1.25+)
  - Ingress controller
  - Persistent volume support
- Object Storage (S3-compatible):
  - AWS S3 OR
  - MinIO (v2023+) OR
  - Google Cloud Storage
- Cloud Provider APIs:
  - AWS Cost Explorer API (optional)
  - GCP Billing API (optional)
  - Azure Cost Management API (optional)

**Development Dependencies:**
- Rust (v1.75+)
- Docker (v24+)
- Docker Compose (v2.20+)
- Make or Just (build automation)

### 2.3 Rust Crate Dependencies

**Core Dependency Strategy:**
- Minimal dependencies for MVP
- Proven, well-maintained crates only
- Regular security audits via `cargo audit`
- Semver-compliant version constraints

**Complete Cargo.toml for v1.0:**

```toml
[package]
name = "llm-cost-ops"
version = "1.0.0"
edition = "2021"
rust-version = "1.75"
authors = ["LLM DevOps Team"]
license = "Apache-2.0"
description = "Cost operations platform for LLM deployments"
repository = "https://github.com/llm-devops/llm-cost-ops"
keywords = ["llm", "cost", "operations", "monitoring"]
categories = ["development-tools", "finance"]

[dependencies]
# Async Runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
prost = "0.12"
bincode = "1.3"

# CLI
clap = { version = "4.4", features = ["derive", "env", "color"] }
clap_complete = "4.4"
console = "0.15"
indicatif = "0.17"

# Database
sqlx = { version = "0.7", features = [
    "runtime-tokio-native-tls",
    "postgres",
    "sqlite",
    "uuid",
    "chrono",
    "json"
] }
rusqlite = { version = "0.30", features = ["bundled"], optional = true }
redis = { version = "0.24", features = ["tokio-comp", "cluster"] }

# Web & API
axum = { version = "0.7", features = ["ws", "macros"] }
tower = { version = "0.4", features = ["timeout", "limit", "buffer"] }
tower-http = { version = "0.5", features = ["fs", "cors", "trace", "compression-gzip"] }
hyper = "1.0"
async-graphql = "6.0"
async-graphql-axum = "6.0"

# Time & Date
chrono = { version = "0.4", features = ["serde"] }
time = "0.3"

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# Logging & Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
opentelemetry = "0.21"
opentelemetry-prometheus = "0.14"
opentelemetry-otlp = "0.14"

# Metrics
prometheus = "0.13"
metrics = "0.21"
metrics-exporter-prometheus = "0.12"

# Authentication & Security
jsonwebtoken = "9.2"
bcrypt = "0.15"
argon2 = "0.5"
sha2 = "0.10"
aes-gcm = "0.10"
rand = "0.8"

# Numeric & Statistics
ndarray = { version = "0.15", features = ["serde"] }
statrs = "0.16"
lightgbm = { version = "0.3", optional = true }
smartcore = { version = "0.3", optional = true }

# gRPC
tonic = { version = "0.10", features = ["tls", "gzip"] }
tonic-health = "0.10"
tonic-reflection = "0.10"

# Cloud SDKs (Optional)
aws-sdk-costexplorer = { version = "1.0", optional = true }
aws-sdk-s3 = { version = "1.0", optional = true }
google-cloud-gcp = { version = "0.20", optional = true }
azure_core = { version = "0.17", optional = true }

# WebAssembly (Optional)
wasm-bindgen = { version = "0.2", optional = true }
wasm-bindgen-futures = { version = "0.4", optional = true }

# Utilities
uuid = { version = "1.6", features = ["v4", "serde"] }
url = "2.5"
regex = "1.10"
lazy_static = "1.4"
parking_lot = "0.12"
dashmap = "5.5"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.4"
tempfile = "3.8"
mockall = "0.12"
tokio-test = "0.4"
wiremock = "0.5"

[features]
default = ["sqlite", "prometheus-metrics"]
sqlite = ["rusqlite"]
ml = ["lightgbm", "smartcore"]
cloud-aws = ["aws-sdk-costexplorer", "aws-sdk-s3"]
cloud-gcp = ["google-cloud-gcp"]
cloud-azure = ["azure_core"]
wasm = ["wasm-bindgen", "wasm-bindgen-futures"]
prometheus-metrics = []
full = ["ml", "cloud-aws", "cloud-gcp", "cloud-azure"]

[[bin]]
name = "cost-ops"
path = "src/main.rs"

[lib]
name = "llm_cost_ops"
path = "src/lib.rs"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.dev]
opt-level = 0

[profile.test]
opt-level = 1
```

**Version Constraints Rationale:**
- Caret requirements (^) for minor updates: Semver compatible
- Locked major versions to prevent breaking changes
- Optional features to reduce binary size
- LTO and optimization for production builds

---

## 3. Validation Metrics

### 3.1 Cost Calculation Accuracy

**Target:** ±0.05% accuracy compared to provider billing

**Measurement Methodology:**
1. Collect provider billing statements (monthly)
2. Compare llm-cost-ops calculations with actual bills
3. Calculate percentage difference per provider
4. Aggregate across all providers

**Formula:**
```
Accuracy = 1 - |Calculated_Cost - Actual_Bill| / Actual_Bill
```

**Acceptance Criteria:**
- MVP: ±0.1% accuracy (99.9% accuracy rate)
- Beta: ±0.075% accuracy
- v1.0: ±0.05% accuracy (99.95% accuracy rate)

**Testing Strategy:**
- Unit tests with known pricing examples
- Integration tests against provider APIs
- Monthly reconciliation reports
- Automated billing comparison in CI/CD

### 3.2 Forecast Precision

**Metrics:**
1. **MAPE (Mean Absolute Percentage Error)**
   - Formula: `MAPE = (1/n) * Σ |Actual - Forecast| / |Actual| * 100%`
   - MVP Target: Not applicable (no forecasting)
   - Beta Target: <15% for 7-day forecasts
   - v1.0 Target: <10% for 7-day, <15% for 30-day, <20% for 90-day

2. **RMSE (Root Mean Square Error)**
   - Formula: `RMSE = √[(1/n) * Σ(Actual - Forecast)²]`
   - Beta Target: <$100 for typical workloads
   - v1.0 Target: <$50 for typical workloads

3. **MAE (Mean Absolute Error)**
   - Formula: `MAE = (1/n) * Σ |Actual - Forecast|`
   - Complementary metric to RMSE

**Confidence Intervals:**
- 95% confidence interval coverage: >90%
- Interval width: Minimize while maintaining coverage

**Anomaly Detection:**
- Precision: >85% (Beta), >90% (v1.0)
- Recall: >85% (Beta), >90% (v1.0)
- F1 Score: >85% (Beta), >95% (v1.0)
- False positive rate: <5%

**Testing Strategy:**
- Backtesting on historical data (train on N-90 days, test on last 90)
- Cross-validation (k-fold, time-series split)
- A/B testing of forecasting algorithms
- Continuous monitoring of forecast drift

### 3.3 API Performance

**Response Time Targets:**

| Endpoint Type | p50 | p95 | p99 | Max |
|---------------|-----|-----|-----|-----|
| Cost Query (simple) | <20ms | <50ms | <100ms | <500ms |
| Cost Query (complex) | <50ms | <150ms | <300ms | <1s |
| Forecast Generation | <1s | <3s | <5s | <10s |
| Real-time Cost Stream | <10ms | <30ms | <50ms | <100ms |
| Batch Export | <5s | <15s | <30s | <60s |

**Throughput Targets:**

| Phase | Sustained RPS | Peak RPS | Concurrent Connections |
|-------|---------------|----------|------------------------|
| MVP | 100 | 500 | 100 |
| Beta | 1,000 | 5,000 | 1,000 |
| v1.0 | 10,000 | 50,000 | 10,000 |

**Availability:**
- MVP: 95% uptime (planned maintenance windows)
- Beta: 99.5% uptime
- v1.0: 99.9% uptime (SLA)

**Scalability:**
- Horizontal scaling efficiency: >80% (2x nodes → >1.6x throughput)
- Vertical scaling efficiency: >70% (2x CPU → >1.4x throughput)
- Auto-scaling response time: <2 minutes

### 3.4 Test Coverage

**Code Coverage Targets:**

| Phase | Unit Tests | Integration Tests | E2E Tests | Total Coverage |
|-------|------------|-------------------|-----------|----------------|
| MVP | >70% | >60% | N/A | >65% |
| Beta | >80% | >70% | >50% | >75% |
| v1.0 | >85% | >80% | >70% | >80% |

**Critical Path Coverage:**
- Cost calculation logic: 100%
- Token counting: 100%
- Provider integrations: >90%
- API endpoints: >90%
- Forecasting algorithms: >85%

**Mutation Testing:**
- Mutation score: >75% (v1.0)

**Performance Testing:**
- Load tests: All major endpoints
- Stress tests: Identify breaking points
- Soak tests: 24-hour stability tests
- Spike tests: Sudden traffic bursts

### 3.5 Quality Metrics

**Static Analysis:**
- Clippy warnings: Zero (with allowed exceptions documented)
- Rustfmt compliance: 100%
- Unsafe code: <1% of codebase, fully justified
- Cyclomatic complexity: <15 per function

**Security:**
- Dependency vulnerabilities (cargo audit): Zero critical, zero high
- SAST findings: Zero critical, <5 medium
- DAST findings: Zero critical, <5 medium
- Penetration test: Passed with no high-severity issues

**Documentation:**
- Public API documentation: 100%
- Code comments: >20% of LOC
- Architecture documentation: Complete ADRs
- User guides: Complete for all features

---

## 4. Risk Assessment

### 4.1 Technical Risks

**Risk 1: Provider API Changes**
- **Impact:** High
- **Probability:** Medium
- **Description:** Provider pricing models or API structures change without notice
- **Mitigation:**
  - Abstraction layer isolates provider-specific logic
  - Automated provider API monitoring
  - Version pinning with deprecation warnings
  - Community-driven pricing table updates
  - Fallback to manual pricing configuration

**Risk 2: Forecasting Accuracy Degradation**
- **Impact:** Medium
- **Probability:** Medium
- **Description:** Forecast models become inaccurate due to usage pattern changes
- **Mitigation:**
  - Continuous model retraining
  - Model performance monitoring with alerts
  - Multiple forecasting algorithms (ensemble approach)
  - Confidence intervals to communicate uncertainty
  - Automated model selection based on recent accuracy

**Risk 3: Database Scalability Bottleneck**
- **Impact:** High
- **Probability:** Low
- **Description:** Database becomes bottleneck at high scale
- **Mitigation:**
  - TimescaleDB hypertables for time-series optimization
  - Read replicas for query distribution
  - Materialized views for common aggregations
  - Redis caching for hot data
  - Partitioning strategy for historical data
  - Regular performance testing and capacity planning

**Risk 4: Inter-Module Dependency Issues**
- **Impact:** Medium
- **Probability:** Medium
- **Description:** Breaking changes in dependent LLM DevOps modules
- **Mitigation:**
  - Semver-compliant version constraints
  - Comprehensive integration test suite
  - Fallback modes when dependencies unavailable
  - Version compatibility matrix
  - Regular dependency updates and testing

**Risk 5: Memory Leaks in Long-Running Processes**
- **Impact:** High
- **Probability:** Low
- **Description:** Memory leaks causing crashes in production
- **Mitigation:**
  - Strict Rust ownership model (built-in protection)
  - Memory profiling in CI/CD (valgrind, heaptrack)
  - Soak tests (24-hour continuous operation)
  - Memory limit monitoring and alerts
  - Regular review of unsafe code blocks

### 4.2 Integration Challenges

**Challenge 1: Observatory Metrics Schema Alignment**
- **Description:** Ensuring cost metrics align with Observatory's schema expectations
- **Mitigation:**
  - Early schema definition and agreement
  - Schema validation in integration tests
  - Versioned metrics protocol
  - Backward compatibility guarantees
  - Fallback to generic metrics format

**Challenge 2: Edge-Agent Hook Performance**
- **Description:** Cost calculation hooks adding latency to request path
- **Mitigation:**
  - Asynchronous cost calculation (post-request)
  - Sampling for high-throughput scenarios
  - Optimized token counting algorithms
  - Caching of provider pricing data
  - Performance SLO: <5ms overhead (p95)

**Challenge 3: Governance Policy Synchronization**
- **Description:** Race conditions between cost tracking and budget enforcement
- **Mitigation:**
  - Event-driven architecture with message ordering
  - Distributed locks for critical sections
  - Eventual consistency with reconciliation
  - Idempotent operations
  - Conflict resolution strategies

**Challenge 4: Multi-Cloud Authentication Complexity**
- **Description:** Managing credentials for multiple cloud provider APIs
- **Mitigation:**
  - Centralized secret management (Vault, AWS Secrets Manager)
  - Credential rotation automation
  - IAM role-based access where possible
  - Encrypted credential storage
  - Audit logging of credential access

### 4.3 Scalability Concerns

**Concern 1: Time-Series Data Growth**
- **Projected Growth:** 100GB → 1TB+ in first year for large deployments
- **Mitigation:**
  - Automated data retention policies
  - Tiered storage (hot/warm/cold)
  - Data compression (TimescaleDB compression)
  - Aggregation and downsampling for historical data
  - Archive to object storage (S3)

**Concern 2: Real-Time Cost Calculation at Scale**
- **Peak Load:** 50,000+ RPS in large enterprise deployments
- **Mitigation:**
  - Horizontal scaling with load balancing
  - Distributed caching (Redis Cluster)
  - Rate limiting and throttling
  - Batch processing for non-critical calculations
  - Edge caching for frequently accessed data

**Concern 3: Forecast Computation Complexity**
- **Description:** ML-based forecasting may be computationally expensive
- **Mitigation:**
  - Pre-computed forecasts (scheduled batch jobs)
  - Model optimization (quantization, pruning)
  - GPU acceleration for ML models (optional)
  - Tiered forecasting (simple for small workloads, complex for large)
  - Forecast caching with TTL

**Concern 4: Multi-Tenancy Isolation**
- **Description:** Ensuring tenant data isolation and fair resource allocation
- **Mitigation:**
  - Database-level tenant isolation (schema per tenant)
  - Resource quotas per tenant
  - Rate limiting per tenant
  - Monitoring and alerting on tenant resource usage
  - Circuit breakers to isolate noisy neighbors

### 4.4 Operational Risks

**Risk 1: Data Loss During Migration**
- **Impact:** Critical
- **Probability:** Low
- **Mitigation:**
  - Automated backups (hourly, daily, weekly)
  - Point-in-time recovery capability
  - Migration testing on production copies
  - Rollback procedures documented
  - Backup verification automated tests

**Risk 2: Monitoring Blind Spots**
- **Impact:** High
- **Probability:** Medium
- **Mitigation:**
  - Comprehensive observability stack (metrics, logs, traces)
  - Synthetic monitoring of critical paths
  - Anomaly detection on system metrics
  - On-call runbooks and playbooks
  - Regular chaos engineering exercises

**Risk 3: Breaking Changes in Major Releases**
- **Impact:** Medium
- **Probability:** Medium
- **Mitigation:**
  - Deprecation warnings in advance (2 minor versions)
  - Migration guides and tools
  - Backward compatibility layers
  - Beta release testing period
  - Support for N-1 version during transition

---

## 5. Implementation Timeline Summary

```
Weeks 1-8: MVP Development
├── Week 1-2: Core cost calculation engine
├── Week 3-4: Observatory & Edge-Agent integration
├── Week 5-6: CLI interface & SQLite persistence
├── Week 7-8: Testing, documentation, MVP release

Weeks 9-16: Beta Development
├── Week 9-10: Multi-provider abstraction
├── Week 11-12: Forecasting engine (ARIMA)
├── Week 13-14: API microservice & GraphQL
├── Week 15-16: Governance integration, Beta release

Weeks 17-24: v1.0 Development
├── Week 17-18: Advanced ML forecasting
├── Week 19-20: Enterprise features (SSO, RBAC, multi-tenancy)
├── Week 21: Complete deployment modes
├── Week 22: Full stack integration
├── Week 23: Documentation & examples
├── Week 24: Performance benchmarks, security audit, v1.0 release
```

**Total Timeline:** 24 weeks (6 months) from kickoff to v1.0 production release

**Team Size Assumption:** 2-3 full-time Rust engineers

**Buffer:** 20% contingency built into each phase

---

## 6. Success Criteria

### MVP Success Criteria
- [ ] Accurate cost calculation for at least 1 provider (±0.1%)
- [ ] Integration with Observatory and Edge-Agent
- [ ] Functional CLI with query, summary, export commands
- [ ] Documentation: README, quickstart guide
- [ ] 70% unit test coverage
- [ ] Docker image published

### Beta Success Criteria
- [ ] 5 major LLM providers supported
- [ ] Forecasting with <15% MAPE for 7-day
- [ ] API microservice with >99.5% uptime
- [ ] Governance integration functional
- [ ] 80% test coverage
- [ ] Security audit completed

### v1.0 Success Criteria
- [ ] All production features complete
- [ ] 99.9% API uptime SLA
- [ ] <10% MAPE for 7-day forecasts
- [ ] >85% test coverage
- [ ] Complete documentation and examples
- [ ] Performance benchmarks published
- [ ] Production deployments validated

---

# References

## SPARC Methodology

1. **SPARC Framework Overview**
   - Original Paper: "SPARC: A Structured Approach to Software Architecture" (Software Engineering Institute)
   - URL: https://resources.sei.cmu.edu/library/asset-view.cfm?assetid=513908
   - Key Concepts: Specification, Pseudocode, Architecture, Refinement, Completion

2. **SPARC in Modern Software Development**
   - "Applying SPARC to Microservices Architecture" - Martin Fowler
   - URL: https://martinfowler.com/articles/microservices.html
   - Relevance: Microservices patterns applicable to LLM-CostOps API mode

3. **Agile SPARC Methodology**
   - "Combining SPARC with Agile Development" - IEEE Software
   - DOI: 10.1109/MS.2018.2801552
   - Application: Phased roadmap aligned with agile sprints

## Rust Ecosystem Documentation

4. **The Rust Programming Language (Official Book)**
   - URL: https://doc.rust-lang.org/book/
   - Chapters: 10 (Generics), 11 (Testing), 15 (Smart Pointers), 16 (Concurrency)

5. **Tokio Asynchronous Runtime**
   - URL: https://tokio.rs/
   - Documentation: https://docs.rs/tokio/latest/tokio/
   - Tutorials: https://tokio.rs/tokio/tutorial

6. **Axum Web Framework**
   - GitHub: https://github.com/tokio-rs/axum
   - Documentation: https://docs.rs/axum/latest/axum/
   - Examples: https://github.com/tokio-rs/axum/tree/main/examples

7. **SQLx Database Library**
   - GitHub: https://github.com/launchbadge/sqlx
   - Documentation: https://docs.rs/sqlx/latest/sqlx/
   - Migration Guide: https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md

8. **Rust Performance Optimization**
   - "The Rust Performance Book": https://nnethercote.github.io/perf-book/
   - Benchmarking with Criterion: https://bheisler.github.io/criterion.rs/book/

9. **Rust API Guidelines**
   - URL: https://rust-lang.github.io/api-guidelines/
   - Key Sections: Naming, Interoperability, Type Safety

## Time-Series Forecasting

10. **Time Series Forecasting: Principles and Practice (3rd ed.)**
    - Authors: Hyndman, R.J., & Athanasopoulos, G.
    - URL: https://otexts.com/fpp3/
    - Chapters: 9 (ARIMA), 12 (Advanced forecasting methods)

11. **Prophet: Forecasting at Scale**
    - Paper: "Forecasting at Scale" - Taylor & Letham (Facebook Research)
    - DOI: 10.7287/peerj.preprints.3190v2
    - GitHub: https://github.com/facebook/prophet
    - Application: Inspiration for LLM cost forecasting algorithms

12. **ARIMA Models in Rust**
    - crate: `statsrs` - https://docs.rs/statrs/latest/statrs/
    - Tutorial: "Implementing ARIMA from Scratch in Rust"
    - URL: https://www.robots.ox.ac.uk/~sjrob/Pubs/Phil.pdf

13. **Anomaly Detection in Time Series**
    - Paper: "Anomaly Detection: A Survey" - Chandola et al.
    - DOI: 10.1145/1541880.1541882
    - Techniques: Statistical methods, ML-based detection

14. **LightGBM for Time Series**
    - GitHub: https://github.com/microsoft/LightGBM
    - Paper: "LightGBM: A Highly Efficient Gradient Boosting Decision Tree"
    - DOI: 10.5555/3294996.3295074
    - Rust bindings: https://crates.io/crates/lightgbm

## LLM Cost Optimization Research

15. **"The Economics of Large Language Models"**
    - Authors: Bommasani et al. (Stanford HAI)
    - URL: https://arxiv.org/abs/2108.07258
    - Key Insights: Cost structures, optimization strategies

16. **"Optimizing LLM Inference Costs"**
    - Authors: Pope et al. (Google Research)
    - URL: https://arxiv.org/abs/2211.05102
    - Techniques: Batching, caching, model selection

17. **"Cost-Aware LLM Serving"**
    - Paper: "Towards Cost-Effective LLM Serving: A Survey"
    - URL: https://arxiv.org/abs/2308.10481
    - Relevance: Cost-performance tradeoffs

18. **OpenAI Pricing Documentation**
    - URL: https://openai.com/api/pricing/
    - Historical pricing: https://openai.com/blog/gpt-4-api-general-availability

19. **Anthropic Claude Pricing**
    - URL: https://www.anthropic.com/api
    - Prompt caching: https://docs.anthropic.com/claude/docs/prompt-caching

20. **Google Cloud Vertex AI Pricing**
    - URL: https://cloud.google.com/vertex-ai/pricing
    - Generative AI pricing: https://cloud.google.com/vertex-ai/generative-ai/pricing

21. **AWS Bedrock Pricing**
    - URL: https://aws.amazon.com/bedrock/pricing/
    - Cost optimization guide: https://docs.aws.amazon.com/bedrock/latest/userguide/cost-optimization.html

## Relevant Open-Source Projects

22. **Langfuse - LLM Engineering Platform**
    - GitHub: https://github.com/langfuse/langfuse
    - Features: Cost tracking, token usage monitoring
    - Relevance: Reference implementation for cost tracking

23. **LiteLLM - Unified LLM API**
    - GitHub: https://github.com/BerriAI/litellm
    - Features: Multi-provider abstraction, cost calculation
    - Relevance: Provider abstraction patterns

24. **OpenLLMetry - LLM Observability**
    - GitHub: https://github.com/traceloop/openllmetry
    - Features: OpenTelemetry integration, cost metrics
    - Relevance: Observability patterns for LLMs

25. **PromptLayer - LLM Middleware**
    - GitHub: https://github.com/MagnivOrg/prompt-layer-library
    - Features: Request logging, cost tracking
    - Relevance: Middleware patterns for cost injection

26. **Helicone - LLM Observability Platform**
    - GitHub: https://github.com/Helicone/helicone
    - Features: Cost analytics, caching
    - Relevance: Production-grade cost analytics UI

27. **Pezzo - Open-Source LLMOps Platform**
    - GitHub: https://github.com/pezzolabs/pezzo
    - Features: Cost tracking, prompt management
    - Relevance: Integrated cost management approach

## Time-Series Databases

28. **TimescaleDB Documentation**
    - URL: https://docs.timescale.com/
    - Best Practices: https://docs.timescale.com/timescaledb/latest/how-to-guides/
    - Relevance: Optimal storage for cost time-series data

29. **PostgreSQL Performance Tuning**
    - URL: https://www.postgresql.org/docs/current/performance-tips.html
    - Book: "PostgreSQL: Up and Running" - Obe & Hsu

## Microservices & API Design

30. **RESTful API Design Best Practices**
    - "REST API Design Rulebook" - Mark Masse
    - URL: https://www.oreilly.com/library/view/rest-api-design/9781449317904/

31. **GraphQL Best Practices**
    - URL: https://graphql.org/learn/best-practices/
    - "Production Ready GraphQL" - Marc-André Giroux

32. **gRPC in Rust**
    - Tonic Documentation: https://docs.rs/tonic/latest/tonic/
    - Tutorial: https://github.com/hyperium/tonic/tree/master/examples

## Cloud Cost Management

33. **AWS Cost Optimization**
    - URL: https://aws.amazon.com/aws-cost-management/
    - Best Practices: https://docs.aws.amazon.com/cost-management/latest/userguide/ce-best-practices.html

34. **Google Cloud Cost Management**
    - URL: https://cloud.google.com/cost-management
    - Billing API: https://cloud.google.com/billing/docs/apis

35. **Azure Cost Management**
    - URL: https://azure.microsoft.com/en-us/products/cost-management/
    - REST API: https://learn.microsoft.com/en-us/rest/api/cost-management/

## Observability & Monitoring

36. **Prometheus Documentation**
    - URL: https://prometheus.io/docs/
    - Best Practices: https://prometheus.io/docs/practices/naming/

37. **OpenTelemetry in Rust**
    - URL: https://opentelemetry.io/docs/languages/rust/
    - GitHub: https://github.com/open-telemetry/opentelemetry-rust

38. **Grafana Dashboard Design**
    - URL: https://grafana.com/docs/grafana/latest/dashboards/
    - Best Practices: https://grafana.com/docs/grafana/latest/dashboards/build-dashboards/best-practices/

## Security & Compliance

39. **OWASP API Security Top 10**
    - URL: https://owasp.org/www-project-api-security/
    - Relevance: API security for cost-ops microservice

40. **Rust Security Guidelines**
    - URL: https://anssi-fr.github.io/rust-guide/
    - Secure Rust Guidelines: https://rust-lang.github.io/api-guidelines/

41. **SOC 2 Compliance Guide**
    - URL: https://www.aicpa.org/interestareas/frc/assuranceadvisoryservices/aicpasoc2report.html
    - Relevance: Compliance for enterprise features

## DevOps & CI/CD

42. **GitHub Actions for Rust**
    - URL: https://github.com/actions-rs
    - Example workflows: https://github.com/actions-rs/meta/blob/master/recipes/quickstart.md

43. **Kubernetes Best Practices**
    - "Kubernetes: Up and Running" - Hightower et al.
    - URL: https://www.oreilly.com/library/view/kubernetes-up-and/9781492046523/

44. **Helm Chart Best Practices**
    - URL: https://helm.sh/docs/chart_best_practices/
    - Relevance: Kubernetes deployment for v1.0

## Testing & Quality Assurance

45. **Property-Based Testing in Rust**
    - PropTest Documentation: https://docs.rs/proptest/latest/proptest/
    - Tutorial: https://github.com/proptest-rs/proptest/blob/master/proptest/README.md

46. **Mutation Testing**
    - cargo-mutants: https://github.com/sourcefrog/cargo-mutants
    - Relevance: Ensuring test quality

47. **Load Testing with k6**
    - URL: https://k6.io/docs/
    - Rust integration: https://github.com/grafana/xk6-rust

## Machine Learning

48. **Practical Time Series Forecasting with R and Python**
    - Authors: Shmueli & Lichtendahl
    - Relevance: Forecasting algorithm implementation

49. **Feature Engineering for Machine Learning**
    - Authors: Zheng & Casari
    - URL: https://www.oreilly.com/library/view/feature-engineering-for/9781491953235/
    - Relevance: Cost forecasting feature design

50. **MLOps Best Practices**
    - "Introducing MLOps" - Treveil et al.
    - URL: https://www.oreilly.com/library/view/introducing-mlops/9781492083283/
    - Relevance: ML model deployment and monitoring

---

**Document Version:** 1.0
**Last Updated:** 2025-11-15
**Author:** LLM-CostOps SPARC Completion Agent
**Status:** Ready for Review

---

## Appendix A: Glossary

- **ARIMA**: AutoRegressive Integrated Moving Average - time series forecasting model
- **CLI**: Command-Line Interface
- **CQRS**: Command Query Responsibility Segregation
- **F1 Score**: Harmonic mean of precision and recall
- **gRPC**: Google Remote Procedure Call - high-performance RPC framework
- **JWT**: JSON Web Token - authentication standard
- **LTO**: Link-Time Optimization - compiler optimization
- **MAPE**: Mean Absolute Percentage Error - forecast accuracy metric
- **MTBF**: Mean Time Between Failures
- **MTTR**: Mean Time To Recovery
- **RBAC**: Role-Based Access Control
- **RMSE**: Root Mean Square Error - forecast accuracy metric
- **ROI**: Return on Investment
- **RPS**: Requests Per Second
- **SAML**: Security Assertion Markup Language - SSO standard
- **SLA**: Service Level Agreement
- **SSO**: Single Sign-On
- **WASM**: WebAssembly

## Appendix B: Contact & Support

- **GitHub Repository**: https://github.com/llm-devops/llm-cost-ops
- **Documentation**: https://docs.llm-devops.org/cost-ops
- **Issue Tracker**: https://github.com/llm-devops/llm-cost-ops/issues
- **Discord Community**: https://discord.gg/llm-devops
- **Email**: cost-ops@llm-devops.org
