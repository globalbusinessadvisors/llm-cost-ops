#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Development Deployment Script
# =============================================================================
# Description: Deploy development environment with Docker Compose
# Usage: ./deploy-dev.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${PROJECT_ROOT}/docker-compose.yml}"
ENV_FILE="${ENV_FILE:-${PROJECT_ROOT}/.env}"
ACTION="${ACTION:-up}"
SERVICES="${SERVICES:-}"
FOLLOW_LOGS="${FOLLOW_LOGS:-false}"
INIT_DB="${INIT_DB:-true}"
HEALTH_CHECK="${HEALTH_CHECK:-true}"
WAIT_TIMEOUT="${WAIT_TIMEOUT:-300}"
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

Deploy and manage LLM Cost Ops development environment.

ACTIONS:
    up              Start all services (default)
    down            Stop all services
    restart         Restart all services
    logs            View logs
    status          Show service status
    clean           Clean up containers and volumes

OPTIONS:
    -h, --help              Show this help message
    -f, --file FILE         Docker Compose file (default: ${COMPOSE_FILE})
    -s, --service SERVICE   Specific service to manage (can specify multiple)
    --follow                Follow logs after deployment
    --no-init               Skip database initialization
    --no-health-check       Skip health checks
    --wait-timeout SEC      Health check timeout in seconds (default: ${WAIT_TIMEOUT})
    --dry-run               Print commands without executing

EXAMPLES:
    # Start all services
    ${0##*/} up

    # Start specific services
    ${0##*/} up --service app --service postgres

    # Start and follow logs
    ${0##*/} up --follow

    # Restart all services
    ${0##*/} restart

    # View logs
    ${0##*/} logs

    # Stop and clean up
    ${0##*/} down
    ${0##*/} clean

ENVIRONMENT VARIABLES:
    COMPOSE_FILE        Override compose file path
    ENV_FILE            Override .env file path
    FOLLOW_LOGS         Follow logs (true/false)
    INIT_DB             Initialize database (true/false)
    HEALTH_CHECK        Perform health checks (true/false)
    DRY_RUN             Dry run mode (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi

    # Check for Docker Compose
    if ! docker compose version &> /dev/null && ! docker-compose version &> /dev/null; then
        log_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi

    # Use docker compose v2 if available, otherwise fall back to docker-compose
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    else
        COMPOSE_CMD="docker-compose"
    fi

    # Check if compose file exists
    if [[ ! -f "${COMPOSE_FILE}" ]]; then
        log_error "Compose file not found: ${COMPOSE_FILE}"
        exit 1
    fi

    # Create .env file if it doesn't exist
    if [[ ! -f "${ENV_FILE}" ]]; then
        log_warn ".env file not found, creating default..."
        cat > "${ENV_FILE}" << 'ENVEOF'
# LLM Cost Ops Development Environment Variables
RUST_LOG=debug
RUST_BACKTRACE=1
LOG_LEVEL=debug
PORT=8080
METRICS_PORT=9090

# Database
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=llm_cost_ops_dev

# Security (change in production!)
JWT_SECRET=dev-secret-change-in-production

# Feature Flags
ENABLE_COMPRESSION=true
ENABLE_RATE_LIMITING=true
ENABLE_METRICS=true
ENABLE_TRACING=true

# Grafana
GF_SECURITY_ADMIN_USER=admin
GF_SECURITY_ADMIN_PASSWORD=admin
ENVEOF
        log_success "Created default .env file"
    fi

    log_success "Prerequisites check passed"
}

create_directories() {
    log_info "Creating required directories..."

    local -a dirs=(
        "${PROJECT_ROOT}/data"
        "${PROJECT_ROOT}/logs"
        "${PROJECT_ROOT}/docker/postgres"
        "${PROJECT_ROOT}/docker/redis"
        "${PROJECT_ROOT}/docker/nats"
        "${PROJECT_ROOT}/docker/prometheus"
        "${PROJECT_ROOT}/docker/grafana/provisioning/datasources"
        "${PROJECT_ROOT}/docker/grafana/provisioning/dashboards"
        "${PROJECT_ROOT}/docker/grafana/dashboards"
        "${PROJECT_ROOT}/docker/pgadmin"
    )

    for dir in "${dirs[@]}"; do
        if [[ ! -d "${dir}" ]]; then
            mkdir -p "${dir}"
            log_info "Created: ${dir}"
        fi
    done

    log_success "Directories ready"
}

initialize_database() {
    if [[ "${INIT_DB}" != "true" ]]; then
        log_info "Skipping database initialization"
        return 0
    fi

    log_info "Initializing database..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would initialize database"
        return 0
    fi

    # Wait for PostgreSQL to be ready
    log_info "Waiting for PostgreSQL to be ready..."
    local retries=0
    while [[ ${retries} -lt 30 ]]; do
        if $COMPOSE_CMD exec -T postgres pg_isready -U postgres >/dev/null 2>&1; then
            log_success "PostgreSQL is ready"
            break
        fi
        ((retries++))
        sleep 2
    done

    if [[ ${retries} -eq 30 ]]; then
        log_error "PostgreSQL failed to become ready"
        return 1
    fi

    # Run migrations
    log_info "Running database migrations..."
    $COMPOSE_CMD exec -T app /app/llm-cost-ops migrate || {
        log_warn "Migration command failed (may already be up to date)"
    }

    log_success "Database initialized"
}

start_services() {
    log_info "Starting services..."

    local -a compose_args=("-f" "${COMPOSE_FILE}")

    if [[ -f "${ENV_FILE}" ]]; then
        compose_args+=("--env-file" "${ENV_FILE}")
    fi

    compose_args+=("up" "-d")

    # Add specific services if requested
    if [[ -n "${SERVICES}" ]]; then
        compose_args+=($SERVICES)
    fi

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute: ${COMPOSE_CMD} ${compose_args[*]}"
        return 0
    fi

    log_info "Executing: ${COMPOSE_CMD} ${compose_args[*]}"

    if $COMPOSE_CMD "${compose_args[@]}"; then
        log_success "Services started"
        return 0
    else
        log_error "Failed to start services"
        return 1
    fi
}

stop_services() {
    log_info "Stopping services..."

    local -a compose_args=("-f" "${COMPOSE_FILE}")
    compose_args+=("down")

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute: ${COMPOSE_CMD} ${compose_args[*]}"
        return 0
    fi

    if $COMPOSE_CMD "${compose_args[@]}"; then
        log_success "Services stopped"
        return 0
    else
        log_error "Failed to stop services"
        return 1
    fi
}

restart_services() {
    log_info "Restarting services..."

    stop_services
    sleep 2
    start_services
}

check_health() {
    if [[ "${HEALTH_CHECK}" != "true" ]]; then
        log_info "Skipping health checks"
        return 0
    fi

    log_info "Performing health checks..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would check service health"
        return 0
    fi

    # Define services and their health check endpoints
    declare -A health_checks=(
        ["app"]="http://localhost:8080/health"
        ["postgres"]="pg_isready"
        ["redis"]="redis-cli ping"
        ["prometheus"]="http://localhost:9091/-/healthy"
        ["grafana"]="http://localhost:3000/api/health"
        ["jaeger"]="http://localhost:14269/"
    )

    local start_time=$(date +%s)
    local all_healthy=false

    while [[ $(( $(date +%s) - start_time )) -lt ${WAIT_TIMEOUT} ]]; do
        all_healthy=true

        for service in "${!health_checks[@]}"; do
            # Skip if service is not running
            if ! $COMPOSE_CMD ps --services --filter "status=running" | grep -q "^${service}$"; then
                continue
            fi

            local check="${health_checks[$service]}"

            if [[ "${check}" == http* ]]; then
                # HTTP health check
                if ! curl -sf "${check}" >/dev/null 2>&1; then
                    all_healthy=false
                    log_info "Waiting for ${service}..."
                fi
            else
                # Command-based health check
                if ! $COMPOSE_CMD exec -T "${service}" ${check} >/dev/null 2>&1; then
                    all_healthy=false
                    log_info "Waiting for ${service}..."
                fi
            fi
        done

        if [[ "${all_healthy}" == "true" ]]; then
            break
        fi

        sleep 5
    done

    if [[ "${all_healthy}" == "true" ]]; then
        log_success "All services are healthy"
        return 0
    else
        log_warn "Some services may not be healthy (timeout reached)"
        show_status
        return 1
    fi
}

show_status() {
    log_info "Service Status:"
    echo ""

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would show service status"
        return 0
    fi

    $COMPOSE_CMD -f "${COMPOSE_FILE}" ps
    echo ""
}

view_logs() {
    log_info "Viewing logs..."

    local -a compose_args=("-f" "${COMPOSE_FILE}" "logs")

    if [[ "${FOLLOW_LOGS}" == "true" ]]; then
        compose_args+=("-f")
    fi

    if [[ -n "${SERVICES}" ]]; then
        compose_args+=($SERVICES)
    fi

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute: ${COMPOSE_CMD} ${compose_args[*]}"
        return 0
    fi

    $COMPOSE_CMD "${compose_args[@]}"
}

clean_environment() {
    log_warn "This will remove all containers, networks, and volumes"
    read -p "Are you sure? (yes/no): " -r
    echo

    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        log_info "Cleanup cancelled"
        return 0
    fi

    log_info "Cleaning up environment..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute: ${COMPOSE_CMD} down -v --remove-orphans"
        return 0
    fi

    $COMPOSE_CMD -f "${COMPOSE_FILE}" down -v --remove-orphans

    # Remove local data directories
    log_info "Removing local data directories..."
    rm -rf "${PROJECT_ROOT}/data" "${PROJECT_ROOT}/logs"

    log_success "Environment cleaned"
}

show_summary() {
    cat << EOF

${GREEN}╔════════════════════════════════════════════════════════════════╗
║              DEVELOPMENT ENVIRONMENT READY                     ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${CYAN}Services:${NC}
    • Application:     http://localhost:8080
    • Metrics:         http://localhost:9090
    • Grafana:         http://localhost:3000 (admin/admin)
    • Prometheus:      http://localhost:9091
    • Jaeger UI:       http://localhost:16686
    • MailHog UI:      http://localhost:8025
    • PgAdmin:         http://localhost:5050
    • Redis Commander: http://localhost:8081

  ${CYAN}Database:${NC}
    • PostgreSQL:      localhost:5432
    • Redis:           localhost:6379

  ${CYAN}Useful Commands:${NC}
    View logs:         ${0##*/} logs --follow
    Restart:           ${0##*/} restart
    Stop:              ${0##*/} down
    Status:            ${0##*/} status

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
            up|down|restart|logs|status|clean)
                ACTION="$1"
                shift
                ;;
            -f|--file)
                COMPOSE_FILE="$2"
                shift 2
                ;;
            -s|--service)
                SERVICES="${SERVICES} $2"
                shift 2
                ;;
            --follow)
                FOLLOW_LOGS="true"
                shift
                ;;
            --no-init)
                INIT_DB="false"
                shift
                ;;
            --no-health-check)
                HEALTH_CHECK="false"
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

    # Check prerequisites
    check_prerequisites

    # Execute action
    case "${ACTION}" in
        up)
            create_directories
            start_services
            check_health
            if [[ "${INIT_DB}" == "true" ]]; then
                initialize_database
            fi
            show_status
            show_summary
            if [[ "${FOLLOW_LOGS}" == "true" ]]; then
                view_logs
            fi
            ;;
        down)
            stop_services
            ;;
        restart)
            restart_services
            check_health
            show_status
            ;;
        logs)
            view_logs
            ;;
        status)
            show_status
            ;;
        clean)
            clean_environment
            ;;
        *)
            log_error "Unknown action: ${ACTION}"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
