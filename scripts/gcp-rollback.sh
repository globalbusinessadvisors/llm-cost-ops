#!/bin/bash
# LLM-CostOps Rollback Script
# Rolls back to a previous revision of the Cloud Run service
#
# Usage: ./scripts/gcp-rollback.sh <environment> [revision]
#        ./scripts/gcp-rollback.sh <environment> --list

set -euo pipefail

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
REVISION="${2:-}"

if [[ -z "$ENVIRONMENT" ]]; then
    echo -e "${RED}Error: Environment required${NC}"
    echo "Usage: $0 <environment> [revision]"
    echo "       $0 <environment> --list"
    exit 1
fi

# Set service name
if [[ "$ENVIRONMENT" != "prod" ]]; then
    FULL_SERVICE_NAME="${SERVICE_NAME}-${ENVIRONMENT}"
else
    FULL_SERVICE_NAME="${SERVICE_NAME}"
fi

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}LLM-CostOps Rollback${NC}"
echo -e "${GREEN}========================================${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Service: ${FULL_SERVICE_NAME}"
echo ""

# List revisions
if [[ "$REVISION" == "--list" ]] || [[ -z "$REVISION" ]]; then
    echo "Available revisions:"
    echo ""
    gcloud run revisions list \
        --service="${FULL_SERVICE_NAME}" \
        --region="${REGION}" \
        --project="${PROJECT_ID}" \
        --format="table(
            metadata.name,
            metadata.creationTimestamp.date('%Y-%m-%d %H:%M:%S'),
            status.conditions[0].status,
            spec.containers[0].image
        )"
    echo ""

    if [[ -z "$REVISION" ]]; then
        echo "To rollback, run:"
        echo "  $0 ${ENVIRONMENT} <revision-name>"
    fi
    exit 0
fi

# Confirm rollback
echo -e "${YELLOW}WARNING: Rolling back to revision '${REVISION}'${NC}"
echo ""

# Get current serving revision
CURRENT_REVISION=$(gcloud run services describe "${FULL_SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --format='value(status.latestReadyRevisionName)')

echo "Current revision: ${CURRENT_REVISION}"
echo "Target revision: ${REVISION}"
echo ""

if [[ "$CURRENT_REVISION" == "$REVISION" ]]; then
    echo -e "${YELLOW}Already serving revision '${REVISION}'${NC}"
    exit 0
fi

# Verify target revision exists
if ! gcloud run revisions describe "${REVISION}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" &>/dev/null; then
    echo -e "${RED}Error: Revision '${REVISION}' not found${NC}"
    exit 1
fi

read -p "Proceed with rollback? (yes/no) " -r
if [[ ! $REPLY == "yes" ]]; then
    echo "Rollback cancelled"
    exit 0
fi

echo ""
echo "Rolling back..."

# Update traffic to target revision
gcloud run services update-traffic "${FULL_SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --to-revisions="${REVISION}=100" \
    --quiet

echo ""
echo -e "${GREEN}Rollback complete!${NC}"
echo ""

# Verify rollback
echo "Verifying rollback..."
sleep 5

# Get service URL
SERVICE_URL=$(gcloud run services describe "${FULL_SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --format='value(status.url)')

# Health check
TOKEN=$(gcloud auth print-identity-token)

for i in {1..5}; do
    HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "Authorization: Bearer ${TOKEN}" \
        "${SERVICE_URL}/health" 2>/dev/null || echo "000")

    if [[ "$HTTP_STATUS" == "200" ]]; then
        echo -e "${GREEN}Rollback verified - service is healthy${NC}"
        break
    fi

    if [[ $i -eq 5 ]]; then
        echo -e "${RED}WARNING: Health check failed after rollback${NC}"
        echo "Consider investigating or rolling forward"
        exit 1
    fi

    echo "Attempt ${i}: HTTP ${HTTP_STATUS}, retrying..."
    sleep 5
done

echo ""
echo "Current serving revision:"
gcloud run services describe "${FULL_SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --format='value(status.latestReadyRevisionName)'
