# Troubleshooting Guide

Common issues and solutions when working with LLM-CostOps.

## Table of Contents

1. [Authentication Issues](#authentication-issues)
2. [API Errors](#api-errors)
3. [Cost Tracking Issues](#cost-tracking-issues)
4. [Pricing Issues](#pricing-issues)
5. [Performance Issues](#performance-issues)
6. [SDK-Specific Issues](#sdk-specific-issues)
7. [Deployment Issues](#deployment-issues)

---

## Authentication Issues

### Error: "Unauthorized" (401)

**Symptom:**
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid or missing API key"
  }
}
```

**Causes & Solutions:**

1. **Missing API Key**
   ```bash
   # ❌ Missing Authorization header
   curl -X GET https://api.llm-cost-ops.dev/api/v1/costs

   # ✅ Include Authorization header
   curl -X GET https://api.llm-cost-ops.dev/api/v1/costs \
     -H "Authorization: Bearer YOUR_API_KEY"
   ```

2. **Incorrect Format**
   ```bash
   # ❌ Wrong format
   -H "Authorization: YOUR_API_KEY"

   # ✅ Correct format
   -H "Authorization: Bearer YOUR_API_KEY"
   ```

3. **Invalid API Key**
   - Verify the API key hasn't been revoked
   - Check for typos or extra spaces
   - Generate a new API key if needed

4. **Expired API Key**
   ```bash
   # Create a new API key
   cost-ops auth create-key --organization org-123 --name "New Key"
   ```

### Error: "Forbidden" (403)

**Symptom:**
```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Insufficient permissions"
  }
}
```

**Solutions:**

1. **Check API Key Permissions**
   ```bash
   # List API key permissions
   cost-ops auth describe-key key_123

   # Grant additional permissions
   cost-ops auth update-key key_123 \
     --add-permissions "usage:write,costs:read"
   ```

2. **Verify Organization Access**
   - Ensure you have access to the organization
   - Check if the organization ID is correct

3. **Contact Admin**
   - Request permission upgrade from organization admin

---

## API Errors

### Error: "Bad Request" (400)

**Symptom:**
```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "organization_id is required",
    "details": {
      "field": "organization_id",
      "reason": "missing_required_field"
    }
  }
}
```

**Common Causes:**

1. **Missing Required Fields**
   ```bash
   # ❌ Missing organization_id
   curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $API_KEY" \
     -d '{
       "provider": "openai",
       "model_id": "gpt-4",
       "input_tokens": 1000,
       "output_tokens": 500
     }'

   # ✅ Include all required fields
   curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $API_KEY" \
     -d '{
       "organization_id": "org-123",
       "provider": "openai",
       "model_id": "gpt-4",
       "input_tokens": 1000,
       "output_tokens": 500,
       "total_tokens": 1500
     }'
   ```

2. **Invalid Field Values**
   ```bash
   # ❌ Negative token count
   "input_tokens": -100

   # ✅ Valid token count
   "input_tokens": 100
   ```

3. **Invalid Date Format**
   ```bash
   # ❌ Wrong format
   "start_date": "2025-01-01"

   # ✅ ISO 8601 format
   "start_date": "2025-01-01T00:00:00Z"
   ```

### Error: "Not Found" (404)

**Symptom:**
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "No pricing found for model: gpt-5"
  }
}
```

**Solutions:**

1. **Add Missing Pricing**
   ```bash
   curl -X POST https://api.llm-cost-ops.dev/api/v1/pricing \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $API_KEY" \
     -d '{
       "provider": "openai",
       "model_id": "gpt-4",
       "input_price_per_1k": 0.01,
       "output_price_per_1k": 0.03,
       "currency": "USD"
     }'
   ```

2. **Check Model ID**
   - Verify the model ID is correct
   - Check for typos (e.g., `gpt-4` not `gpt4`)

3. **List Available Pricing**
   ```bash
   curl -X GET "https://api.llm-cost-ops.dev/api/v1/pricing?provider=openai" \
     -H "Authorization: Bearer $API_KEY"
   ```

### Error: "Rate Limit Exceeded" (429)

**Symptom:**
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests",
    "retry_after": 60
  }
}
```

**Solutions:**

1. **Implement Exponential Backoff**
   ```python
   import time
   from llm_cost_ops import CostOpsClient
   from llm_cost_ops.exceptions import RateLimitError

   client = CostOpsClient(api_key="YOUR_API_KEY")

   def submit_with_retry(usage_data, max_retries=3):
       for attempt in range(max_retries):
           try:
               return client.usage.submit(**usage_data)
           except RateLimitError as e:
               if attempt < max_retries - 1:
                   wait_time = 2 ** attempt  # Exponential backoff
                   print(f"Rate limited. Retrying in {wait_time}s...")
                   time.sleep(wait_time)
               else:
                   raise
   ```

2. **Use Batch Endpoints**
   ```bash
   # Instead of multiple individual requests
   for usage in usages:
       submit_usage(usage)  # ❌ Multiple API calls

   # Use batch endpoint
   submit_usage_batch(usages)  # ✅ Single API call
   ```

3. **Upgrade Your Plan**
   - Contact support to increase rate limits
   - Move to a higher tier plan

### Error: "Internal Server Error" (500)

**Symptom:**
```json
{
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "An unexpected error occurred",
    "request_id": "req-123abc"
  }
}
```

**Solutions:**

1. **Retry the Request**
   - Transient errors often resolve on retry
   - Wait a few seconds before retrying

2. **Check Service Status**
   ```bash
   curl -X GET https://api.llm-cost-ops.dev/health
   ```

3. **Contact Support**
   - Include the `request_id` from the error
   - Provide the request details

---

## Cost Tracking Issues

### Issue: Costs Don't Match Expected Values

**Diagnosis:**

1. **Check Pricing Table**
   ```bash
   curl -X GET "https://api.llm-cost-ops.dev/api/v1/pricing?provider=openai&model_id=gpt-4" \
     -H "Authorization: Bearer $API_KEY"
   ```

2. **Verify Token Counts**
   ```python
   # Ensure total_tokens = input_tokens + output_tokens
   assert usage.total_tokens == usage.input_tokens + usage.output_tokens
   ```

3. **Check for Cached Tokens**
   - Some providers offer discounts for cached tokens
   - Ensure you're tracking cached tokens separately

**Solutions:**

1. **Update Pricing**
   ```bash
   # Update to latest pricing
   curl -X POST https://api.llm-cost-ops.dev/api/v1/pricing \
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

2. **Verify Token Calculation**
   ```python
   # OpenAI example
   response = openai.ChatCompletion.create(
       model="gpt-4",
       messages=[{"role": "user", "content": "Hello"}]
   )

   # Always use the usage from the response
   usage = {
       "input_tokens": response.usage.prompt_tokens,      # ✅
       "output_tokens": response.usage.completion_tokens, # ✅
       "total_tokens": response.usage.total_tokens        # ✅
   }

   # Don't estimate manually
   # "input_tokens": len(message) / 4  # ❌ Inaccurate
   ```

### Issue: Missing Usage Records

**Diagnosis:**

1. **Check if Submission Succeeded**
   ```python
   try:
       result = client.usage.submit(**usage_data)
       print(f"Submitted: {result.usage_id}")
   except Exception as e:
       print(f"Failed: {e}")
   ```

2. **Verify Organization ID**
   ```python
   # Ensure you're querying the correct organization
   costs = client.costs.get(
       organization_id="org-123",  # Must match submission
       start_date=...,
       end_date=...
   )
   ```

3. **Check Date Range**
   ```bash
   # Ensure date range includes the submission time
   curl -X GET "https://api.llm-cost-ops.dev/api/v1/usage/history?organization_id=org-123&start_date=2025-01-15T00:00:00Z&end_date=2025-01-15T23:59:59Z" \
     -H "Authorization: Bearer $API_KEY"
   ```

**Solutions:**

1. **Add Error Handling**
   ```python
   from llm_cost_ops import CostOpsClient
   from llm_cost_ops.exceptions import ApiError

   client = CostOpsClient(api_key="YOUR_API_KEY")

   try:
       result = client.usage.submit(
           organization_id="org-123",
           provider="openai",
           model_id="gpt-4",
           input_tokens=1000,
           output_tokens=500,
           total_tokens=1500
       )
       print(f"✅ Submitted: {result.usage_id}")
   except ApiError as e:
       print(f"❌ Failed: {e.message}")
       # Log error, retry, or alert
   ```

2. **Use Background Tasks**
   ```python
   # FastAPI example
   from fastapi import BackgroundTasks

   def track_usage_background(usage_data):
       try:
           client.usage.submit(**usage_data)
       except Exception as e:
           logger.error(f"Failed to track usage: {e}")
           # Add to retry queue

   @app.post("/api/chat")
   async def chat(message: str, background_tasks: BackgroundTasks):
       response = openai.ChatCompletion.create(...)

       background_tasks.add_task(
           track_usage_background,
           usage_data={...}
       )

       return response
   ```

---

## Pricing Issues

### Issue: "No pricing found for model"

**Solutions:**

1. **Add Pricing for the Model**
   ```bash
   curl -X POST https://api.llm-cost-ops.dev/api/v1/pricing \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $API_KEY" \
     -d '{
       "provider": "openai",
       "model_id": "gpt-4-turbo",
       "input_price_per_1k": 0.01,
       "output_price_per_1k": 0.03,
       "currency": "USD"
     }'
   ```

2. **Check Model Name**
   ```python
   # ❌ Incorrect
   model_id = "gpt4"

   # ✅ Correct
   model_id = "gpt-4"
   ```

3. **Use Default Pricing**
   ```python
   # Set a default/fallback pricing
   client = CostOpsClient(
       api_key="YOUR_API_KEY",
       default_pricing={
           "input_price_per_1k": 0.01,
           "output_price_per_1k": 0.03
       }
   )
   ```

### Issue: Outdated Pricing

**Solutions:**

1. **Add New Pricing with Effective Date**
   ```bash
   # Old pricing automatically expires when new pricing becomes effective
   curl -X POST https://api.llm-cost-ops.dev/api/v1/pricing \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $API_KEY" \
     -d '{
       "provider": "openai",
       "model_id": "gpt-4",
       "input_price_per_1k": 0.005,
       "output_price_per_1k": 0.015,
       "currency": "USD",
       "effective_date": "2025-02-01T00:00:00Z"
     }'
   ```

2. **Import Latest Pricing**
   ```bash
   # Self-hosted: Load latest pricing from LLM-CostOps
   cost-ops pricing sync-latest
   ```

---

## Performance Issues

### Issue: Slow API Responses

**Diagnosis:**

1. **Measure Response Time**
   ```bash
   curl -w "\nTime: %{time_total}s\n" \
     -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123" \
     -H "Authorization: Bearer $API_KEY"
   ```

2. **Check Query Complexity**
   - Large date ranges
   - Complex grouping/filtering
   - Missing indexes (self-hosted)

**Solutions:**

1. **Use Pagination**
   ```python
   # ❌ Fetch all records at once
   all_usage = client.usage.get_history(page_size=10000)

   # ✅ Paginate through results
   page = 1
   while True:
       usage = client.usage.get_history(page=page, page_size=100)
       process(usage.data)
       if not usage.pagination.has_next:
           break
       page += 1
   ```

2. **Reduce Date Range**
   ```python
   # ❌ Query entire year
   costs = client.costs.get(
       start_date="2025-01-01T00:00:00Z",
       end_date="2025-12-31T23:59:59Z"
   )

   # ✅ Query smaller periods
   costs = client.costs.get(
       start_date="2025-01-01T00:00:00Z",
       end_date="2025-01-31T23:59:59Z"
   )
   ```

3. **Use Caching**
   ```python
   from functools import lru_cache
   from datetime import datetime, timedelta

   @lru_cache(maxsize=128)
   def get_daily_costs(date_str):
       return client.costs.get(
           start_date=f"{date_str}T00:00:00Z",
           end_date=f"{date_str}T23:59:59Z"
       )
   ```

4. **Optimize Database (Self-Hosted)**
   ```sql
   -- Add indexes for common queries
   CREATE INDEX idx_usage_org_timestamp
   ON usage_records(organization_id, timestamp);

   CREATE INDEX idx_usage_provider_model
   ON usage_records(provider, model_id);

   -- Analyze tables
   ANALYZE usage_records;
   ANALYZE cost_records;
   ```

### Issue: High Memory Usage

**Solutions:**

1. **Stream Large Exports**
   ```python
   # ❌ Load all data into memory
   all_data = client.usage.get_history(page_size=100000)

   # ✅ Stream data
   with client.usage.stream(start_date=..., end_date=...) as stream:
       for batch in stream:
           process(batch)
   ```

2. **Use Background Workers**
   ```python
   # Process large datasets in background
   from celery import Celery

   @celery.task
   def export_monthly_costs(month):
       costs = client.costs.get(
           start_date=f"{month}-01T00:00:00Z",
           end_date=f"{month}-31T23:59:59Z"
       )
       # Export to S3, send email, etc.
   ```

---

## SDK-Specific Issues

### Python SDK

**Issue: Import Error**
```python
ImportError: No module named 'llm_cost_ops'
```

**Solution:**
```bash
pip install llm-cost-ops
# or
pip install --upgrade llm-cost-ops
```

**Issue: Async/Await Errors**
```python
# ❌ Missing await
result = client.usage.submit_async(...)

# ✅ Use await
result = await client.usage.submit_async(...)
```

### TypeScript SDK

**Issue: Type Errors**
```typescript
// ❌ Wrong type
const costs = await client.costs.get({
  startDate: new Date()  // Error: expecting string
});

// ✅ Correct type
const costs = await client.costs.get({
  startDate: new Date().toISOString()
});
```

---

## Deployment Issues

### Self-Hosted: Database Connection Failed

**Diagnosis:**
```bash
# Test database connectivity
psql -h localhost -U postgres -d costops -c "SELECT 1;"
```

**Solutions:**

1. **Check Connection String**
   ```bash
   # ❌ Wrong format
   DATABASE_URL=postgres://localhost/costops

   # ✅ Correct format
   DATABASE_URL=postgresql://user:password@localhost:5432/costops
   ```

2. **Run Migrations**
   ```bash
   cost-ops migrate --database-url postgresql://user:pass@localhost/costops
   ```

3. **Check Firewall**
   ```bash
   # Test port connectivity
   telnet localhost 5432
   ```

### Kubernetes: Pods Failing

**Diagnosis:**
```bash
kubectl logs -n llm-cost-ops pod-name
kubectl describe pod -n llm-cost-ops pod-name
```

**Solutions:**

1. **Check Resource Limits**
   ```yaml
   resources:
     requests:
       memory: "256Mi"
       cpu: "100m"
     limits:
       memory: "512Mi"
       cpu: "500m"
   ```

2. **Verify ConfigMaps/Secrets**
   ```bash
   kubectl get configmap -n llm-cost-ops
   kubectl get secret -n llm-cost-ops
   ```

3. **Check Health Probes**
   ```yaml
   livenessProbe:
     httpGet:
       path: /health
       port: 8080
     initialDelaySeconds: 30
     periodSeconds: 10
   ```

---

## Getting Help

If you can't resolve your issue:

1. **Check Documentation**
   - [API Reference](api-reference/rest-api/README.md)
   - [FAQ](faq.md)
   - [GitHub Issues](https://github.com/llm-devops/llm-cost-ops/issues)

2. **Community Support**
   - Discord: https://discord.gg/llm-cost-ops
   - GitHub Discussions: https://github.com/llm-devops/llm-cost-ops/discussions

3. **Contact Support**
   - Email: support@llm-cost-ops.dev
   - Include:
     - Request ID (from error response)
     - Full error message
     - Steps to reproduce
     - Environment details (OS, SDK version, etc.)

## Next Steps

- [FAQ](faq.md)
- [Best Practices](guides/best-practices.md)
- [Performance Optimization](guides/performance.md)
