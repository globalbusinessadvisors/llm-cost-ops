#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Logs Aggregation Script
# =============================================================================
# Description: Aggregate and view logs from Docker containers with filtering
#              and service selection
# Usage: ./logs.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${PROJECT_ROOT}/docker-compose.yml}"
SERVICES="${SERVICES:-}"
FOLLOW="${FOLLOW:-false}"
TAIL="${TAIL:-100}"
SINCE="${SINCE:-}"
UNTIL="${UNTIL:-}"
FILTER="${FILTER:-}"
TIMESTAMPS="${TIMESTAMPS:-false}"
NO_COLOR="${NO_COLOR:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

show_usage() {
    cat << EOF
Usage: ${0##*/} [OPTIONS] [SERVICES...]

View and aggregate logs from Docker containers.

OPTIONS:
    -h, --help              Show this help message
    -f, --file FILE         Docker Compose file (default: ${COMPOSE_FILE})
    --follow                Follow log output (tail -f)
    --tail N                Number of lines to show (default: ${TAIL})
    --since TIME            Show logs since timestamp (e.g. 2023-01-01T12:00:00)
    --until TIME            Show logs until timestamp
    --filter PATTERN        Filter logs by pattern (grep)
    --timestamps            Show timestamps
    --no-color              Disable colored output

SERVICES:
    app                     Application service
    postgres                PostgreSQL database
    redis                   Redis cache
    nats                    NATS message broker
    prometheus              Prometheus metrics
    grafana                 Grafana dashboards
    jaeger                  Jaeger tracing
    mailhog                 MailHog email testing
    pgadmin                 PgAdmin database admin
    redis-commander         Redis Commander

EXAMPLES:
    # View all logs
    ${0##*/}

    # Follow application logs
    ${0##*/} --follow app

    # View last 50 lines from multiple services
    ${0##*/} --tail 50 app postgres redis

    # Filter logs by pattern
    ${0##*/} --filter "ERROR" app

    # Show logs since specific time
    ${0##*/} --since "1h" app

    # View logs with timestamps
    ${0##*/} --timestamps --tail 200

TIME FORMATS:
    Relative:  1h, 30m, 24h, 1d
    Absolute:  2023-01-01T12:00:00
    RFC3339:   2023-01-01T12:00:00Z

ENVIRONMENT VARIABLES:
    COMPOSE_FILE        Override compose file path
    FOLLOW              Follow mode (true/false)
    TAIL                Number of lines to show
    FILTER              Filter pattern
    NO_COLOR            Disable colors (true/false)

EOF
}

check_prerequisites() {
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
}

list_available_services() {
    log_info "Available services:"
    $COMPOSE_CMD -f "${COMPOSE_FILE}" config --services | while read -r service; do
        local status
        if $COMPOSE_CMD -f "${COMPOSE_FILE}" ps "${service}" 2>/dev/null | grep -q "Up"; then
            status="${GREEN}[RUNNING]${NC}"
        else
            status="${YELLOW}[STOPPED]${NC}"
        fi
        echo -e "  - ${service} ${status}"
    done
}

build_logs_command() {
    local -a logs_cmd=("$COMPOSE_CMD" "-f" "${COMPOSE_FILE}" "logs")

    # Add follow flag
    if [[ "${FOLLOW}" == "true" ]]; then
        logs_cmd+=("--follow")
    fi

    # Add tail flag
    if [[ -n "${TAIL}" ]] && [[ "${TAIL}" != "all" ]]; then
        logs_cmd+=("--tail=${TAIL}")
    fi

    # Add timestamps flag
    if [[ "${TIMESTAMPS}" == "true" ]]; then
        logs_cmd+=("--timestamps")
    fi

    # Add no-color flag
    if [[ "${NO_COLOR}" == "true" ]]; then
        logs_cmd+=("--no-color")
    fi

    # Add since flag
    if [[ -n "${SINCE}" ]]; then
        logs_cmd+=("--since=${SINCE}")
    fi

    # Add until flag
    if [[ -n "${UNTIL}" ]]; then
        logs_cmd+=("--until=${UNTIL}")
    fi

    # Add services
    if [[ -n "${SERVICES}" ]]; then
        logs_cmd+=(${SERVICES})
    fi

    echo "${logs_cmd[@]}"
}

view_logs() {
    local logs_cmd
    logs_cmd=$(build_logs_command)

    log_info "Viewing logs..."
    if [[ -n "${SERVICES}" ]]; then
        log_info "Services: ${SERVICES}"
    else
        log_info "Services: all"
    fi

    if [[ -n "${FILTER}" ]]; then
        log_info "Filter: ${FILTER}"
    fi

    echo "" >&2

    # Execute logs command with optional filtering
    if [[ -n "${FILTER}" ]]; then
        eval "${logs_cmd}" 2>&1 | grep --color=auto -E "${FILTER}|$"
    else
        eval "${logs_cmd}"
    fi
}

export_logs() {
    local output_file="${1:-logs-$(date +%Y%m%d-%H%M%S).log}"

    log_info "Exporting logs to: ${output_file}"

    local logs_cmd
    logs_cmd=$(build_logs_command)

    # Remove follow flag for export
    logs_cmd="${logs_cmd/--follow/}"

    # Export logs
    eval "${logs_cmd}" > "${output_file}" 2>&1

    log_success "Logs exported to: ${output_file}"
    log_info "File size: $(du -h "${output_file}" | cut -f1)"
}

analyze_logs() {
    log_info "Analyzing logs..."

    local logs_cmd
    logs_cmd=$(build_logs_command)

    # Remove follow flag for analysis
    logs_cmd="${logs_cmd/--follow/}"

    echo ""
    echo "Log Statistics:"
    echo "==============="

    # Total lines
    local total_lines
    total_lines=$(eval "${logs_cmd}" 2>&1 | wc -l)
    echo "Total lines: ${total_lines}"

    # Error count
    local error_count
    error_count=$(eval "${logs_cmd}" 2>&1 | grep -ci "error" || echo "0")
    echo "Errors:      ${error_count}"

    # Warning count
    local warn_count
    warn_count=$(eval "${logs_cmd}" 2>&1 | grep -ci "warn" || echo "0")
    echo "Warnings:    ${warn_count}"

    # Info count
    local info_count
    info_count=$(eval "${logs_cmd}" 2>&1 | grep -ci "info" || echo "0")
    echo "Info:        ${info_count}"

    echo ""

    # Top error messages
    echo "Top Error Messages:"
    echo "==================="
    eval "${logs_cmd}" 2>&1 | grep -i "error" | \
        sed 's/.*\(ERROR\|error\)[: ]*//' | \
        sort | uniq -c | sort -rn | head -10 || echo "None found"

    echo ""

    # Service activity
    echo "Service Activity:"
    echo "================="
    eval "${logs_cmd}" 2>&1 | \
        grep -oP '^\S+' | \
        sort | uniq -c | sort -rn || echo "Unable to parse"

    echo ""
}

show_log_summary() {
    cat << EOF

${CYAN}╔════════════════════════════════════════════════════════════════╗
║                      LOG VIEWER                                ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${BLUE}Mode:${NC}          ${FOLLOW:-view}
  ${BLUE}Lines:${NC}         ${TAIL}
  ${BLUE}Services:${NC}      ${SERVICES:-all}
  ${BLUE}Filter:${NC}        ${FILTER:-none}

  ${CYAN}Useful Commands:${NC}
    Follow logs:       ${0##*/} --follow
    Filter errors:     ${0##*/} --filter "ERROR"
    Last hour:         ${0##*/} --since 1h
    Specific service:  ${0##*/} app
    Export logs:       ${0##*/} --tail 1000 > logs.txt

  ${CYAN}Log Levels:${NC}
    ERROR              Critical errors
    WARN               Warnings
    INFO               Informational messages
    DEBUG              Debug information
    TRACE              Detailed trace information

EOF
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    local action="view"

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
            --follow)
                FOLLOW="true"
                shift
                ;;
            --tail)
                TAIL="$2"
                shift 2
                ;;
            --since)
                SINCE="$2"
                shift 2
                ;;
            --until)
                UNTIL="$2"
                shift 2
                ;;
            --filter)
                FILTER="$2"
                shift 2
                ;;
            --timestamps)
                TIMESTAMPS="true"
                shift
                ;;
            --no-color)
                NO_COLOR="true"
                shift
                ;;
            --list)
                action="list"
                shift
                ;;
            --export)
                action="export"
                shift
                ;;
            --analyze)
                action="analyze"
                shift
                ;;
            -*)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
            *)
                # Treat as service name
                SERVICES="${SERVICES} $1"
                shift
                ;;
        esac
    done

    # Check prerequisites
    check_prerequisites

    # Execute action
    case "${action}" in
        view)
            view_logs
            ;;
        list)
            list_available_services
            ;;
        export)
            export_logs
            ;;
        analyze)
            analyze_logs
            ;;
        *)
            log_error "Unknown action: ${action}"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
