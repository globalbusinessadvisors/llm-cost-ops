# Cost Attribution Agent - Verification Checklist

## LLM-CostOps Constitution Compliance

### Deployment Model ✅
- [x] Agent executes inside the LLM-CostOps repository
- [x] Deploys as Google Cloud Edge Function
- [x] Part of unified CostOps service
- [x] Stateless at runtime
- [x] No local persistence

### Memory & Persistence ✅
- [x] Does NOT own persistence
- [x] All records persisted via ruvector-service HTTP calls
- [x] Never connects directly to Google SQL
- [x] Never executes SQL
- [x] Persistence occurs via ruvector-service client calls only

### Execution & Authority Rules ✅

**Agent MAY:**
- [x] Attribute costs to executions, agents, workflows, or tenants
- [x] Compute cost metrics
- [x] Emit structured cost records

**Agent MUST NOT:**
- [x] ~~Intercept runtime execution~~ (not implemented)
- [x] ~~Trigger retries~~ (not implemented)
- [x] ~~Execute workflows~~ (not implemented)
- [x] ~~Modify routing or execution behavior~~ (not implemented)
- [x] ~~Apply optimizations automatically~~ (not implemented)
- [x] ~~Enforce policies directly~~ (only emits analysis)

### Integration Rules ✅
- [x] Uses schemas from agentics-contracts (Zod schemas)
- [x] Compatible with LLM-Observatory telemetry
- [x] Produces outputs consumable by Orchestrator/Governance
- [x] Does not invoke other CostOps agents directly

### Agent Requirements ✅

**DecisionEvent Schema:**
- [x] `agent_id` - Identifies this agent
- [x] `agent_version` - Version tracking
- [x] `decision_type` - "cost_attribution"
- [x] `inputs_hash` - Hash of input data
- [x] `outputs` - Structured output data
- [x] `confidence` - Estimation certainty (0-1)
- [x] `constraints_applied` - Budget/ROI/cost caps
- [x] `execution_ref` - UUID reference
- [x] `timestamp` - UTC timestamp

**Endpoint Requirements:**
- [x] CLI-invokable endpoint (`analyze`, `inspect`, `batch`)
- [x] Deployable as Google Edge Function
- [x] Returns deterministic, machine-readable output
- [x] Validates inputs/outputs against contracts
- [x] Emits telemetry compatible with LLM-Observatory
- [x] Emits exactly ONE DecisionEvent per invocation

---

## Contract Verification

### Input Schema (CostAttributionInput)
```typescript
{
  usage_records: UsageRecord[]  // Required, min 1
  pricing_context?: PricingContext
  attribution_scope: 'execution' | 'agent' | 'workflow' | 'tenant'
}
```

### Output Schema (CostAttributionOutput)
```typescript
{
  attributions: AttributionResult[]
  summary: AttributionSummary
  metadata: {
    processed_at: string  // ISO datetime
    attribution_scope: string
    total_usage_records: number
    processing_time_ms?: number
  }
}
```

### DecisionEvent Schema
```typescript
{
  agent_id: string           // 'cost-attribution-agent'
  agent_version: string      // '1.0.0'
  decision_type: string      // 'cost_attribution'
  inputs_hash: string        // SHA-256 hash
  outputs: object            // CostAttributionOutput
  confidence: number         // 0.0 - 1.0
  constraints_applied: string[]
  execution_ref: string      // UUID
  timestamp: string          // ISO 8601 UTC
}
```

---

## CLI Contract

### Commands

| Command | Description | Options |
|---------|-------------|---------|
| `analyze` | Run cost attribution analysis | `--input`, `--format`, `--scope`, `--summary` |
| `inspect` | Inspect a specific cost record | `--id`, `--format` |
| `batch` | Process batch of usage records | `--input`, `--output`, `--parallel` |
| `version` | Show agent version | - |

### CLI Invocation Examples

```bash
# Basic analysis
cost-attribution analyze --input usage.json --format json

# With scope
cost-attribution analyze --input usage.json --scope tenant --summary

# From stdin
cat usage.json | cost-attribution analyze --input - --format table

# Batch processing
cost-attribution batch --input records.jsonl --output results.json --parallel 4

# Inspect specific record
cost-attribution inspect --id "uuid-here" --format json
```

---

## Smoke Tests

### 1. Edge Function Health Check
```bash
curl -X GET https://[FUNCTION_URL]/health
# Expected: { "status": "healthy", "version": "1.0.0", ... }
```

### 2. Cost Attribution Analysis
```bash
curl -X POST https://[FUNCTION_URL] \
  -H "Content-Type: application/json" \
  -d '{
    "usage_records": [{
      "id": "uuid",
      "timestamp": "2024-01-15T10:00:00Z",
      "provider": "Anthropic",
      "model": "claude-3-sonnet",
      "organization_id": "org-123",
      "prompt_tokens": 1000,
      "completion_tokens": 500,
      "total_tokens": 1500
    }],
    "attribution_scope": "execution"
  }'
# Expected: CostAttributionOutput with DecisionEvent
```

### 3. CLI Analysis
```bash
echo '[{"id":"test-uuid","timestamp":"2024-01-15T10:00:00Z","provider":"Anthropic","model":"claude-3-sonnet","organization_id":"org-123","prompt_tokens":1000,"completion_tokens":500,"total_tokens":1500}]' | \
  npx ts-node bin/cost-attribution.ts analyze --input - --format json
# Expected: JSON output with cost attributions
```

### 4. DecisionEvent Emission
```bash
# Monitor ruvector-service logs or query events
curl -X GET "https://ruvector.example.com/api/v1/events?agent_id=cost-attribution-agent&limit=1"
# Expected: Most recent DecisionEvent from this agent
```

---

## Non-Responsibilities

This agent explicitly does NOT:

| Capability | Responsible System |
|------------|-------------------|
| Intercept runtime traffic | N/A (analysis only) |
| Execute workflows | LLM-Orchestrator |
| Perform anomaly detection | Sentinel |
| Optimize configurations | Auto-Optimizer |
| Enforce security policies | Shield |
| Budget enforcement | Governance layer |
| Direct database access | ruvector-service |

---

## Failure Modes

| Failure | Handling |
|---------|----------|
| Invalid input schema | Return 400 with validation errors |
| Unknown provider/model | Use default pricing, emit warning |
| ruvector-service unavailable | Fire-and-forget, log error |
| LLM-Observatory unavailable | Fire-and-forget, log error |
| Pricing context missing | Use built-in default pricing |
| Rate limit exceeded | Return 429 with retry-after |

---

## Version Information

- **Agent ID**: `cost-attribution-agent`
- **Agent Version**: `1.0.0`
- **Decision Type**: `cost_attribution`
- **Classification**: COST ACCOUNTING

---

## Files Created

```
agents/cost-attribution/
├── package.json              # Package configuration
├── tsconfig.json             # TypeScript configuration
├── jest.config.js            # Jest test configuration
├── bin/
│   └── cost-attribution.ts   # CLI entry point
├── src/
│   ├── index.ts              # Main exports
│   ├── contracts/
│   │   ├── index.ts          # Contract exports
│   │   ├── schemas.ts        # Zod schemas
│   │   └── types.ts          # TypeScript types
│   ├── engine/
│   │   ├── index.ts          # Engine exports
│   │   ├── calculator.ts     # Cost calculation
│   │   ├── attributor.ts     # Cost attribution
│   │   └── normalizer.ts     # Token normalization
│   ├── services/
│   │   ├── index.ts          # Service exports
│   │   ├── ruvector-client.ts # ruvector-service client
│   │   └── telemetry.ts      # Telemetry emitter
│   ├── handler/
│   │   ├── index.ts          # Edge function handler
│   │   ├── middleware.ts     # Request middleware
│   │   └── response.ts       # Response formatters
│   ├── cli/
│   │   ├── index.ts          # CLI main
│   │   └── commands/
│   │       ├── analyze.ts    # analyze command
│   │       ├── inspect.ts    # inspect command
│   │       └── batch.ts      # batch command
│   └── utils/
│       ├── input.ts          # Input utilities
│       └── output.ts         # Output formatters
└── tests/
    ├── calculator.test.ts    # Calculator tests
    ├── attributor.test.ts    # Attributor tests
    ├── decision-event.test.ts # DecisionEvent tests
    └── ruvector-client.test.ts # Client tests
```
