#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Cleanup Script
# =============================================================================
# Description: Clean up Docker resources including containers, volumes,
#              images, and networks
# Usage: ./cleanup.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${PROJECT_ROOT}/docker-compose.yml}"
CLEANUP_CONTAINERS="${CLEANUP_CONTAINERS:-true}"
CLEANUP_VOLUMES="${CLEANUP_VOLUMES:-false}"
CLEANUP_IMAGES="${CLEANUP_IMAGES:-false}"
CLEANUP_NETWORKS="${CLEANUP_NETWORKS:-true}"
CLEANUP_BUILD_CACHE="${CLEANUP_BUILD_CACHE:-false}"
FORCE="${FORCE:-false}"
DRY_RUN="${DRY_RUN:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

Clean up Docker resources for LLM Cost Ops.

OPTIONS:
    -h, --help              Show this help message
    -f, --file FILE         Docker Compose file (default: ${COMPOSE_FILE})
    --all                   Clean all resources (containers, volumes, images, networks)
    --containers            Clean containers only
    --volumes               Clean volumes
    --images                Clean images
    --networks              Clean networks
    --build-cache           Clean build cache
    --force                 Skip confirmation prompts
    --dry-run               Show what would be deleted without deleting

EXAMPLES:
    # Clean containers and networks (safe)
    ${0##*/}

    # Clean everything
    ${0##*/} --all

    # Clean specific resources
    ${0##*/} --containers --volumes

    # Dry run to see what would be deleted
    ${0##*/} --all --dry-run

    # Force cleanup without confirmation
    ${0##*/} --all --force

WARNING:
    Cleaning volumes will DELETE ALL DATA permanently!
    Always backup important data before cleaning volumes.

ENVIRONMENT VARIABLES:
    COMPOSE_FILE            Override compose file path
    CLEANUP_CONTAINERS      Clean containers (true/false)
    CLEANUP_VOLUMES         Clean volumes (true/false)
    CLEANUP_IMAGES          Clean images (true/false)
    CLEANUP_NETWORKS        Clean networks (true/false)
    CLEANUP_BUILD_CACHE     Clean build cache (true/false)
    FORCE                   Skip confirmations (true/false)
    DRY_RUN                 Dry run mode (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi

    # Use docker compose v2 if available
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    elif docker-compose version &> /dev/null; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD=""
    fi

    log_success "Prerequisites check passed"
}

confirm_action() {
    local message="$1"

    if [[ "${FORCE}" == "true" ]] || [[ "${DRY_RUN}" == "true" ]]; then
        return 0
    fi

    log_warn "${message}"
    read -p "Are you sure you want to continue? (yes/no): " -r
    echo

    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        log_info "Operation cancelled"
        return 1
    fi

    return 0
}

show_current_resources() {
    log_info "Current Docker resources:"
    echo ""

    echo "Containers:"
    docker ps -a --filter "name=llm-cost-ops" --format "table {{.Names}}\t{{.Status}}\t{{.Size}}" 2>/dev/null || echo "  None"
    echo ""

    echo "Volumes:"
    docker volume ls --filter "label=com.llm-cost-ops.description" --format "table {{.Name}}\t{{.Driver}}" 2>/dev/null || echo "  None"
    echo ""

    echo "Images:"
    docker images --filter "reference=*llm-cost-ops*" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}" 2>/dev/null || echo "  None"
    echo ""

    echo "Networks:"
    docker network ls --filter "label=com.llm-cost-ops.description" --format "table {{.Name}}\t{{.Driver}}" 2>/dev/null || echo "  None"
    echo ""
}

cleanup_containers() {
    if [[ "${CLEANUP_CONTAINERS}" != "true" ]]; then
        return 0
    fi

    log_info "Cleaning up containers..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would stop and remove containers:"
        docker ps -a --filter "name=llm-cost-ops" --format "  - {{.Names}}"
        return 0
    fi

    # Stop compose services if compose file exists
    if [[ -n "${COMPOSE_CMD}" ]] && [[ -f "${COMPOSE_FILE}" ]]; then
        log_info "Stopping Docker Compose services..."
        $COMPOSE_CMD -f "${COMPOSE_FILE}" down 2>/dev/null || true
    fi

    # Remove all llm-cost-ops containers
    local containers
    containers=$(docker ps -aq --filter "name=llm-cost-ops" 2>/dev/null || echo "")

    if [[ -n "${containers}" ]]; then
        log_info "Removing containers..."
        echo "${containers}" | xargs docker rm -f 2>/dev/null || true
        log_success "Containers removed"
    else
        log_info "No containers to remove"
    fi
}

cleanup_volumes() {
    if [[ "${CLEANUP_VOLUMES}" != "true" ]]; then
        return 0
    fi

    if ! confirm_action "This will DELETE ALL DATA in volumes!"; then
        return 0
    fi

    log_info "Cleaning up volumes..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would remove volumes:"
        docker volume ls --filter "label=com.llm-cost-ops.description" --format "  - {{.Name}}"
        return 0
    fi

    # Remove compose volumes if compose file exists
    if [[ -n "${COMPOSE_CMD}" ]] && [[ -f "${COMPOSE_FILE}" ]]; then
        log_info "Removing Docker Compose volumes..."
        $COMPOSE_CMD -f "${COMPOSE_FILE}" down -v 2>/dev/null || true
    fi

    # Remove all llm-cost-ops volumes
    local volumes
    volumes=$(docker volume ls -q --filter "label=com.llm-cost-ops.description" 2>/dev/null || echo "")

    if [[ -n "${volumes}" ]]; then
        log_info "Removing volumes..."
        echo "${volumes}" | xargs docker volume rm -f 2>/dev/null || true
        log_success "Volumes removed"
    else
        log_info "No volumes to remove"
    fi

    # Remove local data directories
    if [[ -d "${PROJECT_ROOT}/data" ]]; then
        log_info "Removing local data directory..."
        rm -rf "${PROJECT_ROOT}/data"
    fi

    if [[ -d "${PROJECT_ROOT}/logs" ]]; then
        log_info "Removing local logs directory..."
        rm -rf "${PROJECT_ROOT}/logs"
    fi
}

cleanup_images() {
    if [[ "${CLEANUP_IMAGES}" != "true" ]]; then
        return 0
    fi

    log_info "Cleaning up images..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would remove images:"
        docker images --filter "reference=*llm-cost-ops*" --format "  - {{.Repository}}:{{.Tag}}"
        return 0
    fi

    # Remove all llm-cost-ops images
    local images
    images=$(docker images -q --filter "reference=*llm-cost-ops*" 2>/dev/null || echo "")

    if [[ -n "${images}" ]]; then
        log_info "Removing images..."
        echo "${images}" | xargs docker rmi -f 2>/dev/null || true
        log_success "Images removed"
    else
        log_info "No images to remove"
    fi

    # Remove dangling images
    log_info "Removing dangling images..."
    docker image prune -f >/dev/null 2>&1 || true
}

cleanup_networks() {
    if [[ "${CLEANUP_NETWORKS}" != "true" ]]; then
        return 0
    fi

    log_info "Cleaning up networks..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would remove networks:"
        docker network ls --filter "label=com.llm-cost-ops.description" --format "  - {{.Name}}"
        return 0
    fi

    # Remove compose networks if compose file exists
    if [[ -n "${COMPOSE_CMD}" ]] && [[ -f "${COMPOSE_FILE}" ]]; then
        $COMPOSE_CMD -f "${COMPOSE_FILE}" down 2>/dev/null || true
    fi

    # Remove all llm-cost-ops networks
    local networks
    networks=$(docker network ls -q --filter "label=com.llm-cost-ops.description" 2>/dev/null || echo "")

    if [[ -n "${networks}" ]]; then
        log_info "Removing networks..."
        echo "${networks}" | xargs docker network rm 2>/dev/null || true
        log_success "Networks removed"
    else
        log_info "No networks to remove"
    fi

    # Prune unused networks
    docker network prune -f >/dev/null 2>&1 || true
}

cleanup_build_cache() {
    if [[ "${CLEANUP_BUILD_CACHE}" != "true" ]]; then
        return 0
    fi

    log_info "Cleaning up build cache..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would clean build cache"
        docker buildx du 2>/dev/null || echo "  Build cache size: unknown"
        return 0
    fi

    # Show current cache size
    log_info "Current build cache:"
    docker buildx du 2>/dev/null || docker system df 2>/dev/null || true

    # Clean build cache
    log_info "Removing build cache..."
    docker buildx prune -af 2>/dev/null || docker builder prune -af 2>/dev/null || true

    log_success "Build cache cleaned"
}

show_cleanup_summary() {
    cat << EOF

${GREEN}╔════════════════════════════════════════════════════════════════╗
║                    CLEANUP COMPLETE                            ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${BLUE}Resources cleaned:${NC}
    • Containers:     ${CLEANUP_CONTAINERS}
    • Volumes:        ${CLEANUP_VOLUMES}
    • Images:         ${CLEANUP_IMAGES}
    • Networks:       ${CLEANUP_NETWORKS}
    • Build Cache:    ${CLEANUP_BUILD_CACHE}

  ${BLUE}Disk space reclaimed:${NC}
EOF

    # Show docker system df
    docker system df 2>/dev/null || true

    cat << EOF

  ${BLUE}Additional cleanup (if needed):${NC}
    Clean system:      docker system prune -a
    Clean volumes:     docker volume prune
    Clean everything:  ${0##*/} --all

EOF
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    local cleanup_all="false"

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
            --all)
                cleanup_all="true"
                shift
                ;;
            --containers)
                CLEANUP_CONTAINERS="true"
                shift
                ;;
            --volumes)
                CLEANUP_VOLUMES="true"
                shift
                ;;
            --images)
                CLEANUP_IMAGES="true"
                shift
                ;;
            --networks)
                CLEANUP_NETWORKS="true"
                shift
                ;;
            --build-cache)
                CLEANUP_BUILD_CACHE="true"
                shift
                ;;
            --force)
                FORCE="true"
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

    # Set all cleanup flags if --all is specified
    if [[ "${cleanup_all}" == "true" ]]; then
        CLEANUP_CONTAINERS="true"
        CLEANUP_VOLUMES="true"
        CLEANUP_IMAGES="true"
        CLEANUP_NETWORKS="true"
        CLEANUP_BUILD_CACHE="true"
    fi

    log_info "Starting cleanup process..."

    # Check prerequisites
    check_prerequisites

    # Show current resources
    show_current_resources

    # Confirm if not forced
    if [[ "${CLEANUP_VOLUMES}" == "true" ]] || [[ "${cleanup_all}" == "true" ]]; then
        if ! confirm_action "This cleanup will DELETE DATA. Ensure you have backups!"; then
            exit 0
        fi
    fi

    # Perform cleanup operations
    cleanup_containers
    cleanup_volumes
    cleanup_images
    cleanup_networks
    cleanup_build_cache

    # Show summary
    show_cleanup_summary

    log_success "Cleanup completed successfully!"
    exit 0
}

# Run main function
main "$@"
