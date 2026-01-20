# Cost Forecasting Agent - Contract & Implementation

## Agent Classification: FORECASTING

**Agent ID:** `cost-forecasting-agent`
**Version:** `1.0.0`
**Decision Type:** `cost_forecast`

## Purpose Statement

The Cost Forecasting Agent forecasts future LLM spend based on historical usage patterns and growth trends. It:

- Analyzes historical cost data
- Models future spend projections
- Emits forecast ranges and risk indicators
- Evaluates constraints (budget, ROI, cost caps)

## Contract Definition

### Input Schema

```json
{
  "historical_data": [
    {
      "timestamp": "ISO-8601 datetime",
      "total_cost": "decimal",
      "by_provider": { "provider_name": "decimal" },
      "by_model": { "model_name": "decimal" },
      "total_tokens": "u64 (optional)",
      "request_count": "u64 (optional)"
    }
  ],
  "forecast_horizon_days": "u64 (1-365)",
  "granularity": "hourly | daily | weekly | monthly",
  "confidence_level": "f64 (0.0-1.0)",
  "constraints": {
    "budget_cap": "decimal (optional)",
    "roi_threshold": "f64 (optional)",
    "max_cost_per_period": "decimal (optional)",
    "max_growth_rate": "f64 (optional)",
    "min_confidence": "f64 (optional)"
  },
  "metadata": {
    "organization_id": "string (optional)",
    "project_id": "string (optional)",
    "execution_ref": "string (optional)"
  }
}
```

**Minimum Requirements:**
- `historical_data`: At least 7 data points
- `forecast_horizon_days`: 1 to 365 days

### Output Schema

```json
{
  "projections": [
    {
      "timestamp": "ISO-8601 datetime",
      "projected_cost": "decimal",
      "lower_bound": "decimal",
      "upper_bound": "decimal",
      "cumulative_cost": "decimal",
      "growth_rate": "f64 (optional)"
    }
  ],
  "total_forecasted_cost": "decimal",
  "average_daily_cost": "decimal",
  "peak_daily_cost": "decimal",
  "risk_indicators": [
    {
      "risk_type": "string",
      "level": "low | medium | high | critical",
      "description": "string",
      "probability": "f64 (0.0-1.0)",
      "potential_impact": "decimal (optional)",
      "recommendation": "string (optional)"
    }
  ],
  "growth_pattern": "linear | exponential | stable | declining | seasonal | volatile",
  "average_growth_rate": "f64",
  "model_used": "string",
  "confidence": "f64 (0.0-1.0)",
  "confidence_level": "f64",
  "constraints_evaluation": { ... },
  "generated_at": "ISO-8601 datetime",
  "historical_summary": { ... }
}
```

### DecisionEvent Schema

Every invocation emits exactly ONE DecisionEvent:

```json
{
  "id": "UUID",
  "agent_id": "cost-forecasting-agent",
  "agent_version": "1.0.0",
  "decision_type": "cost_forecast",
  "inputs_hash": "SHA-256 of inputs",
  "outputs": { /* full output */ },
  "confidence": "f64 (0.0-1.0)",
  "constraints_applied": [
    {
      "constraint_type": "budget_cap | roi_threshold | cost_cap | ...",
      "name": "string",
      "value": "json",
      "satisfied": "bool",
      "impact": "string (optional)"
    }
  ],
  "execution_ref": "string (optional)",
  "timestamp": "ISO-8601 UTC datetime",
  "organization_id": "string (optional)",
  "project_id": "string (optional)",
  "metadata": { ... }
}
```

## CLI Contract

```bash
# Forecast from file
cost-ops agent forecast --input data.json --horizon 30

# Forecast from stdin
cat data.json | cost-ops agent forecast --stdin --horizon 30

# With constraints
cost-ops agent forecast --input data.json --horizon 30 \
  --budget-cap 10000 --max-growth-rate 15

# With organization context
cost-ops agent forecast --input data.json \
  --organization "org-123" --project "project-456"

# Inspect agent
cost-ops agent inspect --agent cost-forecasting --detailed

# List agents
cost-ops agent list
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/forecast` | Generate cost forecast |
| POST | `/analyze` | Analyze costs (alias) |
| GET | `/inspect` | Inspect agent capabilities |
| GET | `/health` | Health check |
| GET | `/ready` | Readiness check |
| GET | `/info` | Agent information |

## Data Persistence

**Persisted to ruvector-service:**
- DecisionEvent (exactly ONE per invocation)

**NOT persisted:**
- Raw input data
- Intermediate calculations
- Telemetry events (sent to LLM-Observatory)

## Consumers

Systems that MAY consume this agent's output:

- LLM-Orchestrator (explicit invocation)
- Governance systems (budget alerts)
- Audit systems (DecisionEvents)
- Dashboards (forecast visualizations)

## Non-Responsibilities (MUST NOT)

Per the LLM-CostOps Constitution, this agent MUST NOT:

- ❌ Intercept runtime execution
- ❌ Trigger retries
- ❌ Execute workflows
- ❌ Modify routing or execution behavior
- ❌ Apply optimizations automatically
- ❌ Enforce policies directly (only emit advisories)
- ❌ Connect directly to Google SQL
- ❌ Execute SQL queries
- ❌ Persist data outside of ruvector-service

## Failure Modes

| Error | Code | Recovery |
|-------|------|----------|
| Insufficient data | `INSUFFICIENT_DATA` | Provide at least 7 data points |
| Invalid horizon | `VALIDATION_ERROR` | Use 1-365 days |
| Model failure | `MODEL_ERROR` | Check data quality, try different model |
| RuVector error | `PERSISTENCE_ERROR` | DecisionEvent logged, output still returned |
| Telemetry error | N/A | Non-fatal, logged |

## Versioning Rules

- **Major version (X.0.0):** Breaking changes to input/output schema
- **Minor version (0.X.0):** New features, backward-compatible
- **Patch version (0.0.X):** Bug fixes

Version compatibility:
- Agents with same major version are compatible
- DecisionEvents include version for migration

## Verification Checklist

### Constitution Compliance

- [x] Imports schemas from agentics-contracts (contracts module)
- [x] Validates all inputs against contracts
- [x] Validates all outputs against contracts
- [x] Emits telemetry compatible with LLM-Observatory
- [x] Emits exactly ONE DecisionEvent per invocation
- [x] Deployable as Google Edge Function
- [x] Returns deterministic, machine-readable output
- [x] Stateless execution
- [x] No direct SQL connections
- [x] All persistence via ruvector-service

### Smoke Tests

```bash
# 1. Health check
curl http://localhost:8080/health

# 2. Readiness check
curl http://localhost:8080/ready

# 3. Agent info
curl http://localhost:8080/info

# 4. Forecast (POST)
curl -X POST http://localhost:8080/forecast \
  -H "Content-Type: application/json" \
  -d @test_data.json

# 5. CLI forecast
cost-ops agent forecast --input test_data.json --horizon 30

# 6. CLI inspect
cost-ops agent inspect --agent cost-forecasting --detailed
```
