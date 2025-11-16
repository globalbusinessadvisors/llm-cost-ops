# cURL Examples

Complete collection of cURL examples for the LLM-CostOps API. Perfect for testing, scripting, and learning the API.

## Table of Contents

1. [Authentication](#authentication)
2. [Usage Tracking](#usage-tracking)
3. [Cost Queries](#cost-queries)
4. [Pricing Management](#pricing-management)
5. [Analytics](#analytics)
6. [Batch Operations](#batch-operations)
7. [Error Handling](#error-handling)

## Prerequisites

Set environment variables for easier usage:

```bash
export API_KEY="your_api_key_here"
export BASE_URL="https://api.llm-cost-ops.dev"
export ORG_ID="org-123"
```

## Authentication

### Health Check (No Auth Required)

```bash
curl -X GET $BASE_URL/health
```

### Test Authentication

```bash
curl -X GET $BASE_URL/ready \
  -H "Authorization: Bearer $API_KEY"
```

## Usage Tracking

### Submit OpenAI GPT-4 Usage

```bash
curl -X POST $BASE_URL/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "organization_id": "'$ORG_ID'",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_tokens": 1000,
    "output_tokens": 500,
    "total_tokens": 1500,
    "timestamp": "2025-01-15T10:00:00Z",
    "metadata": {
      "request_id": "req-abc123",
      "user_id": "user-456"
    }
  }'
```

### Submit Anthropic Claude Usage

```bash
curl -X POST $BASE_URL/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "organization_id": "'$ORG_ID'",
    "provider": "anthropic",
    "model_id": "claude-3-sonnet-20240229",
    "input_tokens": 2000,
    "output_tokens": 800,
    "total_tokens": 2800,
    "metadata": {
      "cached_tokens": 500
    }
  }'
```

### Submit Google Vertex AI Usage

```bash
curl -X POST $BASE_URL/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "organization_id": "'$ORG_ID'",
    "provider": "google",
    "model_id": "gemini-pro",
    "input_tokens": 1500,
    "output_tokens": 600,
    "total_tokens": 2100
  }'
```

### Submit with Custom Metadata

```bash
curl -X POST $BASE_URL/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "organization_id": "'$ORG_ID'",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_tokens": 1000,
    "output_tokens": 500,
    "total_tokens": 1500,
    "metadata": {
      "endpoint": "/api/chat",
      "feature": "customer-support",
      "user_id": "user-456",
      "session_id": "sess-789",
      "environment": "production",
      "team": "support",
      "priority": "high",
      "customer_tier": "enterprise"
    }
  }'
```

## Cost Queries

### Get Total Costs (Last 30 Days)

```bash
curl -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z" \
  -H "Authorization: Bearer $API_KEY"
```

### Get Costs Grouped by Provider

```bash
curl -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&group_by=provider" \
  -H "Authorization: Bearer $API_KEY"
```

### Get Costs Grouped by Model

```bash
curl -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&group_by=model" \
  -H "Authorization: Bearer $API_KEY"
```

### Get Daily Cost Breakdown

```bash
curl -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&group_by=day" \
  -H "Authorization: Bearer $API_KEY"
```

### Filter by Provider

```bash
curl -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&provider=openai&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z" \
  -H "Authorization: Bearer $API_KEY"
```

### Filter by Model

```bash
curl -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&model_id=gpt-4&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z" \
  -H "Authorization: Bearer $API_KEY"
```

## Usage History

### Get Recent Usage (Paginated)

```bash
curl -X GET "$BASE_URL/api/v1/usage/history?page=1&page_size=50" \
  -H "Authorization: Bearer $API_KEY"
```

### Get Usage with Filters

```bash
curl -X GET "$BASE_URL/api/v1/usage/history?organization_id=$ORG_ID&provider=openai&model_id=gpt-4&page=1&page_size=20" \
  -H "Authorization: Bearer $API_KEY"
```

### Get Usage by Date Range

```bash
curl -X GET "$BASE_URL/api/v1/usage/history?organization_id=$ORG_ID&start_date=2025-01-15T00:00:00Z&end_date=2025-01-15T23:59:59Z" \
  -H "Authorization: Bearer $API_KEY"
```

## Pricing Management

### List All Pricing Tables

```bash
curl -X GET "$BASE_URL/api/v1/pricing?page=1&page_size=50" \
  -H "Authorization: Bearer $API_KEY"
```

### Filter Pricing by Provider

```bash
curl -X GET "$BASE_URL/api/v1/pricing?provider=openai" \
  -H "Authorization: Bearer $API_KEY"
```

### Create Pricing for OpenAI GPT-4

```bash
curl -X POST $BASE_URL/api/v1/pricing \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "provider": "openai",
    "model_id": "gpt-4",
    "input_price_per_1k": 0.01,
    "output_price_per_1k": 0.03,
    "currency": "USD",
    "effective_date": "2025-01-01T00:00:00Z"
  }'
```

### Create Pricing for Anthropic Claude

```bash
curl -X POST $BASE_URL/api/v1/pricing \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "provider": "anthropic",
    "model_id": "claude-3-sonnet-20240229",
    "input_price_per_1k": 0.003,
    "output_price_per_1k": 0.015,
    "currency": "USD",
    "effective_date": "2025-01-01T00:00:00Z"
  }'
```

### Create Pricing for Google Gemini

```bash
curl -X POST $BASE_URL/api/v1/pricing \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "provider": "google",
    "model_id": "gemini-pro",
    "input_price_per_1k": 0.000125,
    "output_price_per_1k": 0.000375,
    "currency": "USD",
    "effective_date": "2025-01-01T00:00:00Z"
  }'
```

## Analytics

### Get Daily Analytics

```bash
curl -X GET "$BASE_URL/api/v1/analytics?organization_id=$ORG_ID&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&interval=day" \
  -H "Authorization: Bearer $API_KEY"
```

### Get Hourly Analytics

```bash
curl -X GET "$BASE_URL/api/v1/analytics?organization_id=$ORG_ID&start_date=2025-01-15T00:00:00Z&end_date=2025-01-15T23:59:59Z&interval=hour" \
  -H "Authorization: Bearer $API_KEY"
```

### Get Analytics with Specific Metrics

```bash
curl -X GET "$BASE_URL/api/v1/analytics?organization_id=$ORG_ID&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&interval=day&metrics=total_cost,total_tokens,total_requests" \
  -H "Authorization: Bearer $API_KEY"
```

### Get Analytics Grouped by Provider

```bash
curl -X GET "$BASE_URL/api/v1/analytics?organization_id=$ORG_ID&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&interval=day&group_by=provider" \
  -H "Authorization: Bearer $API_KEY"
```

## Batch Operations

### Submit Multiple Usage Records

```bash
curl -X POST $BASE_URL/api/v1/usage/batch \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "records": [
      {
        "organization_id": "'$ORG_ID'",
        "provider": "openai",
        "model_id": "gpt-4",
        "input_tokens": 1000,
        "output_tokens": 500,
        "total_tokens": 1500
      },
      {
        "organization_id": "'$ORG_ID'",
        "provider": "anthropic",
        "model_id": "claude-3-sonnet-20240229",
        "input_tokens": 2000,
        "output_tokens": 800,
        "total_tokens": 2800
      },
      {
        "organization_id": "'$ORG_ID'",
        "provider": "openai",
        "model_id": "gpt-3.5-turbo",
        "input_tokens": 500,
        "output_tokens": 200,
        "total_tokens": 700
      }
    ]
  }'
```

## Scripting Examples

### Daily Cost Report Script

```bash
#!/bin/bash
# daily-cost-report.sh

YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)
START="${YESTERDAY}T00:00:00Z"
END="${YESTERDAY}T23:59:59Z"

echo "=== Daily Cost Report: $YESTERDAY ==="

# Get total costs
RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&start_date=$START&end_date=$END&group_by=provider" \
  -H "Authorization: Bearer $API_KEY")

# Extract total cost using jq
TOTAL_COST=$(echo $RESPONSE | jq -r '.data.total_cost')
echo "Total Cost: \$$TOTAL_COST"

# Extract breakdown
echo ""
echo "By Provider:"
echo $RESPONSE | jq -r '.data.breakdown[] | "  \(.value): $\(.cost) (\(.requests) requests)"'
```

### Weekly Summary Script

```bash
#!/bin/bash
# weekly-summary.sh

WEEK_AGO=$(date -d "7 days ago" +%Y-%m-%d)
TODAY=$(date +%Y-%m-%d)

echo "=== Weekly Cost Summary ==="
echo "Period: $WEEK_AGO to $TODAY"
echo ""

# Get costs grouped by day
curl -s -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&start_date=${WEEK_AGO}T00:00:00Z&end_date=${TODAY}T23:59:59Z&group_by=day" \
  -H "Authorization: Bearer $API_KEY" \
  | jq -r '.data.breakdown[] | "\(.value): $\(.cost) (\(.tokens) tokens, \(.requests) requests)"'
```

### Cost Monitoring Script

```bash
#!/bin/bash
# monitor-costs.sh - Alert if daily costs exceed threshold

THRESHOLD=20.00
TODAY=$(date +%Y-%m-%d)
START="${TODAY}T00:00:00Z"
END="${TODAY}T23:59:59Z"

# Get today's costs
COST=$(curl -s -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&start_date=$START&end_date=$END" \
  -H "Authorization: Bearer $API_KEY" \
  | jq -r '.data.total_cost')

# Compare with threshold
if (( $(echo "$COST > $THRESHOLD" | bc -l) )); then
  echo "⚠️  ALERT: Daily cost $COST exceeds threshold $THRESHOLD"
  # Send alert (email, Slack, PagerDuty, etc.)
else
  echo "✅ Daily cost: $COST (under threshold)"
fi
```

### Export Data Script

```bash
#!/bin/bash
# export-monthly-data.sh

MONTH=$(date +%Y-%m)
START="${MONTH}-01T00:00:00Z"
END="${MONTH}-31T23:59:59Z"
OUTPUT_FILE="costs-${MONTH}.json"

echo "Exporting cost data for $MONTH..."

curl -X GET "$BASE_URL/api/v1/usage/history?organization_id=$ORG_ID&start_date=$START&end_date=$END&page_size=1000" \
  -H "Authorization: Bearer $API_KEY" \
  -o $OUTPUT_FILE

echo "Exported to $OUTPUT_FILE"
```

## Error Handling

### Handle 401 Unauthorized

```bash
RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/api/v1/costs" \
  -H "Authorization: Bearer invalid_key")

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -1)

if [ "$HTTP_CODE" = "401" ]; then
  echo "Error: Unauthorized - Check your API key"
  echo "Response: $BODY"
  exit 1
fi
```

### Handle 404 Not Found

```bash
RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "$BASE_URL/api/v1/pricing/invalid-id" \
  -H "Authorization: Bearer $API_KEY")

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | head -1)

if [ "$HTTP_CODE" = "404" ]; then
  echo "Error: Resource not found"
  echo "Response: $BODY"
  exit 1
fi
```

### Handle Rate Limiting

```bash
#!/bin/bash
# Rate limit aware request

MAX_RETRIES=3
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
  RESPONSE=$(curl -s -w "\n%{http_code}" -X POST $BASE_URL/api/v1/usage \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $API_KEY" \
    -d '{"organization_id":"'$ORG_ID'","provider":"openai","model_id":"gpt-4","input_tokens":1000,"output_tokens":500,"total_tokens":1500}')

  HTTP_CODE=$(echo "$RESPONSE" | tail -1)
  BODY=$(echo "$RESPONSE" | head -1)

  if [ "$HTTP_CODE" = "429" ]; then
    RETRY_AFTER=$(echo "$BODY" | jq -r '.retry_after // 60')
    echo "Rate limited. Retrying after $RETRY_AFTER seconds..."
    sleep $RETRY_AFTER
    RETRY_COUNT=$((RETRY_COUNT + 1))
  else
    echo "Success: $BODY"
    break
  fi
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
  echo "Failed after $MAX_RETRIES retries"
  exit 1
fi
```

## Testing & Debugging

### Verbose Output

```bash
# Show full request/response details
curl -v -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID" \
  -H "Authorization: Bearer $API_KEY"
```

### Save Response to File

```bash
curl -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z" \
  -H "Authorization: Bearer $API_KEY" \
  -o costs.json

# Pretty print with jq
cat costs.json | jq '.'
```

### Measure Response Time

```bash
curl -w "\nTime: %{time_total}s\n" -X GET "$BASE_URL/api/v1/costs?organization_id=$ORG_ID" \
  -H "Authorization: Bearer $API_KEY"
```

## Best Practices

1. **Always use environment variables** for sensitive data (API keys)
2. **Check HTTP status codes** before processing responses
3. **Implement retry logic** for transient failures
4. **Use pagination** for large datasets
5. **Add timestamps** to all usage submissions for accuracy
6. **Include metadata** to help with later analysis
7. **Test with small datasets** before running large batch operations

## Next Steps

- [Python SDK Examples](../python/)
- [TypeScript SDK Examples](../typescript/)
- [Authentication Guide](../../getting-started/authentication.md)
- [API Reference](../../api-reference/rest-api/README.md)
