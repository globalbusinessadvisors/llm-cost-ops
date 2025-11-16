#!/bin/bash
# Kubernetes Manifests Validation Script
# This script validates all Kubernetes manifests using kustomize and kubectl

set -e

echo "========================================="
echo "LLM Cost Ops - Kubernetes Validation"
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo "Checking prerequisites..."
if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}ERROR: kubectl not found. Please install kubectl.${NC}"
    exit 1
fi

if ! kubectl version --client &> /dev/null; then
    echo -e "${RED}ERROR: kubectl not properly configured.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ kubectl found${NC}"

# Check kustomize
if ! kubectl kustomize --help &> /dev/null; then
    echo -e "${YELLOW}WARNING: kustomize not found in kubectl. Install kustomize for full validation.${NC}"
fi

echo ""

# Function to validate manifests
validate_overlay() {
    local overlay=$1
    local path="overlays/${overlay}"

    echo "----------------------------------------"
    echo "Validating ${overlay} overlay..."
    echo "----------------------------------------"

    # Build with kustomize
    if kubectl kustomize "k8s/${path}" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Kustomize build successful${NC}"
    else
        echo -e "${RED}✗ Kustomize build failed${NC}"
        return 1
    fi

    # Dry-run validation
    if kubectl kustomize "k8s/${path}" | kubectl apply --dry-run=client -f - > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Dry-run validation successful${NC}"
    else
        echo -e "${RED}✗ Dry-run validation failed${NC}"
        kubectl kustomize "k8s/${path}" | kubectl apply --dry-run=client -f - || true
        return 1
    fi

    # Count resources
    local resource_count=$(kubectl kustomize "k8s/${path}" | grep -c "^kind:" || echo "0")
    echo -e "${GREEN}✓ Generated ${resource_count} Kubernetes resources${NC}"

    echo ""
}

# Validate base
echo "========================================="
echo "Validating Base Configuration"
echo "========================================="
if kubectl kustomize "k8s/base" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Base configuration is valid${NC}"
    base_resources=$(kubectl kustomize "k8s/base" | grep -c "^kind:" || echo "0")
    echo -e "${GREEN}✓ Base has ${base_resources} resources${NC}"
else
    echo -e "${RED}✗ Base configuration has errors${NC}"
    kubectl kustomize "k8s/base" || true
    exit 1
fi
echo ""

# Validate database
echo "========================================="
echo "Validating Database Configuration"
echo "========================================="
if kubectl kustomize "k8s/database" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Database configuration is valid${NC}"
    db_resources=$(kubectl kustomize "k8s/database" | grep -c "^kind:" || echo "0")
    echo -e "${GREEN}✓ Database has ${db_resources} resources${NC}"
else
    echo -e "${RED}✗ Database configuration has errors${NC}"
    kubectl kustomize "k8s/database" || true
    exit 1
fi
echo ""

# Validate monitoring
echo "========================================="
echo "Validating Monitoring Configuration"
echo "========================================="
if kubectl kustomize "k8s/monitoring" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Monitoring configuration is valid${NC}"
    mon_resources=$(kubectl kustomize "k8s/monitoring" | grep -c "^kind:" || echo "0")
    echo -e "${GREEN}✓ Monitoring has ${mon_resources} resources${NC}"
else
    echo -e "${RED}✗ Monitoring configuration has errors${NC}"
    kubectl kustomize "k8s/monitoring" || true
    exit 1
fi
echo ""

# Validate overlays
echo "========================================="
echo "Validating Overlays"
echo "========================================="
echo ""

validate_overlay "dev" || exit 1
validate_overlay "staging" || exit 1
validate_overlay "prod" || exit 1

# Summary
echo "========================================="
echo "Validation Summary"
echo "========================================="
echo -e "${GREEN}✓ All configurations are valid!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review configurations in k8s/overlays/"
echo "  2. Update secrets with actual values"
echo "  3. Deploy to your cluster:"
echo "     kubectl apply -k k8s/overlays/dev/"
echo ""
echo -e "${YELLOW}Note: Remember to update secrets before deploying to production!${NC}"
echo ""
