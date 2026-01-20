# Budget Enforcement Agent - Contract & Implementation

## Agent Classification: FINANCIAL GOVERNANCE

**Agent ID:** `budget-enforcement-agent`
**Version:** `1.0.0`
**Decision Type:** `budget_constraint_evaluation`

## Purpose Statement

The Budget Enforcement Agent evaluates budget thresholds and emits advisory or gating signals when limits are approached or exceeded. It:

- Analyzes current and projected spend against budget limits
- Computes confidence scores based on data completeness
- Emits advisory signals (informational, warning, gating)
- Persists DecisionEvents to ruvector-service
- Emits telemetry compatible with LLM-Observatory

## Contract Definition

### Input Schema

```json
{
  "request_id": "UUID",
  "budget": {
    "budget_id": "string",
    "name": "string",
    "limit": "decimal",
    "currency": "string (e.g., USD)",
    "period_start": "ISO-8601 datetime",
    "period_end": "ISO-8601 datetime",
    "warning_threshold": "f64 (0.0-1.0, e.g., 0.80 = 80%)",
    "critical_threshold": "f64 (0.0-1.0, e.g., 0.95 = 95%)",
    "gating_threshold": "f64 (0.0-1.0, e.g., 1.0 = 100%)",
    "enable_forecasting": "bool",
    "is_soft_limit": "bool",
    "scope": {
      "type": "tenant | project | agent | model | custom",
      "tenant_id": "string",
      "project_id": "string (optional)",
      "agent_id": "string (optional)",
      "model": "string (optional)",
      "dimensions": "object (for custom scope)"
    }
  },
  "spend_data": {
    "current_spend": "decimal",
    "currency": "string",
    "daily_spend_history": [
      {
        "date": "ISO-8601 datetime",
        "spend": "decimal"
      }
    ],
    "data_completeness": "f64 (0.0-1.0)",
    "data_as_of": "ISO-8601 datetime"
  },
  "execution_ref": "ExecutionRef",
  "include_forecast": "bool",
  "timestamp": "ISO-8601 datetime"
}
```

### Output Schema

```json
{
  "signal_id": "UUID",
  "budget_id": "string",
  "signal_type": "advisory | warning | gating",
  "severity": "info | warning | critical | gating",
  "violation_type": "none | approaching_limit | limit_exceeded | projected_exceedance | unusual_pattern",
  "message": "string (human-readable)",
  "current_spend": "decimal",
  "budget_limit": "decimal",
  "remaining_budget": "decimal",
  "utilization_percent": "f64 (0-100)",
  "projected_spend": "decimal (optional)",
  "projected_utilization": "f64 (optional)",
  "days_remaining": "i64",
  "daily_average": "decimal",
  "recommended_action": "none | monitor | review | reduce_spend | consider_gating | gate",
  "alerts": [
    {
      "alert_type": "string",
      "severity": "info | warning | critical | gating",
      "message": "string",
      "timestamp": "ISO-8601 datetime"
    }
  ],
  "timestamp": "ISO-8601 datetime"
}
```

### DecisionEvent Schema

Every invocation emits exactly ONE DecisionEvent:

```json
{
  "id": "UUID",
  "agent_id": "budget-enforcement-agent",
  "agent_version": "1.0.0",
  "decision_type": "budget_constraint_evaluation",
  "inputs_hash": "SHA-256 of inputs",
  "outputs": { "/* full output */" },
  "confidence": "f64 (0.0-1.0)",
  "constraints_applied": [
    {
      "constraint_type": "budget_cap",
      "violated": "bool",
      "current_value": "string",
      "threshold_value": "string",
      "utilization_percent": "f64"
    }
  ],
  "execution_ref": "ExecutionRef",
  "timestamp": "ISO-8601 UTC datetime"
}
```

## CLI Contract

```bash
# Evaluate budget
llm-cost-ops agent budget-enforcement analyze \
  --tenant-id <tenant> \
  --budget-id <budget> \
  --execution-ref <execution-id>

# Inspect agent
cost-ops agent inspect --agent budget-enforcement --detailed

# List agents
cost-ops agent list
```

## Signal Severity Levels

| Level | Description | Recommended Action |
|-------|-------------|-------------------|
| Info | No action needed, budget on track | None |
| Warning | Approaching budget limit | Monitor |
| Critical | Limit exceeded or imminent | Review/Reduce Spend |
| Gating | Recommend blocking new requests | Gate |

## Violation Types

| Type | Trigger |
|------|---------|
| None | Utilization below warning threshold |
| ApproachingLimit | Utilization >= warning threshold |
| LimitExceeded | Utilization >= critical/gating threshold |
| ProjectedExceedance | Forecast shows exceeding limit |
| UnusualPattern | Abnormal spending detected |

## Confidence Calculation

Confidence is based on:
- **Data completeness** - Higher completeness = more confident
- **Historical data availability** - More data points = more confident (min 7 for forecasting)
- **Data freshness** - Stale data reduces confidence
- **Forecast accuracy** - If using forecast, applies forecast confidence factor

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
| Invalid budget limit | `VALIDATION_ERROR` | Provide positive budget limit |
| Invalid period | `VALIDATION_ERROR` | Ensure end date after start date |
| Invalid thresholds | `VALIDATION_ERROR` | Use values between 0.0 and 1.0 |
| RuVector error | `PERSISTENCE_ERROR` | DecisionEvent logged, output still returned |
| Forecasting error | `FORECASTING_ERROR` | Continues without forecast |

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
