# Video 10: Troubleshooting & Support

## Metadata

- **Duration**: 17-20 minutes
- **Level**: Intermediate
- **Prerequisites**: Videos 01, 02
- **Target Audience**: All users
- **Video ID**: LLMCO-V10-TROUBLESHOOT
- **Version**: 1.0.0

## Learning Objectives

- Diagnose common installation and configuration issues
- Debug SDK integration problems
- Troubleshoot tracking and data collection issues
- Resolve performance and scaling problems
- Use diagnostic tools and logs effectively
- Access community support and resources
- Escalate critical issues properly

## Scene Breakdown

### Scene 1: Opening & Approach
**Duration**: 0:00-1:30

**Narration**:
"Welcome to troubleshooting! Even with careful setup, issues can occur. Today we'll solve the most common problems, learn diagnostic techniques, and show you where to get help. Let's make you self-sufficient at troubleshooting LLM Cost Ops!"

**On-Screen Text**:
- "Troubleshooting Topics:"
  - "Installation & Setup Issues"
  - "SDK Integration Problems"
  - "Tracking & Data Issues"
  - "Performance Problems"
  - "Diagnostic Tools"
  - "Getting Support"

---

### Scene 2: Installation & Setup Issues
**Duration**: 1:30-5:00

**Common Issues & Solutions:**

**Issue 1: Docker Won't Start**
```bash
# Symptom
docker compose up -d
# Error: Error response from daemon: Ports are not available

# Diagnosis
netstat -an | grep 8080
# Output: tcp 0 0 0.0.0.0:8080 0.0.0.0:* LISTEN

# Solution: Port already in use
# Option 1: Change port in docker-compose.yml
ports:
  - "8081:8080"  # Use 8081 instead

# Option 2: Stop conflicting service
lsof -ti:8080 | xargs kill -9
```

**Issue 2: Database Connection Failed**
```bash
# Symptom
docker compose logs api
# Error: connection to database failed: connection refused

# Diagnosis
docker compose ps
# database service not running

# Solution: Check database logs
docker compose logs db

# Common causes:
# 1. Database still initializing (wait 30 seconds)
# 2. Wrong credentials in .env
# 3. Port conflict

# Fix .env file
DATABASE_URL=postgresql://postgres:postgres@db:5432/llm_cost_ops
# Not: localhost (use service name 'db')
```

**Issue 3: Migration Errors**
```bash
# Symptom
Pending migrations failed to apply

# Diagnosis
docker compose exec api ./llm-cost-ops db status
# Shows failed migration

# Solution: Rollback and retry
docker compose exec api ./llm-cost-ops db rollback
docker compose exec api ./llm-cost-ops db migrate

# If still failing, reset (development only!)
docker compose down -v  # Deletes all data!
docker compose up -d
```

**Highlight**: "Check logs first • Verify environment variables • Confirm network connectivity"

---

### Scene 3: SDK Integration Issues
**Duration**: 5:00-9:00

**Issue 1: Import Errors (Python)**
```python
# Symptom
from llm_cost_ops import CostTracker
# ImportError: No module named 'llm_cost_ops'

# Diagnosis
pip list | grep llm-cost-ops
# Not found

# Solution: Install SDK
pip install llm-cost-ops

# Verify installation
python -c "from llm_cost_ops import CostTracker; print('OK')"

# Virtual environment issues
which python  # Ensure using venv python
source venv/bin/activate  # Activate venv
```

**Issue 2: Tracking Not Working**
```typescript
// Symptom
await tracker.track(llmCall);
// No error, but data not appearing in dashboard

// Diagnosis - Enable debug mode
const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,
  debug: true  // Enable verbose logging
});

// Check logs
// Output might show:
// "Failed to send tracking data: 401 Unauthorized"

// Common causes:
// 1. Wrong API key
// 2. API key for different project
// 3. Network blocking requests
// 4. Endpoint URL incorrect

// Solution: Verify configuration
console.log('API Key:', process.env.LCOPS_API_KEY);
console.log('Endpoint:', process.env.LCOPS_ENDPOINT);

// Test connectivity
const response = await fetch(`${endpoint}/health`);
console.log('Health check:', response.status);  // Should be 200

// Verify API key
const testTracker = new CostTracker({
  apiKey: 'test-key',
  endpoint: 'http://localhost:8081'
});

try {
  await testTracker.track(/* ... */);
} catch (error) {
  console.error('Tracking error:', error.message);
  // "Invalid API key" - key is wrong
  // "Network error" - can't reach endpoint
}
```

**Issue 3: TypeScript Type Errors**
```typescript
// Symptom
tracker.track(openai.chat.completions.create(...));
// Error: Argument of type 'Promise<ChatCompletion>' is not assignable

// Solution: Await the promise
const response = await tracker.track(
  await openai.chat.completions.create(...)
);

// Or use the correct overload
const response = await tracker.track(
  openai.chat.completions.create(...),
  { tags: { ... } }
);
```

**Issue 4: Rate Limiting**
```typescript
// Symptom
Error: Rate limit exceeded: 429 Too Many Requests

// Diagnosis
// Tracker has default rate limits
// Sending > 1000 requests/minute

// Solution: Implement buffering
const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,
  bufferSize: 100,      // Batch requests
  bufferTimeout: 5000,  // Flush every 5 seconds
});

// Or implement retry logic
const trackerWithRetry = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,
  maxRetries: 3,
  retryDelay: 1000,
  retryBackoff: 2.0
});
```

**Highlight**: "Enable debug mode • Verify API keys • Check network connectivity"

---

### Scene 4: Data & Tracking Issues
**Duration**: 9:00-12:00

**Issue 1: Costs Are Zero or Incorrect**
```typescript
// Symptom
Dashboard shows requests but $0.00 cost

// Diagnosis
const request = await tracker.getRequest(requestId);
console.log(request);

// Check:
// - model field is set correctly
// - tokens are being tracked
// - provider is recognized

// Common causes:
// 1. Custom model without pricing
// 2. Wrong model name
// 3. Tokens not extracted

// Solution: Explicit pricing
const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,
  customPricing: [{
    provider: 'openai',
    model: 'gpt-4',
    inputTokenCost: 0.03 / 1000,
    outputTokenCost: 0.06 / 1000
  }]
});

// Or verify model name matches
await tracker.track(
  openai.chat.completions.create({
    model: 'gpt-4',  // Exactly: 'gpt-4', not 'GPT-4' or 'gpt4'
  }),
  {
    provider: 'openai',  // Explicitly set provider
    model: 'gpt-4'       // Explicitly set model
  }
);
```

**Issue 2: Missing Requests**
```typescript
// Symptom
Made 100 requests, dashboard shows only 80

// Diagnosis: Check error logs
const errorLogs = await tracker.getErrorLogs({
  startDate: new Date(Date.now() - 3600000),  // Last hour
  limit: 100
});

console.log(`Failed tracking: ${errorLogs.length}`);
errorLogs.forEach(log => {
  console.log(`- ${log.timestamp}: ${log.error}`);
});

// Common causes:
// 1. Network timeouts
// 2. SDK silently failing (strict mode off)
// 3. Buffering not flushed

// Solution: Enable strict mode for debugging
const strictTracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,
  strict: true  // Throws errors instead of silent failure
});

// Ensure buffer is flushed
await tracker.flush();  // Manually flush before exit

// In production, handle graceful shutdown
process.on('SIGTERM', async () => {
  await tracker.flush();
  process.exit(0);
});
```

**Issue 3: Duplicate Requests**
```typescript
// Symptom
Dashboard shows same request multiple times

// Diagnosis
// Retries without idempotency key

// Solution: Use request IDs
await tracker.track(
  llmCall,
  {
    requestId: generateUniqueId(),  // Prevents duplicates
    tags: { ... }
  }
);

// Or use built-in deduplication
const deduplicatingTracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,
  deduplication: {
    enabled: true,
    window: 60000  // 60 second dedup window
  }
});
```

**Highlight**: "Verify pricing • Check error logs • Use request IDs for deduplication"

---

### Scene 5: Performance Issues
**Duration**: 12:00-14:30

**Issue 1: Slow Application Performance**
```typescript
// Symptom
Application latency increased after adding tracking

// Diagnosis
// Measure tracking overhead
const start = performance.now();
await tracker.track(llmCall);
const trackingTime = performance.now() - start;
console.log(`Tracking overhead: ${trackingTime}ms`);

// If > 100ms, investigate

// Solution: Async tracking
const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,
  asyncMode: true,  // Don't block on tracking
  bufferSize: 100    // Batch in background
});

// Tracking happens in background
await tracker.track(llmCall);  // Returns immediately

// Or use fire-and-forget
tracker.trackAsync(llmCall);  // Don't await
const response = await llmCall;  // Continue immediately
```

**Issue 2: High Memory Usage**
```typescript
// Symptom
Memory grows over time, eventual crash

// Diagnosis
// Large buffer not being flushed
console.log(`Buffer size: ${tracker.getBufferSize()}`);

// Solution: Reduce buffer size or timeout
const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!,
  bufferSize: 50,       // Smaller buffer (was 1000)
  bufferTimeout: 2000,  // Flush more frequently (was 30000)
});

// Monitor memory
setInterval(() => {
  const used = process.memoryUsage();
  console.log(`Memory: ${Math.round(used.heapUsed / 1024 / 1024)} MB`);
  console.log(`Buffer: ${tracker.getBufferSize()} items`);
}, 10000);
```

**Issue 3: Database Performance**
```sql
-- Symptom
-- Dashboard slow to load

-- Diagnosis
-- Missing indexes
EXPLAIN ANALYZE
SELECT * FROM tracked_requests
WHERE project_id = 'abc'
  AND timestamp > NOW() - INTERVAL '30 days';

-- Seq Scan (slow!)

-- Solution: Add indexes
CREATE INDEX idx_tracked_requests_project_timestamp
ON tracked_requests(project_id, timestamp DESC);

CREATE INDEX idx_tracked_requests_tags
ON tracked_requests USING GIN(tags);

-- Verify improvement
EXPLAIN ANALYZE
SELECT * FROM tracked_requests
WHERE project_id = 'abc'
  AND timestamp > NOW() - INTERVAL '30 days';

-- Index Scan (fast!)
```

**Highlight**: "Use async mode • Reduce buffer size • Add database indexes"

---

### Scene 6: Diagnostic Tools
**Duration**: 14:30-16:30

**Built-in Diagnostics:**
```bash
# Health check endpoint
curl http://localhost:8080/health

# Response:
# {
#   "status": "healthy",
#   "version": "1.0.0",
#   "uptime": 86400,
#   "database": "connected",
#   "redis": "connected"
# }

# Detailed diagnostics
curl http://localhost:8080/diagnostics

# Response:
# {
#   "database": {
#     "status": "connected",
#     "latency": 5,
#     "activeConnections": 12,
#     "maxConnections": 100
#   },
#   "redis": {
#     "status": "connected",
#     "latency": 1,
#     "memoryUsed": "45.2MB"
#   },
#   "api": {
#     "requestsPerSecond": 123,
#     "averageLatency": 45,
#     "errorRate": 0.001
#   }
# }
```

**CLI Diagnostic Tools:**
```bash
# Test API connectivity
llm-cost-ops test connection \
  --endpoint http://localhost:8080 \
  --api-key your-key

# Validate configuration
llm-cost-ops validate config \
  --config-file production.yaml

# Check database migrations
llm-cost-ops db status

# Test tracking pipeline
llm-cost-ops test track \
  --provider openai \
  --model gpt-3.5-turbo \
  --test-mode

# Export logs for support
llm-cost-ops logs export \
  --since 1h \
  --output support-logs.zip
```

**Highlight**: "Health endpoints • CLI diagnostics • Log export for support"

---

### Scene 7: Getting Support
**Duration**: 16:30-18:30

**Self-Service Resources:**
```markdown
## Documentation
- Official Docs: https://docs.llm-cost-ops.dev
- API Reference: https://api-docs.llm-cost-ops.dev
- SDK Docs (Python): https://python-sdk.llm-cost-ops.dev
- SDK Docs (TypeScript): https://ts-sdk.llm-cost-ops.dev

## Community
- GitHub Discussions: github.com/llm-cost-ops/discussions
- Discord: discord.gg/llm-cost-ops
- Stack Overflow: tag [llm-cost-ops]
- Twitter: @llmcostops

## Issue Reporting
- GitHub Issues: github.com/llm-cost-ops/issues
- Security Issues: security@llm-cost-ops.dev
- Bug Report Template: Use GitHub issue template

## Enterprise Support
- Support Portal: support.llm-cost-ops.dev
- SLA Response Times:
  - P1 (Critical): 1 hour
  - P2 (High): 4 hours
  - P3 (Medium): 1 business day
  - P4 (Low): 3 business days
```

**How to Report Issues:**
```markdown
## Good Bug Report Template

**Environment:**
- LLM Cost Ops Version: 1.0.0
- SDK: Python 3.11
- SDK Version: 1.0.0
- OS: Ubuntu 22.04
- Deployment: Docker Compose

**Expected Behavior:**
Tracking should capture costs for all requests

**Actual Behavior:**
50% of requests not tracked, no errors shown

**Steps to Reproduce:**
1. Install via Docker Compose
2. Run Python SDK example
3. Make 100 requests
4. Check dashboard - only 50 visible

**Logs:**
```
[Attach relevant logs]
```

**Configuration:**
```yaml
[Attach relevant config, redact secrets]
```

**Additional Context:**
- Using OpenAI GPT-4
- Behind corporate proxy
- High request volume (1000/min)
```

**Highlight**: "Check docs first • Provide detailed bug reports • Use community resources"

---

### Scene 8: Preventative Measures
**Duration**: 18:30-19:00

**Best Practices:**
```typescript
// 1. Implement comprehensive error handling
try {
  await tracker.track(llmCall, { tags });
} catch (error) {
  logger.error('Tracking failed:', error);
  // Don't crash - tracking is non-critical
}

// 2. Monitor tracking health
setInterval(async () => {
  const health = await tracker.getHealth();
  if (health.errorRate > 0.05) {  // > 5% errors
    alerting.notify('High tracking error rate');
  }
}, 60000);

// 3. Set up alerts
await tracker.configureAlerts({
  trackingFailureRate: {
    threshold: 0.05,
    window: '5m',
    channels: ['slack']
  },
  bufferOverflow: {
    threshold: 1000,
    channels: ['email']
  }
});

// 4. Regular health checks
// Add to monitoring system
curl http://localhost:8080/health || alert "LLM Cost Ops down"

// 5. Keep SDK updated
npm update llm-cost-ops
pip install --upgrade llm-cost-ops
```

**Highlight**: "Error handling • Health monitoring • Keep updated • Set up alerts"

---

### Scene 9: Series Conclusion
**Duration**: 19:00-20:00

**Narration**:
"Congratulations! You've completed the entire LLM Cost Ops video series. You now know how to install, configure, integrate, optimize, deploy, secure, and troubleshoot the platform. You're ready to take control of your AI costs!

Remember: the community is here to help. Join our Discord, check the docs, and don't hesitate to open issues on GitHub. Happy cost tracking!"

**On-Screen Text**:
- "Series Complete!"
- "You've Learned:"
  - "✅ Platform fundamentals"
  - "✅ SDK integration (Python & TypeScript)"
  - "✅ Analytics & dashboards"
  - "✅ Budget management"
  - "✅ Cost optimization"
  - "✅ Enterprise deployment"
  - "✅ Security & compliance"
  - "✅ Troubleshooting"

**Resources:**
- "📖 Documentation: docs.llm-cost-ops.dev"
- "💬 Discord: discord.gg/llm-cost-ops"
- "⭐ GitHub: github.com/llm-cost-ops"
- "📧 Support: support@llm-cost-ops.dev"

**Call to Action:**
- "Please rate this series!"
- "Share with your team"
- "Star us on GitHub"
- "Join the community"

---

## Post-Production Notes

### Chapter Markers
- 0:00 - Introduction
- 1:30 - Installation Issues
- 5:00 - SDK Problems
- 9:00 - Data Issues
- 12:00 - Performance Issues
- 14:30 - Diagnostic Tools
- 16:30 - Getting Support
- 18:30 - Preventative Measures
- 19:00 - Series Conclusion

### Common Issues Quick Reference Card
Create downloadable PDF with:
- Top 10 issues and solutions
- Diagnostic command cheatsheet
- Support contact information

### Companion Resources
- Troubleshooting flowchart
- Diagnostic script
- Log analysis tool
- Support ticket template

**Script Version**: 1.0.0
**Last Updated**: 2025-11-16
