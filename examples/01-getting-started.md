# Getting Started with LLM-CostOps

This tutorial will walk you through setting up LLM-CostOps and running your first cost analysis.

**Time Required:** ~15 minutes
**Prerequisites:** Rust 1.91+ installed

---

## Step 1: Initialize the Database

First, let's set up the database and run migrations:

```bash
# Navigate to project directory
cd /path/to/llm-cost-ops

# Run the initialization script
./scripts/init-db.sh
```

**What this does:**
- Installs sqlx-cli if needed
- Creates a SQLite database file (`cost-ops.db`)
- Runs all database migrations
- Generates SQLx query cache
- Loads sample pricing data

**Expected Output:**
```
========================================
LLM-CostOps Database Initialization
========================================

Database URL: sqlite:cost-ops.db

✓ sqlx-cli is already installed
✓ Database created successfully
✓ Migrations completed successfully
✓ Query cache generated successfully

Tables created: 3
Default pricing entries: 4

Database initialization complete!
```

---

## Step 2: Verify Installation

Build and test the project:

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Check CLI help
cargo run --release -- --help
```

**Expected Output:**
```
LLM Cost Operations Platform

Usage: cost-ops [OPTIONS] <COMMAND>

Commands:
  init     Initialize database and create schema
  ingest   Ingest usage metrics
  query    Query cost records
  summary  Generate cost summary
  export   Export cost data
  pricing  Manage pricing tables
  help     Print this message or the help of the given subcommand(s)
```

---

## Step 3: View Default Pricing

List the pre-loaded pricing tables:

```bash
cargo run --release -- pricing list
```

**Expected Output:**
```
ID                                   Provider        Model                Effective Date
---------------------------------------------------------------------------------------------
...                                  openai          gpt-4                2024-01-01
...                                  openai          gpt-3.5-turbo        2024-01-01
...                                  anthropic       claude-3-sonnet      2024-02-01
...                                  anthropic       claude-3-opus        2024-02-01

Total pricing tables: 4
```

---

## Step 4: Ingest Sample Usage Data

Load the example usage data:

```bash
cargo run --release -- ingest --file examples/usage.json
```

**Expected Output:**
```
2025-01-15T10:00:00Z INFO llm_cost_ops: LLM-CostOps v0.1.0
2025-01-15T10:00:00Z INFO llm_cost_ops: Ingesting usage from file: "examples/usage.json"
Found 3 usage records
Processing record 550e8400-e29b-41d4-a716-446655440001 - Cost: 0.025000 USD
Processing record 550e8400-e29b-41d4-a716-446655440002 - Cost: 0.020000 USD
Processing record 550e8400-e29b-41d4-a716-446655440003 - Cost: 0.001000 USD
Ingestion complete
```

---

## Step 5: Query Cost Data

View the ingested costs:

```bash
# Table format (default)
cargo run --release -- query --range last-24-hours --output table

# JSON format
cargo run --release -- query --range last-24-hours --output json

# CSV format
cargo run --release -- query --range last-24-hours --output csv
```

**Table Output Example:**
```
ID                                   Provider            Model           Total Cost
-------------------------------------------------------------------------------------
550e8400-e29b-41d4-a716-446655440001 openai              gpt-4           $0.025000
550e8400-e29b-41d4-a716-446655440002 anthropic           claude-3-sonnet $0.020000
550e8400-e29b-41d4-a716-446655440003 openai              gpt-3.5-turbo   $0.001000

Total records: 3
```

---

## Step 6: Generate Cost Summary

Get aggregated cost analysis:

```bash
cargo run --release -- summary --period last-30-days --organization org-example
```

**Expected Output:**
```
=== Cost Summary ===
Period: 2024-12-15 to 2025-01-15
Organization: org-example

Total Cost: $0.046000
Total Requests: 3
Avg Cost/Request: $0.015333

--- By Provider ---
openai: $0.026000
anthropic: $0.020000

--- By Model ---
gpt-4: $0.025000
claude-3-sonnet: $0.020000
gpt-3.5-turbo: $0.001000

--- By Project ---
proj-123: $0.045000
```

---

## Step 7: Export Data

Export cost data for external analysis:

```bash
# Export as JSON
cargo run --release -- export --output costs.json --format json --period last-7-days

# Export as CSV
cargo run --release -- export --output costs.csv --format csv --period last-7-days
```

---

## Step 8: Add Custom Pricing

Add pricing for a model not in defaults:

```bash
cargo run --release -- pricing add \
  --provider google \
  --model gemini-pro \
  --input-price 0.125 \
  --output-price 0.375
```

**What this does:**
- Creates a new pricing table for Google's Gemini Pro
- Input: $0.125 per million tokens
- Output: $0.375 per million tokens
- Effective from current date

---

## Common Issues

### Issue: "cargo: command not found"

**Solution:** Install Rust from https://rustup.rs/

### Issue: "sqlx-cli not found"

**Solution:** Run `./scripts/init-db.sh` which will install it automatically

### Issue: "No pricing found for model X"

**Solution:** Add pricing manually:
```bash
cargo run -- pricing add --provider PROVIDER --model MODEL --input-price X --output-price Y
```

### Issue: Database locked

**Solution:** Only one process can write to SQLite at a time. Close other instances.

---

## Next Steps

- **Tutorial 2:** [Ingesting Real Usage Data](02-ingesting-data.md)
- **Tutorial 3:** [Multi-Provider Cost Analysis](03-multi-provider.md)
- **Tutorial 4:** [Advanced Querying](04-advanced-queries.md)

---

## Quick Reference

```bash
# Initialize database
./scripts/init-db.sh

# Ingest data
cargo run -- ingest --file FILE.json

# Query costs
cargo run -- query --range last-7-days --output table

# Generate summary
cargo run -- summary --period last-30-days

# Export data
cargo run -- export --output FILE --format json

# Manage pricing
cargo run -- pricing list
cargo run -- pricing add --provider X --model Y --input-price A --output-price B
cargo run -- pricing get --provider X --model Y
```

---

**Troubleshooting:** See [Common Issues](#common-issues) section above
**Documentation:** See `README.md` for comprehensive documentation
**Support:** Open an issue at https://github.com/llm-devops/llm-cost-ops/issues
