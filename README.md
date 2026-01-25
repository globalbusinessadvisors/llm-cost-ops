# LLM-CostOps

Enterprise-grade financial intelligence platform for LLM operations. Track tokens, forecast costs, attribute expenses, and optimize spend across multiple providers.

[![Rust](https://img.shields.io/badge/rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![TypeScript](https://img.shields.io/badge/typescript-5.0+-blue.svg)](https://www.typescriptlang.org)
[![Python](https://img.shields.io/badge/python-3.8+-green.svg)](https://www.python.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## Overview

LLM-CostOps is the financial backbone of the LLM DevOps ecosystem, providing:

- **Financial Visibility** - Real-time and historical cost tracking across all LLM operations
- **Predictive Intelligence** - Forecast future costs based on usage patterns and trends
- **ROI Attribution** - Correlate costs with performance outcomes and business value
- **Cost Optimization** - Enable intelligent routing and provider selection based on cost efficiency

## Features

### Core Capabilities

| Feature | Description |
|---------|-------------|
| Token Counting | Real-time, sub-second token capture with streaming support |
| Cost Calculation | Multi-provider pricing with input/output/cached token differentiation |
| Cost Attribution | Multi-dimensional attribution (user, project, org, environment) |
| Forecasting | Linear, moving average, and exponential smoothing models |
| Anomaly Detection | Automatic detection of cost spikes and unusual patterns |
| Report Generation | JSON, CSV, Excel, PDF exports with scheduled delivery |

### Supported Providers

- **OpenAI** - GPT-4, GPT-4 Turbo, GPT-4o, GPT-3.5
- **Anthropic** - Claude Opus, Claude Sonnet, Claude Haiku
- **Google** - Gemini Pro, Gemini Flash
- Custom provider support via rate card configuration

### Governance & FinOps (Phase 4)

- Cost risk signals for anomaly detection
- Budget threshold alerts and enforcement
- Policy violation monitoring
- Performance budgets (token and latency limits)
- Financial approval workflows

## Quick Start

### Prerequisites

- Rust 1.80+ (for core services)
- Node.js 18+ (for TypeScript SDK)
- Python 3.8+ (for Python SDK)
- Docker & Docker Compose (for local development)
- PostgreSQL 15+
- Redis 7+

### Installation

```bash
# Clone the repository
git clone https://github.com/globalbusinessadvisors/llm-cost-ops.git
cd llm-cost-ops

# Copy environment configuration
cp .env.example .env

# Start services with Docker Compose
docker compose up -d

# Or build from source
cargo build --release
```

### Using the SDKs

#### TypeScript/JavaScript

```bash
npm install @llm-dev-ops/llm-cost-ops-sdk
```

```typescript
import { CostOpsClient } from '@llm-dev-ops/llm-cost-ops-sdk';

const client = new CostOpsClient({
  baseUrl: 'https://api.llm-cost-ops.dev',
  apiKey: 'your-api-key'
});

// Track token usage
await client.trackUsage({
  model: 'gpt-4',
  inputTokens: 1500,
  outputTokens: 500,
  project: 'my-project'
});

// Get cost summary
const costs = await client.getCostSummary({
  startDate: '2024-01-01',
  endDate: '2024-01-31',
  groupBy: 'project'
});
```

#### Python

```bash
pip install llm-cost-ops
```

```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(
    base_url="https://api.llm-cost-ops.dev",
    api_key="your-api-key"
)

# Track token usage
client.track_usage(
    model="claude-sonnet-4",
    input_tokens=2000,
    output_tokens=800,
    project="my-project"
)

# Get cost forecast
forecast = client.get_forecast(
    project="my-project",
    horizon_days=30
)
```

#### Rust

```toml
[dependencies]
llm-cost-ops-sdk = "0.1"
```

```rust
use llm_cost_ops_sdk::CostOpsClient;

let client = CostOpsClient::new("https://api.llm-cost-ops.dev", "your-api-key");

// Track usage
client.track_usage(UsageRecord {
    model: "gpt-4".into(),
    input_tokens: 1500,
    output_tokens: 500,
    project: Some("my-project".into()),
    ..Default::default()
}).await?;
```

## Architecture

```
                    +-----------------------------------------------------+
                    |                   LLM DevOps Ecosystem               |
                    +-----------------------------------------------------+
                    |  Intelligence Core                                   |
                    |    --> LLM-Observatory ------+                       |
                    |                               v                       |
                    |  Automation Core             [LLM-CostOps]           |
                    |    +--> LLM-Auto-Optimizer <--+                      |
                    |    +--> LLM-Edge-Agent -------+                      |
                    |                               ^                       |
                    |  Governance Core             |                        |
                    |    +--> LLM-Governance-Core <-+                      |
                    |    +--> LLM-Registry ----------+                     |
                    +-----------------------------------------------------+
```

### Project Structure

```
llm-cost-ops/
├── crates/                    # Rust workspace
│   ├── llm-cost-ops/          # Core library
│   ├── llm-cost-ops-api/      # REST API server
│   ├── llm-cost-ops-cli/      # CLI tool
│   ├── llm-cost-ops-sdk/      # Rust SDK
│   └── llm-cost-ops-compliance/ # Compliance module
├── agents/                    # Edge function agents
│   ├── cost-attribution/      # Cost attribution agent
│   ├── cost-forecasting/      # Forecasting agent
│   ├── budget-enforcement/    # Budget enforcement
│   ├── cost-performance-tradeoff/
│   └── roi-estimation/
├── sdk/                       # TypeScript SDK
├── python-sdk/                # Python SDK
├── docker/                    # Docker configurations
├── k8s/                       # Kubernetes manifests
├── helm/                      # Helm charts
└── docs/                      # Documentation
```

## Configuration

### Environment Variables

```bash
# Application
PORT=8080
METRICS_PORT=9090
LOG_LEVEL=info

# Database
DATABASE_URL=postgres://user:pass@localhost:5432/llm_cost_ops
DATABASE_MAX_CONNECTIONS=20

# Redis
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=your-password

# Security
JWT_SECRET=your-jwt-secret-min-32-chars
CORS_ALLOWED_ORIGINS=http://localhost:3000

# Monitoring
ENABLE_METRICS=true
ENABLE_TRACING=true
```

See [.env.example](.env.example) for full configuration options.

## Deployment

### Docker Compose

```bash
# Development
docker compose up -d

# Production
docker compose -f docker-compose.prod.yml up -d
```

### Kubernetes

```bash
# Using Helm
helm install llm-cost-ops ./helm/llm-cost-ops \
  --namespace llm-cost-ops \
  --create-namespace

# Or raw manifests
kubectl apply -f k8s/
```

### Google Cloud Run

```bash
# Build and deploy
gcloud run deploy llm-cost-ops \
  --source . \
  --region us-central1 \
  --allow-unauthenticated
```

## API Reference

### REST Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/usage` | Record token usage |
| `GET` | `/api/v1/costs` | Query cost data |
| `GET` | `/api/v1/costs/summary` | Get cost summary |
| `GET` | `/api/v1/forecast` | Get cost forecast |
| `GET` | `/api/v1/budget` | Get budget status |
| `POST` | `/api/v1/budget` | Set budget thresholds |
| `GET` | `/api/v1/reports` | List generated reports |
| `POST` | `/api/v1/reports` | Generate a report |
| `GET` | `/api/v1/providers` | List provider pricing |

### Health & Metrics

| Endpoint | Description |
|----------|-------------|
| `/health` | Health check |
| `/health/ready` | Readiness probe |
| `/health/live` | Liveness probe |
| `/metrics` | Prometheus metrics |

## Agents

LLM-CostOps includes specialized agents deployed as edge functions:

| Agent | Purpose |
|-------|---------|
| **Cost Attribution** | Validates input, calculates costs, attributes to dimensions |
| **Cost Forecasting** | Predictive cost modeling based on historical data |
| **Budget Enforcement** | Enforces budget thresholds and limits |
| **Cost-Performance Tradeoff** | Analyzes cost vs performance trade-offs |
| **ROI Estimation** | Correlates costs with business outcomes |

## Integrations

### LLM DevOps Ecosystem

- **LLM-Observatory** - Consumes real-time metrics
- **LLM-Edge-Agent** - Distributed edge deployments
- **LLM-Governance-Core** - Policy integration
- **LLM-Test-Bench** - Cost correlation with benchmarks
- **LLM-Registry** - Model metadata and pricing

### External Services

- Prometheus (metrics)
- Grafana (visualization)
- Jaeger (distributed tracing)
- NATS (event streaming)

## Development

### Building from Source

```bash
# Build all crates
cargo build --release

# Run tests
cargo test --all

# Run with logging
RUST_LOG=debug cargo run --bin llm-cost-ops-api

# Build TypeScript SDK
cd sdk && npm install && npm run build

# Build Python SDK
cd python-sdk && pip install -e ".[dev]"
```

### Running Tests

```bash
# Rust tests
cargo test --all

# TypeScript SDK tests
cd sdk && npm test

# Python SDK tests
cd python-sdk && pytest

# Integration tests
docker compose -f docker-compose.test.yml up -d
cargo test --test integration
```

## Documentation

- [Specification](docs/SPECIFICATION.md) - Detailed functional requirements
- [Architecture](docs/SPARC_PHASE_3_ARCHITECTURE.md) - System architecture
- [Audit System](docs/AUDIT_SYSTEM.md) - Audit logging architecture
- [Compliance](docs/COMPLIANCE_ARCHITECTURE.md) - GDPR and compliance framework
- [Deployment](docs/DEPLOYMENT.md) - Multi-platform deployment guide

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please read our contributing guidelines and code of conduct before submitting PRs.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Support

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/globalbusinessadvisors/llm-cost-ops/issues)
- **Discussions**: [GitHub Discussions](https://github.com/globalbusinessadvisors/llm-cost-ops/discussions)

---

Built with Rust, TypeScript, and Python for the LLM DevOps ecosystem.
