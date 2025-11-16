# Troubleshooting Guide

**Version:** 1.0.0
**Last Updated:** 2025-11-16

Comprehensive troubleshooting guide for the LLM Cost Ops platform. This guide covers common issues, error messages, diagnostic procedures, and solutions for authentication, connectivity, performance, and operational problems.

---

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Authentication Errors](#authentication-errors)
- [Connection Errors](#connection-errors)
- [Database Errors](#database-errors)
- [Rate Limiting Errors](#rate-limiting-errors)
- [Performance Issues](#performance-issues)
- [Memory Issues](#memory-issues)
- [Disk Space Issues](#disk-space-issues)
- [Network Connectivity](#network-connectivity)
- [SSL/TLS Errors](#ssltls-errors)
- [Log Analysis](#log-analysis)
- [Debugging Techniques](#debugging-techniques)
- [Getting Help](#getting-help)

---

## Quick Diagnostics

### Health Check

Check overall system health:

```bash
# API health
curl http://localhost:8080/health

# Expected output:
# {"status":"healthy","version":"1.0.0","uptime_seconds":3600}
```

### CLI Diagnostics

```bash
# Check CLI version
cost-ops --version

# Test database connectivity
cost-ops init --dry-run

# Verify authentication
cost-ops auth whoami

# Test API connectivity
cost-ops query --limit 1
```

### System Requirements Check

```bash
# Check disk space
df -h /var/lib/llm-cost-ops

# Check memory
free -h

# Check CPU
top -bn1 | grep "Cpu(s)"

# Check network
ping -c 3 api.llm-cost-ops.example.com
```

### Log Locations

**Application Logs:**
- Systemd: `journalctl -u llm-cost-ops -f`
- File: `/var/log/llm-cost-ops/app.log`
- Docker: `docker logs llm-cost-ops`
- Kubernetes: `kubectl logs -l app=llm-cost-ops -f`

---

## Authentication Errors

### Error: Invalid API Key

**Error Message:**
```json
{
  "error": {
    "code": "INVALID_API_KEY",
    "message": "Invalid API key"
  }
}
```

**Causes:**
1. API key is incorrect or malformed
2. API key has been revoked
3. API key has expired
4. Wrong API key prefix (using test key in production)

**Solutions:**

1. **Verify API Key:**
```bash
# Check current API key
echo $COST_OPS_API_KEY

# List active API keys
cost-ops auth api-key list

# Create new API key
cost-ops auth api-key create --name "Production Key"
```

2. **Check API Key Format:**
```bash
# Production keys should start with sk_live_
echo $COST_OPS_API_KEY | grep '^sk_live_'

# Test keys start with sk_test_
echo $COST_OPS_API_KEY | grep '^sk_test_'
```

3. **Update API Key:**
```bash
export COST_OPS_API_KEY=sk_live_new_key_here

# Or update config file
vi ~/.config/cost-ops/config.toml
```

---

### Error: Token Expired

**Error Message:**
```json
{
  "error": {
    "code": "TOKEN_EXPIRED",
    "message": "Token expired"
  }
}
```

**Causes:**
1. JWT access token has expired (typically after 1 hour)
2. System clock is incorrect

**Solutions:**

1. **Refresh Token:**
```bash
# Login again to get new token
cost-ops auth login

# Or use refresh token (if using SDK)
curl -X POST http://localhost:8080/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"YOUR_REFRESH_TOKEN"}'
```

2. **Check System Time:**
```bash
# Verify system time
date
timedatectl

# Sync time (if incorrect)
sudo ntpdate -s time.nist.gov
```

---

### Error: Insufficient Permissions

**Error Message:**
```json
{
  "error": {
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Insufficient permissions"
  }
}
```

**Causes:**
1. User lacks required permissions for the operation
2. API key has limited scopes
3. RBAC policy restricts access

**Solutions:**

1. **Check Current Permissions:**
```bash
cost-ops auth whoami
# Output shows roles and permissions
```

2. **Request Permission Elevation:**
Contact your administrator to:
- Assign appropriate role (viewer, analyst, engineer, admin)
- Add required scopes to API key
- Update RBAC policies

3. **Use Correct API Key:**
```bash
# Use admin API key for administrative operations
export COST_OPS_API_KEY=sk_admin_abc123
```

---

### Error: Missing Credentials

**Error Message:**
```json
{
  "error": {
    "code": "MISSING_CREDENTIALS",
    "message": "Missing authentication credentials"
  }
}
```

**Causes:**
1. No API key or JWT token provided
2. Authorization header missing
3. Environment variable not set

**Solutions:**

1. **Set API Key:**
```bash
export COST_OPS_API_KEY=sk_live_abc123
```

2. **Login:**
```bash
cost-ops auth login
```

3. **Check Headers (API Calls):**
```bash
# Correct header format
curl -H "Authorization: Bearer sk_live_abc123" \
  http://localhost:8080/api/v1/costs
```

---

## Connection Errors

### Error: Connection Refused

**Error Message:**
```
Error: Connection refused (os error 111)
```

**Causes:**
1. API server is not running
2. Wrong host or port
3. Firewall blocking connection
4. Server crashed or failed to start

**Solutions:**

1. **Check Server Status:**
```bash
# Systemd
systemctl status llm-cost-ops

# Docker
docker ps | grep llm-cost-ops

# Kubernetes
kubectl get pods -l app=llm-cost-ops
```

2. **Start Server:**
```bash
# Systemd
sudo systemctl start llm-cost-ops

# Docker
docker start llm-cost-ops

# CLI
cost-ops server
```

3. **Verify Host and Port:**
```bash
# Check configuration
cost-ops config get api.bind
cost-ops config get api.port

# Test connectivity
nc -zv localhost 8080
```

4. **Check Firewall:**
```bash
# UFW (Ubuntu)
sudo ufw status
sudo ufw allow 8080/tcp

# Firewalld (RHEL/CentOS)
sudo firewall-cmd --list-ports
sudo firewall-cmd --add-port=8080/tcp --permanent
```

5. **Check Logs for Startup Errors:**
```bash
journalctl -u llm-cost-ops -n 100
```

---

### Error: Connection Timeout

**Error Message:**
```
Error: Connection timed out after 30 seconds
```

**Causes:**
1. Network latency or packet loss
2. Server overloaded
3. DNS resolution issues
4. Intermediate proxy timeout

**Solutions:**

1. **Increase Timeout:**
```bash
# CLI with custom timeout
cost-ops --timeout 60 query

# API request with timeout
curl --max-time 60 http://localhost:8080/api/v1/costs
```

2. **Check Network Latency:**
```bash
ping -c 10 api.llm-cost-ops.example.com
traceroute api.llm-cost-ops.example.com
```

3. **Test DNS:**
```bash
nslookup api.llm-cost-ops.example.com
dig api.llm-cost-ops.example.com
```

4. **Check Server Load:**
```bash
# CPU and memory
top

# Network connections
netstat -an | grep 8080 | wc -l
```

---

## Database Errors

### Error: Database Connection Failed

**Error Message:**
```
Error: DATABASE_ERROR - Failed to connect to database
```

**Causes:**
1. Database server is down
2. Wrong connection credentials
3. Network connectivity issues
4. Connection pool exhausted

**Solutions:**

1. **Verify Database Status:**
```bash
# PostgreSQL
sudo systemctl status postgresql
psql -U postgres -c "SELECT version();"

# SQLite
sqlite3 /var/lib/llm-cost-ops/cost-ops.db ".databases"
```

2. **Test Connection:**
```bash
# PostgreSQL
psql "postgresql://user:pass@localhost:5432/costops"

# Using cost-ops CLI
cost-ops init --database-url "postgresql://user:pass@localhost/costops" --dry-run
```

3. **Check Connection String:**
```bash
# View current database URL
cost-ops config get database.url

# Update database URL
export COST_OPS_DATABASE_URL="postgresql://user:pass@localhost/costops"
```

4. **Check Connection Pool:**
```toml
# config.toml
[database]
pool_size = 20  # Increase if exhausted
max_lifetime_secs = 3600
idle_timeout_secs = 600
```

---

### Error: Too Many Connections

**Error Message:**
```
Error: FATAL: remaining connection slots are reserved
```

**Causes:**
1. PostgreSQL max_connections limit reached
2. Connection pool not properly releasing connections
3. Connection leaks in application

**Solutions:**

1. **Check Active Connections:**
```sql
-- PostgreSQL
SELECT count(*) FROM pg_stat_activity WHERE datname = 'costops';
SELECT * FROM pg_stat_activity WHERE datname = 'costops';
```

2. **Increase PostgreSQL max_connections:**
```bash
# Edit postgresql.conf
sudo vi /etc/postgresql/15/main/postgresql.conf

# Update max_connections
max_connections = 200

# Restart PostgreSQL
sudo systemctl restart postgresql
```

3. **Reduce Pool Size:**
```toml
[database]
pool_size = 10  # Reduce from 20
```

4. **Kill Idle Connections:**
```sql
-- Kill connections idle for more than 1 hour
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = 'costops'
  AND state = 'idle'
  AND state_change < NOW() - INTERVAL '1 hour';
```

---

### Error: Migration Failed

**Error Message:**
```
Error: Migration 003_add_forecasting failed
```

**Causes:**
1. Schema conflicts
2. Missing dependencies
3. Interrupted migration
4. Insufficient database permissions

**Solutions:**

1. **Check Migration Status:**
```sql
-- PostgreSQL
SELECT * FROM _sqlx_migrations ORDER BY version;
```

2. **Revert Last Migration:**
```bash
sqlx migrate revert --database-url "$DATABASE_URL"
```

3. **Force Re-run Migration:**
```bash
# Delete migration record
sqlx database drop --database-url "$DATABASE_URL"
sqlx database create --database-url "$DATABASE_URL"
sqlx migrate run --database-url "$DATABASE_URL"
```

4. **Check Permissions:**
```sql
-- Verify user permissions
SELECT has_table_privilege('costops_user', 'usage_records', 'INSERT');
```

---

## Rate Limiting Errors

### Error: Rate Limit Exceeded

**Error Message:**
```json
{
  "error": {
    "code": "TOO_MANY_REQUESTS",
    "message": "Rate limit exceeded. Try again in 3600 seconds."
  }
}
```

**Causes:**
1. Exceeded requests per hour limit
2. Burst limit exceeded
3. Aggressive polling or retry logic

**Solutions:**

1. **Check Rate Limit Headers:**
```bash
curl -I http://localhost:8080/api/v1/costs
# X-RateLimit-Limit: 1000
# X-RateLimit-Remaining: 0
# X-RateLimit-Reset: 1700000000
```

2. **Wait for Reset:**
```bash
# Calculate wait time
reset_time=$(curl -I http://localhost:8080/api/v1/costs | grep X-RateLimit-Reset | cut -d: -f2)
current_time=$(date +%s)
wait_seconds=$((reset_time - current_time))
echo "Wait $wait_seconds seconds"
```

3. **Implement Exponential Backoff:**
```bash
#!/bin/bash
retry=0
max_retries=5
while [ $retry -lt $max_retries ]; do
  if cost-ops query; then
    break
  fi
  wait=$((2 ** retry))
  echo "Retry $retry, waiting $wait seconds..."
  sleep $wait
  retry=$((retry + 1))
done
```

4. **Upgrade Plan:**
Contact sales to upgrade to higher tier with increased limits.

5. **Disable Rate Limiting (Development Only):**
```toml
[rate_limiting]
enable = false
```

---

## Performance Issues

### Slow Query Performance

**Symptoms:**
- API requests taking >1 second
- Database queries timing out
- High CPU usage

**Diagnostic Steps:**

1. **Enable Slow Query Logging:**
```toml
[database]
slow_query_threshold_ms = 1000
log_statements = true
```

2. **Identify Slow Queries:**
```bash
# Check application logs
grep "slow query" /var/log/llm-cost-ops/app.log

# PostgreSQL slow query log
sudo tail -f /var/log/postgresql/postgresql-15-main.log | grep "duration"
```

3. **Analyze Query Plans:**
```sql
-- PostgreSQL
EXPLAIN ANALYZE
SELECT * FROM usage_records
WHERE organization_id = 'org-123'
  AND timestamp >= NOW() - INTERVAL '30 days';
```

**Solutions:**

1. **Add Indexes:**
```sql
-- Create composite index for common queries
CREATE INDEX idx_usage_org_timestamp
ON usage_records(organization_id, timestamp DESC);

-- Create index for provider filtering
CREATE INDEX idx_usage_provider
ON usage_records(provider);
```

2. **Optimize Queries:**
```sql
-- Use covering index
SELECT id, timestamp, total_cost
FROM usage_records
WHERE organization_id = 'org-123'
  AND timestamp >= '2025-11-01'
ORDER BY timestamp DESC
LIMIT 100;
```

3. **Increase Connection Pool:**
```toml
[database]
pool_size = 30  # Increase from 20
```

4. **Enable Query Caching:**
```toml
[cache]
enable = true
ttl_secs = 300
```

---

### High Memory Usage

**Symptoms:**
- Server using >2GB memory
- OOM (Out of Memory) errors
- Frequent garbage collection

**Diagnostic Steps:**

1. **Check Memory Usage:**
```bash
# Overall memory
free -h

# Process memory
ps aux | grep cost-ops

# Detailed breakdown
top -p $(pgrep cost-ops)
```

2. **Monitor Metrics:**
```bash
# Prometheus metrics
curl http://localhost:9090/metrics | grep memory
```

**Solutions:**

1. **Limit Result Sets:**
```toml
[api]
max_page_size = 100  # Limit to 100 records per page
```

2. **Stream Large Exports:**
```bash
# Use streaming for large exports
cost-ops export --output large-export.csv --stream
```

3. **Adjust Export Limits:**
```toml
[export]
max_export_size = 52428800  # 50 MB (reduce from 100 MB)
```

4. **Increase Available Memory:**
```bash
# Docker
docker run -m 4g llm-cost-ops

# Kubernetes
resources:
  limits:
    memory: 4Gi
```

5. **Optimize Database Queries:**
- Use pagination
- Filter results server-side
- Avoid SELECT *

---

### High CPU Usage

**Symptoms:**
- CPU usage >80%
- Slow response times
- Request timeouts

**Diagnostic Steps:**

1. **Identify CPU-Intensive Processes:**
```bash
# Overall CPU usage
top

# Process-specific
pidstat -u 1 -p $(pgrep cost-ops)
```

2. **Profile Application:**
```bash
# Enable CPU profiling
RUST_LOG=debug cost-ops server --profile
```

**Solutions:**

1. **Increase Worker Threads:**
```toml
[api]
workers = 16  # Increase to match CPU cores
```

2. **Optimize Calculations:**
```bash
# Check for expensive forecasting operations
grep "forecast" /var/log/llm-cost-ops/app.log
```

3. **Disable Unused Features:**
```toml
[forecasting]
enable = false  # Disable if not used

[observability.tracing]
enable = false  # Reduce overhead
```

4. **Scale Horizontally:**
```bash
# Add more instances
kubectl scale deployment llm-cost-ops --replicas=5
```

---

## Memory Issues

### Error: Out of Memory (OOM)

**Error Message:**
```
Error: Cannot allocate memory
Killed
```

**Causes:**
1. Memory leak
2. Large export or query
3. Insufficient system memory
4. Memory limits too low

**Solutions:**

1. **Increase Memory Limits:**
```yaml
# Docker Compose
services:
  cost-ops:
    mem_limit: 4g

# Kubernetes
resources:
  limits:
    memory: 4Gi
```

2. **Enable Memory Limits:**
```toml
[api]
max_request_size = 5242880  # 5 MB

[export]
max_export_size = 52428800  # 50 MB
```

3. **Use Streaming:**
```bash
# Stream large exports
cost-ops export --output data.csv --stream
```

4. **Restart Service Periodically:**
```bash
# Systemd timer for weekly restart
sudo systemctl edit --force --full llm-cost-ops.timer
```

---

## Disk Space Issues

### Error: No Space Left on Device

**Error Message:**
```
Error: No space left on device (os error 28)
```

**Causes:**
1. Log files consuming disk space
2. Export files not cleaned up
3. Database growth
4. Temporary files

**Solutions:**

1. **Check Disk Usage:**
```bash
df -h
du -sh /var/lib/llm-cost-ops/*
du -sh /var/log/llm-cost-ops/*
```

2. **Clean Up Logs:**
```bash
# Rotate logs
sudo logrotate -f /etc/logrotate.d/llm-cost-ops

# Delete old logs
find /var/log/llm-cost-ops -name "*.log.gz" -mtime +30 -delete
```

3. **Clean Up Exports:**
```bash
# Delete old exports
find /var/lib/llm-cost-ops/exports -type f -mtime +7 -delete
```

4. **Enable Automatic Cleanup:**
```toml
[export]
retention_days = 7  # Auto-delete after 7 days

[logging.rotation]
max_backups = 10
compress = true
```

5. **Vacuum Database (PostgreSQL):**
```sql
VACUUM FULL;
REINDEX DATABASE costops;
```

6. **Expand Disk:**
```bash
# Resize volume (cloud provider specific)
# AWS EBS, GCP Persistent Disk, etc.

# Resize partition
sudo resize2fs /dev/sda1
```

---

## Network Connectivity

### Error: DNS Resolution Failed

**Error Message:**
```
Error: failed to lookup address information: Name or service not known
```

**Causes:**
1. DNS server unreachable
2. Incorrect hostname
3. /etc/hosts misconfiguration
4. Network interface down

**Solutions:**

1. **Test DNS:**
```bash
nslookup api.llm-cost-ops.example.com
dig api.llm-cost-ops.example.com
```

2. **Check /etc/resolv.conf:**
```bash
cat /etc/resolv.conf
# Should contain:
# nameserver 8.8.8.8
# nameserver 8.8.4.4
```

3. **Use IP Address:**
```bash
# Bypass DNS
cost-ops --api-url http://192.168.1.10:8080 query
```

4. **Add to /etc/hosts:**
```bash
echo "192.168.1.10 api.llm-cost-ops.example.com" | sudo tee -a /etc/hosts
```

---

### Error: Network Unreachable

**Error Message:**
```
Error: Network is unreachable (os error 101)
```

**Causes:**
1. Network interface down
2. No default route
3. Firewall blocking traffic
4. VPN disconnected

**Solutions:**

1. **Check Network Interface:**
```bash
ip addr show
ip link set eth0 up
```

2. **Check Routing:**
```bash
ip route show
# Add default route if missing
sudo ip route add default via 192.168.1.1
```

3. **Test Connectivity:**
```bash
ping -c 3 8.8.8.8
curl -v http://api.llm-cost-ops.example.com/health
```

---

## SSL/TLS Errors

### Error: Certificate Verification Failed

**Error Message:**
```
Error: SSL certificate problem: unable to get local issuer certificate
```

**Causes:**
1. Self-signed certificate
2. Expired certificate
3. Certificate chain incomplete
4. CA certificate not trusted

**Solutions:**

1. **Verify Certificate:**
```bash
openssl s_client -connect api.example.com:443 -showcerts
```

2. **Install CA Certificate:**
```bash
# Ubuntu/Debian
sudo cp ca-cert.crt /usr/local/share/ca-certificates/
sudo update-ca-certificates

# RHEL/CentOS
sudo cp ca-cert.crt /etc/pki/ca-trust/source/anchors/
sudo update-ca-trust
```

3. **Disable Verification (Development Only):**
```bash
curl -k https://api.example.com/health
```

4. **Update Certificate:**
```bash
# Let's Encrypt
sudo certbot renew

# Manual
sudo cp new-cert.crt /etc/ssl/certs/server.crt
sudo systemctl restart llm-cost-ops
```

---

## Log Analysis

### Enable Debug Logging

```bash
# CLI
cost-ops -vvv query

# Configuration
export RUST_LOG=debug

# Config file
[logging]
level = "debug"
```

### Common Log Patterns

**Authentication Failures:**
```bash
grep "authentication failed" /var/log/llm-cost-ops/app.log
```

**Database Errors:**
```bash
grep "DATABASE_ERROR" /var/log/llm-cost-ops/app.log
```

**Slow Queries:**
```bash
grep "slow query" /var/log/llm-cost-ops/app.log | head -20
```

**Request Errors:**
```bash
grep "status=500" /var/log/llm-cost-ops/app.log
```

### Structured Log Queries (JSON)

```bash
# Extract error messages
jq -r 'select(.level=="ERROR") | .message' /var/log/llm-cost-ops/app.log

# Count errors by type
jq -r 'select(.level=="ERROR") | .error_code' /var/log/llm-cost-ops/app.log | sort | uniq -c

# Find slow requests
jq -r 'select(.duration_ms > 1000) | {path, duration_ms}' /var/log/llm-cost-ops/app.log
```

---

## Debugging Techniques

### Enable Request Tracing

```bash
# Add trace header to requests
curl -H "X-Trace-ID: debug-123" http://localhost:8080/api/v1/costs

# Search logs for trace ID
grep "debug-123" /var/log/llm-cost-ops/app.log
```

### Test API Endpoints

```bash
# Health check
curl -v http://localhost:8080/health

# Submit test usage
curl -X POST http://localhost:8080/api/v1/usage \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "test-org",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_tokens": 100,
    "output_tokens": 50,
    "total_tokens": 150
  }'
```

### Database Query Testing

```sql
-- Test usage query
SELECT * FROM usage_records
WHERE organization_id = 'org-123'
ORDER BY timestamp DESC
LIMIT 10;

-- Check indexes
SELECT schemaname, tablename, indexname
FROM pg_indexes
WHERE tablename = 'usage_records';

-- Analyze table
ANALYZE usage_records;
```

### Performance Profiling

```bash
# CPU profiling
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Memory profiling with valgrind
valgrind --leak-check=full ./target/release/cost-ops server

# Benchmark
ab -n 1000 -c 10 http://localhost:8080/health
```

---

## Getting Help

### Before Requesting Support

1. **Check health status:**
```bash
cost-ops health
curl http://localhost:8080/health
```

2. **Gather version information:**
```bash
cost-ops --version
uname -a
```

3. **Collect logs:**
```bash
# Last 100 lines
tail -100 /var/log/llm-cost-ops/app.log > support-logs.txt

# Full logs (compressed)
tar -czf logs.tar.gz /var/log/llm-cost-ops/
```

4. **Export configuration (redact secrets):**
```bash
cost-ops config show --redact > config-export.toml
```

### Support Channels

**Community:**
- GitHub Issues: https://github.com/llm-cost-ops/llm-cost-ops/issues
- Discord: https://discord.gg/llm-cost-ops
- Forum: https://forum.llm-cost-ops.example.com

**Enterprise:**
- Email: support@llm-cost-ops.example.com
- Slack: #llm-cost-ops-support
- Phone: 1-800-LLM-COST

### Bug Reports

When filing a bug report, include:

1. **Environment:**
   - OS and version
   - LLM Cost Ops version
   - Deployment method (Docker, K8s, binary)

2. **Configuration:**
   - Redacted config file
   - Environment variables (redacted)

3. **Reproduction Steps:**
   - Exact commands or API calls
   - Expected behavior
   - Actual behavior

4. **Logs:**
   - Application logs with timestamps
   - Error messages
   - Stack traces (if available)

5. **Impact:**
   - Frequency of issue
   - Number of users affected
   - Workaround (if known)

---

**See Also:**

- [API Reference](./api-reference.md)
- [CLI Reference](./cli-reference.md)
- [Configuration Reference](./configuration.md)
- [FAQ](./faq.md)
