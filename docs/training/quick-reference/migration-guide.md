# Migration Guide

**Version:** 1.0.0
**Last Updated:** 2025-11-16

Comprehensive migration guide for moving to LLM Cost Ops from manual tracking, other cost tools, or upgrading between versions.

---

## Table of Contents

- [Migrating from Manual Tracking](#migrating-from-manual-tracking)
- [Migrating from Other Cost Tools](#migrating-from-other-cost-tools)
- [Version Upgrade Guides](#version-upgrade-guides)
- [Breaking Changes](#breaking-changes)
- [Migration Checklist](#migration-checklist)
- [Rollback Procedures](#rollback-procedures)
- [Testing Migration](#testing-migration)

---

## Migrating from Manual Tracking

### Overview

If you're currently tracking LLM costs manually (spreadsheets, scripts, databases), this guide will help you migrate to LLM Cost Ops.

### Pre-Migration Assessment

**1. Inventory Your Current Data:**
- [ ] Identify all data sources (spreadsheets, databases, log files)
- [ ] Document data format and structure
- [ ] Estimate total data volume
- [ ] Identify data quality issues
- [ ] List all LLM providers and models in use

**2. Define Migration Scope:**
- [ ] Decide how much historical data to migrate
- [ ] Identify critical vs. optional data
- [ ] Set migration timeline
- [ ] Determine acceptable downtime
- [ ] Plan parallel run period

**3. Prepare Your Team:**
- [ ] Identify migration team members
- [ ] Schedule training sessions
- [ ] Create communication plan
- [ ] Define success criteria
- [ ] Plan rollback strategy

### Step 1: Install LLM Cost Ops

**Install CLI:**
```bash
# Download and install
curl -L https://github.com/llm-cost-ops/releases/latest/download/cost-ops-linux-amd64 -o cost-ops
chmod +x cost-ops
sudo mv cost-ops /usr/local/bin/

# Verify installation
cost-ops --version
```

**Initialize Database:**
```bash
# For development/testing
cost-ops init --database-url sqlite:cost-ops.db

# For production
cost-ops init --database-url postgresql://user:password@localhost:5432/costops

# Run migrations
cost-ops migrate run --database-url postgresql://localhost/costops
```

### Step 2: Set Up Pricing Data

**Add Current Pricing:**
```bash
# OpenAI pricing
cost-ops pricing add \
  --provider openai \
  --model gpt-4-turbo \
  --input-price 10.0 \
  --output-price 30.0 \
  --effective-date 2024-04-09

# Anthropic pricing
cost-ops pricing add \
  --provider anthropic \
  --model claude-3-5-sonnet-20241022 \
  --input-price 3.0 \
  --output-price 15.0 \
  --effective-date 2024-10-22

# Add pricing for all providers/models you use
```

**Historical Pricing (if needed):**
```bash
# Add historical pricing for accurate cost calculation
cost-ops pricing add \
  --provider openai \
  --model gpt-4 \
  --input-price 30.0 \
  --output-price 60.0 \
  --effective-date 2023-03-14 \
  --end-date 2024-04-08
```

### Step 3: Export Historical Data

**From Spreadsheet:**

If using Excel/Google Sheets:
1. Export to CSV format
2. Ensure columns match expected format

**Expected CSV Format:**
```csv
timestamp,provider,model,organization_id,prompt_tokens,completion_tokens,total_tokens,user_id,project_id
2025-01-15T10:00:00Z,openai,gpt-4-turbo,org-abc,1000,500,1500,user-123,proj-xyz
2025-01-15T10:05:00Z,anthropic,claude-3-5-sonnet,org-abc,1200,600,1800,user-456,proj-xyz
```

**From Database:**

Export to JSON:
```sql
-- PostgreSQL example
COPY (
  SELECT
    timestamp,
    provider,
    model,
    organization_id,
    prompt_tokens,
    completion_tokens,
    total_tokens,
    user_id,
    project_id
  FROM usage_data
  WHERE timestamp >= '2024-01-01'
) TO '/tmp/usage_export.csv' WITH CSV HEADER;
```

**From Log Files:**

Parse logs using script:
```python
# parse_logs.py
import json
import re
from datetime import datetime

def parse_openai_log(line):
    # Parse your log format
    match = re.search(r'tokens=(\d+)', line)
    if match:
        return {
            'timestamp': datetime.now().isoformat(),
            'provider': 'openai',
            'model': 'gpt-4-turbo',
            'total_tokens': int(match.group(1))
        }
    return None

with open('usage.json', 'w') as out:
    with open('application.log') as f:
        for line in f:
            usage = parse_openai_log(line)
            if usage:
                out.write(json.dumps(usage) + '\n')
```

### Step 4: Transform Data

**Create Transformation Script:**

```python
# transform_data.py
import csv
import json
from datetime import datetime

def transform_csv_to_json(csv_file, json_file):
    """Transform CSV to LLM Cost Ops JSON format"""
    with open(csv_file) as f:
        reader = csv.DictReader(f)
        records = []

        for row in reader:
            # Transform to expected format
            record = {
                'timestamp': row['timestamp'],
                'provider': row['provider'].lower(),
                'model': {
                    'name': row['model']
                },
                'organization_id': row.get('organization_id', 'default-org'),
                'project_id': row.get('project_id'),
                'user_id': row.get('user_id'),
                'prompt_tokens': int(row.get('prompt_tokens', 0)),
                'completion_tokens': int(row.get('completion_tokens', 0)),
                'total_tokens': int(row['total_tokens'])
            }

            # Add optional fields if present
            if 'cached_tokens' in row:
                record['cached_tokens'] = int(row['cached_tokens'])
            if 'latency_ms' in row:
                record['latency_ms'] = int(row['latency_ms'])

            records.append(record)

    with open(json_file, 'w') as f:
        json.dump(records, f, indent=2)

# Transform data
transform_csv_to_json('usage_export.csv', 'usage_import.json')
```

**Validate Transformed Data:**
```bash
# Check JSON is valid
cat usage_import.json | jq . > /dev/null

# Count records
jq 'length' usage_import.json

# Sample first record
jq '.[0]' usage_import.json
```

### Step 5: Import Data

**Test Import (Small Sample):**
```bash
# Test with first 100 records
jq '.[:100]' usage_import.json > test_import.json

# Validate import
cost-ops ingest --file test_import.json --validate-only

# Import test data
cost-ops ingest --file test_import.json

# Verify import
cost-ops query --range last-7-days --output table
```

**Full Import:**
```bash
# Import all data
cost-ops ingest --file usage_import.json --batch-size 1000

# Monitor import progress
tail -f /var/log/cost-ops/import.log

# Verify total records
cost-ops query --output json | jq 'length'
```

**Import Large Files:**
```bash
# For very large files, split into chunks
split -l 10000 usage_import.json chunk_

# Import each chunk
for file in chunk_*; do
  echo "Importing $file..."
  cost-ops ingest --file $file --batch-size 1000
  echo "Completed $file"
done
```

### Step 6: Verify Migration

**Data Verification:**
```bash
# Check total usage records
cost-ops query --output json | jq 'length'

# Verify costs calculated
cost-ops summary --period last-30-days

# Compare with original data
# Export from Cost Ops
cost-ops export --output migrated_costs.csv --format csv

# Compare totals
original_total=$(awk -F',' '{sum+=$7} END {print sum}' original_data.csv)
migrated_total=$(awk -F',' '{sum+=$5} END {print sum}' migrated_costs.csv)

echo "Original total: $original_total"
echo "Migrated total: $migrated_total"
```

**Reconciliation Report:**
```bash
# Generate reconciliation report
cost-ops report generate \
  --type reconciliation \
  --start-date 2024-01-01 \
  --end-date 2025-01-31 \
  --output reconciliation.pdf
```

### Step 7: Integrate with Applications

**Update Application Code:**

**Before (Manual Tracking):**
```python
# Old manual tracking
def track_openai_usage(prompt_tokens, completion_tokens):
    with open('usage.csv', 'a') as f:
        writer = csv.writer(f)
        writer.writerow([
            datetime.now().isoformat(),
            'openai',
            'gpt-4',
            prompt_tokens,
            completion_tokens
        ])
```

**After (LLM Cost Ops):**
```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(api_key=os.environ['COST_OPS_API_KEY'])

def track_openai_usage(prompt_tokens, completion_tokens):
    client.usage.create(
        provider='openai',
        model='gpt-4-turbo',
        organization_id='org-abc',
        prompt_tokens=prompt_tokens,
        completion_tokens=completion_tokens,
        total_tokens=prompt_tokens + completion_tokens
    )
```

**Add Error Handling:**
```python
from llm_cost_ops.exceptions import APIError

def track_usage_safe(usage_data):
    max_retries = 3
    for attempt in range(max_retries):
        try:
            return client.usage.create(**usage_data)
        except APIError as e:
            if attempt < max_retries - 1:
                time.sleep(2 ** attempt)
            else:
                # Log failure and continue
                logger.error(f"Failed to track usage: {e}")
                # Optionally queue for retry
                retry_queue.put(usage_data)
```

### Step 8: Parallel Run

**Run Both Systems in Parallel:**
```python
def track_usage_parallel(usage_data):
    # Track in both old and new systems
    try:
        # New system
        client.usage.create(**usage_data)
    except Exception as e:
        logger.error(f"Cost Ops error: {e}")

    # Old system (keep as backup)
    try:
        write_to_csv(usage_data)
    except Exception as e:
        logger.error(f"CSV error: {e}")
```

**Monitor and Compare:**
```bash
# Daily comparison
./compare_systems.sh 2025-01-15

# Weekly reconciliation
./weekly_reconciliation.sh
```

### Step 9: Cutover

**Preparation:**
1. Verify all data migrated correctly
2. Ensure team trained on new system
3. Update documentation
4. Communicate cutover date
5. Prepare rollback plan

**Cutover Checklist:**
- [ ] Final data verification completed
- [ ] All applications updated to use Cost Ops
- [ ] Old system in read-only mode
- [ ] Monitoring and alerts configured
- [ ] Support team ready
- [ ] Rollback plan tested

**Execute Cutover:**
```bash
# 1. Stop new writes to old system
./stop_old_system.sh

# 2. Final data sync
./final_sync.sh

# 3. Switch applications to Cost Ops only
./enable_cost_ops.sh

# 4. Verify new system
cost-ops query --range last-24-hours
./verify_cutover.sh

# 5. Monitor for issues
tail -f /var/log/cost-ops/application.log
```

### Step 10: Decommission Old System

**After Successful Cutover (30 days):**
```bash
# 1. Archive old data
./archive_old_system.sh

# 2. Create final backup
./backup_old_data.sh

# 3. Document archive location
echo "Old data archived to: /backups/old-cost-tracking-$(date +%Y%m%d).tar.gz"

# 4. Decommission old infrastructure
./decommission_old_system.sh
```

---

## Migrating from Other Cost Tools

### From Helicone

**Export Data from Helicone:**
```bash
# Use Helicone API
curl -X GET "https://api.helicone.ai/v1/usage" \
  -H "Authorization: Bearer $HELICONE_API_KEY" \
  -H "Content-Type: application/json" \
  > helicone_export.json
```

**Transform Data:**
```python
# transform_helicone.py
import json

def transform_helicone_to_cost_ops(helicone_file, cost_ops_file):
    with open(helicone_file) as f:
        helicone_data = json.load(f)

    cost_ops_data = []
    for record in helicone_data:
        cost_ops_record = {
            'timestamp': record['created_at'],
            'provider': record['provider'],
            'model': {'name': record['model']},
            'organization_id': record.get('user_id', 'default-org'),
            'prompt_tokens': record['usage']['prompt_tokens'],
            'completion_tokens': record['usage']['completion_tokens'],
            'total_tokens': record['usage']['total_tokens'],
            'metadata': {
                'helicone_request_id': record['request_id']
            }
        }
        cost_ops_data.append(cost_ops_record)

    with open(cost_ops_file, 'w') as f:
        json.dump(cost_ops_data, f, indent=2)

transform_helicone_to_cost_ops('helicone_export.json', 'cost_ops_import.json')
```

**Import to Cost Ops:**
```bash
cost-ops ingest --file cost_ops_import.json
```

### From LangSmith

**Export from LangSmith:**
```python
# export_langsmith.py
from langsmith import Client

client = Client()

# Export runs
runs = client.list_runs(
    project_name="my-project",
    start_time="2024-01-01T00:00:00Z"
)

# Transform to Cost Ops format
cost_ops_data = []
for run in runs:
    if run.total_tokens:
        cost_ops_data.append({
            'timestamp': run.start_time.isoformat(),
            'provider': 'openai',  # Adjust based on your setup
            'model': {'name': run.extra.get('model', 'unknown')},
            'organization_id': 'default-org',
            'prompt_tokens': run.prompt_tokens,
            'completion_tokens': run.completion_tokens,
            'total_tokens': run.total_tokens
        })

# Save to file
with open('langsmith_export.json', 'w') as f:
    json.dump(cost_ops_data, f)
```

### From LangFuse

**Export from LangFuse:**
```typescript
// export_langfuse.ts
import { Langfuse } from 'langfuse';

const langfuse = new Langfuse({
  publicKey: process.env.LANGFUSE_PUBLIC_KEY,
  secretKey: process.env.LANGFUSE_SECRET_KEY
});

const traces = await langfuse.getTraces({
  fromTimestamp: '2024-01-01T00:00:00Z'
});

const costOpsData = traces.map(trace => ({
  timestamp: trace.timestamp,
  provider: 'openai',
  model: { name: trace.metadata.model },
  organization_id: trace.userId || 'default-org',
  prompt_tokens: trace.usage.promptTokens,
  completion_tokens: trace.usage.completionTokens,
  total_tokens: trace.usage.totalTokens
}));

fs.writeFileSync('langfuse_export.json', JSON.stringify(costOpsData, null, 2));
```

### From Custom Solutions

**Adapter Pattern:**
```python
# custom_adapter.py
from abc import ABC, abstractmethod

class CostTrackingAdapter(ABC):
    @abstractmethod
    def export_data(self, start_date, end_date):
        pass

    @abstractmethod
    def transform_to_cost_ops(self, raw_data):
        pass

class MyCorporateToolAdapter(CostTrackingAdapter):
    def export_data(self, start_date, end_date):
        # Export from your corporate tool
        return custom_api.get_usage(start_date, end_date)

    def transform_to_cost_ops(self, raw_data):
        # Transform to Cost Ops format
        return [{
            'timestamp': item['date'],
            'provider': item['llm_provider'],
            'model': {'name': item['model_name']},
            'organization_id': item['department'],
            'prompt_tokens': item['input_tokens'],
            'completion_tokens': item['output_tokens'],
            'total_tokens': item['total_tokens']
        } for item in raw_data]

# Use adapter
adapter = MyCorporateToolAdapter()
data = adapter.export_data('2024-01-01', '2024-12-31')
transformed = adapter.transform_to_cost_ops(data)

# Import to Cost Ops
with open('corporate_export.json', 'w') as f:
    json.dump(transformed, f)
```

---

## Version Upgrade Guides

### Upgrading from 0.x to 1.0

**Breaking Changes:**
- API endpoint paths changed from `/v1` to `/api/v1`
- Authentication header format changed
- Database schema updated
- Configuration file format changed

**Migration Steps:**

**1. Backup Current System:**
```bash
# Backup database
pg_dump costops > costops_backup_$(date +%Y%m%d).sql

# Backup configuration
cp /etc/cost-ops/config.toml /etc/cost-ops/config.toml.bak

# Backup application data
tar -czf cost-ops-backup-$(date +%Y%m%d).tar.gz /var/lib/cost-ops
```

**2. Update Application:**
```bash
# Download new version
curl -L https://github.com/llm-cost-ops/releases/download/v1.0.0/cost-ops-linux-amd64 -o cost-ops-new

# Stop old version
systemctl stop cost-ops

# Replace binary
sudo mv cost-ops-new /usr/local/bin/cost-ops
sudo chmod +x /usr/local/bin/cost-ops

# Verify version
cost-ops --version
```

**3. Update Configuration:**

**Old format (0.x):**
```toml
database_url = "postgresql://localhost/costops"
server_port = 8080
```

**New format (1.0):**
```toml
[database]
url = "postgresql://localhost/costops"

[server]
port = 8080
```

**4. Run Database Migration:**
```bash
# Run migrations
cost-ops migrate run --database-url postgresql://localhost/costops

# Verify migration
cost-ops migrate status
```

**5. Update API Clients:**

**Old (0.x):**
```python
client = CostOpsClient(
    base_url="https://api.example.com/v1",
    api_key="key_abc123"
)
```

**New (1.0):**
```python
client = CostOpsClient(
    base_url="https://api.example.com",  # No /v1
    api_key="sk_live_abc123"  # New key format
)
```

**6. Update Authentication:**

**Old API key format:**
```
key_abc123xyz
```

**New API key format:**
```
sk_live_abc123xyz  # Includes prefix
```

**Migrate API keys:**
```bash
# Create new API keys
cost-ops auth create-api-key \
  --name "Migration Key" \
  --scopes "usage:write,costs:read"

# Update applications with new keys
export COST_OPS_API_KEY="sk_live_new_key"

# Revoke old keys after verification
cost-ops auth revoke-api-key --id old_key_id
```

**7. Test Migration:**
```bash
# Start new version
systemctl start cost-ops

# Verify health
curl http://localhost:8080/health

# Test API
cost-ops query --range last-24-hours

# Run integration tests
./run_integration_tests.sh
```

### Upgrading from 1.0 to 1.1 (Minor Version)

Minor version upgrades typically don't have breaking changes.

**1. Update Binary:**
```bash
# Download new version
curl -L https://github.com/llm-cost-ops/releases/download/v1.1.0/cost-ops-linux-amd64 -o cost-ops

# Replace binary
sudo mv cost-ops /usr/local/bin/cost-ops
sudo chmod +x /usr/local/bin/cost-ops
```

**2. Run Migrations (if any):**
```bash
cost-ops migrate run --database-url postgresql://localhost/costops
```

**3. Restart Service:**
```bash
systemctl restart cost-ops
```

**4. Verify:**
```bash
cost-ops --version
curl http://localhost:8080/health
```

---

## Breaking Changes

### Version 1.0.0

**API Changes:**
- **Endpoint paths:** `/v1/*` → `/api/v1/*`
- **Authentication header:** `X-API-Key: key` → `Authorization: Bearer sk_live_key`
- **Response format:** Top-level `data` wrapper added
- **Error format:** Standardized error response structure

**Database Schema:**
- **usage_records:** Added `cached_tokens` and `reasoning_tokens` columns
- **cost_records:** Changed `cost` type from DECIMAL to TEXT (for precision)
- **pricing_tables:** Added `pricing_structure` JSON column

**Configuration:**
- **Format:** Flat structure → Nested TOML sections
- **Environment variables:** `DATABASE_URL` → `DATABASE__URL`
- **Defaults:** Changed default port from 3000 to 8080

**SDK Changes:**
- **Method names:** `create_usage()` → `usage.create()`
- **Response types:** Direct objects → Response wrappers with `.data`
- **Error handling:** String errors → Typed exceptions

### Version 0.9.0 to 1.0.0 Compatibility

**Backwards Compatibility:**
- Old API endpoints supported with deprecation warnings (removed in 2.0)
- Old API keys continue to work (migration recommended)
- Database automatically migrated on startup

**Recommended Actions:**
1. Update API endpoints in applications
2. Migrate to new API key format
3. Update SDK to 1.0.0+
4. Review configuration file
5. Test thoroughly before production deployment

---

## Migration Checklist

### Pre-Migration

- [ ] Backup all data (database, files, configurations)
- [ ] Document current system architecture
- [ ] Identify all integrations and dependencies
- [ ] Create migration timeline
- [ ] Assign migration team roles
- [ ] Prepare rollback plan
- [ ] Set up test environment
- [ ] Review migration documentation

### Planning

- [ ] Define migration scope
- [ ] Estimate data volume
- [ ] Plan downtime window
- [ ] Identify critical vs. optional data
- [ ] Create data transformation scripts
- [ ] Prepare validation criteria
- [ ] Schedule team training
- [ ] Communicate to stakeholders

### Execution

- [ ] Install LLM Cost Ops in test environment
- [ ] Set up pricing data
- [ ] Transform historical data
- [ ] Test import with sample data
- [ ] Validate data accuracy
- [ ] Import full dataset
- [ ] Verify all data migrated
- [ ] Update application integrations
- [ ] Configure monitoring and alerts
- [ ] Test end-to-end workflows

### Validation

- [ ] Verify record counts match
- [ ] Reconcile cost totals
- [ ] Test all API endpoints
- [ ] Validate reports accuracy
- [ ] Check error handling
- [ ] Test performance under load
- [ ] Verify security controls
- [ ] Test backup and restore

### Cutover

- [ ] Final data sync
- [ ] Switch applications to new system
- [ ] Update DNS/load balancers
- [ ] Enable monitoring
- [ ] Verify production traffic
- [ ] Monitor for errors
- [ ] Communicate completion
- [ ] Document any issues

### Post-Migration

- [ ] Verify system stability (24-48 hours)
- [ ] Reconcile first week of data
- [ ] Address any issues
- [ ] Update documentation
- [ ] Train team on new features
- [ ] Decommission old system
- [ ] Archive old data
- [ ] Conduct retrospective

---

## Rollback Procedures

### Emergency Rollback

**If critical issues occur during migration:**

**1. Stop New System:**
```bash
# Stop Cost Ops
systemctl stop cost-ops

# Or kill processes
pkill -f cost-ops
```

**2. Restore Old System:**
```bash
# Restore old binary
sudo cp /backups/cost-ops-old /usr/local/bin/cost-ops

# Restore old configuration
sudo cp /backups/config.toml.bak /etc/cost-ops/config.toml

# Start old system
systemctl start cost-ops-old
```

**3. Switch Applications Back:**
```bash
# Update environment variables
export COST_OPS_URL="https://old-api.example.com"
export COST_OPS_API_KEY="old_key_abc123"

# Restart applications
./restart_applications.sh
```

**4. Notify Team:**
```bash
# Send notification
./notify_team.sh "Rollback completed. Old system restored."
```

### Database Rollback

**If database migration fails:**

**1. Restore Database:**
```bash
# Drop new database
psql -c "DROP DATABASE costops;"

# Restore from backup
psql -c "CREATE DATABASE costops;"
psql costops < costops_backup_20250115.sql
```

**2. Verify Restore:**
```bash
# Check record counts
psql costops -c "SELECT COUNT(*) FROM usage_records;"

# Test queries
cost-ops query --range last-24-hours
```

### Partial Rollback

**If only specific features need rollback:**

**1. Disable New Features:**
```toml
# config.toml
[features]
forecasting = false
anomaly_detection = false
new_export_formats = false
```

**2. Use Feature Flags:**
```python
# Application code
if feature_flags.is_enabled('use_new_cost_ops'):
    client = NewCostOpsClient()
else:
    client = OldCostTracker()
```

---

## Testing Migration

### Test Plan

**1. Unit Tests:**
```bash
# Test data transformation
python -m pytest tests/test_transformation.py

# Test import logic
python -m pytest tests/test_import.py

# Test API compatibility
python -m pytest tests/test_api_migration.py
```

**2. Integration Tests:**
```bash
# Test end-to-end flow
./tests/integration/test_migration_flow.sh

# Test with sample data
cost-ops ingest --file tests/fixtures/sample_usage.json
cost-ops query --range last-7-days
```

**3. Data Validation:**
```python
# validate_migration.py
def validate_migration():
    # Compare record counts
    old_count = get_old_system_count()
    new_count = get_new_system_count()
    assert old_count == new_count, f"Count mismatch: {old_count} vs {new_count}"

    # Compare totals
    old_total = get_old_system_total()
    new_total = get_new_system_total()
    diff = abs(old_total - new_total)
    assert diff < 0.01, f"Total mismatch: {old_total} vs {new_total}"

    # Sample records
    for record_id in get_sample_ids():
        old_record = get_old_record(record_id)
        new_record = get_new_record(record_id)
        assert_records_match(old_record, new_record)

validate_migration()
```

**4. Performance Tests:**
```bash
# Load test
./tests/performance/load_test.sh

# Benchmark queries
./tests/performance/benchmark_queries.sh

# Monitor resource usage
./tests/performance/monitor_resources.sh
```

**5. User Acceptance Testing:**
```markdown
**UAT Checklist:**
- [ ] User can log in successfully
- [ ] User can view historical data
- [ ] User can create new usage records
- [ ] User can query costs
- [ ] User can generate reports
- [ ] User can export data
- [ ] All dashboards display correctly
- [ ] Alerts are working
```

### Test Data Sets

**Minimal Test Set:**
```json
[
  {
    "timestamp": "2025-01-15T10:00:00Z",
    "provider": "openai",
    "model": {"name": "gpt-4-turbo"},
    "organization_id": "test-org",
    "prompt_tokens": 100,
    "completion_tokens": 50,
    "total_tokens": 150
  }
]
```

**Comprehensive Test Set:**
- Multiple providers (OpenAI, Anthropic, Google)
- Multiple models
- Date range: 1 year
- Volume: 10,000+ records
- Edge cases: zero tokens, large values, special characters

### Validation Scripts

**Compare Systems:**
```bash
#!/bin/bash
# compare_systems.sh

echo "Comparing old and new systems..."

# Export from old system
./export_old_system.sh > old_export.csv

# Export from new system
cost-ops export --output new_export.csv --format csv

# Compare
diff <(sort old_export.csv) <(sort new_export.csv)

if [ $? -eq 0 ]; then
    echo "✓ Systems match"
else
    echo "✗ Systems differ"
    exit 1
fi
```

**Reconciliation Report:**
```python
# reconciliation.py
def generate_reconciliation_report():
    report = {
        'migration_date': datetime.now(),
        'old_system': {
            'total_records': get_old_count(),
            'total_cost': get_old_total_cost(),
            'date_range': get_old_date_range()
        },
        'new_system': {
            'total_records': get_new_count(),
            'total_cost': get_new_total_cost(),
            'date_range': get_new_date_range()
        },
        'differences': {
            'record_count_diff': abs(get_old_count() - get_new_count()),
            'cost_diff': abs(get_old_total_cost() - get_new_total_cost())
        },
        'status': 'PASS' if is_within_threshold() else 'FAIL'
    }

    with open('reconciliation_report.json', 'w') as f:
        json.dump(report, f, indent=2)

generate_reconciliation_report()
```

---

## Support During Migration

### Migration Support

**Email:** migrations@llm-cost-ops.com
**Slack:** #migrations channel
**Office Hours:** Tuesday/Thursday 2-4 PM EST

### Resources

- **Migration FAQ:** https://docs.llm-cost-ops.com/migration/faq
- **Video Tutorials:** https://www.youtube.com/llmcostops
- **Community Forum:** https://community.llm-cost-ops.com

### Professional Services

For complex migrations, we offer professional services:
- Migration planning and assessment
- Data transformation assistance
- Custom integration development
- Training and support
- Post-migration optimization

Contact: services@llm-cost-ops.com

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0
