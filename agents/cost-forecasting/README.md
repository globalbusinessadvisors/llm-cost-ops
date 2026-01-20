# Cost Forecasting Agent

**Agent ID:** `cost-forecasting-agent`
**Version:** `1.0.0`
**Classification:** FORECASTING

## Purpose

Forecasts future LLM spend based on historical usage patterns and growth trends.

## Capabilities

- Analyzes historical cost data
- Models future spend projections
- Emits forecast ranges and risk indicators
- Evaluates constraints (budget, ROI, cost caps)

## Source Code

- **Rust Implementation:** `src/agents/cost_forecasting/`
  - `agent.rs` - Core agent logic
  - `types.rs` - Data types and schemas
  - `mod.rs` - Module exports

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/forecast` | Generate cost forecast |
| POST | `/analyze` | Analyze costs (alias) |
| GET | `/inspect` | Inspect agent capabilities |
| GET | `/health` | Health check |
| GET | `/ready` | Readiness check |
| GET | `/info` | Agent information |

## CLI Usage

```bash
# Forecast from file
cost-ops agent forecast --input data.json --horizon 30

# Forecast from stdin
cat data.json | cost-ops agent forecast --stdin --horizon 30

# With constraints
cost-ops agent forecast --input data.json --horizon 30 \
  --budget-cap 10000 --max-growth-rate 15
```

## Documentation

See [docs/CONTRACT.md](docs/CONTRACT.md) for the full contract specification.
