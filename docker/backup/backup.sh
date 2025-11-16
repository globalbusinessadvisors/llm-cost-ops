#!/bin/sh
# Backup script for LLM Cost Ops - Production
# Version: 1.0.0

set -e

# Configuration
BACKUP_DIR="${BACKUP_DIR:-/backup}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"

# Database configuration
POSTGRES_HOST="${POSTGRES_HOST:-postgres-primary}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-llm_cost_ops}"
PGPASSWORD="${POSTGRES_PASSWORD}"
export PGPASSWORD

# S3 configuration (optional)
S3_BUCKET="${S3_BUCKET:-}"
AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-}"
AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}"

# Logging
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1"
}

error() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1" >&2
}

# Create backup directory
mkdir -p "$BACKUP_DIR/postgres"
mkdir -p "$BACKUP_DIR/grafana"
mkdir -p "$BACKUP_DIR/prometheus"

# Backup PostgreSQL
backup_postgres() {
    log "Starting PostgreSQL backup..."

    BACKUP_FILE="$BACKUP_DIR/postgres/postgres_${TIMESTAMP}.sql.gz"

    if pg_dump -h "$POSTGRES_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" | gzip > "$BACKUP_FILE"; then
        log "PostgreSQL backup completed: $BACKUP_FILE"

        # Generate checksum
        sha256sum "$BACKUP_FILE" > "$BACKUP_FILE.sha256"

        return 0
    else
        error "PostgreSQL backup failed"
        return 1
    fi
}

# Backup Grafana
backup_grafana() {
    log "Starting Grafana backup..."

    BACKUP_FILE="$BACKUP_DIR/grafana/grafana_${TIMESTAMP}.tar.gz"

    if [ -d "/data/grafana" ]; then
        if tar -czf "$BACKUP_FILE" -C /data grafana; then
            log "Grafana backup completed: $BACKUP_FILE"
            sha256sum "$BACKUP_FILE" > "$BACKUP_FILE.sha256"
            return 0
        else
            error "Grafana backup failed"
            return 1
        fi
    else
        log "Grafana data directory not found, skipping..."
        return 0
    fi
}

# Backup Prometheus
backup_prometheus() {
    log "Starting Prometheus backup..."

    BACKUP_FILE="$BACKUP_DIR/prometheus/prometheus_${TIMESTAMP}.tar.gz"

    if [ -d "/data/prometheus" ]; then
        if tar -czf "$BACKUP_FILE" -C /data prometheus; then
            log "Prometheus backup completed: $BACKUP_FILE"
            sha256sum "$BACKUP_FILE" > "$BACKUP_FILE.sha256"
            return 0
        else
            error "Prometheus backup failed"
            return 1
        fi
    else
        log "Prometheus data directory not found, skipping..."
        return 0
    fi
}

# Upload to S3 (optional)
upload_to_s3() {
    if [ -n "$S3_BUCKET" ] && [ -n "$AWS_ACCESS_KEY_ID" ]; then
        log "Uploading backups to S3..."

        # Install AWS CLI if not present
        if ! command -v aws > /dev/null 2>&1; then
            apk add --no-cache aws-cli
        fi

        aws s3 sync "$BACKUP_DIR" "s3://$S3_BUCKET/backups/$(date +%Y/%m/%d)/" \
            --exclude "*" \
            --include "*.gz" \
            --include "*.sha256"

        log "S3 upload completed"
    fi
}

# Cleanup old backups
cleanup_old_backups() {
    log "Cleaning up backups older than $RETENTION_DAYS days..."

    find "$BACKUP_DIR" -type f -name "*.gz" -mtime +$RETENTION_DAYS -delete
    find "$BACKUP_DIR" -type f -name "*.sha256" -mtime +$RETENTION_DAYS -delete

    log "Cleanup completed"
}

# Main backup routine
main() {
    log "=== Starting backup routine ==="

    backup_postgres
    backup_grafana
    backup_prometheus
    upload_to_s3
    cleanup_old_backups

    log "=== Backup routine completed ==="
}

# Run backup
main "$@"
