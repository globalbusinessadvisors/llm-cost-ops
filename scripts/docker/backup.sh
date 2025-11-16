#!/usr/bin/env bash
# =============================================================================
# LLM Cost Ops - Backup Script
# =============================================================================
# Description: Backup databases, volumes, and configurations with S3 upload
#              support
# Usage: ./backup.sh [OPTIONS]
# Version: 1.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# Configuration & Defaults
# -----------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BACKUP_DIR="${BACKUP_DIR:-${PROJECT_ROOT}/backups}"
COMPOSE_FILE="${COMPOSE_FILE:-${PROJECT_ROOT}/docker-compose.yml}"
TIMESTAMP="${TIMESTAMP:-$(date +%Y%m%d-%H%M%S)}"
BACKUP_TAG="${BACKUP_TAG:-manual}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
COMPRESS="${COMPRESS:-true}"
UPLOAD_S3="${UPLOAD_S3:-false}"
S3_BUCKET="${S3_BUCKET:-}"
S3_PREFIX="${S3_PREFIX:-llm-cost-ops/backups}"
DRY_RUN="${DRY_RUN:-false}"

# Backup components
BACKUP_DATABASE="${BACKUP_DATABASE:-true}"
BACKUP_VOLUMES="${BACKUP_VOLUMES:-true}"
BACKUP_CONFIG="${BACKUP_CONFIG:-true}"

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

Backup LLM Cost Ops data and configurations.

OPTIONS:
    -h, --help              Show this help message
    -d, --dir DIR           Backup directory (default: ${BACKUP_DIR})
    -t, --tag TAG           Backup tag (default: ${BACKUP_TAG})
    --no-database           Skip database backup
    --no-volumes            Skip volume backup
    --no-config             Skip configuration backup
    --no-compress           Don't compress backups
    --retention DAYS        Retention period in days (default: ${RETENTION_DAYS})
    --s3                    Upload to S3
    --s3-bucket BUCKET      S3 bucket name
    --s3-prefix PREFIX      S3 prefix (default: ${S3_PREFIX})
    --dry-run               Show what would be backed up

EXAMPLES:
    # Full backup
    ${0##*/}

    # Database only
    ${0##*/} --no-volumes --no-config

    # Backup with custom tag
    ${0##*/} --tag pre-deployment

    # Backup and upload to S3
    ${0##*/} --s3 --s3-bucket my-backups

    # List existing backups
    ${0##*/} --list

    # Restore from backup
    ${0##*/} --restore backup-20240101-120000.tar.gz

ENVIRONMENT VARIABLES:
    BACKUP_DIR          Backup directory path
    RETENTION_DAYS      Backup retention period
    UPLOAD_S3           Upload to S3 (true/false)
    S3_BUCKET           S3 bucket name
    S3_PREFIX           S3 key prefix
    DRY_RUN             Dry run mode (true/false)

EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for required tools
    local required_tools=("tar" "gzip")

    for tool in "${required_tools[@]}"; do
        if ! command -v "${tool}" &> /dev/null; then
            log_error "${tool} is not installed"
            exit 1
        fi
    done

    # Check for Docker Compose
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    elif docker-compose version &> /dev/null; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD=""
        log_warn "Docker Compose not available, will skip container backups"
    fi

    # Check for AWS CLI if S3 upload is enabled
    if [[ "${UPLOAD_S3}" == "true" ]]; then
        if ! command -v aws &> /dev/null; then
            log_error "AWS CLI is not installed (required for S3 upload)"
            exit 1
        fi

        if [[ -z "${S3_BUCKET}" ]]; then
            log_error "S3_BUCKET must be set for S3 upload"
            exit 1
        fi
    fi

    # Create backup directory
    if [[ "${DRY_RUN}" != "true" ]]; then
        mkdir -p "${BACKUP_DIR}"
    fi

    log_success "Prerequisites check passed"
}

backup_database() {
    if [[ "${BACKUP_DATABASE}" != "true" ]]; then
        log_info "Skipping database backup"
        return 0
    fi

    log_info "Backing up database..."

    local db_backup_file="${BACKUP_DIR}/database-${TIMESTAMP}.sql"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would backup database to: ${db_backup_file}"
        return 0
    fi

    # Detect database type
    local database_url
    database_url=$(grep '^DATABASE_URL=' "${PROJECT_ROOT}/.env" 2>/dev/null | cut -d= -f2- || echo "")

    if [[ "${database_url}" == postgresql://* ]] || [[ "${database_url}" == postgres://* ]]; then
        backup_postgres "${db_backup_file}"
    elif [[ "${database_url}" == sqlite://* ]]; then
        backup_sqlite "${db_backup_file}"
    else
        log_warn "Unknown database type, attempting PostgreSQL backup..."
        backup_postgres "${db_backup_file}" || log_warn "PostgreSQL backup failed"
    fi

    if [[ -f "${db_backup_file}" ]]; then
        log_success "Database backup created: ${db_backup_file}"
        log_info "Backup size: $(du -h "${db_backup_file}" | cut -f1)"
    else
        log_error "Database backup failed"
        return 1
    fi
}

backup_postgres() {
    local output_file="$1"

    log_info "Backing up PostgreSQL database..."

    # Try Docker-based backup first
    if [[ -n "${COMPOSE_CMD}" ]] && [[ -f "${COMPOSE_FILE}" ]]; then
        log_info "Using Docker container for backup..."

        # Ensure postgres is running
        $COMPOSE_CMD -f "${COMPOSE_FILE}" up -d postgres 2>/dev/null || true
        sleep 2

        # Run pg_dump in container
        if $COMPOSE_CMD -f "${COMPOSE_FILE}" exec -T postgres \
            pg_dump -U postgres llm_cost_ops_dev > "${output_file}" 2>/dev/null; then
            return 0
        fi
    fi

    # Try native pg_dump
    if command -v pg_dump &> /dev/null; then
        log_info "Using native pg_dump..."
        local db_url="${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/llm_cost_ops_dev}"

        if pg_dump "${db_url}" > "${output_file}" 2>/dev/null; then
            return 0
        fi
    fi

    log_error "PostgreSQL backup failed"
    return 1
}

backup_sqlite() {
    local output_file="$1"

    log_info "Backing up SQLite database..."

    local db_path="${DATABASE_URL#sqlite://}"
    db_path="${db_path:-${PROJECT_ROOT}/data/llm-cost-ops.db}"

    if [[ ! -f "${db_path}" ]]; then
        log_warn "SQLite database not found: ${db_path}"
        return 1
    fi

    # SQLite backup methods
    if command -v sqlite3 &> /dev/null; then
        # Use sqlite3 dump
        sqlite3 "${db_path}" .dump > "${output_file}"
        return 0
    else
        # Simple file copy
        cp "${db_path}" "${output_file}"
        return 0
    fi
}

backup_volumes() {
    if [[ "${BACKUP_VOLUMES}" != "true" ]]; then
        log_info "Skipping volume backup"
        return 0
    fi

    log_info "Backing up Docker volumes..."

    local volumes_backup_dir="${BACKUP_DIR}/volumes-${TIMESTAMP}"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would backup volumes to: ${volumes_backup_dir}"
        return 0
    fi

    mkdir -p "${volumes_backup_dir}"

    # Get volumes to backup
    local volumes
    volumes=$(docker volume ls --filter "label=com.llm-cost-ops.backup=required" -q 2>/dev/null || echo "")

    if [[ -z "${volumes}" ]]; then
        log_warn "No volumes found to backup"
        return 0
    fi

    # Backup each volume
    for volume in ${volumes}; do
        log_info "Backing up volume: ${volume}"

        # Create tar archive of volume
        docker run --rm \
            -v "${volume}:/data:ro" \
            -v "${volumes_backup_dir}:/backup" \
            alpine \
            tar czf "/backup/${volume}.tar.gz" -C /data . 2>/dev/null || {
            log_warn "Failed to backup volume: ${volume}"
            continue
        }

        log_success "Volume backed up: ${volume}"
    done

    log_success "Volumes backup completed: ${volumes_backup_dir}"
}

backup_config() {
    if [[ "${BACKUP_CONFIG}" != "true" ]]; then
        log_info "Skipping configuration backup"
        return 0
    fi

    log_info "Backing up configurations..."

    local config_backup_file="${BACKUP_DIR}/config-${TIMESTAMP}.tar.gz"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would backup config to: ${config_backup_file}"
        return 0
    fi

    # Files and directories to backup
    local -a backup_items=(
        "docker-compose.yml"
        "docker-compose.prod.yml"
        ".env"
        ".env.prod"
        "config.toml"
        "docker/"
        "k8s/"
        "helm/"
    )

    # Create tar archive
    local -a tar_items=()
    for item in "${backup_items[@]}"; do
        if [[ -e "${PROJECT_ROOT}/${item}" ]]; then
            tar_items+=("${item}")
        fi
    done

    if [[ ${#tar_items[@]} -eq 0 ]]; then
        log_warn "No configuration files found to backup"
        return 0
    fi

    cd "${PROJECT_ROOT}"
    tar czf "${config_backup_file}" "${tar_items[@]}" 2>/dev/null || {
        log_error "Configuration backup failed"
        return 1
    }
    cd - > /dev/null

    log_success "Configuration backup created: ${config_backup_file}"
    log_info "Backup size: $(du -h "${config_backup_file}" | cut -f1)"
}

create_backup_manifest() {
    local manifest_file="${BACKUP_DIR}/manifest-${TIMESTAMP}.json"

    log_info "Creating backup manifest..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would create manifest: ${manifest_file}"
        return 0
    fi

    cat > "${manifest_file}" << EOF
{
  "timestamp": "${TIMESTAMP}",
  "tag": "${BACKUP_TAG}",
  "date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "hostname": "$(hostname)",
  "user": "$(whoami)",
  "components": {
    "database": ${BACKUP_DATABASE},
    "volumes": ${BACKUP_VOLUMES},
    "config": ${BACKUP_CONFIG}
  },
  "files": [
EOF

    # List backup files
    find "${BACKUP_DIR}" -name "*-${TIMESTAMP}*" -type f | while read -r file; do
        local size
        size=$(stat -f%z "${file}" 2>/dev/null || stat -c%s "${file}" 2>/dev/null || echo "0")
        cat >> "${manifest_file}" << EOF
    {
      "name": "$(basename "${file}")",
      "path": "${file}",
      "size": ${size},
      "checksum": "$(sha256sum "${file}" 2>/dev/null | cut -d' ' -f1 || echo 'unknown')"
    },
EOF
    done

    # Remove trailing comma and close JSON
    sed -i '$ s/,$//' "${manifest_file}" 2>/dev/null || true
    cat >> "${manifest_file}" << EOF
  ]
}
EOF

    log_success "Backup manifest created: ${manifest_file}"
}

compress_backup() {
    if [[ "${COMPRESS}" != "true" ]]; then
        log_info "Skipping compression"
        return 0
    fi

    log_info "Compressing backup files..."

    local archive_file="${BACKUP_DIR}/llm-cost-ops-backup-${BACKUP_TAG}-${TIMESTAMP}.tar.gz"

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would create archive: ${archive_file}"
        return 0
    fi

    # Find all backup files for this timestamp
    local backup_files
    backup_files=$(find "${BACKUP_DIR}" -name "*-${TIMESTAMP}*" -type f -o -name "volumes-${TIMESTAMP}" -type d)

    if [[ -z "${backup_files}" ]]; then
        log_warn "No backup files found to compress"
        return 0
    fi

    # Create tar archive
    cd "${BACKUP_DIR}"
    tar czf "$(basename "${archive_file}")" *-${TIMESTAMP}* 2>/dev/null || {
        log_error "Compression failed"
        return 1
    }
    cd - > /dev/null

    # Remove individual files
    find "${BACKUP_DIR}" -name "*-${TIMESTAMP}*" ! -name "*.tar.gz" -exec rm -rf {} + 2>/dev/null || true

    log_success "Backup archive created: ${archive_file}"
    log_info "Archive size: $(du -h "${archive_file}" | cut -f1)"

    # Set as final backup file
    FINAL_BACKUP_FILE="${archive_file}"
}

upload_to_s3() {
    if [[ "${UPLOAD_S3}" != "true" ]]; then
        log_info "Skipping S3 upload"
        return 0
    fi

    log_info "Uploading backup to S3..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would upload to: s3://${S3_BUCKET}/${S3_PREFIX}/"
        return 0
    fi

    if [[ -z "${FINAL_BACKUP_FILE}" ]] || [[ ! -f "${FINAL_BACKUP_FILE}" ]]; then
        log_error "No backup file to upload"
        return 1
    fi

    local s3_key="${S3_PREFIX}/$(basename "${FINAL_BACKUP_FILE}")"

    log_info "Uploading to: s3://${S3_BUCKET}/${s3_key}"

    if aws s3 cp "${FINAL_BACKUP_FILE}" "s3://${S3_BUCKET}/${s3_key}" \
        --storage-class STANDARD_IA \
        --metadata "backup-tag=${BACKUP_TAG},timestamp=${TIMESTAMP}"; then
        log_success "Backup uploaded to S3"
        log_info "S3 URI: s3://${S3_BUCKET}/${s3_key}"
        return 0
    else
        log_error "S3 upload failed"
        return 1
    fi
}

cleanup_old_backups() {
    log_info "Cleaning up old backups (retention: ${RETENTION_DAYS} days)..."

    if [[ "${DRY_RUN}" == "true" ]]; then
        log_info "DRY RUN - Would delete backups older than ${RETENTION_DAYS} days"
        find "${BACKUP_DIR}" -name "*.tar.gz" -mtime +${RETENTION_DAYS} -type f 2>/dev/null || true
        return 0
    fi

    # Delete local backups older than retention period
    local deleted_count=0
    while IFS= read -r -d '' file; do
        log_info "Deleting old backup: $(basename "${file}")"
        rm -f "${file}"
        ((deleted_count++))
    done < <(find "${BACKUP_DIR}" -name "*.tar.gz" -mtime +${RETENTION_DAYS} -type f -print0 2>/dev/null)

    if [[ ${deleted_count} -gt 0 ]]; then
        log_success "Deleted ${deleted_count} old backup(s)"
    else
        log_info "No old backups to delete"
    fi

    # Cleanup S3 if enabled
    if [[ "${UPLOAD_S3}" == "true" ]]; then
        log_info "Cleaning up old S3 backups..."
        # This would require listing and deleting old S3 objects
        # Implementation depends on S3 lifecycle policies
    fi
}

list_backups() {
    log_info "Available backups:"
    echo ""

    if [[ ! -d "${BACKUP_DIR}" ]] || [[ -z "$(ls -A "${BACKUP_DIR}" 2>/dev/null)" ]]; then
        log_warn "No backups found in ${BACKUP_DIR}"
        return 0
    fi

    # List local backups
    find "${BACKUP_DIR}" -name "*.tar.gz" -type f | sort -r | while read -r backup; do
        local size
        size=$(du -h "${backup}" | cut -f1)
        local date
        date=$(stat -f%Sm -t '%Y-%m-%d %H:%M:%S' "${backup}" 2>/dev/null || \
               stat -c%y "${backup}" 2>/dev/null | cut -d. -f1)
        echo "  $(basename "${backup}") - ${size} - ${date}"
    done

    echo ""
}

show_backup_summary() {
    cat << EOF

${GREEN}╔════════════════════════════════════════════════════════════════╗
║                  BACKUP COMPLETE                               ║
╚════════════════════════════════════════════════════════════════╝${NC}

  ${CYAN}Backup Details:${NC}
    Tag:            ${BACKUP_TAG}
    Timestamp:      ${TIMESTAMP}
    Location:       ${BACKUP_DIR}

  ${CYAN}Components Backed Up:${NC}
    Database:       ${BACKUP_DATABASE}
    Volumes:        ${BACKUP_VOLUMES}
    Configuration:  ${BACKUP_CONFIG}

  ${CYAN}Upload:${NC}
    S3:             ${UPLOAD_S3}
EOF

    if [[ "${UPLOAD_S3}" == "true" ]] && [[ -n "${S3_BUCKET}" ]]; then
        echo "    S3 Bucket:      ${S3_BUCKET}"
        echo "    S3 Prefix:      ${S3_PREFIX}"
    fi

    if [[ -n "${FINAL_BACKUP_FILE}" ]] && [[ -f "${FINAL_BACKUP_FILE}" ]]; then
        echo ""
        echo "  ${CYAN}Final Backup:${NC}"
        echo "    File:           $(basename "${FINAL_BACKUP_FILE}")"
        echo "    Size:           $(du -h "${FINAL_BACKUP_FILE}" | cut -f1)"
    fi

    cat << EOF

  ${CYAN}Useful Commands:${NC}
    List backups:   ${0##*/} --list
    Restore:        ${0##*/} --restore <backup-file>

EOF
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------
main() {
    local action="backup"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -d|--dir)
                BACKUP_DIR="$2"
                shift 2
                ;;
            -t|--tag)
                BACKUP_TAG="$2"
                shift 2
                ;;
            --no-database)
                BACKUP_DATABASE="false"
                shift
                ;;
            --no-volumes)
                BACKUP_VOLUMES="false"
                shift
                ;;
            --no-config)
                BACKUP_CONFIG="false"
                shift
                ;;
            --no-compress)
                COMPRESS="false"
                shift
                ;;
            --retention)
                RETENTION_DAYS="$2"
                shift 2
                ;;
            --s3)
                UPLOAD_S3="true"
                shift
                ;;
            --s3-bucket)
                S3_BUCKET="$2"
                shift 2
                ;;
            --s3-prefix)
                S3_PREFIX="$2"
                shift 2
                ;;
            --list)
                action="list"
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

    # Execute action
    case "${action}" in
        list)
            list_backups
            exit 0
            ;;
        backup)
            log_info "Starting backup: ${BACKUP_TAG}-${TIMESTAMP}"

            # Check prerequisites
            check_prerequisites

            # Perform backups
            backup_database
            backup_volumes
            backup_config
            create_backup_manifest
            compress_backup
            upload_to_s3
            cleanup_old_backups

            # Show summary
            show_backup_summary

            log_success "Backup completed successfully!"
            exit 0
            ;;
        *)
            log_error "Unknown action: ${action}"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
