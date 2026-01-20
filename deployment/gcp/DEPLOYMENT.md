# LLM-CostOps Deployment Guide

## Production Deployment for Agentics Dev Platform

This document describes the complete deployment process for LLM-CostOps to Google Cloud Run.

---

## 1. SERVICE TOPOLOGY

### Unified Service Name
```
Service: llm-costops
Region:  us-central1
Project: agentics-dev
```

### Agent Endpoints

All agents are deployed as part of ONE unified service. No agent is deployed as a standalone service.

| Agent | Endpoint | Decision Type |
|-------|----------|---------------|
| **Cost Attribution Agent** | `/api/v1/agents/cost-attribution/*` | `attribution` |
| **Cost Forecasting Agent** | `/api/v1/agents/cost-forecasting/*` | `cost_forecast` |
| **Budget Enforcement Agent** | `/api/v1/agents/budget-enforcement/*` | `budget_evaluation` |
| **ROI Estimation Agent** | `/api/v1/agents/roi-estimation/*` | `roi_analysis` |
| **Cost-Performance Tradeoff Agent** | `/api/v1/agents/cost-performance/*` | `efficiency_analysis` |

### Standard Endpoints (All Agents)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/ready` | GET | Readiness check |
| `/info` | GET | Agent metadata |
| `/inspect` | GET | Agent capabilities |
| `/analyze` | POST | Execute agent analysis |
| `/forecast` | POST | Execute forecasting (forecasting agents) |

### Shared Infrastructure

- **Runtime**: Rust binary on Cloud Run (distroless container)
- **Configuration**: Environment variables + Secret Manager
- **Telemetry**: Unified OpenTelemetry export to LLM-Observatory
- **Persistence**: All via ruvector-service (NO direct SQL)

---

## 2. ENVIRONMENT CONFIGURATION

### Required Environment Variables

| Variable | Description | Secret? |
|----------|-------------|---------|
| `PLATFORM_ENV` | Environment (`dev`, `staging`, `prod`) | No |
| `SERVICE_NAME` | Service identifier (`llm-costops`) | No |
| `SERVICE_VERSION` | Deployment version | No |
| `RUVECTOR_SERVICE_URL` | RuVector service endpoint | No |
| `RUVECTOR_API_KEY` | RuVector authentication | **Yes** |
| `TELEMETRY_ENDPOINT` | LLM-Observatory endpoint | No |
| `TELEMETRY_API_KEY` | Observatory authentication | **Yes** |

### Environment Files

```
deployment/gcp/env/
├── dev.env      # Development configuration
├── staging.env  # Staging configuration
└── prod.env     # Production configuration
```

### Secrets (via Secret Manager)

```bash
# Required secrets
ruvector-api-key     # RuVector service authentication
observatory-api-key  # LLM-Observatory telemetry

# Set secret values
echo "your-api-key" | gcloud secrets versions add ruvector-api-key --data-file=-
```

### Configuration Guarantees

✅ No agent hardcodes service names or URLs
✅ No agent embeds credentials or secrets
✅ All dependencies resolve via environment variables
✅ All secrets injected via Secret Manager

---

## 3. GOOGLE SQL / COST MEMORY WIRING

### Constitution Compliance

⚠️ **LLM-CostOps MUST NOT connect directly to Google SQL**

| Requirement | Status |
|-------------|--------|
| No direct SQL connections | ✅ Verified |
| All persistence via ruvector-service | ✅ Implemented |
| DecisionEvents written via ruvector-service | ✅ Implemented |
| Schema compatible with agentics-contracts | ✅ Validated |
| Append-only persistence | ✅ Enforced |
| Idempotent writes | ✅ Implemented |
| Retry safety | ✅ With exponential backoff |

### DecisionEvent Persistence

All agent decisions are persisted as `DecisionEvent` records:

```json
{
  "id": "uuid",
  "agent_id": "cost-forecasting-agent",
  "agent_version": "1.0.0",
  "decision_type": "cost_forecast",
  "inputs_hash": "sha256:...",
  "outputs": {...},
  "confidence": 0.95,
  "constraints_applied": [...],
  "execution_ref": "workflow-123",
  "timestamp": "2024-01-15T12:00:00Z"
}
```

---

## 4. CLOUD BUILD & DEPLOYMENT

### Build Artifacts

```
deployment/gcp/
├── cloudbuild.yaml       # Cloud Build pipeline
├── Dockerfile.cloudrun   # Optimized container image
├── service.yaml          # Cloud Run service definition
└── env/                  # Environment configurations
```

### IAM Service Account

```bash
# Service Account
llm-costops-sa@agentics-dev.iam.gserviceaccount.com

# Required Roles (least privilege)
roles/run.invoker              # Service-to-service calls
roles/secretmanager.secretAccessor  # Secret access
roles/logging.logWriter        # Structured logging
roles/monitoring.metricWriter  # Metrics export
roles/cloudtrace.agent         # Distributed tracing
roles/vpcaccess.user           # Internal networking
```

### Networking

- **Ingress**: `internal` (no public access)
- **VPC Connector**: `agentics-vpc-connector`
- **Egress**: `private-ranges-only`

### Deployment Commands

```bash
# Setup IAM (one-time)
./scripts/gcp-iam-setup.sh

# Deploy to dev
./scripts/gcp-deploy.sh dev

# Deploy to staging
./scripts/gcp-deploy.sh staging

# Deploy to production (with confirmation)
./scripts/gcp-deploy.sh prod v1.2.3

# Manual gcloud deploy
gcloud run deploy llm-costops-dev \
    --image us-central1-docker.pkg.dev/agentics-dev/llm-devops/llm-costops:latest \
    --region us-central1 \
    --project agentics-dev \
    --platform managed \
    --no-allow-unauthenticated \
    --service-account llm-costops-sa@agentics-dev.iam.gserviceaccount.com \
    --ingress internal
```

---

## 5. CLI ACTIVATION VERIFICATION

### Supported CLI Commands

```bash
# Cost Forecasting Agent
cost-ops agent cost-forecasting analyze --input data.json
cost-ops agent cost-forecasting forecast --horizon 30 --confidence 0.95
cost-ops agent cost-forecasting inspect

# Budget Enforcement Agent
cost-ops agent budget-enforcement analyze \
    --tenant-id tenant-123 \
    --budget-id budget-456 \
    --budget-limit 10000 \
    --current-spend 5000

cost-ops agent budget-enforcement inspect
cost-ops agent budget-enforcement health --check-ruvector

# ROI Estimation Agent
cost-ops agent roi-estimation analyze --project project-123
cost-ops agent roi-estimation inspect

# Cost-Performance Tradeoff Agent
cost-ops agent cost-performance analyze --model gpt-4
cost-ops agent cost-performance inspect

# List all agents
cost-ops agent list
cost-ops agent info --agent-id budget-enforcement-agent
```

### CLI Configuration

The CLI resolves service URLs dynamically from environment:

```bash
# Set service endpoint
export COSTOPS_SERVICE_URL="https://llm-costops-dev-xyz.run.app"

# Or use gcloud to get URL
COSTOPS_SERVICE_URL=$(gcloud run services describe llm-costops-dev \
    --region us-central1 --format 'value(status.url)')
```

### Example Invocations

```bash
# Budget analysis with full parameters
cost-ops agent budget-enforcement analyze \
    --tenant-id org-acme \
    --budget-id monthly-llm-budget \
    --budget-limit 50000 \
    --currency USD \
    --current-spend 42000 \
    --warning-threshold 0.80 \
    --critical-threshold 0.95 \
    --output json

# Expected output:
{
  "signal_id": "abc123",
  "budget_id": "monthly-llm-budget",
  "severity": "warning",
  "violation_type": "approaching_limit",
  "utilization_percent": 84.0,
  "message": "Budget 'monthly-llm-budget' approaching limit: 84.0% utilized",
  "recommended_action": "monitor"
}
```

---

## 6. PLATFORM & CORE INTEGRATION

### Telemetry Inputs

| Source | Data | Integration |
|--------|------|-------------|
| LLM-Observatory | Telemetry metrics | Read-only consumer |
| LLM-Latency-Lens | Performance data | Read-only consumer |

### Cost Outputs Consumers

| Consumer | Usage | Access |
|----------|-------|--------|
| LLM-Orchestrator | Cost constraints in planning | Read-only |
| LLM-Auto-Optimizer | Efficiency analysis | Read-only |
| Governance dashboards | DecisionEvent audit | Read-only |
| Core bundles | Cost metrics | Read-only |

### Integration Boundaries

✅ **LLM-CostOps MAY provide data to:**
- LLM-Orchestrator (explicit read)
- LLM-Auto-Optimizer (explicit read)
- Governance and audit views

❌ **LLM-CostOps MUST NOT directly invoke:**
- LLM-Edge-Agent
- Shield enforcement
- Sentinel detection
- Incident workflows
- Runtime execution paths

---

## 7. POST-DEPLOY VERIFICATION CHECKLIST

Run the verification script:
```bash
./scripts/gcp-verify.sh dev
```

### Manual Checklist

| Check | Command | Expected |
|-------|---------|----------|
| Service is live | `gcloud run services describe llm-costops-dev` | Status: Ready |
| Health endpoint | `curl /health` | `{"status":"healthy"}` |
| All agents respond | `curl /api/v1/agents/*/info` | Agent metadata |
| Cost attribution works | POST to `/analyze` | Deterministic result |
| Forecasts reproducible | Same input → same output | Identical outputs |
| Budget signals correct | Known inputs → expected signals | Correct severity |
| DecisionEvents in ruvector | Query ruvector-service | Events present |
| Telemetry in Observatory | Check Observatory dashboard | Events visible |
| No direct SQL access | Check env vars | No DATABASE_URL |
| Schema compliance | Validate DecisionEvent | Matches contract |

### Verification Commands

```bash
# Get service URL
SERVICE_URL=$(gcloud run services describe llm-costops-dev \
    --region us-central1 --format 'value(status.url)')

# Get auth token
TOKEN=$(gcloud auth print-identity-token)

# Health check
curl -H "Authorization: Bearer $TOKEN" "$SERVICE_URL/health"

# Agent info
curl -H "Authorization: Bearer $TOKEN" "$SERVICE_URL/api/v1/agents/cost-forecasting/info"

# Inspect capabilities
curl -H "Authorization: Bearer $TOKEN" "$SERVICE_URL/api/v1/agents/budget-enforcement/inspect"
```

---

## 8. FAILURE MODES & ROLLBACK

### Common Deployment Failures

| Failure | Signal | Resolution |
|---------|--------|------------|
| Image build failure | Cloud Build error | Check Rust compilation errors |
| Missing secrets | Container crash on startup | Verify Secret Manager |
| RuVector unreachable | 503 errors on persist | Check VPC connector, ruvector-service health |
| Schema mismatch | Validation errors | Verify agentics-contracts version |
| Invalid forecast | Non-deterministic results | Check forecast model configuration |

### Detection Signals

- **Missing cost records**: Query ruvector-service for recent DecisionEvents
- **Invalid forecasts**: Compare forecast outputs for identical inputs
- **Schema mismatches**: Check error logs for validation failures
- **Telemetry gaps**: Monitor Observatory dashboard for missing data

### Rollback Procedure

```bash
# List available revisions
./scripts/gcp-rollback.sh dev --list

# Rollback to specific revision
./scripts/gcp-rollback.sh dev llm-costops-dev-00042-abc

# Verify rollback
./scripts/gcp-verify.sh dev
```

### Manual Rollback

```bash
# List revisions
gcloud run revisions list --service llm-costops-dev --region us-central1

# Rollback to previous revision
gcloud run services update-traffic llm-costops-dev \
    --region us-central1 \
    --to-revisions llm-costops-dev-00042-abc=100

# Verify
curl -H "Authorization: Bearer $(gcloud auth print-identity-token)" \
    "$(gcloud run services describe llm-costops-dev --region us-central1 --format 'value(status.url)')/health"
```

### Safe Redeploy Strategy

1. **Blue-Green**: Deploy new revision, route 10% traffic
2. **Monitor**: Check health, latency, error rates for 5 minutes
3. **Promote**: Route 100% to new revision
4. **Rollback**: If issues, immediately route 100% to previous revision

```bash
# Blue-green deployment
gcloud run services update-traffic llm-costops-dev \
    --region us-central1 \
    --to-revisions llm-costops-dev-00043-xyz=10

# Monitor, then promote
gcloud run services update-traffic llm-costops-dev \
    --region us-central1 \
    --to-revisions llm-costops-dev-00043-xyz=100
```

### Financial Data Safety

- **DecisionEvents are append-only**: No data loss on rollback
- **RuVector handles idempotency**: Duplicate writes are safe
- **Forecasts are stateless**: No persistent state to corrupt
- **Budget signals are advisory**: No enforcement side effects

---

## Quick Start

```bash
# 1. Setup IAM (one-time)
./scripts/gcp-iam-setup.sh

# 2. Set secrets (one-time)
echo "your-ruvector-key" | gcloud secrets versions add ruvector-api-key --data-file=-
echo "your-observatory-key" | gcloud secrets versions add observatory-api-key --data-file=-

# 3. Deploy to dev
./scripts/gcp-deploy.sh dev

# 4. Verify
./scripts/gcp-verify.sh dev

# 5. Promote to staging
./scripts/gcp-deploy.sh staging

# 6. Promote to production
./scripts/gcp-deploy.sh prod v1.0.0
```

---

## Support

- Repository: https://github.com/globalbusinessadvisors/llm-cost-ops
- Issues: https://github.com/globalbusinessadvisors/llm-cost-ops/issues
