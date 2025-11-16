# Configuration Reference

**Version:** 1.0.0
**Last Updated:** 2025-11-16

Complete configuration reference for the LLM Cost Ops platform. This document covers all configuration options, environment variables, validation rules, and examples for different deployment environments.

---

## Table of Contents

- [Configuration Overview](#configuration-overview)
- [Configuration File Format](#configuration-file-format)
- [Database Configuration](#database-configuration)
- [API Server Configuration](#api-server-configuration)
- [Authentication Configuration](#authentication-configuration)
- [Logging and Observability](#logging-and-observability)
- [Export and Reporting](#export-and-reporting)
- [Forecasting Configuration](#forecasting-configuration)
- [Rate Limiting](#rate-limiting)
- [Compression](#compression)
- [Email Configuration](#email-configuration)
- [Environment Variables](#environment-variables)
- [Validation Rules](#validation-rules)
- [Environment Examples](#environment-examples)
- [Docker Configuration](#docker-configuration)
- [Kubernetes Configuration](#kubernetes-configuration)

---

## Configuration Overview

### Configuration Precedence

Configuration values are loaded in the following order (later sources override earlier ones):

1. Default values (hardcoded in application)
2. System config file (`/etc/llm-cost-ops/config.toml`)
3. User config file (`~/.config/llm-cost-ops/config.toml`)
4. Local config file (`./cost-ops.toml`)
5. Environment variables
6. Command-line flags

### Configuration File Locations

**System-wide:**
- Linux: `/etc/llm-cost-ops/config.toml`
- macOS: `/Library/Application Support/llm-cost-ops/config.toml`
- Windows: `C:\ProgramData\llm-cost-ops\config.toml`

**User-specific:**
- Linux/macOS: `~/.config/llm-cost-ops/config.toml`
- Windows: `%APPDATA%\llm-cost-ops\config.toml`

**Project-specific:**
- Current directory: `./cost-ops.toml`

### Specifying Config File

**CLI:**
```bash
cost-ops --config /path/to/config.toml server
```

**Environment Variable:**
```bash
export COST_OPS_CONFIG=/path/to/config.toml
```

---

## Configuration File Format

LLM Cost Ops supports TOML, YAML, and JSON configuration files.

### TOML Format (Recommended)

**File:** `config.toml`

```toml
# LLM Cost Ops Configuration

[database]
url = "postgresql://user:pass@localhost/costops"
pool_size = 20
max_lifetime_secs = 3600
idle_timeout_secs = 600
connection_timeout_secs = 30

[api]
bind = "0.0.0.0"
port = 8080
workers = 8
request_timeout_secs = 30
max_request_size = 10485760  # 10 MB

[auth]
jwt_secret = "your-secret-key-here"
jwt_expiry_secs = 3600
refresh_token_expiry_secs = 2592000  # 30 days
api_key_hash_iterations = 10000

[logging]
level = "info"
format = "json"
file = "/var/log/llm-cost-ops/app.log"
max_size_mb = 100
max_backups = 10

[observability]
enable_metrics = true
metrics_port = 9090
enable_tracing = true
tracing_endpoint = "http://localhost:4317"
service_name = "llm-cost-ops"

[export]
output_dir = "/var/lib/llm-cost-ops/exports"
max_export_size = 104857600  # 100 MB
enable_compression = true

[export.email]
smtp_host = "smtp.example.com"
smtp_port = 587
smtp_username = "reports@example.com"
smtp_password = "password"
use_starttls = true
from_email = "reports@example.com"
from_name = "LLM Cost Ops"

[forecasting]
enable = true
default_horizon_days = 30
default_model = "exponential_smoothing"
confidence_level = 0.95

[rate_limiting]
enable = true
requests_per_hour = 10000
burst_size = 200
```

### YAML Format

**File:** `config.yaml`

```yaml
database:
  url: postgresql://user:pass@localhost/costops
  pool_size: 20
  max_lifetime_secs: 3600

api:
  bind: 0.0.0.0
  port: 8080
  workers: 8

auth:
  jwt_secret: your-secret-key-here
  jwt_expiry_secs: 3600

logging:
  level: info
  format: json
```

### JSON Format

**File:** `config.json`

```json
{
  "database": {
    "url": "postgresql://user:pass@localhost/costops",
    "pool_size": 20
  },
  "api": {
    "bind": "0.0.0.0",
    "port": 8080
  },
  "logging": {
    "level": "info",
    "format": "json"
  }
}
```

---

## Database Configuration

### PostgreSQL Configuration

**Production-recommended configuration:**

```toml
[database]
# Database connection URL
# Format: postgresql://username:password@host:port/database?options
url = "postgresql://costops:password@localhost:5432/costops?sslmode=require"

# Connection pool size
# Recommended: 2-4x CPU cores
pool_size = 20

# Maximum connection lifetime (seconds)
# Recommended: 1-2 hours
max_lifetime_secs = 3600

# Idle connection timeout (seconds)
# Connections idle longer than this are closed
idle_timeout_secs = 600

# Connection timeout (seconds)
# How long to wait for a connection from the pool
connection_timeout_secs = 30

# Acquire timeout (seconds)
# How long to wait to acquire a connection
acquire_timeout_secs = 5

# Enable statement logging
log_statements = false

# Enable slow query logging (milliseconds)
# Queries taking longer than this are logged
slow_query_threshold_ms = 1000

# Enable connection retry
enable_retry = true
max_retries = 3
retry_delay_ms = 1000
```

**Connection URL Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `sslmode` | SSL mode (disable, allow, prefer, require, verify-ca, verify-full) | prefer |
| `application_name` | Application name for pg_stat_activity | llm-cost-ops |
| `connect_timeout` | Connection timeout (seconds) | 30 |
| `tcp_user_timeout` | TCP timeout (milliseconds) | 0 |
| `options` | Additional server options | |

**Example URLs:**

```toml
# Local development
url = "postgresql://localhost/costops"

# Production with SSL
url = "postgresql://user:pass@db.example.com/costops?sslmode=verify-full"

# Unix socket
url = "postgresql:///costops?host=/var/run/postgresql"

# With connection pooling (PgBouncer)
url = "postgresql://user:pass@pgbouncer:6432/costops?pool_mode=transaction"
```

### SQLite Configuration

**Development/testing configuration:**

```toml
[database]
# SQLite database file path
url = "sqlite:///var/lib/llm-cost-ops/cost-ops.db"

# Journal mode: DELETE, TRUNCATE, PERSIST, MEMORY, WAL, OFF
# WAL recommended for production use
journal_mode = "WAL"

# Synchronous mode: OFF, NORMAL, FULL, EXTRA
# NORMAL is a good balance for WAL mode
synchronous = "NORMAL"

# Cache size (pages, negative = KB)
cache_size = -64000  # 64 MB

# Busy timeout (milliseconds)
busy_timeout_ms = 5000

# Enable foreign keys
foreign_keys = true

# Auto vacuum mode
auto_vacuum = "INCREMENTAL"
```

**In-Memory SQLite (Testing):**

```toml
[database]
url = "sqlite::memory:"
```

### Database Migration Settings

```toml
[database.migrations]
# Automatically run migrations on startup
auto_migrate = false

# Migrations directory
migrations_dir = "./migrations"

# Enable migration locking
enable_locking = true

# Migration timeout (seconds)
timeout_secs = 300
```

---

## API Server Configuration

```toml
[api]
# Bind address
# Use "0.0.0.0" for all interfaces
# Use "127.0.0.1" for localhost only
bind = "0.0.0.0"

# Port number
port = 8080

# Worker threads
# Auto-detect: 0 (uses CPU count)
# Recommended: CPU count * 2
workers = 0

# Request timeout (seconds)
request_timeout_secs = 30

# Maximum request body size (bytes)
# Default: 10 MB
max_request_size = 10485760

# Keep-alive timeout (seconds)
keep_alive_secs = 75

# Enable graceful shutdown
graceful_shutdown = true

# Shutdown timeout (seconds)
shutdown_timeout_secs = 30

# Enable HTTP/2
enable_http2 = true

# Enable compression
enable_compression = true

# Compression level (1-9)
compression_level = 6

# CORS configuration
[api.cors]
# Allowed origins (["*"] for all)
allowed_origins = ["https://app.example.com"]

# Allowed methods
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]

# Allowed headers
allowed_headers = ["Authorization", "Content-Type", "X-Request-ID"]

# Exposed headers
exposed_headers = ["X-RateLimit-Limit", "X-RateLimit-Remaining"]

# Allow credentials
allow_credentials = true

# Max age (seconds)
max_age_secs = 3600

# TLS/SSL configuration
[api.tls]
# Enable TLS
enable = false

# Certificate file path
cert_file = "/etc/ssl/certs/server.crt"

# Private key file path
key_file = "/etc/ssl/private/server.key"

# Client certificate verification
verify_client = false

# CA certificate file
ca_file = "/etc/ssl/certs/ca.crt"
```

---

## Authentication Configuration

```toml
[auth]
# JWT secret key for signing tokens
# IMPORTANT: Use a strong, random secret in production
# Generate with: openssl rand -base64 32
jwt_secret = "your-256-bit-secret-key-here"

# JWT token expiry (seconds)
# Default: 1 hour
jwt_expiry_secs = 3600

# Refresh token expiry (seconds)
# Default: 30 days
refresh_token_expiry_secs = 2592000

# JWT issuer
jwt_issuer = "llm-cost-ops"

# JWT audience
jwt_audience = "llm-cost-ops-api"

# API key configuration
[auth.api_key]
# Hash algorithm: sha256, sha512
hash_algorithm = "sha256"

# Hash iterations for PBKDF2
hash_iterations = 10000

# API key prefix (for key identification)
key_prefix = "sk_"

# API key expiry (seconds, 0 = no expiry)
default_expiry_secs = 0

# Minimum key length
min_key_length = 32

# RBAC configuration
[auth.rbac]
# Enable RBAC
enable = true

# Default role for new users
default_role = "viewer"

# Enable permission caching
cache_permissions = true

# Permission cache TTL (seconds)
cache_ttl_secs = 300

# OAuth 2.0 / SSO configuration
[auth.oauth]
# Enable OAuth
enable = false

# Provider: generic, google, github, okta, azure_ad
provider = "generic"

# Client ID
client_id = ""

# Client secret
client_secret = ""

# Authorization URL
auth_url = "https://provider.com/oauth/authorize"

# Token URL
token_url = "https://provider.com/oauth/token"

# Redirect URI
redirect_uri = "https://api.example.com/auth/callback"

# Scopes
scopes = ["openid", "email", "profile"]

# SAML SSO configuration
[auth.saml]
# Enable SAML
enable = false

# Entity ID
entity_id = "https://api.example.com/saml/metadata"

# Single Sign-On URL
sso_url = "https://idp.example.com/sso"

# Single Logout URL
slo_url = "https://idp.example.com/slo"

# X.509 certificate
certificate = "/etc/saml/idp-cert.pem"

# Attribute mappings
email_attribute = "email"
name_attribute = "displayName"
```

---

## Logging and Observability

### Logging Configuration

```toml
[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log format: text, json
format = "json"

# Log output: stdout, stderr, file
output = "stdout"

# Log file path (when output = "file")
file = "/var/log/llm-cost-ops/app.log"

# File rotation
[logging.rotation]
# Enable log rotation
enable = true

# Maximum file size (MB)
max_size_mb = 100

# Maximum number of backup files
max_backups = 10

# Maximum age (days)
max_age_days = 30

# Compress rotated files
compress = true

# Log filtering
[logging.filters]
# Filter by module
modules = ["llm_cost_ops=info", "sqlx=warn", "hyper=warn"]

# Exclude patterns
exclude = ["health_check", "metrics"]
```

### Observability Configuration

```toml
[observability]
# Service name for tracing/metrics
service_name = "llm-cost-ops"

# Environment name
environment = "production"

# Instance ID (auto-generated if not set)
instance_id = ""

# Metrics configuration
[observability.metrics]
# Enable Prometheus metrics
enable = true

# Metrics bind address
bind = "0.0.0.0"

# Metrics port
port = 9090

# Metrics path
path = "/metrics"

# Include default process metrics
include_process_metrics = true

# Include Go runtime metrics
include_runtime_metrics = true

# Metrics prefix
prefix = "llm_cost_ops"

# Tracing configuration
[observability.tracing]
# Enable distributed tracing
enable = true

# Tracing backend: otlp, jaeger, zipkin
backend = "otlp"

# OTLP endpoint (for OpenTelemetry)
otlp_endpoint = "http://localhost:4317"

# Jaeger endpoint
jaeger_endpoint = "http://localhost:14268/api/traces"

# Zipkin endpoint
zipkin_endpoint = "http://localhost:9411/api/v2/spans"

# Sampling rate (0.0 - 1.0)
# 1.0 = trace all requests
# 0.1 = trace 10% of requests
sampling_rate = 1.0

# Enable trace context propagation
propagate_context = true

# Health check configuration
[observability.health]
# Enable health endpoints
enable = true

# Health check interval (seconds)
check_interval_secs = 30

# Health check timeout (seconds)
check_timeout_secs = 5

# Components to check
components = ["database", "redis", "api"]
```

---

## Export and Reporting

```toml
[export]
# Output directory for exports
output_dir = "/var/lib/llm-cost-ops/exports"

# Maximum export size (bytes)
# Default: 100 MB
max_export_size = 104857600

# Enable compression
enable_compression = true

# Compression format: gzip, zstd
compression_format = "gzip"

# Compression level (1-9 for gzip, 1-22 for zstd)
compression_level = 6

# Export retention (days)
# Automatically delete exports older than this
retention_days = 7

# Export formats configuration
[export.formats]
# Enable CSV export
csv = true

# Enable JSON export
json = true

# Enable JSONL export
jsonl = true

# Enable Excel export
xlsx = true

# Enable Parquet export
parquet = false

# CSV configuration
[export.formats.csv]
# Delimiter character
delimiter = ","

# Quote character
quote = "\""

# Escape character
escape = "\\"

# Include header row
include_header = true

# XLSX configuration
[export.formats.xlsx]
# Maximum rows per sheet
max_rows_per_sheet = 1048576

# Include formulas
include_formulas = false

# Email delivery configuration
[export.email]
# SMTP host
smtp_host = "smtp.example.com"

# SMTP port
smtp_port = 587

# SMTP username
smtp_username = "reports@example.com"

# SMTP password (use environment variable in production)
smtp_password = "${SMTP_PASSWORD}"

# Enable TLS
use_tls = false

# Enable STARTTLS
use_starttls = true

# From email address
from_email = "reports@example.com"

# From name
from_name = "LLM Cost Ops Reports"

# Default recipients
default_recipients = ["finance@example.com"]

# Email template
template_name = "default"

# Connection timeout (seconds)
timeout_secs = 30

# Storage configuration (S3, GCS, Azure Blob)
[export.storage]
# Storage backend: local, s3, gcs, azure
backend = "local"

# S3 configuration
[export.storage.s3]
bucket = "llm-cost-ops-exports"
region = "us-east-1"
access_key_id = "${AWS_ACCESS_KEY_ID}"
secret_access_key = "${AWS_SECRET_ACCESS_KEY}"
endpoint = ""  # For S3-compatible storage

# GCS configuration
[export.storage.gcs]
bucket = "llm-cost-ops-exports"
project_id = "my-project"
credentials_file = "/etc/gcs/credentials.json"

# Azure Blob configuration
[export.storage.azure]
account_name = "costopsexports"
account_key = "${AZURE_STORAGE_KEY}"
container = "exports"

# Report templates
[export.templates]
# Templates directory
templates_dir = "/etc/llm-cost-ops/templates"

# Default template
default_template = "executive_summary"

# Scheduled reports
[[export.scheduled_reports]]
name = "Daily Cost Summary"
type = "cost_summary"
format = "pdf"
schedule = "0 9 * * *"  # Daily at 9 AM
timezone = "America/New_York"
enabled = true
recipients = ["finance@example.com"]

[[export.scheduled_reports]]
name = "Weekly Usage Report"
type = "usage_analysis"
format = "xlsx"
schedule = "0 9 * * 1"  # Mondays at 9 AM
timezone = "America/New_York"
enabled = true
recipients = ["ops@example.com"]
```

---

## Forecasting Configuration

```toml
[forecasting]
# Enable forecasting
enable = true

# Default forecast horizon (days)
default_horizon_days = 30

# Maximum forecast horizon (days)
max_horizon_days = 90

# Default forecast model
# Options: linear, moving_average, exponential_smoothing, prophet
default_model = "exponential_smoothing"

# Default confidence level
# Options: 0.80, 0.90, 0.95, 0.99
confidence_level = 0.95

# Minimum historical data points
min_data_points = 30

# Enable automatic retraining
auto_retrain = true

# Retrain interval (hours)
retrain_interval_hours = 24

# Model configuration
[forecasting.models]
# Linear regression
[forecasting.models.linear]
enable = true
regularization = "none"  # none, l1, l2, elastic_net

# Moving average
[forecasting.models.moving_average]
enable = true
window_size = 7

# Exponential smoothing
[forecasting.models.exponential_smoothing]
enable = true
alpha = 0.3  # Level smoothing
beta = 0.1   # Trend smoothing
gamma = 0.1  # Seasonal smoothing
seasonal_periods = 7  # Weekly seasonality

# Facebook Prophet
[forecasting.models.prophet]
enable = false
yearly_seasonality = true
weekly_seasonality = true
daily_seasonality = false
changepoint_prior_scale = 0.05

# Anomaly detection
[forecasting.anomaly_detection]
# Enable anomaly detection
enable = true

# Detection method: zscore, iqr, prophet
method = "zscore"

# Sensitivity: low, medium, high
sensitivity = "medium"

# Z-score threshold
zscore_threshold = 3.0

# IQR multiplier
iqr_multiplier = 1.5

# Minimum anomaly severity
min_severity = "medium"

# Budget forecasting
[forecasting.budget]
# Enable budget forecasting
enable = true

# Default budget period: daily, weekly, monthly, yearly
default_period = "monthly"

# Alert thresholds (percentage)
alert_thresholds = [0.50, 0.75, 0.90]

# Alert cooldown (hours)
alert_cooldown_hours = 24
```

---

## Rate Limiting

```toml
[rate_limiting]
# Enable rate limiting
enable = true

# Rate limit algorithm: token_bucket, leaky_bucket, fixed_window, sliding_window
algorithm = "token_bucket"

# Requests per hour (global default)
requests_per_hour = 10000

# Burst size
burst_size = 200

# Rate limit by key: ip, user, api_key, organization
limit_by = "api_key"

# Enable rate limit headers
include_headers = true

# Per-tier limits
[rate_limiting.tiers]
[rate_limiting.tiers.free]
requests_per_hour = 100
burst_size = 10

[rate_limiting.tiers.developer]
requests_per_hour = 1000
burst_size = 50

[rate_limiting.tiers.professional]
requests_per_hour = 10000
burst_size = 200

[rate_limiting.tiers.enterprise]
requests_per_hour = 100000
burst_size = 500

# Per-endpoint limits
[[rate_limiting.endpoints]]
path = "/api/v1/usage"
method = "POST"
requests_per_hour = 50000
burst_size = 1000

[[rate_limiting.endpoints]]
path = "/api/v1/export"
method = "POST"
requests_per_hour = 100
burst_size = 10

# Redis configuration for distributed rate limiting
[rate_limiting.redis]
url = "redis://localhost:6379/0"
pool_size = 10
```

---

## Compression

```toml
[compression]
# Enable response compression
enable = true

# Compression algorithm: gzip, brotli, zstd
algorithm = "gzip"

# Compression level (1-9 for gzip, 1-11 for brotli, 1-22 for zstd)
level = 6

# Minimum response size to compress (bytes)
min_size = 1024

# Compress types (MIME types)
compress_types = [
  "application/json",
  "application/xml",
  "text/plain",
  "text/html",
  "text/css",
  "text/javascript",
  "application/javascript"
]
```

---

## Email Configuration

```toml
[email]
# Email provider: smtp, sendgrid, mailgun, ses
provider = "smtp"

# SMTP configuration
[email.smtp]
host = "smtp.example.com"
port = 587
username = "user@example.com"
password = "${SMTP_PASSWORD}"
use_tls = false
use_starttls = true
timeout_secs = 30

# SendGrid configuration
[email.sendgrid]
api_key = "${SENDGRID_API_KEY}"

# Mailgun configuration
[email.mailgun]
api_key = "${MAILGUN_API_KEY}"
domain = "example.com"

# AWS SES configuration
[email.ses]
region = "us-east-1"
access_key_id = "${AWS_ACCESS_KEY_ID}"
secret_access_key = "${AWS_SECRET_ACCESS_KEY}"

# Email defaults
[email.defaults]
from_email = "noreply@example.com"
from_name = "LLM Cost Ops"
reply_to = "support@example.com"
```

---

## Environment Variables

All configuration values can be set via environment variables using the format `COST_OPS_<SECTION>_<KEY>`.

### Examples

```bash
# Database
export COST_OPS_DATABASE_URL="postgresql://localhost/costops"
export COST_OPS_DATABASE_POOL_SIZE=20

# API
export COST_OPS_API_PORT=8080
export COST_OPS_API_WORKERS=8

# Authentication
export COST_OPS_AUTH_JWT_SECRET="your-secret-key"
export COST_OPS_AUTH_JWT_EXPIRY_SECS=3600

# Logging
export COST_OPS_LOGGING_LEVEL=info
export COST_OPS_LOGGING_FORMAT=json

# Observability
export COST_OPS_OBSERVABILITY_METRICS_ENABLE=true
export COST_OPS_OBSERVABILITY_TRACING_OTLP_ENDPOINT="http://localhost:4317"
```

### Sensitive Values

**Never commit sensitive values to version control.** Use environment variables or secrets management:

```bash
# Use .env file (not committed)
export COST_OPS_AUTH_JWT_SECRET=$(cat /run/secrets/jwt_secret)
export COST_OPS_DATABASE_URL=$(cat /run/secrets/database_url)
export SMTP_PASSWORD=$(cat /run/secrets/smtp_password)
```

### Variable Substitution in Config

Config files support environment variable substitution:

```toml
[database]
url = "${DATABASE_URL}"

[auth]
jwt_secret = "${JWT_SECRET}"

[export.email]
smtp_password = "${SMTP_PASSWORD}"
```

---

## Validation Rules

### Required Fields

The following fields are **required**:

- `database.url` - Database connection URL
- `auth.jwt_secret` - JWT secret key (production)

### Value Constraints

| Field | Constraint |
|-------|-----------|
| `database.pool_size` | 1 - 100 |
| `api.port` | 1 - 65535 |
| `api.workers` | 0 - 1024 (0 = auto) |
| `auth.jwt_expiry_secs` | 60 - 86400 |
| `logging.level` | trace, debug, info, warn, error |
| `forecasting.confidence_level` | 0.0 - 1.0 |
| `rate_limiting.requests_per_hour` | 1 - 1000000 |

### URL Formats

**Database URLs:**
```
postgresql://user:pass@host:port/db?options
sqlite:///path/to/db.sqlite
sqlite::memory:
```

**Redis URLs:**
```
redis://localhost:6379/0
redis://:password@localhost:6379/0
rediss://localhost:6380/0  # TLS
```

---

## Environment Examples

### Development

```toml
# config.dev.toml

[database]
url = "sqlite:///tmp/cost-ops-dev.db"
journal_mode = "WAL"

[api]
bind = "127.0.0.1"
port = 8080
workers = 2

[auth]
jwt_secret = "dev-secret-not-for-production"
jwt_expiry_secs = 86400  # 24 hours for convenience

[logging]
level = "debug"
format = "text"
output = "stdout"

[observability]
enable_metrics = false
enable_tracing = false

[rate_limiting]
enable = false
```

### Staging

```toml
# config.staging.toml

[database]
url = "${DATABASE_URL}"
pool_size = 10
max_lifetime_secs = 1800

[api]
bind = "0.0.0.0"
port = 8080
workers = 4
enable_http2 = true

[api.cors]
allowed_origins = ["https://staging.example.com"]

[auth]
jwt_secret = "${JWT_SECRET}"
jwt_expiry_secs = 3600

[logging]
level = "info"
format = "json"
output = "stdout"

[observability]
enable_metrics = true
metrics_port = 9090
enable_tracing = true
otlp_endpoint = "http://jaeger:4317"

[rate_limiting]
enable = true
requests_per_hour = 5000
```

### Production

```toml
# config.prod.toml

[database]
url = "${DATABASE_URL}"
pool_size = 20
max_lifetime_secs = 3600
idle_timeout_secs = 600
connection_timeout_secs = 30
log_statements = false
slow_query_threshold_ms = 1000

[api]
bind = "0.0.0.0"
port = 8080
workers = 16
request_timeout_secs = 30
max_request_size = 10485760
keep_alive_secs = 75
graceful_shutdown = true
enable_http2 = true
enable_compression = true

[api.cors]
allowed_origins = ["https://app.example.com"]
allow_credentials = true

[api.tls]
enable = true
cert_file = "/etc/ssl/certs/server.crt"
key_file = "/etc/ssl/private/server.key"

[auth]
jwt_secret = "${JWT_SECRET}"
jwt_expiry_secs = 3600
refresh_token_expiry_secs = 2592000

[auth.rbac]
enable = true
cache_permissions = true
cache_ttl_secs = 300

[logging]
level = "info"
format = "json"
output = "file"
file = "/var/log/llm-cost-ops/app.log"

[logging.rotation]
enable = true
max_size_mb = 100
max_backups = 30
compress = true

[observability]
service_name = "llm-cost-ops"
environment = "production"

[observability.metrics]
enable = true
port = 9090
include_process_metrics = true

[observability.tracing]
enable = true
backend = "otlp"
otlp_endpoint = "http://otel-collector:4317"
sampling_rate = 0.1  # 10% sampling

[export]
output_dir = "/var/lib/llm-cost-ops/exports"
max_export_size = 104857600
enable_compression = true
retention_days = 7

[export.storage]
backend = "s3"

[export.storage.s3]
bucket = "llm-cost-ops-exports-prod"
region = "us-east-1"

[forecasting]
enable = true
default_horizon_days = 30
auto_retrain = true

[rate_limiting]
enable = true
requests_per_hour = 100000
burst_size = 500
```

---

## Docker Configuration

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  cost-ops:
    image: llm-cost-ops:latest
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - COST_OPS_DATABASE_URL=postgresql://postgres:password@db:5432/costops
      - COST_OPS_AUTH_JWT_SECRET=${JWT_SECRET}
      - COST_OPS_LOGGING_LEVEL=info
      - COST_OPS_OBSERVABILITY_METRICS_ENABLE=true
    volumes:
      - ./config.toml:/etc/llm-cost-ops/config.toml:ro
      - exports:/var/lib/llm-cost-ops/exports
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=costops
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres-data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  postgres-data:
  exports:
```

### Docker Environment File

```bash
# .env
JWT_SECRET=your-secret-key-here
DATABASE_URL=postgresql://postgres:password@db:5432/costops
SMTP_PASSWORD=smtp-password
```

---

## Kubernetes Configuration

### ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: llm-cost-ops-config
  namespace: llm-cost-ops
data:
  config.toml: |
    [database]
    pool_size = 20
    max_lifetime_secs = 3600

    [api]
    bind = "0.0.0.0"
    port = 8080
    workers = 8

    [logging]
    level = "info"
    format = "json"

    [observability.metrics]
    enable = true
    port = 9090
```

### Secret

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: llm-cost-ops-secrets
  namespace: llm-cost-ops
type: Opaque
stringData:
  database-url: postgresql://user:pass@postgres:5432/costops
  jwt-secret: your-secret-key-here
  smtp-password: smtp-password
```

### Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-cost-ops
  namespace: llm-cost-ops
spec:
  replicas: 3
  selector:
    matchLabels:
      app: llm-cost-ops
  template:
    metadata:
      labels:
        app: llm-cost-ops
    spec:
      containers:
      - name: cost-ops
        image: llm-cost-ops:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: COST_OPS_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: llm-cost-ops-secrets
              key: database-url
        - name: COST_OPS_AUTH_JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: llm-cost-ops-secrets
              key: jwt-secret
        volumeMounts:
        - name: config
          mountPath: /etc/llm-cost-ops
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: llm-cost-ops-config
```

---

**See Also:**

- [API Reference](./api-reference.md)
- [CLI Reference](./cli-reference.md)
- [Troubleshooting Guide](./troubleshooting.md)
