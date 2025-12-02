# LLM Cost Ops - CLI

[![Crates.io](https://img.shields.io/crates/v/llm-cost-ops-cli.svg)](https://crates.io/crates/llm-cost-ops-cli)
[![Documentation](https://docs.rs/llm-cost-ops-cli/badge.svg)](https://docs.rs/llm-cost-ops-cli)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Command-line interface for LLM Cost Ops platform**

A powerful, user-friendly CLI tool for managing LLM costs, generating reports, and administering the Cost Ops platform.

## Features

- **Database Management** - Initialize, migrate, and manage databases
- **Data Import/Export** - Import usage data from various formats
- **Report Generation** - Generate and download cost/usage reports
- **Cost Queries** - Query and analyze cost data
- **User Management** - Create and manage users and API keys
- **Organization Management** - Manage organizations and permissions
- **Server Management** - Start and manage API servers
- **Interactive Mode** - REPL-style interactive shell
- **Benchmarks** - Run performance benchmarks and generate reports

## Installation

### From crates.io

```bash
cargo install llm-cost-ops-cli
```

### From source

```bash
git clone https://github.com/globalbusinessadvisors/llm-cost-ops
cd llm-cost-ops
cargo install --path crates/llm-cost-ops-cli
```

### Verify installation

```bash
cost-ops --version
```

## Quick Start

### Initialize Database

```bash
# SQLite (default)
cost-ops init --database-url sqlite:cost-ops.db

# PostgreSQL
cost-ops init --database-url postgresql://user:pass@localhost/costops
```

### Import Usage Data

```bash
# Import from OpenAI format
cost-ops import \
  --file usage-data.json \
  --format openai \
  --organization org-123

# Import from Anthropic format
cost-ops import \
  --file claude-usage.csv \
  --format anthropic \
  --organization org-123

# Batch import
cost-ops import \
  --directory ./usage-logs \
  --format openai \
  --batch-size 1000
```

### Query Costs

```bash
# Query costs for organization
cost-ops query \
  --organization org-123 \
  --start-date 2024-01-01 \
  --end-date 2024-01-31

# Query with grouping
cost-ops query \
  --organization org-123 \
  --group-by provider,model \
  --output table

# Export to CSV
cost-ops query \
  --organization org-123 \
  --output csv \
  --output-file costs.csv
```

### Generate Reports

```bash
# Cost report
cost-ops report \
  --type cost \
  --organization org-123 \
  --format pdf \
  --output monthly-report.pdf

# Usage report
cost-ops report \
  --type usage \
  --organization org-123 \
  --format excel \
  --output usage-report.xlsx

# Email report
cost-ops report \
  --type cost \
  --organization org-123 \
  --format csv \
  --email finance@example.com
```

### Forecast Costs

```bash
# 30-day forecast
cost-ops forecast \
  --organization org-123 \
  --horizon 30 \
  --model linear-trend

# Anomaly detection
cost-ops forecast anomalies \
  --organization org-123 \
  --lookback-days 90
```

### User Management

```bash
# Create user
cost-ops user create \
  --username user@example.com \
  --organization org-123 \
  --role admin

# Generate API key
cost-ops user api-key \
  --username user@example.com \
  --expires-in 90d

# List users
cost-ops user list --organization org-123

# Update roles
cost-ops user update-roles \
  --username user@example.com \
  --roles admin,analyst
```

### Organization Management

```bash
# Create organization
cost-ops org create \
  --name "Acme Corp" \
  --id acme-corp

# Set budget
cost-ops org budget \
  --organization org-123 \
  --amount 10000 \
  --period monthly

# List organizations
cost-ops org list
```

### Server Management

```bash
# Start API server
cost-ops server start \
  --port 8080 \
  --database-url postgresql://user:pass@localhost/costops

# Start with custom config
cost-ops server start --config config.toml

# Health check
cost-ops server health --url http://localhost:8080
```

## Commands

### Database Commands

```bash
# Initialize database
cost-ops init --database-url <url>

# Run migrations
cost-ops migrate up

# Rollback migrations
cost-ops migrate down

# Reset database (WARNING: deletes all data)
cost-ops migrate reset --confirm

# Database status
cost-ops db status
```

### Import Commands

```bash
# Import usage data
cost-ops import \
  --file <path> \
  --format <openai|anthropic|vertex|azure> \
  --organization <org-id>

# Import pricing data
cost-ops import pricing \
  --file pricing.json \
  --provider openai

# Import batch
cost-ops import batch \
  --directory <path> \
  --pattern "*.json"
```

### Query Commands

```bash
# Query costs
cost-ops query \
  [--organization <org-id>] \
  [--provider <provider>] \
  [--model <model>] \
  [--start-date <date>] \
  [--end-date <date>] \
  [--group-by <field>] \
  [--output <format>]

# Top consumers
cost-ops query top \
  --by cost \
  --limit 10

# Usage summary
cost-ops query summary \
  --organization org-123 \
  --period monthly
```

### Report Commands

```bash
# Generate report
cost-ops report \
  --type <cost|usage|forecast|audit> \
  --organization <org-id> \
  --format <csv|excel|json|pdf> \
  [--output <file>] \
  [--email <address>]

# List reports
cost-ops report list [--organization <org-id>]

# Download report
cost-ops report download --id <report-id> --output <file>

# Schedule report
cost-ops report schedule \
  --type cost \
  --cron "0 0 * * 1" \
  --email finance@example.com
```

### Forecast Commands

```bash
# Cost forecast
cost-ops forecast cost \
  --organization <org-id> \
  --horizon <days> \
  [--model <linear-trend|moving-average|exponential-smoothing>]

# Usage forecast
cost-ops forecast usage \
  --organization <org-id> \
  --horizon <days>

# Detect anomalies
cost-ops forecast anomalies \
  --organization <org-id> \
  --lookback-days <days>
```

### User Commands

```bash
# Create user
cost-ops user create \
  --username <email> \
  --organization <org-id> \
  --role <role>

# List users
cost-ops user list [--organization <org-id>]

# Generate API key
cost-ops user api-key \
  --username <email> \
  [--expires-in <duration>]

# Revoke API key
cost-ops user revoke-key --key-id <id>

# Update roles
cost-ops user update-roles \
  --username <email> \
  --roles <role1,role2>
```

### Organization Commands

```bash
# Create organization
cost-ops org create --name <name> --id <id>

# List organizations
cost-ops org list

# Set budget
cost-ops org budget \
  --organization <org-id> \
  --amount <amount> \
  --period <monthly|quarterly|yearly>

# View budget status
cost-ops org budget-status --organization <org-id>
```

### Server Commands

```bash
# Start server
cost-ops server start \
  [--port <port>] \
  [--host <host>] \
  [--database-url <url>] \
  [--config <file>]

# Health check
cost-ops server health --url <url>

# Generate config
cost-ops server config --output config.toml
```

### Benchmark Commands

```bash
# Run all benchmarks
cost-ops run

# Run with custom output directory
cost-ops run --output ./my-benchmarks

# Run specific benchmarks
cost-ops run --filter cost_calculation

# Skip summary report
cost-ops run --no-summary
```

For detailed benchmark documentation, see [BENCHMARKS.md](BENCHMARKS.md).

## Configuration

### Configuration File

Create `~/.config/cost-ops/config.toml`:

```toml
[database]
url = "postgresql://user:pass@localhost/costops"

[server]
host = "0.0.0.0"
port = 8080

[auth]
jwt_secret = "your-secret-key"

[defaults]
organization = "org-123"
output_format = "table"
```

### Environment Variables

```bash
export COST_OPS_DATABASE_URL="postgresql://user:pass@localhost/costops"
export COST_OPS_API_URL="http://localhost:8080"
export COST_OPS_API_KEY="your-api-key"
export COST_OPS_ORG="org-123"
```

## Interactive Mode

Start an interactive shell:

```bash
cost-ops shell
```

```
cost-ops> query --organization org-123
cost-ops> report --type cost --format csv
cost-ops> forecast --horizon 30
cost-ops> exit
```

## Output Formats

- **table** - Formatted ASCII table (default)
- **json** - JSON format
- **csv** - CSV format
- **yaml** - YAML format
- **excel** - Excel spreadsheet
- **pdf** - PDF report

## Examples

### Monthly Cost Report Pipeline

```bash
#!/bin/bash
# Generate monthly cost report and email it

ORG="org-123"
MONTH=$(date +%Y-%m)

cost-ops report \
  --type cost \
  --organization $ORG \
  --format pdf \
  --start-date "${MONTH}-01" \
  --output "monthly-report-${MONTH}.pdf" \
  --email finance@example.com
```

### Import and Analyze

```bash
#!/bin/bash
# Import usage data and analyze

cost-ops import \
  --file ./usage-data.json \
  --format openai \
  --organization org-123

cost-ops query \
  --organization org-123 \
  --group-by model \
  --output table
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Links

- **Documentation**: [https://docs.rs/llm-cost-ops-cli](https://docs.rs/llm-cost-ops-cli)
- **Core Library**: [https://crates.io/crates/llm-cost-ops](https://crates.io/crates/llm-cost-ops)
- **Repository**: [https://github.com/globalbusinessadvisors/llm-cost-ops](https://github.com/globalbusinessadvisors/llm-cost-ops)
