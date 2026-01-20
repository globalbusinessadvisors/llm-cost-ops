# Cost Attribution Agent CLI

## Overview

The Cost Attribution Agent provides a command-line interface for deterministic LLM cost tracking and attribution. All commands emit DecisionEvents to the RuVector service for learning and improvement.

## Installation

```bash
cd /workspaces/cost-ops/agents/cost-attribution
npm install
npm run build
```

## Usage

```bash
# Run directly
node dist/cli/index.js <command> [options]

# Or after npm link
cost-attribution <command> [options]
```

## Commands

### 1. analyze

Run cost attribution analysis on usage records.

```bash
# From stdin (JSONL format)
echo '{"executionId":"ex1","agentId":"ag1","provider":"anthropic","model":"claude-3-opus","inputTokens":1000,"outputTokens":500,"requestCount":1,"timestamp":"2024-01-01T00:00:00Z"}' | cost-attribution analyze

# From file (JSON)
cost-attribution analyze --input usage.json --format table

# From file (JSONL)
cost-attribution analyze --input usage.jsonl --format json

# With summary statistics
cost-attribution analyze --input usage.json --summary

# Different attribution scopes
cost-attribution analyze --input usage.json --scope agent
cost-attribution analyze --input usage.json --scope workflow
cost-attribution analyze --input usage.json --scope tenant
```

**Options:**
- `-i, --input <file>` - Input file (JSON or JSONL) or "-" for stdin (default: "-")
- `-f, --format <format>` - Output format: json or table (default: "table")
- `-s, --scope <scope>` - Attribution scope: execution, agent, workflow, or tenant (default: "execution")
- `--summary` - Show summary statistics (default: false)

**Output:**
- JSON format: Array of CostRecord objects
- Table format: Human-readable table with columns for execution, agent, provider, model, tokens, and cost
- Summary: Aggregate statistics including total cost, average cost, token counts

### 2. inspect

Inspect a specific cost record and its decision history.

```bash
# Inspect by execution ID
cost-attribution inspect --id execution-123

# JSON output
cost-attribution inspect --id execution-123 --format json
```

**Options:**
- `-i, --id <id>` - Cost record ID (execution ID) - **required**
- `-f, --format <format>` - Output format: json or table (default: "table")

**Output:**
- Cost record details
- Decision history from RuVector service

### 3. batch

Process multiple usage records in parallel with configurable worker count.

```bash
# Process with 4 workers (default)
cost-attribution batch --input usage-batch.json --output costs.json

# Process with 8 workers
cost-attribution batch --input usage-batch.json --output costs.json --parallel 8

# Table format output
cost-attribution batch --input usage-batch.json --output costs.txt --format table --parallel 4
```

**Options:**
- `-i, --input <file>` - Input file (JSON or JSONL) - **required**
- `-o, --output <file>` - Output file for results - **required**
- `-p, --parallel <number>` - Number of parallel workers (default: 4)
- `-f, --format <format>` - Output format: json or table (default: "json")

**Output:**
- Writes cost records to output file
- Progress messages to stderr
- DecisionEvent with performance metrics

### 4. version

Show agent version and build information.

```bash
cost-attribution version
```

**Output (JSON):**
```json
{
  "version": "1.0.0",
  "name": "@llm-costops/cost-attribution-agent",
  "node": "v20.10.0",
  "platform": "linux",
  "arch": "x64"
}
```

## Input Format

### UsageRecord (JSON)

```json
{
  "executionId": "exec-123",
  "agentId": "agent-456",
  "workflowId": "workflow-789",
  "tenantId": "tenant-001",
  "provider": "anthropic",
  "model": "claude-3-opus",
  "inputTokens": 1000,
  "outputTokens": 500,
  "cachedInputTokens": 200,
  "requestCount": 1,
  "timestamp": "2024-01-01T00:00:00Z",
  "metadata": {}
}
```

**Required fields:**
- `executionId` - Unique execution identifier
- `agentId` - Agent identifier
- `provider` - LLM provider (e.g., "anthropic", "openai")
- `model` - Model identifier (e.g., "claude-3-opus", "gpt-4")
- `inputTokens` - Number of input tokens
- `outputTokens` - Number of output tokens
- `requestCount` - Number of requests (for per-request pricing)
- `timestamp` - ISO 8601 timestamp

**Optional fields:**
- `workflowId` - Workflow identifier (for workflow-scoped attribution)
- `tenantId` - Tenant identifier (for tenant-scoped attribution)
- `cachedInputTokens` - Number of cached input tokens (for cache discount)
- `metadata` - Additional metadata

### JSONL Format

For batch processing, use one JSON record per line:

```jsonl
{"executionId":"ex1","agentId":"ag1","provider":"anthropic","model":"claude-3-opus","inputTokens":1000,"outputTokens":500,"requestCount":1,"timestamp":"2024-01-01T00:00:00Z"}
{"executionId":"ex2","agentId":"ag2","provider":"openai","model":"gpt-4","inputTokens":2000,"outputTokens":1000,"requestCount":1,"timestamp":"2024-01-01T01:00:00Z"}
```

## Output Format

### CostRecord (JSON)

```json
{
  "executionId": "exec-123",
  "agentId": "agent-456",
  "workflowId": "workflow-789",
  "tenantId": "tenant-001",
  "provider": "anthropic",
  "model": "claude-3-opus",
  "inputTokens": 1000,
  "outputTokens": 500,
  "cachedInputTokens": 200,
  "inputTokenCost": "0.0150000000",
  "outputTokenCost": "0.0375000000",
  "cachedInputTokenCost": "0.0003000000",
  "requestCost": "0.0000000000",
  "totalCost": "0.0528000000",
  "currency": "USD",
  "timestamp": "2024-01-01T00:00:00Z",
  "calculatedAt": "2024-01-01T00:01:00Z"
}
```

**Cost breakdown:**
- `inputTokenCost` - Cost for input tokens (decimal string, 10 digits precision)
- `outputTokenCost` - Cost for output tokens
- `cachedInputTokenCost` - Cost for cached tokens (discounted)
- `requestCost` - Cost for requests (per-request pricing)
- `totalCost` - Total cost (sum of all costs)
- `currency` - Currency code (USD, EUR, GBP, JPY)
- `calculatedAt` - Timestamp when cost was calculated

### Table Format

```
┌──────────────────┬─────────────┬──────────┬──────────────────┬──────────────┬───────────────┬──────────────┬──────────┐
│ Execution ID     │ Agent       │ Provider │ Model            │ Input Tokens │ Output Tokens │ Total Cost   │ Currency │
├──────────────────┼─────────────┼──────────┼──────────────────┼──────────────┼───────────────┼──────────────┼──────────┤
│ exec-123         │ agent-456   │ anthropic│ claude-3-opus    │ 1,000        │ 500           │ 0.052800     │ USD      │
└──────────────────┴─────────────┴──────────┴──────────────────┴──────────────┴───────────────┴──────────────┴──────────┘
```

### Summary Statistics

```
┌──────────────────────────────┬──────────────────────┐
│ Metric                       │ Value                │
├──────────────────────────────┼──────────────────────┤
│ Total Records                │ 10                   │
│ Total Cost                   │ 0.528000 USD         │
│ Avg Cost per Record          │ 0.052800 USD         │
│ Total Input Tokens           │ 10,000               │
│ Total Output Tokens          │ 5,000                │
│ Total Cached Tokens          │ 2,000                │
│ Unique Providers             │ 2                    │
│ Unique Models                │ 3                    │
└──────────────────────────────┴──────────────────────┘
```

## DecisionEvent Emission

Every CLI invocation emits a DecisionEvent to stderr in JSON format. This enables the RuVector service to learn from CLI usage patterns.

**DecisionEvent structure:**

```json
{
  "_type": "decision_event",
  "id": "evt_1234567890_123456",
  "type": "cost-attribution",
  "timestamp": "2024-01-01T00:00:00.000Z",
  "input": {
    "command": "analyze",
    "args": {
      "input": "usage.json",
      "format": "table",
      "scope": "execution",
      "summary": false
    },
    "usageRecords": 10
  },
  "output": {
    "success": true,
    "recordsProcessed": 10,
    "totalCost": "0.528000",
    "currency": "USD"
  },
  "confidence": 0.95,
  "metadata": {
    "executionTimeMs": 45,
    "inputRecords": 10,
    "outputRecords": 10
  }
}
```

**Event types:**
- `cost-attribution` - Analyze command
- `cost-inspection` - Inspect command
- `batch-processing` - Batch command

**Confidence levels:**
- `0.95` - Successful execution
- `0.5` - Failed execution

## Pricing Configuration

The CLI includes default pricing tables for common providers:

**Anthropic:**
- `claude-3-opus`: $15/1M input, $75/1M output, $1.50/1M cached
- `claude-3-sonnet`: $3/1M input, $15/1M output, $0.30/1M cached

**OpenAI:**
- `gpt-4`: $30/1M input, $60/1M output

In production, pricing would be loaded from a configuration service or database.

## Environment Variables

- `RUVECTOR_SERVICE_URL` - URL for RuVector service (default: `http://localhost:8080`)

## Exit Codes

- `0` - Success
- `1` - Error (validation, file not found, calculation error)

## Examples

### Example 1: Single Usage Record

```bash
echo '{"executionId":"ex1","agentId":"coder","provider":"anthropic","model":"claude-3-opus","inputTokens":1000,"outputTokens":500,"requestCount":1,"timestamp":"2024-01-20T00:00:00Z"}' | cost-attribution analyze --format json
```

**Output:**
```json
[
  {
    "executionId": "ex1",
    "agentId": "coder",
    "provider": "anthropic",
    "model": "claude-3-opus",
    "inputTokens": 1000,
    "outputTokens": 500,
    "cachedInputTokens": 0,
    "inputTokenCost": "0.0150000000",
    "outputTokenCost": "0.0375000000",
    "cachedInputTokenCost": "0.0000000000",
    "requestCost": "0.0000000000",
    "totalCost": "0.0525000000",
    "currency": "USD",
    "timestamp": "2024-01-20T00:00:00.000Z",
    "calculatedAt": "2024-01-20T00:01:00.000Z"
  }
]
```

### Example 2: Batch Processing

Create `batch-input.jsonl`:
```jsonl
{"executionId":"ex1","agentId":"coder","provider":"anthropic","model":"claude-3-opus","inputTokens":1000,"outputTokens":500,"requestCount":1,"timestamp":"2024-01-20T00:00:00Z"}
{"executionId":"ex2","agentId":"tester","provider":"anthropic","model":"claude-3-sonnet","inputTokens":2000,"outputTokens":1000,"requestCount":1,"timestamp":"2024-01-20T01:00:00Z"}
{"executionId":"ex3","agentId":"reviewer","provider":"openai","model":"gpt-4","inputTokens":1500,"outputTokens":750,"requestCount":1,"timestamp":"2024-01-20T02:00:00Z"}
```

Run batch:
```bash
cost-attribution batch --input batch-input.jsonl --output costs.json --parallel 4
```

**Stderr output:**
```
Processing 3 records with 4 workers...
✓ Processed 3 records
✓ Results written to costs.json
{"_type":"decision_event","id":"evt_...","type":"batch-processing",...}
```

### Example 3: Summary Statistics

```bash
cost-attribution analyze --input batch-input.jsonl --summary --format table
```

**Output:**
```
┌──────────────────────────────┬──────────────────────┐
│ Metric                       │ Value                │
├──────────────────────────────┼──────────────────────┤
│ Total Records                │ 3                    │
│ Total Cost                   │ 0.136500 USD         │
│ Avg Cost per Record          │ 0.045500 USD         │
│ Total Input Tokens           │ 4,500                │
│ Total Output Tokens          │ 2,250                │
│ Total Cached Tokens          │ 0                    │
│ Unique Providers             │ 2                    │
│ Unique Models                │ 3                    │
└──────────────────────────────┴──────────────────────┘
```

## Architecture

The CLI follows a clean, deterministic architecture:

1. **Input Layer** (`utils/input.ts`)
   - Reads from files or stdin
   - Supports JSON and JSONL formats
   - Validates usage records

2. **Processing Layer** (`engine/calculator.ts`)
   - Pure, deterministic cost calculations
   - Decimal precision (no floating-point errors)
   - Supports multiple pricing models

3. **Output Layer** (`utils/output.ts`)
   - Formats results as JSON or tables
   - Generates summary statistics
   - Machine-readable output

4. **Event Layer** (`types/decision-event.ts`)
   - Emits DecisionEvents for learning
   - Tracks execution metrics
   - Integrates with RuVector service

5. **Command Layer** (`cli/commands/*.ts`)
   - Command implementations
   - Option parsing
   - Error handling

## Development

```bash
# Install dependencies
npm install

# Build
npm run build

# Run in development mode
npm run dev -- analyze --input test.json

# Run tests
npm test

# Lint
npm run lint
```

## License

MIT
