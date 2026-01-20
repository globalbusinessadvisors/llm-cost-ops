# Budget Enforcement Agent

**Agent ID:** `budget-enforcement-agent`
**Version:** `1.0.0`
**Classification:** FINANCIAL GOVERNANCE

## Purpose

Evaluates budget thresholds and emits advisory or gating signals when limits are approached or exceeded. This agent does NOT enforce budgets directly - it only emits signals that downstream systems can consume.

## Capabilities

- Analyze current and projected spend against budget limits
- Compute confidence scores based on data completeness
- Emit advisory signals (informational, warning, gating)
- Persist DecisionEvents to ruvector-service
- Emit telemetry compatible with LLM-Observatory

## What This Agent MUST NOT DO

- Intercept runtime execution
- Trigger retries
- Execute workflows
- Modify routing or execution behavior
- Apply optimizations automatically
- Enforce policies directly (only emit constraints/advisories)
- Execute SQL directly
- Connect to Google SQL

## Source Code

- **Rust Implementation:** `crates/llm-cost-ops/src/agents/budget_enforcement.rs`

## Signal Types

| Severity | Description |
|----------|-------------|
| Info | Informational, no action needed |
| Warning | Approaching limit |
| Critical | Limit exceeded or imminent |
| Gating | Recommend blocking |

## CLI Usage

```bash
llm-cost-ops agent budget-enforcement analyze \
  --tenant-id <tenant> \
  --budget-id <budget> \
  --execution-ref <execution-id>
```

## Documentation

See [docs/CONTRACT.md](docs/CONTRACT.md) for the full contract specification.
