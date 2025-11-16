# Glossary

**Version:** 1.0.0
**Last Updated:** 2025-11-16

Complete glossary of terms used in the LLM Cost Ops platform. Terms are organized alphabetically with clear definitions, related terms, and examples where helpful.

---

## A

### Aggregation
The process of combining multiple cost or usage records into summary statistics.

**Example:** Aggregating daily costs by provider to see monthly totals per provider.

**Related Terms:** [Cost Summary](#cost-summary), [Group By](#group-by)

---

### Anomaly Detection
Automated identification of unusual patterns in cost or usage data that deviate significantly from expected behavior.

**Methods:**
- Z-score analysis
- IQR (Interquartile Range)
- Statistical process control

**Example:** Detecting when daily costs suddenly spike to 3x the normal amount.

**Related Terms:** [Forecast](#forecast), [Alert](#alert), [Threshold](#threshold)

---

### API (Application Programming Interface)
A set of HTTP endpoints that allow programmatic access to the LLM Cost Ops platform.

**Endpoints:**
- `/api/v1/usage` - Usage tracking
- `/api/v1/costs` - Cost queries
- `/api/v1/pricing` - Pricing management

**Related Terms:** [REST API](#rest-api), [SDK](#sdk), [Authentication](#authentication)

---

### API Key
A secret token used to authenticate requests to the API.

**Format:** `sk_{environment}_{random_string}`
- `sk_test_` - Development/testing keys
- `sk_live_` - Production keys
- `sk_admin_` - Administrative keys

**Example:** `sk_live_a1b2c3d4e5f6g7h8i9j0`

**Related Terms:** [Authentication](#authentication), [JWT](#jwt), [Bearer Token](#bearer-token)

---

### Audit Log
A chronological record of all actions performed in the system, used for security, compliance, and troubleshooting.

**Captured Information:**
- User/API key performing action
- Action type (create, update, delete, read)
- Timestamp
- Resource affected
- IP address
- Request/response data

**Example:**
```json
{
  "timestamp": "2025-01-15T10:00:00Z",
  "user_id": "user-123",
  "action": "usage.create",
  "resource_id": "usage-abc",
  "ip_address": "192.168.1.1"
}
```

**Related Terms:** [Compliance](#compliance), [GDPR](#gdpr), [SOC2](#soc2)

---

## B

### Backoff
A retry strategy where the delay between retry attempts increases exponentially.

**Example:** First retry after 1s, second after 2s, third after 4s, etc.

**Related Terms:** [Retry](#retry), [Rate Limiting](#rate-limiting)

---

### Batch Operation
Processing multiple records in a single API request to improve performance and reduce overhead.

**Example:**
```python
# Batch create multiple usage records
client.usage.batch_create([
    {"provider": "openai", ...},
    {"provider": "anthropic", ...}
])
```

**Related Terms:** [Pagination](#pagination), [Performance](#performance)

---

### Bearer Token
An authentication method where the token is included in the Authorization header.

**Format:** `Authorization: Bearer <token>`

**Example:**
```bash
curl -H "Authorization: Bearer sk_live_abc123" \
  https://api.example.com/api/v1/usage
```

**Related Terms:** [API Key](#api-key), [JWT](#jwt), [Authentication](#authentication)

---

### Budget
A predefined spending limit for an organization or project over a specific time period.

**Components:**
- Limit amount (e.g., $10,000)
- Period (daily, weekly, monthly, quarterly)
- Alert thresholds (e.g., 80%, 90%, 100%)

**Example:** Monthly budget of $5,000 with alerts at 80% ($4,000) and 95% ($4,750).

**Related Terms:** [Alert](#alert), [Forecast](#forecast), [Cost Control](#cost-control)

---

## C

### Cached Tokens
Previously processed tokens that are reused from cache, typically at a reduced cost.

**Example:** When using Anthropic's prompt caching, repeated portions of the prompt are charged at a discounted rate.

**Pricing Impact:** Often 90% discount compared to regular input tokens.

**Related Terms:** [Input Tokens](#input-tokens), [Pricing Structure](#pricing-structure)

---

### CLI (Command-Line Interface)
A terminal-based interface for interacting with LLM Cost Ops.

**Binary Name:** `cost-ops`

**Common Commands:**
```bash
cost-ops ingest --file usage.json
cost-ops query --range last-7-days
cost-ops export --output costs.csv
```

**Related Terms:** [API](#api), [SDK](#sdk)

---

### Completion Tokens
The number of tokens in the model's response (output).

**Also Known As:** Output tokens, generated tokens

**Pricing:** Usually higher than input tokens (e.g., 3x for GPT-4).

**Related Terms:** [Input Tokens](#input-tokens), [Total Tokens](#total-tokens), [Token](#token)

---

### Compliance
Adherence to regulatory requirements and industry standards for data handling and security.

**Supported Frameworks:**
- GDPR (General Data Protection Regulation)
- SOC2 (Service Organization Control 2)
- HIPAA (Health Insurance Portability and Accountability Act)
- PCI DSS (Payment Card Industry Data Security Standard)

**Related Terms:** [Audit Log](#audit-log), [GDPR](#gdpr), [SOC2](#soc2)

---

### Cost Calculation
The process of computing the monetary cost of LLM usage based on token counts and pricing tables.

**Formula:**
```
Input Cost = (Prompt Tokens / 1,000,000) × Input Price per Million
Output Cost = (Completion Tokens / 1,000,000) × Output Price per Million
Total Cost = Input Cost + Output Cost
```

**Related Terms:** [Pricing Structure](#pricing-structure), [Token](#token), [Usage Record](#usage-record)

---

### Cost Record
A database record representing the calculated cost for a usage record.

**Fields:**
- Usage ID (foreign key)
- Input cost
- Output cost
- Total cost
- Currency
- Timestamp

**Related Terms:** [Usage Record](#usage-record), [Pricing Table](#pricing-table)

---

### Cost Summary
An aggregated view of costs over a time period, often grouped by provider, model, organization, or project.

**Example:**
```
Total Cost: $1,234.56
Total Requests: 10,000
Average Cost per Request: $0.123

By Provider:
- OpenAI: $800.00
- Anthropic: $434.56
```

**Related Terms:** [Aggregation](#aggregation), [Report](#report)

---

## D

### Database Migration
The process of updating the database schema to a new version.

**Tool:** SQLx migrations

**Commands:**
```bash
cost-ops migrate run    # Apply migrations
cost-ops migrate revert # Undo last migration
cost-ops migrate status # Check status
```

**Related Terms:** [Version](#version), [Schema](#schema)

---

## E

### Export
The process of extracting data from the system in a specific format.

**Supported Formats:**
- CSV (Comma-Separated Values)
- JSON (JavaScript Object Notation)
- Excel (XLSX)
- JSON Lines (JSONL)
- PDF (for reports)

**Example:**
```bash
cost-ops export --output costs.csv --format csv
```

**Related Terms:** [Report](#report), [Format](#format)

---

## F

### Forecast
A prediction of future costs or usage based on historical data and statistical models.

**Models:**
- Linear regression
- Moving average
- Exponential smoothing
- ARIMA (future)

**Output:**
- Predicted values
- Confidence intervals
- Trend direction

**Example:** "Based on current trends, we predict $5,000 in costs next month (±$500)."

**Related Terms:** [Anomaly Detection](#anomaly-detection), [Trend](#trend)

---

## G

### GDPR (General Data Protection Regulation)
European Union regulation on data protection and privacy.

**Key Requirements:**
- Right to access
- Right to deletion (right to be forgotten)
- Data portability
- Consent management
- Breach notification

**Related Terms:** [Compliance](#compliance), [PII](#pii), [Data Retention](#data-retention)

---

### Group By
Aggregating data by one or more dimensions.

**Common Dimensions:**
- Provider (openai, anthropic, etc.)
- Model (gpt-4, claude-3, etc.)
- Organization
- Project
- Time period (day, week, month)

**Example:**
```bash
cost-ops query --group-by provider,model
```

**Related Terms:** [Aggregation](#aggregation), [Cost Summary](#cost-summary)

---

## H

### Health Check
An HTTP endpoint that indicates system health status.

**Endpoints:**
- `/health` - Overall health
- `/health/ready` - Readiness probe
- `/health/live` - Liveness probe

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-01-15T10:00:00Z",
  "checks": {
    "database": "ok",
    "redis": "ok"
  }
}
```

**Related Terms:** [Monitoring](#monitoring), [Observability](#observability)

---

### Horizon
The time period into the future for which a forecast is generated.

**Example:** A 30-day horizon means predicting costs for the next 30 days.

**Related Terms:** [Forecast](#forecast), [Prediction](#prediction)

---

## I

### Idempotency
The property that performing an operation multiple times has the same effect as performing it once.

**Example:** Creating a usage record with the same ID twice will not create duplicate records.

**Implementation:** Use unique IDs (UUIDs) for all records.

**Related Terms:** [UUID](#uuid), [Retry](#retry)

---

### Ingestion
The process of importing usage data into the system.

**Methods:**
- File upload (JSON, CSV)
- API endpoint
- Streaming
- Webhooks

**Example:**
```bash
cost-ops ingest --file usage.json
```

**Related Terms:** [Usage Record](#usage-record), [Batch Operation](#batch-operation)

---

### Input Tokens
The number of tokens in the prompt or request sent to the LLM.

**Also Known As:** Prompt tokens

**Components:**
- System instructions
- User message
- Context/history
- Examples

**Related Terms:** [Completion Tokens](#completion-tokens), [Token](#token), [Cached Tokens](#cached-tokens)

---

## J

### JSON (JavaScript Object Notation)
A lightweight data interchange format.

**Example:**
```json
{
  "provider": "openai",
  "model": "gpt-4-turbo",
  "prompt_tokens": 1000,
  "completion_tokens": 500
}
```

**Related Terms:** [Export](#export), [API](#api)

---

### JWT (JSON Web Token)
A compact, URL-safe token format used for authentication and authorization.

**Structure:** `header.payload.signature`

**Use Cases:**
- User authentication
- Session management
- API authorization

**Example:**
```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyLTEyMyIsImV4cCI6MTY0MDk5NTIwMH0.abc123xyz
```

**Related Terms:** [Authentication](#authentication), [API Key](#api-key)

---

## K

### Kubernetes (K8s)
Container orchestration platform for deploying and managing the LLM Cost Ops system.

**Resources:**
- Deployment
- Service
- ConfigMap
- Secret
- Ingress
- HorizontalPodAutoscaler

**Related Terms:** [Deployment](#deployment), [Helm](#helm)

---

## L

### Latency
The time taken for an LLM API request to complete.

**Measured In:** Milliseconds (ms)

**Tracked For:**
- Performance monitoring
- SLA compliance
- Cost-performance trade-offs

**Example:** A request taking 1,500ms has a latency of 1.5 seconds.

**Related Terms:** [Performance](#performance), [Metrics](#metrics)

---

### LLM (Large Language Model)
An AI model trained on large amounts of text data.

**Supported Providers:**
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude)
- Google (Gemini, PaLM)
- Azure OpenAI
- AWS Bedrock
- Cohere
- Mistral

**Related Terms:** [Provider](#provider), [Model](#model)

---

## M

### Metadata
Additional contextual information attached to usage records.

**Common Fields:**
- Request ID
- Customer ID
- Endpoint
- User agent
- Geographic region

**Example:**
```json
{
  "metadata": {
    "request_id": "req-abc123",
    "customer_id": "cust-xyz",
    "endpoint": "/chat/completions"
  }
}
```

**Related Terms:** [Usage Record](#usage-record), [Tags](#tags)

---

### Metrics
Quantitative measurements of system performance and usage.

**Types:**
- Counters (requests, errors)
- Gauges (active connections, memory usage)
- Histograms (latency distribution, cost distribution)

**Format:** Prometheus metrics

**Related Terms:** [Observability](#observability), [Monitoring](#monitoring)

---

### Migration
The process of moving data from one system to another or updating to a new version.

**Types:**
- Data migration (from other tools)
- Database migration (schema updates)
- Version migration (upgrades)

**Related Terms:** [Version](#version), [Database Migration](#database-migration)

---

### Model
A specific LLM variant with defined capabilities and pricing.

**Examples:**
- `gpt-4-turbo` (OpenAI)
- `claude-3-5-sonnet-20241022` (Anthropic)
- `gemini-1.5-pro` (Google)

**Attributes:**
- Name
- Context window
- Capabilities
- Pricing

**Related Terms:** [Provider](#provider), [Pricing Structure](#pricing-structure)

---

### Multi-Tenancy
The ability to support multiple isolated organizations within a single deployment.

**Isolation Levels:**
- Organization level
- Project level
- User level

**Benefits:**
- Resource sharing
- Cost efficiency
- Centralized management

**Related Terms:** [Organization](#organization), [Project](#project), [RBAC](#rbac)

---

## O

### Observability
The ability to understand system state through logs, metrics, and traces.

**Components:**
- Metrics (Prometheus)
- Logs (structured JSON)
- Traces (OpenTelemetry)

**Related Terms:** [Monitoring](#monitoring), [Metrics](#metrics), [Tracing](#tracing)

---

### Organization
The top-level entity for multi-tenant isolation.

**Attributes:**
- Name
- Slug (URL-friendly identifier)
- Settings
- Users
- Projects

**Example:** `org-acme-corp`

**Related Terms:** [Multi-Tenancy](#multi-tenancy), [Project](#project)

---

## P

### Pagination
Splitting large result sets into smaller pages for better performance.

**Parameters:**
- `page` - Page number (1-indexed)
- `per_page` - Results per page (default: 100, max: 1000)

**Example:**
```bash
curl "https://api.example.com/api/v1/costs?page=2&per_page=100"
```

**Related Terms:** [API](#api), [Performance](#performance)

---

### PII (Personally Identifiable Information)
Data that can identify an individual person.

**Examples:**
- Email addresses
- Names
- IP addresses
- User IDs (in some contexts)

**Handling:**
- Encryption
- Access controls
- Audit logging
- Deletion on request

**Related Terms:** [GDPR](#gdpr), [Compliance](#compliance), [Security](#security)

---

### Pricing Structure
The pricing model for a specific provider and model.

**Types:**
1. **Per-Token:** Charge based on input/output tokens
2. **Per-Request:** Fixed cost per request
3. **Tiered:** Volume-based discounts

**Example:**
```json
{
  "type": "per_token",
  "input_price_per_million": 10.0,
  "output_price_per_million": 30.0,
  "currency": "USD"
}
```

**Related Terms:** [Pricing Table](#pricing-table), [Cost Calculation](#cost-calculation)

---

### Pricing Table
A database record defining pricing for a provider/model combination during a specific time period.

**Fields:**
- Provider
- Model
- Pricing structure (JSON)
- Currency
- Effective date
- End date (optional)

**Related Terms:** [Pricing Structure](#pricing-structure), [Cost Calculation](#cost-calculation)

---

### Project
A subdivision within an organization for tracking costs separately.

**Use Cases:**
- Different products
- Different departments
- Different environments (dev, staging, prod)

**Example:** `proj-mobile-app`, `proj-web-platform`

**Related Terms:** [Organization](#organization), [Multi-Tenancy](#multi-tenancy)

---

### Provider
A company or service that offers LLM APIs.

**Supported Providers:**
- `openai` - OpenAI
- `anthropic` - Anthropic
- `google` - Google (Vertex AI)
- `azure` - Azure OpenAI
- `aws` - AWS Bedrock
- `cohere` - Cohere
- `mistral` - Mistral AI

**Related Terms:** [Model](#model), [LLM](#llm)

---

## R

### Rate Limiting
Controlling the number of requests allowed within a time window.

**Default Limits:**
- 1,000 requests per minute per API key
- 10,000 requests per hour per organization

**Response:** HTTP 429 Too Many Requests

**Headers:**
- `X-RateLimit-Limit` - Total allowed
- `X-RateLimit-Remaining` - Remaining in window
- `X-RateLimit-Reset` - When limit resets

**Related Terms:** [API](#api), [Backoff](#backoff)

---

### RBAC (Role-Based Access Control)
Authorization model based on user roles.

**Built-in Roles:**
- **Admin:** Full access to everything
- **Developer:** Create/read usage and costs
- **Analyst:** Read-only access to costs and reports
- **Viewer:** Read-only access to dashboards

**Permissions:**
- `usage:write`, `usage:read`
- `costs:read`
- `pricing:write`, `pricing:read`
- `org:admin`

**Related Terms:** [Authentication](#authentication), [Authorization](#authorization)

---

### Reasoning Tokens
Special tokens used by reasoning models (e.g., OpenAI o1) for internal thinking.

**Characteristics:**
- Not visible in response
- Charged separately
- Only for reasoning-capable models

**Related Terms:** [Token](#token), [Input Tokens](#input-tokens)

---

### Report
A formatted document containing cost or usage analysis.

**Types:**
- Cost summary
- Usage analysis
- Forecast report
- Audit report
- Budget report

**Formats:**
- PDF
- Excel
- HTML
- JSON

**Related Terms:** [Export](#export), [Cost Summary](#cost-summary)

---

### REST API
Representational State Transfer - a standard architectural style for APIs.

**Characteristics:**
- HTTP methods (GET, POST, PUT, DELETE)
- Resource-based URLs
- Stateless
- JSON request/response

**Related Terms:** [API](#api), [HTTP](#http)

---

### Retry
Automatically re-attempting a failed operation.

**Strategy:**
- Max retries: 3
- Backoff: Exponential (1s, 2s, 4s)
- Retryable errors: 429, 500, 503

**Related Terms:** [Backoff](#backoff), [Rate Limiting](#rate-limiting)

---

## S

### Schema
The structure of the database tables and relationships.

**Key Tables:**
- `usage_records` - Raw usage data
- `cost_records` - Calculated costs
- `pricing_tables` - Pricing information
- `organizations` - Multi-tenant orgs
- `users` - User accounts

**Related Terms:** [Database Migration](#database-migration), [Model](#model)

---

### SDK (Software Development Kit)
Client libraries for integrating with LLM Cost Ops.

**Available SDKs:**
- Python (`pip install llm-cost-ops`)
- TypeScript (`npm install @llm-cost-ops/sdk`)
- Go (`go get github.com/llm-cost-ops/sdk-go`)
- Rust (`llm-cost-ops = "1.0"`)

**Related Terms:** [API](#api), [CLI](#cli)

---

### Security
Measures to protect the system and data.

**Features:**
- TLS/HTTPS encryption
- API key authentication
- JWT tokens
- RBAC authorization
- Audit logging
- Rate limiting
- Input validation
- SQL injection prevention

**Related Terms:** [Authentication](#authentication), [Compliance](#compliance)

---

### SOC2 (Service Organization Control 2)
An auditing standard for service providers.

**Trust Principles:**
- Security
- Availability
- Processing integrity
- Confidentiality
- Privacy

**Related Terms:** [Compliance](#compliance), [Audit Log](#audit-log)

---

## T

### Tags
Labels attached to usage records for categorization and filtering.

**Examples:**
- `["production", "api"]`
- `["development", "testing"]`
- `["customer-facing", "internal"]`

**Use Cases:**
- Cost allocation
- Filtering reports
- Usage analysis

**Related Terms:** [Metadata](#metadata), [Usage Record](#usage-record)

---

### Token
The basic unit of text processed by LLMs.

**Approximation:**
- 1 token ≈ 4 characters
- 1 token ≈ 0.75 words
- 100 tokens ≈ 75 words

**Types:**
- Input/Prompt tokens
- Output/Completion tokens
- Cached tokens
- Reasoning tokens

**Related Terms:** [Input Tokens](#input-tokens), [Completion Tokens](#completion-tokens)

---

### Total Tokens
The sum of input and output tokens for a request.

**Formula:**
```
Total Tokens = Prompt Tokens + Completion Tokens
```

**Note:** Some models have separate counts for cached and reasoning tokens.

**Related Terms:** [Token](#token), [Input Tokens](#input-tokens), [Completion Tokens](#completion-tokens)

---

### Tracing
Recording the path and timing of requests through the system.

**Format:** OpenTelemetry Protocol (OTLP)

**Information Captured:**
- Trace ID
- Span ID
- Parent span
- Duration
- Attributes

**Tools:** Jaeger, Zipkin, Honeycomb

**Related Terms:** [Observability](#observability), [Monitoring](#monitoring)

---

### Trend
The general direction of costs or usage over time.

**Types:**
- Upward trend (increasing)
- Downward trend (decreasing)
- Stable (no significant change)
- Seasonal (repeating patterns)

**Related Terms:** [Forecast](#forecast), [Anomaly Detection](#anomaly-detection)

---

## U

### Usage Record
A database record representing a single LLM API call.

**Required Fields:**
- Timestamp
- Provider
- Model
- Organization ID
- Prompt tokens
- Completion tokens
- Total tokens

**Optional Fields:**
- Project ID
- User ID
- Cached tokens
- Reasoning tokens
- Latency
- Tags
- Metadata

**Related Terms:** [Cost Record](#cost-record), [Ingestion](#ingestion)

---

### UUID (Universally Unique Identifier)
A 128-bit identifier guaranteed to be unique.

**Format:** `550e8400-e29b-41d4-a716-446655440000`

**Use:** Primary keys for all records

**Related Terms:** [Idempotency](#idempotency), [Schema](#schema)

---

## V

### Validation
Checking that data meets required format and constraints.

**Checks:**
- Required fields present
- Data types correct
- Values within valid ranges
- Format matches specification

**Example Errors:**
- "prompt_tokens must be greater than 0"
- "provider must be one of: openai, anthropic, ..."
- "timestamp must be in ISO 8601 format"

**Related Terms:** [API](#api), [Error Handling](#error-handling)

---

### Version
The release version of the LLM Cost Ops software.

**Format:** Semantic Versioning (SemVer)
- `MAJOR.MINOR.PATCH`
- Example: `1.0.0`

**Rules:**
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

**Related Terms:** [Migration](#migration), [Breaking Changes](#breaking-changes)

---

## W

### Webhook
An HTTP callback that sends data to a URL when an event occurs.

**Events:**
- Usage created
- Cost threshold exceeded
- Anomaly detected
- Report generated

**Example:**
```json
{
  "event": "cost.threshold.exceeded",
  "timestamp": "2025-01-15T10:00:00Z",
  "data": {
    "organization_id": "org-abc",
    "threshold": 1000.0,
    "current_cost": 1050.0
  }
}
```

**Related Terms:** [Event](#event), [Alert](#alert)

---

## Z

### Z-Score
A statistical measurement of how many standard deviations a value is from the mean.

**Use:** Anomaly detection

**Formula:**
```
Z-Score = (Value - Mean) / Standard Deviation
```

**Threshold:** Typically, |z| > 3.0 indicates an anomaly

**Example:** If daily cost is $1,000 with mean $500 and std dev $100:
```
Z-Score = (1000 - 500) / 100 = 5.0
```
This is an anomaly (z > 3).

**Related Terms:** [Anomaly Detection](#anomaly-detection), [Threshold](#threshold)

---

## Additional Terms

### Alert
A notification sent when a condition is met.

**Triggers:**
- Budget threshold exceeded
- Anomaly detected
- Cost spike detected
- System error

**Channels:**
- Email
- Slack
- Webhook
- PagerDuty

**Related Terms:** [Budget](#budget), [Webhook](#webhook)

---

### Authorization
The process of determining what actions a user can perform.

**Related Terms:** [RBAC](#rbac), [Authentication](#authentication)

---

### Breaking Changes
Changes that are not backward compatible and require updates to client code.

**Examples:**
- Renamed API endpoints
- Changed response format
- Removed fields
- Changed authentication method

**Related Terms:** [Version](#version), [Migration](#migration)

---

### Cost Control
Mechanisms to manage and limit spending.

**Methods:**
- Budgets with alerts
- Rate limiting
- Approval workflows
- Auto-shutoff

**Related Terms:** [Budget](#budget), [Alert](#alert)

---

### Data Retention
How long data is stored before being deleted.

**Policies:**
- Usage records: 2 years default
- Audit logs: 7 years (compliance)
- Exports: 30 days

**Related Terms:** [Compliance](#compliance), [GDPR](#gdpr)

---

### Deployment
The process of installing and running the LLM Cost Ops system.

**Methods:**
- Kubernetes (recommended for production)
- Docker
- Binary installation
- From source

**Related Terms:** [Kubernetes](#kubernetes), [Helm](#helm)

---

### Error Handling
Managing and responding to errors gracefully.

**Best Practices:**
- Return meaningful error messages
- Use appropriate HTTP status codes
- Log errors for debugging
- Retry transient errors

**Related Terms:** [Validation](#validation), [Retry](#retry)

---

### Event
An occurrence in the system that may trigger actions.

**Types:**
- Usage created
- Cost calculated
- Threshold exceeded
- Report generated

**Related Terms:** [Webhook](#webhook), [Alert](#alert)

---

### Format
The structure and encoding of exported data.

**Supported Formats:**
- CSV - Spreadsheet compatible
- JSON - Machine readable
- Excel - Full formatting
- PDF - Reports

**Related Terms:** [Export](#export), [Report](#report)

---

### Helm
A package manager for Kubernetes.

**Chart:** Pre-configured Kubernetes manifests

**Commands:**
```bash
helm install cost-ops ./helm/cost-ops
helm upgrade cost-ops ./helm/cost-ops
```

**Related Terms:** [Kubernetes](#kubernetes), [Deployment](#deployment)

---

### HTTP
HyperText Transfer Protocol - the foundation of the REST API.

**Methods:**
- GET - Retrieve resources
- POST - Create resources
- PUT - Update resources
- DELETE - Remove resources

**Related Terms:** [REST API](#rest-api), [API](#api)

---

### Monitoring
Continuous observation of system health and performance.

**Tools:**
- Prometheus (metrics)
- Grafana (dashboards)
- Alertmanager (alerts)

**Related Terms:** [Observability](#observability), [Metrics](#metrics)

---

### Performance
How efficiently the system operates.

**Metrics:**
- Request latency
- Throughput (requests/second)
- Query time
- Resource usage

**Optimization:**
- Caching
- Indexing
- Batch operations
- Connection pooling

**Related Terms:** [Latency](#latency), [Optimization](#optimization)

---

### Prediction
An estimate of future values based on historical data.

**Related Terms:** [Forecast](#forecast), [Trend](#trend)

---

### Threshold
A boundary value that triggers an action when crossed.

**Examples:**
- Budget threshold: 80% of limit
- Anomaly threshold: Z-score > 3.0
- Rate limit: 1000 requests/minute

**Related Terms:** [Alert](#alert), [Budget](#budget), [Anomaly Detection](#anomaly-detection)

---

## Acronyms Quick Reference

- **API** - Application Programming Interface
- **CLI** - Command-Line Interface
- **CRUD** - Create, Read, Update, Delete
- **CSV** - Comma-Separated Values
- **GDPR** - General Data Protection Regulation
- **HTTP** - HyperText Transfer Protocol
- **HTTPS** - HTTP Secure
- **IQR** - Interquartile Range
- **JSON** - JavaScript Object Notation
- **JWT** - JSON Web Token
- **K8s** - Kubernetes
- **LLM** - Large Language Model
- **OTLP** - OpenTelemetry Protocol
- **PII** - Personally Identifiable Information
- **RBAC** - Role-Based Access Control
- **REST** - Representational State Transfer
- **SDK** - Software Development Kit
- **SOC2** - Service Organization Control 2
- **SQL** - Structured Query Language
- **TLS** - Transport Layer Security
- **UUID** - Universally Unique Identifier

---

## See Also

### Documentation
- [Getting Started Guide](/docs/training/user-guides/getting-started.md)
- [API Reference](/docs/training/reference/api-reference.md)
- [Architecture Overview](/docs/SPECIFICATION.md)

### Quick Reference
- [Quick Reference Card](/docs/training/quick-reference/quick-reference-card.md)
- [Cheat Sheet](/docs/training/quick-reference/cheat-sheet.md)
- [FAQ](/docs/training/reference/faq.md)

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0

For questions about terminology or to suggest additions, contact documentation@llm-cost-ops.com
