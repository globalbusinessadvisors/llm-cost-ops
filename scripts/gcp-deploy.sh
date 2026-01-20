#!/bin/bash
# LLM-CostOps Google Cloud Run Deployment Script
# Usage: ./scripts/gcp-deploy.sh <environment> [version]
#
# Environments: dev, staging, prod
# Version: git tag or commit SHA (default: current commit)

set -euo pipefail

# Configuration
PROJECT_ID="agentics-dev"
REGION="us-central1"
SERVICE_NAME="llm-costops"
ARTIFACT_REGISTRY="us-central1-docker.pkg.dev/${PROJECT_ID}/llm-devops"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
ENVIRONMENT="${1:-}"
VERSION="${2:-$(git rev-parse --short HEAD)}"

if [[ -z "$ENVIRONMENT" ]]; then
    echo -e "${RED}Error: Environment required${NC}"
    echo "Usage: $0 <environment> [version]"
    echo "Environments: dev, staging, prod"
    exit 1
fi

if [[ ! "$ENVIRONMENT" =~ ^(dev|staging|prod)$ ]]; then
    echo -e "${RED}Error: Invalid environment '${ENVIRONMENT}'${NC}"
    echo "Valid environments: dev, staging, prod"
    exit 1
fi

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}LLM-CostOps Deployment${NC}"
echo -e "${GREEN}========================================${NC}"
echo "Environment: ${ENVIRONMENT}"
echo "Version: ${VERSION}"
echo "Project: ${PROJECT_ID}"
echo "Region: ${REGION}"
echo ""

# Confirm production deployment
if [[ "$ENVIRONMENT" == "prod" ]]; then
    echo -e "${YELLOW}WARNING: You are deploying to PRODUCTION${NC}"
    read -p "Are you sure you want to continue? (yes/no) " -r
    if [[ ! $REPLY == "yes" ]]; then
        echo "Deployment cancelled"
        exit 0
    fi
fi

# Set service name with environment suffix
if [[ "$ENVIRONMENT" != "prod" ]]; then
    FULL_SERVICE_NAME="${SERVICE_NAME}-${ENVIRONMENT}"
else
    FULL_SERVICE_NAME="${SERVICE_NAME}"
fi

# Load environment configuration
ENV_FILE="deployment/gcp/env/${ENVIRONMENT}.env"
if [[ ! -f "$ENV_FILE" ]]; then
    echo -e "${RED}Error: Environment file not found: ${ENV_FILE}${NC}"
    exit 1
fi

echo -e "${GREEN}Step 1: Building Docker image...${NC}"
docker build \
    -t "${ARTIFACT_REGISTRY}/${SERVICE_NAME}:${VERSION}" \
    -t "${ARTIFACT_REGISTRY}/${SERVICE_NAME}:${ENVIRONMENT}-latest" \
    -f deployment/gcp/Dockerfile.cloudrun \
    .

echo -e "${GREEN}Step 2: Pushing to Artifact Registry...${NC}"
docker push "${ARTIFACT_REGISTRY}/${SERVICE_NAME}:${VERSION}"
docker push "${ARTIFACT_REGISTRY}/${SERVICE_NAME}:${ENVIRONMENT}-latest"

echo -e "${GREEN}Step 3: Deploying to Cloud Run...${NC}"

# Build environment variables string
ENV_VARS="PLATFORM_ENV=${ENVIRONMENT}"
ENV_VARS="${ENV_VARS},SERVICE_NAME=${SERVICE_NAME}"
ENV_VARS="${ENV_VARS},SERVICE_VERSION=${VERSION}"

# Set memory and CPU based on environment
case "$ENVIRONMENT" in
    dev)
        MEMORY="512Mi"
        CPU="1"
        MIN_INSTANCES="0"
        MAX_INSTANCES="10"
        ;;
    staging)
        MEMORY="1Gi"
        CPU="2"
        MIN_INSTANCES="1"
        MAX_INSTANCES="20"
        ;;
    prod)
        MEMORY="2Gi"
        CPU="2"
        MIN_INSTANCES="2"
        MAX_INSTANCES="100"
        ;;
esac

# Deploy to Cloud Run
gcloud run deploy "${FULL_SERVICE_NAME}" \
    --image "${ARTIFACT_REGISTRY}/${SERVICE_NAME}:${VERSION}" \
    --region "${REGION}" \
    --project "${PROJECT_ID}" \
    --platform managed \
    --no-allow-unauthenticated \
    --service-account "llm-costops-sa@${PROJECT_ID}.iam.gserviceaccount.com" \
    --set-env-vars "${ENV_VARS}" \
    --set-secrets "RUVECTOR_API_KEY=ruvector-api-key:latest,TELEMETRY_API_KEY=observatory-api-key:latest" \
    --memory "${MEMORY}" \
    --cpu "${CPU}" \
    --min-instances "${MIN_INSTANCES}" \
    --max-instances "${MAX_INSTANCES}" \
    --timeout "60s" \
    --concurrency "100" \
    --ingress "internal" \
    --vpc-connector "agentics-vpc-connector" \
    --quiet

echo -e "${GREEN}Step 4: Verifying deployment...${NC}"

# Get service URL
SERVICE_URL=$(gcloud run services describe "${FULL_SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --format='value(status.url)')

echo "Service URL: ${SERVICE_URL}"

# Get identity token for internal service
TOKEN=$(gcloud auth print-identity-token)

# Health check
echo "Running health check..."
for i in {1..10}; do
    HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" \
        -H "Authorization: Bearer ${TOKEN}" \
        "${SERVICE_URL}/health" 2>/dev/null || echo "000")

    if [[ "$HTTP_STATUS" == "200" ]]; then
        echo -e "${GREEN}Health check passed!${NC}"
        break
    fi

    if [[ $i -eq 10 ]]; then
        echo -e "${RED}Health check failed after 10 attempts${NC}"
        exit 1
    fi

    echo "Attempt ${i}: HTTP ${HTTP_STATUS}, retrying in 5s..."
    sleep 5
done

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Deployment Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo "Service: ${FULL_SERVICE_NAME}"
echo "Version: ${VERSION}"
echo "URL: ${SERVICE_URL}"
echo ""
echo "Agent Endpoints:"
echo "  - Health:            ${SERVICE_URL}/health"
echo "  - Cost Attribution:  ${SERVICE_URL}/api/v1/agents/cost-attribution"
echo "  - Cost Forecasting:  ${SERVICE_URL}/api/v1/agents/cost-forecasting"
echo "  - Budget Enforcement: ${SERVICE_URL}/api/v1/agents/budget-enforcement"
echo "  - ROI Estimation:    ${SERVICE_URL}/api/v1/agents/roi-estimation"
echo "  - Cost-Performance:  ${SERVICE_URL}/api/v1/agents/cost-performance"
