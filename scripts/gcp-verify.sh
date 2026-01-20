#!/bin/bash
# LLM-CostOps Post-Deployment Verification Script
# Usage: ./scripts/gcp-verify.sh <environment>
#
# Performs comprehensive verification of the deployed service

set -euo pipefail

# Configuration
PROJECT_ID="agentics-dev"
REGION="us-central1"
SERVICE_NAME="llm-costops"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Parse arguments
ENVIRONMENT="${1:-}"

if [[ -z "$ENVIRONMENT" ]]; then
    echo -e "${RED}Error: Environment required${NC}"
    echo "Usage: $0 <environment>"
    exit 1
fi

# Set service name
if [[ "$ENVIRONMENT" != "prod" ]]; then
    FULL_SERVICE_NAME="${SERVICE_NAME}-${ENVIRONMENT}"
else
    FULL_SERVICE_NAME="${SERVICE_NAME}"
fi

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}LLM-CostOps Verification${NC}"
echo -e "${GREEN}========================================${NC}"
echo "Environment: ${ENVIRONMENT}"
echo ""

# Get service URL
SERVICE_URL=$(gcloud run services describe "${FULL_SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --format='value(status.url)')

if [[ -z "$SERVICE_URL" ]]; then
    echo -e "${RED}Error: Service not found${NC}"
    exit 1
fi

echo "Service URL: ${SERVICE_URL}"

# Get identity token
TOKEN=$(gcloud auth print-identity-token)

PASS_COUNT=0
FAIL_COUNT=0

# Helper function for API calls
call_api() {
    local endpoint="$1"
    local method="${2:-GET}"
    local data="${3:-}"

    if [[ -n "$data" ]]; then
        curl -s -X "${method}" \
            -H "Authorization: Bearer ${TOKEN}" \
            -H "Content-Type: application/json" \
            -d "${data}" \
            "${SERVICE_URL}${endpoint}"
    else
        curl -s -X "${method}" \
            -H "Authorization: Bearer ${TOKEN}" \
            "${SERVICE_URL}${endpoint}"
    fi
}

# Test function
run_test() {
    local name="$1"
    local expected="$2"
    local actual="$3"

    if [[ "$actual" == *"$expected"* ]]; then
        echo -e "  ${GREEN}✓${NC} ${name}"
        ((PASS_COUNT++))
    else
        echo -e "  ${RED}✗${NC} ${name}"
        echo "    Expected: ${expected}"
        echo "    Actual: ${actual}"
        ((FAIL_COUNT++))
    fi
}

echo ""
echo -e "${YELLOW}1. Health & Readiness Checks${NC}"
echo "-------------------------------------------"

# Health check
HEALTH_RESPONSE=$(call_api "/health")
run_test "Health endpoint responds" "healthy" "$HEALTH_RESPONSE"

# Readiness check
READY_RESPONSE=$(call_api "/ready")
run_test "Readiness endpoint responds" "ready" "$READY_RESPONSE"

echo ""
echo -e "${YELLOW}2. Agent Info Endpoints${NC}"
echo "-------------------------------------------"

# Cost Forecasting Agent info
FORECAST_INFO=$(call_api "/api/v1/agents/cost-forecasting/info" 2>/dev/null || echo "{}")
run_test "Cost Forecasting Agent info" "cost-forecasting-agent" "$FORECAST_INFO"

# Budget Enforcement Agent info
BUDGET_INFO=$(call_api "/api/v1/agents/budget-enforcement/info" 2>/dev/null || echo "{}")
run_test "Budget Enforcement Agent info" "budget-enforcement" "$BUDGET_INFO"

echo ""
echo -e "${YELLOW}3. Agent Inspect Endpoints${NC}"
echo "-------------------------------------------"

# Cost Forecasting inspect
FORECAST_INSPECT=$(call_api "/api/v1/agents/cost-forecasting/inspect" 2>/dev/null || echo "{}")
run_test "Cost Forecasting inspect endpoint" "capabilities" "$FORECAST_INSPECT"

# Budget Enforcement inspect
BUDGET_INSPECT=$(call_api "/api/v1/agents/budget-enforcement/inspect" 2>/dev/null || echo "{}")
run_test "Budget Enforcement inspect endpoint" "capabilities" "$BUDGET_INSPECT"

echo ""
echo -e "${YELLOW}4. Cost Forecasting Agent - Analyze${NC}"
echo "-------------------------------------------"

# Create test forecast request
NOW=$(date -u +%Y-%m-%dT%H:%M:%SZ)
HISTORICAL_DATA="["
for i in {14..1}; do
    DATE=$(date -u -d "-${i} days" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -v-${i}d +%Y-%m-%dT%H:%M:%SZ)
    COST=$((100 + i * 5))
    if [[ $i -gt 1 ]]; then
        HISTORICAL_DATA="${HISTORICAL_DATA}{\"timestamp\":\"${DATE}\",\"total_cost\":${COST},\"total_tokens\":1000000,\"request_count\":1000},"
    else
        HISTORICAL_DATA="${HISTORICAL_DATA}{\"timestamp\":\"${DATE}\",\"total_cost\":${COST},\"total_tokens\":1000000,\"request_count\":1000}"
    fi
done
HISTORICAL_DATA="${HISTORICAL_DATA}]"

FORECAST_REQUEST='{
    "historical_data": '"${HISTORICAL_DATA}"',
    "forecast_horizon_days": 30,
    "confidence_level": 0.95,
    "metadata": {
        "organization_id": "test-org",
        "execution_ref": "verify-test"
    }
}'

FORECAST_RESPONSE=$(call_api "/api/v1/agents/cost-forecasting/forecast" "POST" "$FORECAST_REQUEST" 2>/dev/null || echo "{}")
run_test "Cost Forecasting analyze produces result" "success" "$FORECAST_RESPONSE"

echo ""
echo -e "${YELLOW}5. Budget Enforcement Agent - Analyze${NC}"
echo "-------------------------------------------"

BUDGET_REQUEST='{
    "tenant_id": "test-tenant",
    "budget_id": "test-budget",
    "budget_limit": 10000,
    "currency": "USD",
    "current_spend": 5000,
    "warning_threshold": 0.80,
    "critical_threshold": 0.95
}'

BUDGET_RESPONSE=$(call_api "/api/v1/agents/budget-enforcement/analyze" "POST" "$BUDGET_REQUEST" 2>/dev/null || echo "{}")
run_test "Budget Enforcement analyze produces result" "signal" "$BUDGET_RESPONSE"

echo ""
echo -e "${YELLOW}6. Constitution Compliance Checks${NC}"
echo "-------------------------------------------"

# Verify no direct SQL access (should not have DATABASE_URL)
SERVICE_ENVS=$(gcloud run services describe "${FULL_SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --format='value(spec.template.spec.containers[0].env)' 2>/dev/null || echo "")

if [[ "$SERVICE_ENVS" != *"DATABASE_URL"* ]]; then
    echo -e "  ${GREEN}✓${NC} No direct DATABASE_URL configured"
    ((PASS_COUNT++))
else
    echo -e "  ${RED}✗${NC} WARNING: DATABASE_URL found (violates Constitution)"
    ((FAIL_COUNT++))
fi

if [[ "$SERVICE_ENVS" == *"RUVECTOR"* ]]; then
    echo -e "  ${GREEN}✓${NC} RuVector service configured for persistence"
    ((PASS_COUNT++))
else
    echo -e "  ${YELLOW}⚠${NC} RuVector configuration not found in env vars"
fi

echo ""
echo -e "${YELLOW}7. Service Configuration${NC}"
echo "-------------------------------------------"

# Get service details
SERVICE_DETAILS=$(gcloud run services describe "${FULL_SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --format='json' 2>/dev/null || echo "{}")

# Check ingress (should be internal)
INGRESS=$(echo "$SERVICE_DETAILS" | grep -o '"ingress":"[^"]*"' | head -1 || echo "")
if [[ "$INGRESS" == *"internal"* ]] || [[ "$INGRESS" == *"INGRESS_TRAFFIC_INTERNAL_ONLY"* ]]; then
    echo -e "  ${GREEN}✓${NC} Ingress is internal-only"
    ((PASS_COUNT++))
else
    echo -e "  ${YELLOW}⚠${NC} Ingress may not be internal-only"
fi

# Check authentication
if [[ "$SERVICE_DETAILS" == *"noAuthentication"* ]] || [[ "$SERVICE_DETAILS" == *"allUsers"* ]]; then
    echo -e "  ${RED}✗${NC} WARNING: Service may allow unauthenticated access"
    ((FAIL_COUNT++))
else
    echo -e "  ${GREEN}✓${NC} Service requires authentication"
    ((PASS_COUNT++))
fi

echo ""
echo "=========================================="
echo "VERIFICATION SUMMARY"
echo "=========================================="
echo -e "Passed: ${GREEN}${PASS_COUNT}${NC}"
echo -e "Failed: ${RED}${FAIL_COUNT}${NC}"
echo ""

if [[ $FAIL_COUNT -gt 0 ]]; then
    echo -e "${RED}Some checks failed. Review the output above.${NC}"
    exit 1
else
    echo -e "${GREEN}All verification checks passed!${NC}"
    exit 0
fi
