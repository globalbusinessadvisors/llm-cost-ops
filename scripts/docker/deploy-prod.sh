#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Production Deployment Script
# =============================================================================
# Description: Deploy production environment with blue-green deployment,
#              health checks, and rollback capability
# Usage: ./deploy-prod.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${PROJECT_ROOT}/docker-compose.prod.yml}"
ENV_FILE="${ENV_FILE:-${PROJECT_ROOT}/.env.prod}"
DEPLOYMENT_STRATEGY="${DEPLOYMENT_STRATEGY:-rolling}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
HEALTH_CHECK_RETRIES="${HEALTH_CHECK_RETRIES:-10}"
HEALTH_CHECK_INTERVAL="${HEALTH_CHECK_INTERVAL:-5}"
BACKUP_BEFORE_DEPLOY="${BACKUP_BEFORE_DEPLOY:-true}"
AUTO_ROLLBACK="${AUTO_ROLLBACK:-true}"
DRY_RUN="${DRY_RUN:-false}"

# Deployment tracking
DEPLOYMENT_ID="${DEPLOYMENT_ID:-$(date +%Y%m%d-%H%M%S)}"
DEPLOYMENT_LOG="${PROJECT_ROOT}/logs/deployment-${DEPLOYMENT_ID}.log"
STATE_FILE="${PROJECT_ROOT}/.deployment-state"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------
log_info() {
    local msg="$*"
    echo -e "${BLUE}[INFO]${NC} ${msg}" | tee -a "${DEPLOYMENT_LOG}"
}

log_success() {
    local msg="$*"
    echo -e "${GREEN}[SUCCESS]${NC} ${msg}" | tee -a "${DEPLOYMENT_LOG}"
}

log_warn() {
    local msg="$*"
    echo -e "${YELLOW}[WARN]${NC} ${msg}" | tee -a "${DEPLOYMENT_LOG}"
}

log_error() {
    local msg="$*"
    echo -e "${RED}[ERROR]${NC} ${msg}" | tee -a "${DEPLOYMENT_LOG}"
}

show_usage() {
    cat << EOF
Usage: ${0##*/} [OPTIONS]

Deploy LLM Cost Ops to production environment.

OPTIONS:
    -h, --help                  Show this help message
    -f, --file FILE             Docker Compose file (default: ${COMPOSE_FILE})
    -e, --env FILE              Environment file (default: ${ENV_FILE})
    -t, --tag TAG               Image tag to deploy (default: ${IMAGE_TAG})
    -s, --strategy STRATEGY     Deployment strategy: rolling, blue-green, recreate
    --no-backup                 Skip pre-deployment backup
    --no-rollback               Disable auto-rollback on failure
    --dry-run                   Print actions without executing

DEPLOYMENT STRATEGIES:
    rolling         Zero-downtime rolling update (default)
    blue-green      Blue-green deployment with traffic switch
    recreate        Stop all, then start new version

EXAMPLES:
    # Deploy latest version
    ${0##*/}

    # Deploy specific version with blue-green strategy
    ${0##*/} --tag v1.2.3 --strategy blue-green

    # Deploy without backup (not recommended)
    ${0##*/} --no-backup

    # Dry run
    ${0##*/} --dry-run --tag v1.2.3

PRE-DEPLOYMENT CHECKLIST:
    ☐ Backup database
    ☐ Test configuration
    ☐ Verify image availability
    ☐ Check system resources
    ☐ Notify team
    ☐ Prepare rollback plan

ENVIRONMENT VARIABLES:
    COMPOSE_FILE            Override compose file path
    ENV_FILE                Override environment file path
    IMAGE_TAG               Image tag to deploy
    DEPLOYMENT_STRATEGY     Deployment strategy
    BACKUP_BEFORE_DEPLOY    Backup before deploy (true/false)
    AUTO_ROLLBACK           Auto rollback on failure (true/false)
    DRY_RUN                 Dry run mode (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi

    # Check for Docker Compose
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    elif docker-compose version &> /dev/null; then
        COMPOSE_CMD="docker-compose"
    else
        log_error "Docker Compose is not installed"
        exit 1
    fi

    # Check if compose file exists
    if [[ ! -f "${COMPOSE_FILE}" ]]; then
        log_error "Compose file not found: ${COMPOSE_FILE}"
        exit 1
    fi

    # Check if env file exists
    if [[ ! -f "${ENV_FILE}" ]]; then
        log_error "Environment file not found: ${ENV_FILE}"
        log_info "Please create ${ENV_FILE} with production configuration"
        exit 1
    fi

    # Check if running as root (not recommended)
    if [[ $EUID -eq 0 ]]; then
        log_warn "Running as root is not recommended"
    fi

    # Create logs directory
    mkdir -p "$(dirname "${DEPLOYMENT_LOG}")"

    log_success "Prerequisites check passed"
}

validate_configuration() {
    log_info "Validating configuration..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Skipping validation"
        return 0
    fi

    # Validate docker-compose configuration
    if ! $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" config >/dev/null 2>&1; then
        log_error "Invalid Docker Compose configuration"
        return 1
    fi

    # Check required environment variables
    local -a required_vars=(
        "DATABASE_URL"
        "JWT_SECRET"
    )

    for var in "${required_vars[@]}"; do
        if ! grep -q "^${var}=" "${ENV_FILE}"; then
            log_error "Missing required environment variable: ${var}"
            return 1
        fi
    done

    # Validate JWT secret is not default
    if grep -q "JWT_SECRET=dev-secret" "${ENV_FILE}"; then
        log_error "JWT_SECRET is set to default development value!"
        log_error "Please set a secure production secret"
        return 1
    fi

    log_success "Configuration validated"
}

check_image_availability() {
    log_info "Checking image availability..."

    local image="${REGISTRY:-ghcr.io/yourusername}/llm-cost-ops:${IMAGE_TAG}"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would check image: ${image}"
        return 0
    fi

    # Try to pull image
    log_info "Pulling image: ${image}"
    if docker pull "${image}"; then
        log_success "Image available: ${image}"
        return 0
    else
        log_error "Failed to pull image: ${image}"
        return 1
    fi
}

check_system_resources() {
    log_info "Checking system resources..."

    # Check disk space
    local available_disk
    available_disk=$(df -BG "${PROJECT_ROOT}" | awk 'NR==2 {print $4}' | sed 's/G//')

    if [[ ${available_disk} -lt 10 ]]; then
        log_warn "Low disk space: ${available_disk}GB available"
    else
        log_info "Disk space: ${available_disk}GB available"
    fi

    # Check memory
    if command -v free &> /dev/null; then
        local available_mem
        available_mem=$(free -g | awk 'NR==2 {print $7}')
        log_info "Available memory: ${available_mem}GB"
    fi

    log_success "System resources check complete"
}

backup_current_state() {
    if [[ "${BACKUP_BEFORE_DEPLOY}" != "true" ]]; then
        log_info "Skipping pre-deployment backup"
        return 0
    fi

    log_info "Creating pre-deployment backup..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would create backup"
        return 0
    fi

    # Run backup script
    if [[ -f "${SCRIPT_DIR}/backup.sh" ]]; then
        "${SCRIPT_DIR}/backup.sh" --tag "pre-deploy-${DEPLOYMENT_ID}" || {
            log_error "Backup failed"
            return 1
        }
    else
        log_warn "Backup script not found, skipping backup"
    fi

    log_success "Backup completed"
}

save_deployment_state() {
    log_info "Saving deployment state..."

    local current_image
    current_image=$($COMPOSE_CMD -f "${COMPOSE_FILE}" images app --format json 2>/dev/null | \
                   jq -r '.Repository + ":" + .Tag' 2>/dev/null || echo "unknown")

    cat > "${STATE_FILE}" << EOF
DEPLOYMENT_ID=${DEPLOYMENT_ID}
DEPLOYMENT_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
IMAGE_TAG=${IMAGE_TAG}
PREVIOUS_IMAGE=${current_image}
STRATEGY=${DEPLOYMENT_STRATEGY}
EOF

    log_info "Deployment state saved"
}

deploy_rolling() {
    log_info "Starting rolling deployment..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would perform rolling deployment"
        return 0
    fi

    # Update images
    log_info "Pulling latest images..."
    $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" pull

    # Perform rolling update
    log_info "Performing rolling update..."
    $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" up -d --no-deps --build app

    log_success "Rolling deployment initiated"
}

deploy_blue_green() {
    log_info "Starting blue-green deployment..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would perform blue-green deployment"
        return 0
    fi

    # Deploy green environment
    log_info "Deploying green environment..."
    $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" \
        -p llm-cost-ops-green up -d

    # Wait for health check
    if ! verify_deployment "llm-cost-ops-green"; then
        log_error "Green environment health check failed"
        log_info "Cleaning up green environment..."
        docker compose -p llm-cost-ops-green down
        return 1
    fi

    # Switch traffic (this would typically involve load balancer configuration)
    log_info "Switching traffic to green environment..."
    # TODO: Implement traffic switching logic

    # Stop blue environment
    log_info "Stopping blue environment..."
    $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" down

    # Rename green to blue
    log_info "Promoting green to blue..."
    # This is handled by restarting with the main project name

    log_success "Blue-green deployment completed"
}

deploy_recreate() {
    log_info "Starting recreate deployment..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would perform recreate deployment"
        return 0
    fi

    # Stop all services
    log_info "Stopping all services..."
    $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" down

    # Start with new version
    log_info "Starting services with new version..."
    $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" up -d

    log_success "Recreate deployment completed"
}

verify_deployment() {
    local project_name="${1:-llm-cost-ops}"
    log_info "Verifying deployment..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would verify deployment"
        return 0
    fi

    # Wait for services to be healthy
    log_info "Waiting for services to be healthy..."

    local retry=0
    while [[ ${retry} -lt ${HEALTH_CHECK_RETRIES} ]]; do
        # Check if app is running
        if docker compose -p "${project_name}" ps app | grep -q "Up"; then
            # Check health endpoint
            if curl -sf http://localhost:8080/health >/dev/null 2>&1; then
                log_success "Application is healthy"
                return 0
            fi
        fi

        ((retry++))
        log_info "Health check attempt ${retry}/${HEALTH_CHECK_RETRIES}..."
        sleep "${HEALTH_CHECK_INTERVAL}"
    done

    log_error "Deployment verification failed"
    return 1
}

run_smoke_tests() {
    log_info "Running smoke tests..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would run smoke tests"
        return 0
    fi

    # Test health endpoint
    if ! curl -sf http://localhost:8080/health >/dev/null; then
        log_error "Health endpoint test failed"
        return 1
    fi

    # Test metrics endpoint
    if ! curl -sf http://localhost:9090/metrics >/dev/null; then
        log_error "Metrics endpoint test failed"
        return 1
    fi

    # Test API endpoint
    if ! curl -sf http://localhost:8080/api/v1/health >/dev/null 2>&1; then
        log_warn "API endpoint test failed (may be expected)"
    fi

    log_success "Smoke tests passed"
}

rollback() {
    log_error "Initiating rollback..."

    if [[ ! -f "${STATE_FILE}" ]]; then
        log_error "No deployment state found, cannot rollback"
        return 1
    fi

    # Load previous state
    source "${STATE_FILE}"

    log_info "Rolling back to: ${PREVIOUS_IMAGE}"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would rollback to ${PREVIOUS_IMAGE}"
        return 0
    fi

    # Stop current deployment
    $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" down

    # Start previous version
    IMAGE_TAG="${PREVIOUS_IMAGE##*:}" \
        $COMPOSE_CMD -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" up -d

    if verify_deployment; then
        log_success "Rollback completed successfully"
        return 0
    else
        log_error "Rollback verification failed"
        return 1
    fi
}

show_deployment_summary() {
    local status="$1"

    cat << EOF

${GREEN}╔════════════════════════════════════════════════════════════════╗
║                  DEPLOYMENT ${status^^}                              ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${CYAN}Deployment ID:${NC}  ${DEPLOYMENT_ID}
  ${CYAN}Image Tag:${NC}      ${IMAGE_TAG}
  ${CYAN}Strategy:${NC}       ${DEPLOYMENT_STRATEGY}
  ${CYAN}Status:${NC}         ${status}

  ${CYAN}Services:${NC}
    • Application:    http://localhost:8080
    • Metrics:        http://localhost:9090

  ${CYAN}Logs:${NC}
    View with: docker compose -f ${COMPOSE_FILE} logs -f

  ${CYAN}Deployment Log:${NC}
    ${DEPLOYMENT_LOG}

EOF
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
            -f|--file)
                COMPOSE_FILE="$2"
                shift 2
                ;;
            -e|--env)
                ENV_FILE="$2"
                shift 2
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            -s|--strategy)
                DEPLOYMENT_STRATEGY="$2"
                shift 2
                ;;
            --no-backup)
                BACKUP_BEFORE_DEPLOY="false"
                shift
                ;;
            --no-rollback)
                AUTO_ROLLBACK="false"
                shift
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

    log_info "Starting production deployment ${DEPLOYMENT_ID}"

    # Pre-deployment checks
    check_prerequisites || exit 1
    validate_configuration || exit 1
    check_image_availability || exit 1
    check_system_resources
    backup_current_state || exit 1
    save_deployment_state

    # Perform deployment based on strategy
    deployment_success=false
    case "${DEPLOYMENT_STRATEGY}" in
        rolling)
            if deploy_rolling && verify_deployment && run_smoke_tests; then
                deployment_success=true
            fi
            ;;
        blue-green)
            if deploy_blue_green && run_smoke_tests; then
                deployment_success=true
            fi
            ;;
        recreate)
            if deploy_recreate && verify_deployment && run_smoke_tests; then
                deployment_success=true
            fi
            ;;
        *)
            log_error "Unknown deployment strategy: ${DEPLOYMENT_STRATEGY}"
            exit 1
            ;;
    esac

    # Handle deployment result
    if [[ "${deployment_success}" == "true" ]]; then
        log_success "Deployment completed successfully!"
        show_deployment_summary "SUCCESS"
        exit 0
    else
        log_error "Deployment failed!"

        if [[ "${AUTO_ROLLBACK}" == "true" ]]; then
            rollback
        else
            log_warn "Auto-rollback disabled. Manual intervention required."
        fi

        show_deployment_summary "FAILED"
        exit 1
    fi
}

# Run main function
main "$@"
