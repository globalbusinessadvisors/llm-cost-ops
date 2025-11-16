# CLI Reference

**Version:** 1.0.0
**Last Updated:** 2025-11-16
**Binary Name:** `cost-ops`

Complete command-line interface reference for the LLM Cost Ops platform. The CLI provides full platform functionality for usage tracking, cost analysis, reporting, and administration.

---

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Authentication](#authentication)
- [Global Flags](#global-flags)
- [Commands](#commands)
  - [init](#init)
  - [ingest](#ingest)
  - [query](#query)
  - [summary](#summary)
  - [export](#export)
  - [pricing](#pricing)
  - [forecast](#forecast)
  - [report](#report)
  - [auth](#auth)
  - [config](#config)
  - [server](#server)
- [Output Formats](#output-formats)
- [Environment Variables](#environment-variables)
- [Shell Completion](#shell-completion)
- [Scripting and Automation](#scripting-and-automation)

---

## Installation

### Binary Installation

Download the latest release for your platform:

**Linux:**
```bash
curl -L https://github.com/llm-cost-ops/releases/latest/download/cost-ops-linux-amd64 -o cost-ops
chmod +x cost-ops
sudo mv cost-ops /usr/local/bin/
```

**macOS:**
```bash
curl -L https://github.com/llm-cost-ops/releases/latest/download/cost-ops-darwin-amd64 -o cost-ops
chmod +x cost-ops
sudo mv cost-ops /usr/local/bin/
```

**Windows:**
```powershell
Invoke-WebRequest -Uri https://github.com/llm-cost-ops/releases/latest/download/cost-ops-windows-amd64.exe -OutFile cost-ops.exe
```

### Package Managers

**Homebrew (macOS/Linux):**
```bash
brew install llm-cost-ops/tap/cost-ops
```

**Snap (Linux):**
```bash
sudo snap install cost-ops
```

**Cargo (from source):**
```bash
cargo install cost-ops
```

### Build from Source

```bash
git clone https://github.com/llm-cost-ops/llm-cost-ops.git
cd llm-cost-ops
cargo build --release
./target/release/cost-ops --version
```

### Verify Installation

```bash
cost-ops --version
# Output: cost-ops 1.0.0
```

---

## Configuration

The CLI reads configuration from multiple sources in this order:

1. Command-line flags
2. Environment variables
3. Configuration file
4. Default values

### Configuration File

Create `~/.config/cost-ops/config.toml`:

```toml
# Database connection
[database]
url = "postgresql://user:pass@localhost/costops"
pool_size = 10

# API server settings
[api]
bind = "0.0.0.0"
port = 8080

# Logging configuration
[logging]
level = "info"  # debug, info, warn, error
json = false

# Default organization
[defaults]
organization_id = "org-123"

# Authentication
[auth]
api_key = "sk_live_abc123"
api_url = "https://api.llm-cost-ops.example.com"
```

### Configuration File Locations

The CLI searches for config files in:

1. `./cost-ops.toml` (current directory)
2. `~/.config/cost-ops/config.toml` (user config)
3. `/etc/cost-ops/config.toml` (system config)

Specify custom config file:
```bash
cost-ops --config /path/to/config.toml query
```

---

## Authentication

### API Key Authentication

Set your API key:

```bash
export COST_OPS_API_KEY=sk_live_abc123
```

Or use the config file:
```toml
[auth]
api_key = "sk_live_abc123"
```

Or pass as flag:
```bash
cost-ops --api-key sk_live_abc123 query
```

### Interactive Login

```bash
cost-ops auth login
# Enter your email: user@example.com
# Enter your password: ********
# Successfully logged in!
```

Login credentials are stored in `~/.config/cost-ops/credentials`:

```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "expires_at": "2025-11-16T11:00:00Z"
}
```

### Logout

```bash
cost-ops auth logout
```

### Check Authentication

```bash
cost-ops auth whoami
# Authenticated as: user@example.com
# Organization: org-123
# Roles: admin
```

---

## Global Flags

Global flags can be used with any command:

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--config` | `-c` | Config file path | Auto-detected |
| `--verbose` | `-v` | Verbose output (-v, -vv, -vvv) | 0 |
| `--quiet` | `-q` | Suppress output | false |
| `--output` | `-o` | Output format (json, table, csv, yaml) | table |
| `--no-color` | | Disable colored output | false |
| `--api-key` | | API key for authentication | From env/config |
| `--api-url` | | API base URL | From config |
| `--help` | `-h` | Show help | |
| `--version` | `-V` | Show version | |

### Examples

**Verbose output:**
```bash
cost-ops -v query
cost-ops -vv query  # More verbose
cost-ops -vvv query # Debug level
```

**JSON output:**
```bash
cost-ops --output json query
```

**Custom config:**
```bash
cost-ops --config /etc/cost-ops/prod.toml query
```

---

## Commands

### init

Initialize database and create schema.

**Usage:**
```bash
cost-ops init [OPTIONS]
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--database-url` | Database URL | From config |
| `--force` | Force re-initialization | false |
| `--seed` | Seed with sample data | false |

**Examples:**

Initialize with SQLite:
```bash
cost-ops init --database-url sqlite:cost-ops.db
```

Initialize with PostgreSQL:
```bash
cost-ops init --database-url postgresql://user:pass@localhost/costops
```

Force re-initialization:
```bash
cost-ops init --force
```

Initialize with sample data:
```bash
cost-ops init --seed
```

**Output:**
```
Initializing database...
Running migrations...
✓ Migration 001_initial_schema applied
✓ Migration 002_add_pricing applied
✓ Migration 003_add_forecasting applied
Database initialized successfully!
```

---

### ingest

Ingest usage metrics from files or stdin.

**Usage:**
```bash
cost-ops ingest [OPTIONS]
```

**Options:**

| Flag | Short | Description | Required |
|------|-------|-------------|----------|
| `--file` | `-f` | Input file path (JSON/JSONL) | Yes |
| `--provider` | `-p` | Override provider | No |
| `--format` | | Input format (json, jsonl) | Auto-detect |
| `--batch-size` | | Batch size for processing | 100 |
| `--dry-run` | | Validate without ingesting | false |

**Examples:**

Ingest from JSON file:
```bash
cost-ops ingest --file usage.json
```

Ingest from JSONL:
```bash
cost-ops ingest --file usage.jsonl --format jsonl
```

Ingest from stdin:
```bash
cat usage.json | cost-ops ingest --file -
```

Dry run (validate only):
```bash
cost-ops ingest --file usage.json --dry-run
```

**Input Format (JSON):**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "timestamp": "2025-11-16T10:00:00Z",
    "provider": "openai",
    "model": {
      "name": "gpt-4-turbo",
      "context_window": 128000
    },
    "organization_id": "org-123",
    "prompt_tokens": 1500,
    "completion_tokens": 800,
    "total_tokens": 2300,
    "cached_tokens": 500
  }
]
```

**Input Format (JSONL):**
```json
{"id": "550e8400-e29b-41d4-a716-446655440001", "timestamp": "2025-11-16T10:00:00Z", "provider": "openai", "model": {"name": "gpt-4-turbo"}, "organization_id": "org-123", "prompt_tokens": 1500, "completion_tokens": 800, "total_tokens": 2300}
{"id": "550e8400-e29b-41d4-a716-446655440002", "timestamp": "2025-11-16T10:05:00Z", "provider": "anthropic", "model": {"name": "claude-3-sonnet-20240229"}, "organization_id": "org-123", "prompt_tokens": 2000, "completion_tokens": 1000, "total_tokens": 3000}
```

**Output:**
```
Ingesting usage data from usage.json...
✓ Validated 100 records
Processing batches...
[=============================>] 100/100
✓ Successfully ingested 100 records
✗ Failed: 0 records

Summary:
  Total records: 100
  Successful: 100
  Failed: 0
  Total cost: $12.34 USD
  Processing time: 2.3s
```

---

### query

Query cost records with filtering.

**Usage:**
```bash
cost-ops query [OPTIONS]
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--range` | Time range preset | last-24-hours |
| `--start-date` | Start date (ISO 8601) | |
| `--end-date` | End date (ISO 8601) | |
| `--organization` | Organization ID | From config |
| `--provider` | Provider filter | |
| `--model` | Model filter | |
| `--group-by` | Group by dimension | |
| `--limit` | Result limit | 100 |

**Time Range Presets:**
- `last-hour` - Last hour
- `last-24-hours` - Last 24 hours
- `last-7-days` - Last 7 days
- `last-30-days` - Last 30 days
- `this-week` - Current week (Mon-Sun)
- `last-week` - Previous week
- `this-month` - Current month
- `last-month` - Previous month
- `this-year` - Current year

**Examples:**

Query last 24 hours:
```bash
cost-ops query --range last-24-hours
```

Query with date range:
```bash
cost-ops query --start-date 2025-11-01 --end-date 2025-11-16
```

Filter by provider:
```bash
cost-ops query --provider openai --range last-7-days
```

Filter by model:
```bash
cost-ops query --model gpt-4-turbo --range last-30-days
```

Group by provider:
```bash
cost-ops query --group-by provider --range last-30-days
```

Multiple filters:
```bash
cost-ops query --provider openai --model gpt-4 --range last-7-days --group-by day
```

JSON output:
```bash
cost-ops query --output json --range last-7-days > costs.json
```

**Table Output:**
```
┌──────────────────────┬──────────┬───────────────┬────────────┬──────────────┬──────────┐
│ Timestamp            │ Provider │ Model         │ Tokens     │ Cost         │ Currency │
├──────────────────────┼──────────┼───────────────┼────────────┼──────────────┼──────────┤
│ 2025-11-16 10:00:00 │ openai   │ gpt-4-turbo   │ 2300       │ $0.023       │ USD      │
│ 2025-11-16 10:05:00 │ anthropic│ claude-3-opus │ 3000       │ $0.045       │ USD      │
│ 2025-11-16 10:10:00 │ openai   │ gpt-4-turbo   │ 1800       │ $0.018       │ USD      │
└──────────────────────┴──────────┴───────────────┴────────────┴──────────────┴──────────┘

Total: $0.086 USD (3 records, 7100 tokens)
```

**JSON Output:**
```json
{
  "data": [
    {
      "timestamp": "2025-11-16T10:00:00Z",
      "provider": "openai",
      "model_id": "gpt-4-turbo",
      "total_tokens": 2300,
      "total_cost": 0.023,
      "currency": "USD"
    }
  ],
  "summary": {
    "total_cost": 0.086,
    "total_records": 3,
    "total_tokens": 7100
  }
}
```

---

### summary

Generate cost summary reports.

**Usage:**
```bash
cost-ops summary [OPTIONS]
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--period` | Time period | last-30-days |
| `--organization` | Organization ID | From config |
| `--breakdown` | Include breakdowns | false |

**Examples:**

Monthly summary:
```bash
cost-ops summary --period last-30-days
```

With breakdowns:
```bash
cost-ops summary --period last-30-days --breakdown
```

Custom organization:
```bash
cost-ops summary --organization org-456 --period last-7-days
```

**Output:**
```
=== Cost Summary ===
Period: 2025-10-17 to 2025-11-16
Organization: org-123

Total Cost: $1,234.56 USD
Total Requests: 25,000
Total Tokens: 15,000,000
Average Cost/Request: $0.0494

--- By Provider ---
openai:     $789.12 (64.0%)  15,000 requests
anthropic:  $445.44 (36.0%)  10,000 requests

--- By Model ---
gpt-4-turbo:           $520.30  10,000 requests
claude-3-opus:         $380.20   8,000 requests
gpt-3.5-turbo:         $268.82   5,000 requests
claude-3-sonnet:        $65.24   2,000 requests

--- Daily Trend ---
2025-11-16: $52.34  1,200 requests  ↑
2025-11-15: $48.90  1,150 requests  ↑
2025-11-14: $45.67  1,100 requests  ↑
...

Trend: Increasing (+8.5% vs last period)
```

---

### export

Export cost and usage data.

**Usage:**
```bash
cost-ops export [OPTIONS]
```

**Options:**

| Flag | Short | Description | Required |
|------|-------|-------------|----------|
| `--output` | `-o` | Output file path | Yes |
| `--format` | `-f` | Export format | json |
| `--period` | | Time period | last-30-days |
| `--start-date` | | Start date | |
| `--end-date` | | End date | |
| `--compress` | | Compress output (gzip) | false |

**Supported Formats:**
- `json` - JSON format
- `jsonl` - JSON Lines (newline-delimited)
- `csv` - Comma-separated values
- `xlsx` - Excel spreadsheet
- `parquet` - Apache Parquet

**Examples:**

Export to JSON:
```bash
cost-ops export --output costs.json --format json --period last-30-days
```

Export to CSV:
```bash
cost-ops export --output costs.csv --format csv --period last-7-days
```

Export to Excel:
```bash
cost-ops export --output costs.xlsx --format xlsx --period last-month
```

Export with compression:
```bash
cost-ops export --output costs.json.gz --format json --compress
```

Custom date range:
```bash
cost-ops export --output costs.csv --format csv \
  --start-date 2025-11-01 --end-date 2025-11-16
```

**Output:**
```
Exporting cost data...
Date range: 2025-10-17 to 2025-11-16
Format: CSV

[=============================>] 100%

✓ Exported 25,000 records
✓ File size: 5.2 MB
✓ Saved to: costs.csv
```

---

### pricing

Manage pricing tables.

**Usage:**
```bash
cost-ops pricing <SUBCOMMAND>
```

**Subcommands:**
- `list` - List pricing tables
- `add` - Add new pricing
- `get` - Get pricing by ID
- `update` - Update pricing
- `delete` - Delete pricing

#### pricing list

List all pricing tables.

**Usage:**
```bash
cost-ops pricing list [OPTIONS]
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--provider` | Filter by provider | |
| `--model` | Filter by model | |
| `--active-only` | Show only active pricing | true |

**Examples:**

List all pricing:
```bash
cost-ops pricing list
```

Filter by provider:
```bash
cost-ops pricing list --provider openai
```

Include inactive pricing:
```bash
cost-ops pricing list --active-only=false
```

**Output:**
```
┌──────────────┬──────────┬────────────────┬───────────┬────────────┬──────────────────────┐
│ ID           │ Provider │ Model          │ Input/1K  │ Output/1K  │ Effective Date       │
├──────────────┼──────────┼────────────────┼───────────┼────────────┼──────────────────────┤
│ price-abc123 │ openai   │ gpt-4-turbo    │ $0.0100   │ $0.0300    │ 2025-11-01           │
│ price-abc124 │ anthropic│ claude-3-opus  │ $0.0150   │ $0.0750    │ 2025-11-01           │
│ price-abc125 │ openai   │ gpt-3.5-turbo  │ $0.0005   │ $0.0015    │ 2025-11-01           │
└──────────────┴──────────┴────────────────┴───────────┴────────────┴──────────────────────┘

Total: 3 active pricing tables
```

#### pricing add

Add new pricing table.

**Usage:**
```bash
cost-ops pricing add [OPTIONS]
```

**Options:**

| Flag | Description | Required |
|------|-------------|----------|
| `--provider` | Provider name | Yes |
| `--model` | Model name | Yes |
| `--input-price` | Input price per million tokens | Yes |
| `--output-price` | Output price per million tokens | Yes |
| `--effective-date` | Effective date (ISO 8601) | Today |
| `--currency` | Currency code | USD |

**Examples:**

Add OpenAI pricing:
```bash
cost-ops pricing add \
  --provider openai \
  --model gpt-4-turbo \
  --input-price 10.0 \
  --output-price 30.0
```

Add with effective date:
```bash
cost-ops pricing add \
  --provider anthropic \
  --model claude-3-sonnet-20240229 \
  --input-price 3.0 \
  --output-price 15.0 \
  --effective-date 2025-12-01
```

**Output:**
```
Adding pricing for openai/gpt-4-turbo...
✓ Pricing created successfully
  ID: price-abc123
  Input: $10.00 per 1M tokens
  Output: $30.00 per 1M tokens
  Effective: 2025-11-16
```

#### pricing get

Get pricing details.

**Usage:**
```bash
cost-ops pricing get [OPTIONS]
```

**Options:**

| Flag | Description | Required |
|------|-------------|----------|
| `--provider` | Provider name | Yes |
| `--model` | Model name | Yes |
| `--date` | Date for pricing lookup | Today |

**Examples:**

Get current pricing:
```bash
cost-ops pricing get --provider openai --model gpt-4-turbo
```

Get historical pricing:
```bash
cost-ops pricing get --provider openai --model gpt-4-turbo --date 2025-10-01
```

**Output:**
```
Pricing for openai/gpt-4-turbo
Effective: 2025-11-01 to present

Input:  $10.00 per 1M tokens
Output: $30.00 per 1M tokens
Currency: USD

Pricing Structure: Per-Token
  • Standard input pricing
  • Standard output pricing
  • 50% discount for cached tokens
```

---

### forecast

Generate cost forecasts and detect anomalies.

**Usage:**
```bash
cost-ops forecast <SUBCOMMAND>
```

**Subcommands:**
- `generate` - Generate cost forecast
- `anomalies` - Detect cost anomalies
- `budget` - Budget forecasting

#### forecast generate

Generate cost forecast.

**Usage:**
```bash
cost-ops forecast generate [OPTIONS]
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--horizon` | Forecast horizon (days) | 30 |
| `--model` | Forecast model | exponential_smoothing |
| `--organization` | Organization ID | From config |

**Forecast Models:**
- `linear` - Linear regression
- `moving_average` - Moving average
- `exponential_smoothing` - Exponential smoothing

**Examples:**

Generate 30-day forecast:
```bash
cost-ops forecast generate --horizon 30
```

Use specific model:
```bash
cost-ops forecast generate --model linear --horizon 60
```

**Output:**
```
Generating cost forecast...
Model: Exponential Smoothing
Horizon: 30 days
Historical period: 2025-10-17 to 2025-11-16

Forecast Summary:
  Total predicted cost: $1,380.50
  Average daily cost: $46.02
  Trend: Increasing (+2.3%)
  Seasonality: Detected (weekly pattern)

Next 7 Days:
  2025-11-17: $45.67 ($38.20 - $53.14)
  2025-11-18: $46.12 ($38.50 - $53.74)
  2025-11-19: $46.58 ($38.81 - $54.35)
  2025-11-20: $47.05 ($39.13 - $54.97)
  2025-11-21: $47.52 ($39.45 - $55.59)
  2025-11-22: $48.00 ($39.78 - $56.22)
  2025-11-23: $48.48 ($40.11 - $56.85)

Forecast accuracy metrics:
  MAPE: 5.2%
  RMSE: $2.34
  MAE: $1.87
```

#### forecast anomalies

Detect cost anomalies.

**Usage:**
```bash
cost-ops forecast anomalies [OPTIONS]
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--period` | Time period | last-30-days |
| `--method` | Detection method | zscore |
| `--sensitivity` | Sensitivity level | medium |

**Detection Methods:**
- `zscore` - Z-score method
- `iqr` - Interquartile range
- `prophet` - Facebook Prophet

**Sensitivity Levels:**
- `low` - Fewer alerts, only severe anomalies
- `medium` - Balanced
- `high` - More alerts, catch minor anomalies

**Examples:**

Detect anomalies:
```bash
cost-ops forecast anomalies --period last-30-days
```

High sensitivity:
```bash
cost-ops forecast anomalies --sensitivity high
```

**Output:**
```
Detecting cost anomalies...
Period: 2025-10-17 to 2025-11-16
Method: Z-Score
Sensitivity: Medium

Found 3 anomalies:

⚠️  HIGH SEVERITY
Date: 2025-11-10
Actual: $125.50
Expected: $45.67
Deviation: +174.8%
Z-Score: 3.5
Description: Significant cost spike detected

⚠️  MEDIUM SEVERITY
Date: 2025-11-05
Actual: $72.30
Expected: $46.20
Deviation: +56.5%
Z-Score: 2.1
Description: Moderate cost increase

⚠️  MEDIUM SEVERITY
Date: 2025-10-28
Actual: $18.90
Expected: $44.50
Deviation: -57.5%
Z-Score: -2.3
Description: Unusually low cost
```

---

### report

Generate and manage reports.

**Usage:**
```bash
cost-ops report <SUBCOMMAND>
```

**Subcommands:**
- `generate` - Generate one-time report
- `schedule` - Schedule recurring report
- `list` - List scheduled reports
- `cancel` - Cancel scheduled report

#### report generate

Generate a report.

**Usage:**
```bash
cost-ops report generate [OPTIONS]
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--type` | Report type | cost_summary |
| `--period` | Time period | last-30-days |
| `--format` | Output format | pdf |
| `--output` | Output file path | |
| `--email` | Email recipients | |

**Report Types:**
- `cost_summary` - Cost summary report
- `usage_analysis` - Usage analysis
- `forecast` - Forecast report
- `budget` - Budget vs actual
- `audit` - Audit trail

**Examples:**

Generate cost summary:
```bash
cost-ops report generate --type cost_summary --period last-month
```

Save to file:
```bash
cost-ops report generate --type cost_summary --output report.pdf
```

Email report:
```bash
cost-ops report generate --type cost_summary --email finance@example.com
```

---

### auth

Authentication management.

**Subcommands:**
- `login` - Login with credentials
- `logout` - Logout
- `whoami` - Show current user
- `api-key` - Manage API keys

#### auth login

Interactive login.

**Usage:**
```bash
cost-ops auth login
```

**Examples:**

```bash
cost-ops auth login
# Enter your email: user@example.com
# Enter your password: ********
# ✓ Successfully logged in!
```

Non-interactive:
```bash
cost-ops auth login --email user@example.com --password-stdin < password.txt
```

#### auth api-key

Manage API keys.

**Subcommands:**
- `create` - Create API key
- `list` - List API keys
- `revoke` - Revoke API key

**Examples:**

Create API key:
```bash
cost-ops auth api-key create --name "Production Key" --scopes usage:write,costs:read
```

List API keys:
```bash
cost-ops auth api-key list
```

Revoke API key:
```bash
cost-ops auth api-key revoke key_abc123
```

---

### config

Configuration management.

**Subcommands:**
- `show` - Show current configuration
- `set` - Set configuration value
- `get` - Get configuration value

**Examples:**

Show config:
```bash
cost-ops config show
```

Set value:
```bash
cost-ops config set defaults.organization_id org-123
```

Get value:
```bash
cost-ops config get database.url
```

---

### server

Start API server.

**Usage:**
```bash
cost-ops server [OPTIONS]
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--bind` | Bind address | 0.0.0.0 |
| `--port` | Port number | 8080 |
| `--workers` | Worker threads | CPU count |

**Examples:**

Start server:
```bash
cost-ops server
```

Custom port:
```bash
cost-ops server --port 3000
```

Production mode:
```bash
cost-ops server --bind 0.0.0.0 --port 8080 --workers 8
```

---

## Output Formats

### Table Format (default)

Human-readable table output.

```bash
cost-ops query --output table
```

### JSON Format

Machine-readable JSON.

```bash
cost-ops query --output json
```

```json
{
  "data": [...],
  "summary": {...}
}
```

### CSV Format

Comma-separated values.

```bash
cost-ops query --output csv
```

```csv
timestamp,provider,model_id,total_tokens,total_cost,currency
2025-11-16T10:00:00Z,openai,gpt-4-turbo,2300,0.023,USD
```

### YAML Format

YAML output.

```bash
cost-ops query --output yaml
```

```yaml
data:
  - timestamp: 2025-11-16T10:00:00Z
    provider: openai
    model_id: gpt-4-turbo
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `COST_OPS_API_KEY` | API key for authentication | |
| `COST_OPS_API_URL` | API base URL | https://api.llm-cost-ops.example.com |
| `COST_OPS_CONFIG` | Config file path | Auto-detect |
| `DATABASE_URL` | Database connection URL | sqlite:cost-ops.db |
| `RUST_LOG` | Log level (debug, info, warn, error) | info |
| `NO_COLOR` | Disable colored output | false |

**Examples:**

```bash
export COST_OPS_API_KEY=sk_live_abc123
export DATABASE_URL=postgresql://user:pass@localhost/costops
export RUST_LOG=debug

cost-ops query
```

---

## Shell Completion

Generate shell completion scripts for faster command entry.

### Bash

```bash
cost-ops completion bash > /etc/bash_completion.d/cost-ops
source ~/.bashrc
```

### Zsh

```bash
cost-ops completion zsh > /usr/local/share/zsh/site-functions/_cost-ops
```

### Fish

```bash
cost-ops completion fish > ~/.config/fish/completions/cost-ops.fish
```

### PowerShell

```powershell
cost-ops completion powershell >> $PROFILE
```

---

## Scripting and Automation

### Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Authentication error |
| 4 | API error |
| 5 | Database error |

### JSON Output for Scripting

```bash
#!/bin/bash

# Get costs as JSON
costs=$(cost-ops query --output json --range last-24-hours)

# Parse with jq
total_cost=$(echo "$costs" | jq -r '.summary.total_cost')

echo "Total cost: $total_cost"
```

### Batch Processing

```bash
#!/bin/bash

# Process multiple files
for file in usage_*.json; do
  echo "Processing $file..."
  cost-ops ingest --file "$file"
done
```

### Cron Jobs

```bash
# Daily cost report at 9 AM
0 9 * * * /usr/local/bin/cost-ops report generate --type cost_summary --period yesterday --email finance@example.com

# Weekly export every Monday at 8 AM
0 8 * * 1 /usr/local/bin/cost-ops export --output /backups/costs_$(date +\%Y-\%m-\%d).csv --period last-week
```

### Error Handling

```bash
#!/bin/bash
set -e

if ! cost-ops query --range last-24-hours > /dev/null 2>&1; then
  echo "Error querying costs" >&2
  exit 1
fi

echo "Query successful"
```

---

**See Also:**

- [API Reference](./api-reference.md)
- [Configuration Reference](./configuration.md)
- [Troubleshooting Guide](./troubleshooting.md)
