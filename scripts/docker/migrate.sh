#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Database Migration Script
# =============================================================================
# Description: Run database migrations for SQLite and PostgreSQL with
#              rollback support and verification
# Usage: ./migrate.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-${PROJECT_ROOT}/docker-compose.yml}"
DATABASE_TYPE="${DATABASE_TYPE:-postgres}"
DATABASE_URL="${DATABASE_URL:-}"
MIGRATION_DIR="${MIGRATION_DIR:-}"
ACTION="${ACTION:-up}"
STEPS="${STEPS:-}"
DRY_RUN="${DRY_RUN:-false}"
FORCE="${FORCE:-false}"

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

Run database migrations for LLM Cost Ops.

ACTIONS:
    up              Run pending migrations (default)
    down            Rollback last migration
    status          Show migration status
    create          Create a new migration file
    reset           Reset database (drop and recreate)
    verify          Verify migration integrity

OPTIONS:
    -h, --help              Show this help message
    -d, --database TYPE     Database type: sqlite, postgres (default: ${DATABASE_TYPE})
    -u, --url URL           Database URL
    -s, --steps N           Number of migrations to run/rollback
    -f, --file FILE         Docker Compose file
    --force                 Force migration without confirmation
    --dry-run               Show what would be executed

EXAMPLES:
    # Run all pending migrations
    ${0##*/} up

    # Rollback last migration
    ${0##*/} down

    # Rollback 3 migrations
    ${0##*/} down --steps 3

    # Check migration status
    ${0##*/} status

    # Run migrations in Docker container
    ${0##*/} up --database postgres

    # Reset database (DESTRUCTIVE!)
    ${0##*/} reset --force

ENVIRONMENT VARIABLES:
    DATABASE_TYPE       Database type (sqlite/postgres)
    DATABASE_URL        Database connection URL
    MIGRATION_DIR       Custom migration directory
    DRY_RUN             Dry run mode (true/false)
    FORCE               Skip confirmations (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for required tools based on database type
    case "${DATABASE_TYPE}" in
        sqlite)
            if ! command -v sqlite3 &> /dev/null; then
                log_warn "sqlite3 not found, will use Docker container"
            fi
            ;;
        postgres|postgresql)
            if ! command -v psql &> /dev/null; then
                log_warn "psql not found, will use Docker container"
            fi
            DATABASE_TYPE="postgres"
            ;;
        *)
            log_error "Unknown database type: ${DATABASE_TYPE}"
            log_info "Supported types: sqlite, postgres"
            exit 1
            ;;
    esac

    # Check for Docker Compose
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    elif docker-compose version &> /dev/null; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD=""
    fi

    # Set migration directory based on database type
    if [[ -z "${MIGRATION_DIR}" ]]; then
        if [[ "${DATABASE_TYPE}" == "sqlite" ]]; then
            MIGRATION_DIR="${PROJECT_ROOT}/migrations"
        else
            MIGRATION_DIR="${PROJECT_ROOT}/migrations_postgres"
        fi
    fi

    # Verify migration directory exists
    if [[ ! -d "${MIGRATION_DIR}" ]]; then
        log_error "Migration directory not found: ${MIGRATION_DIR}"
        exit 1
    fi

    log_success "Prerequisites check passed"
}

detect_database_url() {
    if [[ -n "${DATABASE_URL}" ]]; then
        log_info "Using provided DATABASE_URL"
        return 0
    fi

    # Try to detect from environment files
    local env_files=(
        "${PROJECT_ROOT}/.env"
        "${PROJECT_ROOT}/.env.local"
        "${PROJECT_ROOT}/.env.dev"
    )

    for env_file in "${env_files[@]}"; do
        if [[ -f "${env_file}" ]]; then
            local url
            url=$(grep '^DATABASE_URL=' "${env_file}" | cut -d= -f2- | tr -d '"' || echo "")
            if [[ -n "${url}" ]]; then
                DATABASE_URL="${url}"
                log_info "Detected DATABASE_URL from ${env_file}"
                return 0
            fi
        fi
    done

    # Set default based on database type
    if [[ "${DATABASE_TYPE}" == "sqlite" ]]; then
        DATABASE_URL="sqlite://${PROJECT_ROOT}/data/llm-cost-ops.db"
        log_info "Using default SQLite database: ${DATABASE_URL}"
    else
        DATABASE_URL="postgresql://postgres:postgres@localhost:5432/llm_cost_ops_dev"
        log_info "Using default PostgreSQL database: ${DATABASE_URL}"
    fi
}

run_migration_in_docker() {
    local action="$1"

    log_info "Running migration in Docker container..."

    if [[ -z "${COMPOSE_CMD}" ]]; then
        log_error "Docker Compose not available"
        return 1
    fi

    if [[ ! -f "${COMPOSE_FILE}" ]]; then
        log_error "Compose file not found: ${COMPOSE_FILE}"
        return 1
    fi

    # Ensure database is running
    log_info "Ensuring database is running..."
    $COMPOSE_CMD -f "${COMPOSE_FILE}" up -d postgres 2>/dev/null || true
    sleep 3

    # Run migration in app container
    local migrate_cmd="/app/llm-cost-ops migrate"

    case "${action}" in
        up)
            migrate_cmd="${migrate_cmd}"
            ;;
        down)
            migrate_cmd="${migrate_cmd} --rollback"
            if [[ -n "${STEPS}" ]]; then
                migrate_cmd="${migrate_cmd} --steps ${STEPS}"
            fi
            ;;
        *)
            log_error "Unknown action: ${action}"
            return 1
            ;;
    esac

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute: ${migrate_cmd}"
        return 0
    fi

    log_info "Executing: ${migrate_cmd}"

    # Run migration
    if $COMPOSE_CMD -f "${COMPOSE_FILE}" run --rm app ${migrate_cmd}; then
        log_success "Migration completed successfully"
        return 0
    else
        log_error "Migration failed"
        return 1
    fi
}

run_migration_native() {
    local action="$1"

    log_info "Running migration natively..."

    # Check if binary exists
    local binary="${PROJECT_ROOT}/target/release/llm-cost-ops"
    if [[ ! -f "${binary}" ]]; then
        binary="${PROJECT_ROOT}/target/debug/llm-cost-ops"
    fi

    if [[ ! -f "${binary}" ]]; then
        log_error "Binary not found. Please build the project first."
        log_info "Run: cargo build --release"
        return 1
    fi

    local migrate_cmd="${binary} migrate"

    case "${action}" in
        up)
            migrate_cmd="${migrate_cmd}"
            ;;
        down)
            migrate_cmd="${migrate_cmd} --rollback"
            if [[ -n "${STEPS}" ]]; then
                migrate_cmd="${migrate_cmd} --steps ${STEPS}"
            fi
            ;;
        *)
            log_error "Unknown action: ${action}"
            return 1
            ;;
    esac

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would execute: ${migrate_cmd}"
        return 0
    fi

    log_info "Executing: ${migrate_cmd}"

    # Set environment variable
    export DATABASE_URL="${DATABASE_URL}"

    # Run migration
    if ${migrate_cmd}; then
        log_success "Migration completed successfully"
        return 0
    else
        log_error "Migration failed"
        return 1
    fi
}

migrate_up() {
    log_info "Running migrations..."

    if run_migration_in_docker "up" 2>/dev/null || run_migration_native "up"; then
        return 0
    else
        return 1
    fi
}

migrate_down() {
    log_warn "Rolling back migrations..."

    if [[ "${FORCE}" != "true" ]]; then
        read -p "Are you sure you want to rollback? (yes/no): " -r
        echo
        if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
            log_info "Rollback cancelled"
            return 0
        fi
    fi

    if run_migration_in_docker "down" 2>/dev/null || run_migration_native "down"; then
        return 0
    else
        return 1
    fi
}

show_migration_status() {
    log_info "Migration status:"
    echo ""

    # Count migration files
    local total_migrations
    total_migrations=$(find "${MIGRATION_DIR}" -name "*.sql" -type f | wc -l)

    echo "  Migration directory: ${MIGRATION_DIR}"
    echo "  Total migrations:    ${total_migrations}"
    echo "  Database type:       ${DATABASE_TYPE}"
    echo "  Database URL:        ${DATABASE_URL}"
    echo ""

    # List migrations
    log_info "Available migrations:"
    find "${MIGRATION_DIR}" -name "*.sql" -type f | sort | while read -r migration; do
        echo "  - $(basename "${migration}")"
    done
    echo ""

    # Try to get applied migrations from database
    case "${DATABASE_TYPE}" in
        postgres)
            if command -v psql &> /dev/null && [[ "${DATABASE_URL}" == postgresql://* ]]; then
                log_info "Applied migrations:"
                psql "${DATABASE_URL}" -c "SELECT version, applied_at FROM schema_migrations ORDER BY version;" 2>/dev/null || \
                    log_warn "Could not query migration table (may not exist yet)"
            else
                log_warn "psql not available, cannot check applied migrations"
            fi
            ;;
        sqlite)
            if command -v sqlite3 &> /dev/null; then
                local db_path="${DATABASE_URL#sqlite://}"
                if [[ -f "${db_path}" ]]; then
                    log_info "Applied migrations:"
                    sqlite3 "${db_path}" "SELECT version, applied_at FROM schema_migrations ORDER BY version;" 2>/dev/null || \
                        log_warn "Could not query migration table (may not exist yet)"
                else
                    log_warn "Database file not found: ${db_path}"
                fi
            else
                log_warn "sqlite3 not available, cannot check applied migrations"
            fi
            ;;
    esac
}

create_migration() {
    log_info "Creating new migration..."

    # Get migration name
    local name="${1:-new_migration}"
    name=$(echo "${name}" | tr '[:upper:]' '[:lower:]' | tr ' ' '_')

    # Generate timestamp
    local timestamp
    timestamp=$(date +%Y%m%d%H%M%S)

    # Create migration files
    local up_file="${MIGRATION_DIR}/${timestamp}_${name}.up.sql"
    local down_file="${MIGRATION_DIR}/${timestamp}_${name}.down.sql"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would create:"
        echo "  - ${up_file}"
        echo "  - ${down_file}"
        return 0
    fi

    # Create up migration
    cat > "${up_file}" << 'EOF'
-- Migration: ${name}
-- Created: ${timestamp}

-- Add your migration SQL here

EOF

    # Create down migration
    cat > "${down_file}" << 'EOF'
-- Rollback migration: ${name}
-- Created: ${timestamp}

-- Add your rollback SQL here

EOF

    log_success "Created migration files:"
    echo "  UP:   ${up_file}"
    echo "  DOWN: ${down_file}"
}

reset_database() {
    log_error "DESTRUCTIVE OPERATION: This will drop and recreate the database!"

    if [[ "${FORCE}" != "true" ]]; then
        read -p "Are you absolutely sure? Type 'RESET' to confirm: " -r
        echo
        if [[ "$REPLY" != "RESET" ]]; then
            log_info "Reset cancelled"
            return 0
        fi
    fi

    log_warn "Resetting database..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would reset database"
        return 0
    fi

    case "${DATABASE_TYPE}" in
        postgres)
            if command -v psql &> /dev/null; then
                local db_name="${DATABASE_URL##*/}"
                local base_url="${DATABASE_URL%/*}"

                # Drop and recreate database
                psql "${base_url}/postgres" -c "DROP DATABASE IF EXISTS ${db_name};" 2>/dev/null || true
                psql "${base_url}/postgres" -c "CREATE DATABASE ${db_name};" 2>/dev/null || true

                log_success "Database reset"
            else
                log_error "psql not available"
                return 1
            fi
            ;;
        sqlite)
            local db_path="${DATABASE_URL#sqlite://}"
            if [[ -f "${db_path}" ]]; then
                rm -f "${db_path}"
                log_success "Database file removed: ${db_path}"
            fi
            ;;
    esac

    # Run migrations
    migrate_up
}

verify_migrations() {
    log_info "Verifying migrations..."

    local errors=0

    # Check for SQL syntax errors
    find "${MIGRATION_DIR}" -name "*.sql" -type f | while read -r migration; do
        log_info "Checking: $(basename "${migration}")"

        # Basic syntax checks
        if ! grep -q ";" "${migration}"; then
            log_warn "  Missing semicolon in ${migration}"
            ((errors++))
        fi

        # Check for common issues
        if grep -qi "drop database" "${migration}"; then
            log_warn "  Contains DROP DATABASE command: ${migration}"
        fi
    done

    if [[ ${errors} -eq 0 ]]; then
        log_success "All migrations verified"
        return 0
    else
        log_warn "Found ${errors} potential issues"
        return 1
    fi
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
            up|down|status|create|reset|verify)
                ACTION="$1"
                shift
                ;;
            -d|--database)
                DATABASE_TYPE="$2"
                shift 2
                ;;
            -u|--url)
                DATABASE_URL="$2"
                shift 2
                ;;
            -s|--steps)
                STEPS="$2"
                shift 2
                ;;
            -f|--file)
                COMPOSE_FILE="$2"
                shift 2
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
                # Treat as migration name for create action
                if [[ "${ACTION}" == "create" ]]; then
                    MIGRATION_NAME="$1"
                    shift
                else
                    log_error "Unknown option: $1"
                    show_usage
                    exit 1
                fi
                ;;
        esac
    done

    log_info "Starting database migration: ${ACTION}"

    # Check prerequisites
    check_prerequisites

    # Detect database URL
    detect_database_url

    # Execute action
    case "${ACTION}" in
        up)
            migrate_up || exit 1
            ;;
        down)
            migrate_down || exit 1
            ;;
        status)
            show_migration_status
            ;;
        create)
            create_migration "${MIGRATION_NAME:-new_migration}"
            ;;
        reset)
            reset_database || exit 1
            ;;
        verify)
            verify_migrations || exit 1
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
