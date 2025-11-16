#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Helm Deployment Script
# =============================================================================
# Description: Deploy to Kubernetes using Helm with chart management,
#              values file selection, and release management
# Usage: ./deploy-helm.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CHART_DIR="${CHART_DIR:-${PROJECT_ROOT}/helm/llm-cost-ops}"
RELEASE_NAME="${RELEASE_NAME:-llm-cost-ops}"
NAMESPACE="${NAMESPACE:-llm-cost-ops}"
ENVIRONMENT="${ENVIRONMENT:-dev}"
VALUES_FILE="${VALUES_FILE:-}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
ACTION="${ACTION:-install}"
WAIT="${WAIT:-true}"
TIMEOUT="${TIMEOUT:-5m}"
ATOMIC="${ATOMIC:-true}"
CREATE_NAMESPACE="${CREATE_NAMESPACE:-true}"
DRY_RUN="${DRY_RUN:-false}"

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
Usage: ${0##*/} [OPTIONS] [ACTION]

Deploy LLM Cost Ops using Helm.

ACTIONS:
    install         Install a new release (default)
    upgrade         Upgrade an existing release
    uninstall       Uninstall a release
    rollback        Rollback to previous release
    status          Show release status
    test            Run helm tests

OPTIONS:
    -h, --help              Show this help message
    -r, --release NAME      Release name (default: ${RELEASE_NAME})
    -n, --namespace NS      Kubernetes namespace (default: ${NAMESPACE})
    -e, --env ENV           Environment: dev, staging, prod (default: ${ENVIRONMENT})
    -f, --values FILE       Additional values file
    -t, --tag TAG           Image tag to deploy (default: ${IMAGE_TAG})
    --set KEY=VALUE         Set values on the command line
    --no-wait               Don't wait for deployment to complete
    --no-atomic             Don't rollback on failure
    --timeout DURATION      Timeout for operations (default: ${TIMEOUT})
    --dry-run               Simulate deployment

EXAMPLES:
    # Install with default values
    ${0##*/} install

    # Upgrade with specific tag
    ${0##*/} upgrade --tag v1.2.3

    # Install to production
    ${0##*/} install --env prod --tag v1.0.0

    # Install with custom values
    ${0##*/} install --values custom-values.yaml

    # Uninstall release
    ${0##*/} uninstall

    # Rollback to previous revision
    ${0##*/} rollback

    # Dry run
    ${0##*/} install --dry-run

ENVIRONMENT VARIABLES:
    CHART_DIR           Helm chart directory
    RELEASE_NAME        Helm release name
    NAMESPACE           Kubernetes namespace
    ENVIRONMENT         Deployment environment
    IMAGE_TAG           Image tag to deploy
    DRY_RUN             Dry run mode (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for helm
    if ! command -v helm &> /dev/null; then
        log_error "Helm is not installed"
        log_info "Install from: https://helm.sh/docs/intro/install/"
        exit 1
    fi

    # Check helm version
    local helm_version
    helm_version=$(helm version --short 2>/dev/null || echo "unknown")
    log_info "Helm version: ${helm_version}"

    # Check for kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is not installed"
        exit 1
    fi

    # Check cluster connectivity
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi

    local current_context
    current_context=$(kubectl config current-context)
    log_info "Current cluster: ${current_context}"

    # Check if chart exists
    if [[ ! -f "${CHART_DIR}/Chart.yaml" ]]; then
        log_error "Helm chart not found: ${CHART_DIR}/Chart.yaml"
        exit 1
    fi

    log_success "Prerequisites check passed"
}

validate_chart() {
    log_info "Validating Helm chart..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Skipping chart validation"
        return 0
    fi

    # Lint chart
    if helm lint "${CHART_DIR}" >/dev/null 2>&1; then
        log_success "Chart validation passed"
        return 0
    else
        log_error "Chart validation failed"
        helm lint "${CHART_DIR}"
        return 1
    fi
}

select_values_file() {
    # If values file explicitly specified, use it
    if [[ -n "${VALUES_FILE}" ]]; then
        if [[ ! -f "${VALUES_FILE}" ]]; then
            log_error "Values file not found: ${VALUES_FILE}"
            exit 1
        fi
        log_info "Using values file: ${VALUES_FILE}"
        return 0
    fi

    # Otherwise, select based on environment
    local env_values="${CHART_DIR}/values-${ENVIRONMENT}.yaml"

    if [[ -f "${env_values}" ]]; then
        VALUES_FILE="${env_values}"
        log_info "Using environment values: ${VALUES_FILE}"
    else
        VALUES_FILE="${CHART_DIR}/values.yaml"
        log_info "Using default values: ${VALUES_FILE}"
    fi
}

build_helm_command() {
    local action="$1"
    local -a helm_cmd=("helm" "${action}")

    case "${action}" in
        install|upgrade)
            helm_cmd+=("${RELEASE_NAME}" "${CHART_DIR}")

            # Add namespace
            helm_cmd+=("--namespace" "${NAMESPACE}")

            # Create namespace if needed
            if [[ "${CREATE_NAMESPACE}" == "true" ]] && [[ "${action}" == "install" ]]; then
                helm_cmd+=("--create-namespace")
            fi

            # Add values file
            if [[ -n "${VALUES_FILE}" ]]; then
                helm_cmd+=("--values" "${VALUES_FILE}")
            fi

            # Add image tag
            helm_cmd+=("--set" "image.tag=${IMAGE_TAG}")

            # Add custom set values
            for set_value in "${SET_VALUES[@]}"; do
                helm_cmd+=("--set" "${set_value}")
            done

            # Add wait flag
            if [[ "${WAIT}" == "true" ]]; then
                helm_cmd+=("--wait")
            fi

            # Add timeout
            helm_cmd+=("--timeout" "${TIMEOUT}")

            # Add atomic flag for upgrades
            if [[ "${ATOMIC}" == "true" ]]; then
                helm_cmd+=("--atomic")
            fi

            # Install or upgrade specific flags
            if [[ "${action}" == "upgrade" ]]; then
                helm_cmd+=("--install")  # Install if not exists
                helm_cmd+=("--cleanup-on-fail")
            fi

            # Add dry-run flag
            if [[ "${DRY_RUN}" == "true" ]]; then
                helm_cmd+=("--dry-run" "--debug")
            fi
            ;;

        uninstall)
            helm_cmd+=("${RELEASE_NAME}")
            helm_cmd+=("--namespace" "${NAMESPACE}")

            if [[ "${WAIT}" == "true" ]]; then
                helm_cmd+=("--wait")
            fi
            ;;

        rollback)
            helm_cmd+=("${RELEASE_NAME}")
            helm_cmd+=("--namespace" "${NAMESPACE}")

            if [[ "${WAIT}" == "true" ]]; then
                helm_cmd+=("--wait")
            fi

            helm_cmd+=("--timeout" "${TIMEOUT}")

            if [[ "${DRY_RUN}" == "true" ]]; then
                helm_cmd+=("--dry-run")
            fi
            ;;

        status|test)
            helm_cmd+=("${RELEASE_NAME}")
            helm_cmd+=("--namespace" "${NAMESPACE}")
            ;;
    esac

    echo "${helm_cmd[@]}"
}

helm_install() {
    log_info "Installing Helm release: ${RELEASE_NAME}"

    local helm_cmd
    helm_cmd=$(build_helm_command "install")

    log_info "Executing: ${helm_cmd}"

    if eval "${helm_cmd}"; then
        log_success "Release installed successfully"
        return 0
    else
        log_error "Installation failed"
        return 1
    fi
}

helm_upgrade() {
    log_info "Upgrading Helm release: ${RELEASE_NAME}"

    # Check if release exists
    if ! helm status "${RELEASE_NAME}" -n "${NAMESPACE}" &> /dev/null; then
        log_info "Release not found, will install instead"
    fi

    local helm_cmd
    helm_cmd=$(build_helm_command "upgrade")

    log_info "Executing: ${helm_cmd}"

    if eval "${helm_cmd}"; then
        log_success "Release upgraded successfully"
        return 0
    else
        log_error "Upgrade failed"
        return 1
    fi
}

helm_uninstall() {
    log_info "Uninstalling Helm release: ${RELEASE_NAME}"

    # Confirm uninstall for production
    if [[ "${ENVIRONMENT}" == "prod" ]] || [[ "${ENVIRONMENT}" == "production" ]]; then
        log_warn "You are about to uninstall from PRODUCTION!"
        read -p "Are you sure? (yes/no): " -r
        echo
        if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
            log_info "Uninstall cancelled"
            return 0
        fi
    fi

    local helm_cmd
    helm_cmd=$(build_helm_command "uninstall")

    log_info "Executing: ${helm_cmd}"

    if eval "${helm_cmd}"; then
        log_success "Release uninstalled successfully"
        return 0
    else
        log_error "Uninstall failed"
        return 1
    fi
}

helm_rollback() {
    log_info "Rolling back Helm release: ${RELEASE_NAME}"

    # Get current revision
    local current_revision
    current_revision=$(helm status "${RELEASE_NAME}" -n "${NAMESPACE}" -o json 2>/dev/null | \
        jq -r '.version' || echo "0")

    if [[ "${current_revision}" == "0" ]] || [[ "${current_revision}" == "1" ]]; then
        log_error "Cannot rollback: no previous revision available"
        return 1
    fi

    log_info "Current revision: ${current_revision}"
    log_info "Rolling back to revision: $((current_revision - 1))"

    local helm_cmd
    helm_cmd=$(build_helm_command "rollback")
    helm_cmd="${helm_cmd} $((current_revision - 1))"

    log_info "Executing: ${helm_cmd}"

    if eval "${helm_cmd}"; then
        log_success "Rollback completed successfully"
        return 0
    else
        log_error "Rollback failed"
        return 1
    fi
}

helm_status() {
    log_info "Checking release status: ${RELEASE_NAME}"

    local helm_cmd
    helm_cmd=$(build_helm_command "status")

    eval "${helm_cmd}"
}

helm_test() {
    log_info "Running Helm tests: ${RELEASE_NAME}"

    local helm_cmd
    helm_cmd=$(build_helm_command "test")

    if eval "${helm_cmd}"; then
        log_success "Tests passed"
        return 0
    else
        log_error "Tests failed"
        return 1
    fi
}

show_release_info() {
    log_info "Release Information:"

    # Get release info
    local release_info
    release_info=$(helm list -n "${NAMESPACE}" -o json | \
        jq -r ".[] | select(.name==\"${RELEASE_NAME}\")")

    if [[ -n "${release_info}" ]]; then
        local status chart revision updated
        status=$(echo "${release_info}" | jq -r '.status')
        chart=$(echo "${release_info}" | jq -r '.chart')
        revision=$(echo "${release_info}" | jq -r '.revision')
        updated=$(echo "${release_info}" | jq -r '.updated')

        echo ""
        echo "  Release:    ${RELEASE_NAME}"
        echo "  Namespace:  ${NAMESPACE}"
        echo "  Status:     ${status}"
        echo "  Chart:      ${chart}"
        echo "  Revision:   ${revision}"
        echo "  Updated:    ${updated}"
        echo ""
    else
        log_warn "Release not found: ${RELEASE_NAME}"
    fi
}

show_deployment_summary() {
    cat << EOF

${GREEN}╔════════════════════════════════════════════════════════════════╗
║                 HELM DEPLOYMENT COMPLETE                       ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${CYAN}Release:${NC}        ${RELEASE_NAME}
  ${CYAN}Namespace:${NC}      ${NAMESPACE}
  ${CYAN}Environment:${NC}    ${ENVIRONMENT}
  ${CYAN}Image Tag:${NC}      ${IMAGE_TAG}

  ${CYAN}Useful Commands:${NC}
    Status:            helm status ${RELEASE_NAME} -n ${NAMESPACE}
    History:           helm history ${RELEASE_NAME} -n ${NAMESPACE}
    Get values:        helm get values ${RELEASE_NAME} -n ${NAMESPACE}
    Get manifest:      helm get manifest ${RELEASE_NAME} -n ${NAMESPACE}
    Upgrade:           ${0##*/} upgrade --tag <version>
    Rollback:          ${0##*/} rollback
    Uninstall:         ${0##*/} uninstall

  ${CYAN}Access Application:${NC}
    Port forward:      kubectl port-forward -n ${NAMESPACE} svc/${RELEASE_NAME} 8080:8080
    View logs:         kubectl logs -n ${NAMESPACE} -l app.kubernetes.io/name=llm-cost-ops -f

EOF

    # Show ingress if available
    local ingress_host
    ingress_host=$(kubectl get ingress -n "${NAMESPACE}" "${RELEASE_NAME}" \
        -o jsonpath='{.spec.rules[0].host}' 2>/dev/null || echo "")

    if [[ -n "${ingress_host}" ]]; then
        echo -e "    URL:               https://${ingress_host}"
    fi

    echo ""
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    local -a SET_VALUES=()

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            install|upgrade|uninstall|rollback|status|test)
                ACTION="$1"
                shift
                ;;
            -r|--release)
                RELEASE_NAME="$2"
                shift 2
                ;;
            -n|--namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            -e|--env)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -f|--values)
                VALUES_FILE="$2"
                shift 2
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            --set)
                SET_VALUES+=("$2")
                shift 2
                ;;
            --no-wait)
                WAIT="false"
                shift
                ;;
            --no-atomic)
                ATOMIC="false"
                shift
                ;;
            --timeout)
                TIMEOUT="$2"
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

    log_info "Starting Helm deployment"
    log_info "Action: ${ACTION}"
    log_info "Release: ${RELEASE_NAME}"
    log_info "Namespace: ${NAMESPACE}"

    # Check prerequisites
    check_prerequisites || exit 1

    # Validate chart
    validate_chart || exit 1

    # Select values file
    select_values_file

    # Execute action
    case "${ACTION}" in
        install)
            helm_install || exit 1
            show_release_info
            show_deployment_summary
            ;;
        upgrade)
            helm_upgrade || exit 1
            show_release_info
            show_deployment_summary
            ;;
        uninstall)
            helm_uninstall || exit 1
            ;;
        rollback)
            helm_rollback || exit 1
            show_release_info
            ;;
        status)
            helm_status
            show_release_info
            ;;
        test)
            helm_test || exit 1
            ;;
        *)
            log_error "Unknown action: ${ACTION}"
            show_usage
            exit 1
            ;;
    esac

    log_success "Operation completed successfully!"
    exit 0
}

# Run main function
main "$@"
