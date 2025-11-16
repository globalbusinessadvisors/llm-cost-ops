#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Kubernetes Deployment Script
# =============================================================================
# Description: Deploy to Kubernetes with namespace management, secret handling,
#              and health verification using kubectl and kustomize
# Usage: ./deploy-k8s.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
K8S_DIR="${K8S_DIR:-${PROJECT_ROOT}/k8s}"
NAMESPACE="${NAMESPACE:-llm-cost-ops}"
ENVIRONMENT="${ENVIRONMENT:-dev}"
OVERLAY_DIR="${K8S_DIR}/overlays/${ENVIRONMENT}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
WAIT_TIMEOUT="${WAIT_TIMEOUT:-300s}"
DRY_RUN="${DRY_RUN:-false}"
SKIP_SECRETS="${SKIP_SECRETS:-false}"
CREATE_NAMESPACE="${CREATE_NAMESPACE:-true}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

show_usage() {
    cat << EOF
Usage: ${0##*/} [OPTIONS]

Deploy LLM Cost Ops to Kubernetes cluster.

OPTIONS:
    -h, --help              Show this help message
    -n, --namespace NS      Kubernetes namespace (default: ${NAMESPACE})
    -e, --env ENV           Environment: dev, staging, prod (default: ${ENVIRONMENT})
    -t, --tag TAG           Image tag to deploy (default: ${IMAGE_TAG})
    -k, --kubeconfig FILE   Path to kubeconfig file
    --skip-secrets          Skip secret creation
    --no-create-ns          Don't create namespace if missing
    --wait-timeout DURATION Timeout for rollout (default: ${WAIT_TIMEOUT})
    --dry-run               Print actions without executing

EXAMPLES:
    # Deploy to development
    ${0##*/} --env dev

    # Deploy to production with specific tag
    ${0##*/} --env prod --tag v1.2.3

    # Deploy to custom namespace
    ${0##*/} --namespace my-namespace --env staging

    # Dry run
    ${0##*/} --dry-run --env prod

REQUIREMENTS:
    - kubectl installed and configured
    - kustomize installed (or kubectl with kustomize support)
    - Access to target cluster
    - Required secrets configured

ENVIRONMENT VARIABLES:
    NAMESPACE           Kubernetes namespace
    ENVIRONMENT         Deployment environment
    IMAGE_TAG           Image tag to deploy
    KUBECONFIG          Path to kubeconfig
    DRY_RUN             Dry run mode (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is not installed"
        exit 1
    fi

    # Check kubectl version
    local kubectl_version
    kubectl_version=$(kubectl version --client -o json 2>/dev/null | jq -r '.clientVersion.gitVersion' || echo "unknown")
    log_info "kubectl version: ${kubectl_version}"

    # Check for kustomize (standalone or kubectl kustomize)
    if command -v kustomize &> /dev/null; then
        KUSTOMIZE_CMD="kustomize build"
        local kustomize_version
        kustomize_version=$(kustomize version --short 2>/dev/null || echo "unknown")
        log_info "kustomize version: ${kustomize_version}"
    elif kubectl kustomize --help &> /dev/null; then
        KUSTOMIZE_CMD="kubectl kustomize"
        log_info "Using kubectl kustomize"
    else
        log_error "kustomize is not available"
        log_info "Install with: curl -s \"https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh\" | bash"
        exit 1
    fi

    # Check cluster connectivity
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        log_info "Configure with: kubectl config use-context <context>"
        exit 1
    fi

    local current_context
    current_context=$(kubectl config current-context)
    log_info "Current cluster: ${current_context}"

    # Verify overlay directory exists
    if [[ ! -d "${OVERLAY_DIR}" ]]; then
        log_error "Overlay directory not found: ${OVERLAY_DIR}"
        exit 1
    fi

    log_success "Prerequisites check passed"
}

confirm_production_deployment() {
    if [[ "${ENVIRONMENT}" == "prod" ]] || [[ "${ENVIRONMENT}" == "production" ]]; then
        log_warn "You are about to deploy to PRODUCTION!"
        log_info "Namespace: ${NAMESPACE}"
        log_info "Image tag: ${IMAGE_TAG}"
        log_info "Cluster: $(kubectl config current-context)"
        echo ""
        read -p "Are you sure you want to continue? (yes/no): " -r
        echo

        if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
            log_info "Deployment cancelled"
            exit 0
        fi
    fi
}

create_namespace() {
    if [[ "${CREATE_NAMESPACE}" != "true" ]]; then
        log_info "Skipping namespace creation"
        return 0
    fi

    log_info "Checking namespace: ${NAMESPACE}"

    if kubectl get namespace "${NAMESPACE}" &> /dev/null; then
        log_info "Namespace already exists: ${NAMESPACE}"
        return 0
    fi

    log_info "Creating namespace: ${NAMESPACE}"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would create namespace: ${NAMESPACE}"
        return 0
    fi

    kubectl create namespace "${NAMESPACE}" || {
        log_error "Failed to create namespace"
        return 1
    }

    # Label namespace
    kubectl label namespace "${NAMESPACE}" \
        environment="${ENVIRONMENT}" \
        app=llm-cost-ops \
        --overwrite

    log_success "Namespace created: ${NAMESPACE}"
}

setup_secrets() {
    if [[ "${SKIP_SECRETS}" == "true" ]]; then
        log_info "Skipping secret creation"
        return 0
    fi

    log_info "Setting up secrets..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would setup secrets"
        return 0
    fi

    # Check if secrets already exist
    if kubectl get secret llm-cost-ops-secrets -n "${NAMESPACE}" &> /dev/null; then
        log_info "Secrets already exist"
        return 0
    fi

    # Create secrets from environment file or prompted values
    local env_file="${PROJECT_ROOT}/.env.${ENVIRONMENT}"

    if [[ -f "${env_file}" ]]; then
        log_info "Creating secrets from: ${env_file}"

        # Extract required secrets
        local database_url
        local jwt_secret
        local redis_url

        database_url=$(grep '^DATABASE_URL=' "${env_file}" | cut -d= -f2- || echo "")
        jwt_secret=$(grep '^JWT_SECRET=' "${env_file}" | cut -d= -f2- || echo "")
        redis_url=$(grep '^REDIS_URL=' "${env_file}" | cut -d= -f2- || echo "")

        if [[ -z "${jwt_secret}" ]] || [[ "${jwt_secret}" == *"dev-secret"* ]]; then
            log_error "JWT_SECRET is not set or using development value"
            log_info "Please set a secure JWT_SECRET in ${env_file}"
            return 1
        fi

        kubectl create secret generic llm-cost-ops-secrets \
            -n "${NAMESPACE}" \
            --from-literal=database-url="${database_url:-postgresql://user:pass@postgres:5432/llm_cost_ops}" \
            --from-literal=jwt-secret="${jwt_secret}" \
            --from-literal=redis-url="${redis_url:-redis://redis:6379/0}" \
            --dry-run=client -o yaml | kubectl apply -f -

        log_success "Secrets created"
    else
        log_warn "Environment file not found: ${env_file}"
        log_info "Please create secrets manually:"
        echo "  kubectl create secret generic llm-cost-ops-secrets -n ${NAMESPACE} \\"
        echo "    --from-literal=database-url=... \\"
        echo "    --from-literal=jwt-secret=... \\"
        echo "    --from-literal=redis-url=..."
    fi
}

update_image_tag() {
    log_info "Updating image tag to: ${IMAGE_TAG}"

    local kustomization_file="${OVERLAY_DIR}/kustomization.yaml"

    if [[ ! -f "${kustomization_file}" ]]; then
        log_error "Kustomization file not found: ${kustomization_file}"
        return 1
    fi

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would update image tag"
        return 0
    fi

    # Update image tag in kustomization.yaml
    cd "${OVERLAY_DIR}"
    kustomize edit set image "llm-cost-ops=ghcr.io/yourusername/llm-cost-ops:${IMAGE_TAG}" || {
        log_error "Failed to update image tag"
        return 1
    }
    cd - > /dev/null

    log_success "Image tag updated"
}

apply_manifests() {
    log_info "Applying Kubernetes manifests..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would apply manifests"
        log_info "Generated manifests:"
        $KUSTOMIZE_CMD "${OVERLAY_DIR}"
        return 0
    fi

    # Build and apply with kustomize
    $KUSTOMIZE_CMD "${OVERLAY_DIR}" | kubectl apply -n "${NAMESPACE}" -f - || {
        log_error "Failed to apply manifests"
        return 1
    }

    log_success "Manifests applied"
}

wait_for_rollout() {
    log_info "Waiting for deployment rollout..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would wait for rollout"
        return 0
    fi

    # Wait for deployment
    if kubectl rollout status deployment/llm-cost-ops \
        -n "${NAMESPACE}" \
        --timeout="${WAIT_TIMEOUT}"; then
        log_success "Deployment rolled out successfully"
        return 0
    else
        log_error "Deployment rollout failed or timed out"
        return 1
    fi
}

verify_deployment() {
    log_info "Verifying deployment..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would verify deployment"
        return 0
    fi

    # Check pods
    log_info "Checking pods..."
    kubectl get pods -n "${NAMESPACE}" -l app=llm-cost-ops

    # Check if pods are running
    local ready_pods
    ready_pods=$(kubectl get pods -n "${NAMESPACE}" -l app=llm-cost-ops \
        -o jsonpath='{.items[?(@.status.phase=="Running")].metadata.name}' | wc -w)

    if [[ ${ready_pods} -eq 0 ]]; then
        log_error "No pods are running"
        log_info "Pod details:"
        kubectl describe pods -n "${NAMESPACE}" -l app=llm-cost-ops
        return 1
    fi

    log_info "Running pods: ${ready_pods}"

    # Check services
    log_info "Checking services..."
    kubectl get services -n "${NAMESPACE}" -l app=llm-cost-ops

    # Check ingress
    if kubectl get ingress -n "${NAMESPACE}" llm-cost-ops &> /dev/null; then
        log_info "Checking ingress..."
        kubectl get ingress -n "${NAMESPACE}" llm-cost-ops
    fi

    log_success "Deployment verified"
}

run_health_checks() {
    log_info "Running health checks..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would run health checks"
        return 0
    fi

    # Get a pod name
    local pod
    pod=$(kubectl get pods -n "${NAMESPACE}" -l app=llm-cost-ops \
        -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")

    if [[ -z "${pod}" ]]; then
        log_warn "No pods found for health check"
        return 1
    fi

    log_info "Testing health endpoint on pod: ${pod}"

    # Port-forward and test (with timeout)
    kubectl port-forward -n "${NAMESPACE}" "${pod}" 8080:8080 &
    local pf_pid=$!
    sleep 3

    if curl -sf http://localhost:8080/health >/dev/null 2>&1; then
        log_success "Health check passed"
        kill ${pf_pid} 2>/dev/null || true
        return 0
    else
        log_warn "Health check failed (may not be exposed)"
        kill ${pf_pid} 2>/dev/null || true
        return 0  # Don't fail deployment for this
    fi
}

show_deployment_info() {
    cat << EOF

${GREEN}╔════════════════════════════════════════════════════════════════╗
║              KUBERNETES DEPLOYMENT COMPLETE                    ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${CYAN}Environment:${NC}    ${ENVIRONMENT}
  ${CYAN}Namespace:${NC}      ${NAMESPACE}
  ${CYAN}Image Tag:${NC}      ${IMAGE_TAG}
  ${CYAN}Cluster:${NC}        $(kubectl config current-context)

  ${CYAN}Useful Commands:${NC}
    View pods:         kubectl get pods -n ${NAMESPACE}
    View logs:         kubectl logs -n ${NAMESPACE} -l app=llm-cost-ops -f
    Describe pod:      kubectl describe pod -n ${NAMESPACE} <pod-name>
    Port forward:      kubectl port-forward -n ${NAMESPACE} svc/llm-cost-ops 8080:8080
    Scale:             kubectl scale deployment/llm-cost-ops -n ${NAMESPACE} --replicas=3
    Delete:            kubectl delete -k ${OVERLAY_DIR} -n ${NAMESPACE}

  ${CYAN}Monitoring:${NC}
    Dashboard:         kubectl proxy
                       http://localhost:8001/api/v1/namespaces/kubernetes-dashboard/services/https:kubernetes-dashboard:/proxy/

EOF

    # Show service endpoints if available
    local service_ip
    service_ip=$(kubectl get svc llm-cost-ops -n "${NAMESPACE}" \
        -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")

    if [[ -n "${service_ip}" ]]; then
        echo -e "  ${CYAN}Service IP:${NC}     ${service_ip}"
    fi

    # Show ingress hostname if available
    local ingress_host
    ingress_host=$(kubectl get ingress llm-cost-ops -n "${NAMESPACE}" \
        -o jsonpath='{.spec.rules[0].host}' 2>/dev/null || echo "")

    if [[ -n "${ingress_host}" ]]; then
        echo -e "  ${CYAN}Ingress Host:${NC}   https://${ingress_host}"
    fi

    echo ""
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -n|--namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            -e|--env)
                ENVIRONMENT="$2"
                OVERLAY_DIR="${K8S_DIR}/overlays/${ENVIRONMENT}"
                shift 2
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            -k|--kubeconfig)
                export KUBECONFIG="$2"
                shift 2
                ;;
            --skip-secrets)
                SKIP_SECRETS="true"
                shift
                ;;
            --no-create-ns)
                CREATE_NAMESPACE="false"
                shift
                ;;
            --wait-timeout)
                WAIT_TIMEOUT="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN="true"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    log_info "Starting Kubernetes deployment"
    log_info "Environment: ${ENVIRONMENT}"
    log_info "Namespace: ${NAMESPACE}"

    # Execute deployment steps
    check_prerequisites || exit 1
    confirm_production_deployment
    create_namespace || exit 1
    setup_secrets || exit 1
    update_image_tag || exit 1
    apply_manifests || exit 1
    wait_for_rollout || exit 1
    verify_deployment || exit 1
    run_health_checks

    log_success "Deployment completed successfully!"
    show_deployment_info

    exit 0
}

# Run main function
main "$@"
