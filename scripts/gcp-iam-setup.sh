#!/bin/bash
# LLM-CostOps IAM Setup Script
# Creates service account and assigns minimal required permissions
#
# Usage: ./scripts/gcp-iam-setup.sh

set -euo pipefail

PROJECT_ID="agentics-dev"
SERVICE_ACCOUNT_NAME="llm-costops-sa"
SERVICE_ACCOUNT_EMAIL="${SERVICE_ACCOUNT_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"

echo "========================================="
echo "LLM-CostOps IAM Setup"
echo "========================================="
echo "Project: ${PROJECT_ID}"
echo "Service Account: ${SERVICE_ACCOUNT_EMAIL}"
echo ""

# Create service account if it doesn't exist
echo "Step 1: Creating service account..."
if gcloud iam service-accounts describe "${SERVICE_ACCOUNT_EMAIL}" --project="${PROJECT_ID}" &>/dev/null; then
    echo "Service account already exists"
else
    gcloud iam service-accounts create "${SERVICE_ACCOUNT_NAME}" \
        --project="${PROJECT_ID}" \
        --display-name="LLM-CostOps Service Account" \
        --description="Service account for LLM-CostOps Cloud Run services"
    echo "Service account created"
fi

echo ""
echo "Step 2: Assigning IAM roles (least privilege)..."

# Required roles for Cloud Run operation
ROLES=(
    # Cloud Run invoker (for service-to-service calls)
    "roles/run.invoker"

    # Secret Manager access (for API keys)
    "roles/secretmanager.secretAccessor"

    # Cloud Logging (for structured logs)
    "roles/logging.logWriter"

    # Cloud Monitoring (for metrics)
    "roles/monitoring.metricWriter"

    # Cloud Trace (for distributed tracing)
    "roles/cloudtrace.agent"

    # VPC Access (for internal networking)
    "roles/vpcaccess.user"
)

for ROLE in "${ROLES[@]}"; do
    echo "  Assigning ${ROLE}..."
    gcloud projects add-iam-policy-binding "${PROJECT_ID}" \
        --member="serviceAccount:${SERVICE_ACCOUNT_EMAIL}" \
        --role="${ROLE}" \
        --condition=None \
        --quiet
done

echo ""
echo "Step 3: Creating secrets in Secret Manager..."

# Check/create secrets (values must be set manually or via Terraform)
SECRETS=(
    "ruvector-api-key"
    "observatory-api-key"
)

for SECRET in "${SECRETS[@]}"; do
    if gcloud secrets describe "${SECRET}" --project="${PROJECT_ID}" &>/dev/null; then
        echo "  Secret '${SECRET}' already exists"
    else
        echo "  Creating secret '${SECRET}' (placeholder)..."
        echo "PLACEHOLDER_VALUE" | gcloud secrets create "${SECRET}" \
            --project="${PROJECT_ID}" \
            --replication-policy="automatic" \
            --data-file=-
        echo "  WARNING: Update secret '${SECRET}' with actual value"
    fi
done

echo ""
echo "Step 4: Granting secret access to service account..."

for SECRET in "${SECRETS[@]}"; do
    echo "  Granting access to ${SECRET}..."
    gcloud secrets add-iam-policy-binding "${SECRET}" \
        --project="${PROJECT_ID}" \
        --member="serviceAccount:${SERVICE_ACCOUNT_EMAIL}" \
        --role="roles/secretmanager.secretAccessor" \
        --quiet
done

echo ""
echo "========================================="
echo "IAM Setup Complete"
echo "========================================="
echo ""
echo "Service Account: ${SERVICE_ACCOUNT_EMAIL}"
echo ""
echo "Assigned Roles:"
for ROLE in "${ROLES[@]}"; do
    echo "  - ${ROLE}"
done
echo ""
echo "IMPORTANT: Update the following secrets with actual values:"
for SECRET in "${SECRETS[@]}"; do
    echo "  - ${SECRET}"
done
echo ""
echo "To update a secret:"
echo "  echo 'actual-value' | gcloud secrets versions add SECRET_NAME --data-file=-"
