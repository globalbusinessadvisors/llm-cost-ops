# LLM Cost Ops - Docker Deployment Scripts

Comprehensive build and deployment scripts for LLM Cost Ops Docker containerization.

## Table of Contents

- [Overview](#overview)
- [Scripts](#scripts)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
- [Environment Variables](#environment-variables)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

## Overview

This directory contains production-ready scripts for building, deploying, and managing LLM Cost Ops in containerized environments. All scripts include:

- Comprehensive error handling
- Dry-run mode for testing
- Detailed logging and progress output
- Built-in help documentation
- Idempotent operations where possible
- Environment variable support

## Scripts

### 1. build.sh - Build Docker Images

Build Docker images with multi-architecture support, caching, and tag management.

```bash
./build.sh [OPTIONS]
```

**Features:**
- Multi-arch support (amd64, arm64)
- Build cache optimization (inline, registry, local)
- Automatic tag generation (version, commit, latest)
- Build arguments and labels
- Progress output and verification

**Examples:**
```bash
# Build for current platform
./build.sh

# Build multi-arch and push
./build.sh --platforms linux/amd64,linux/arm64 --push

# Build with custom tag
./build.sh --tag v1.0.0

# Dry run
./build.sh --dry-run --platforms linux/amd64,linux/arm64
```

### 2. push.sh - Push Images to Registry

Push Docker images to multiple registries with authentication and verification.

```bash
./push.sh [OPTIONS]
```

**Features:**
- Support for Docker Hub, GHCR, ECR, GCR
- Automatic authentication handling
- Multi-arch manifest creation
- Image verification after push
- Retry logic for failures

**Examples:**
```bash
# Push to GitHub Container Registry
./push.sh --ghcr

# Push to multiple registries
./push.sh --docker --ghcr --ecr

# Push specific version
./push.sh --version v1.2.3 --ghcr

# Push to all configured registries
./push.sh --all
```

### 3. deploy-dev.sh - Deploy Development Environment

Deploy development stack with Docker Compose, health checks, and initialization.

```bash
./deploy-dev.sh [OPTIONS] [ACTION]
```

**Features:**
- Complete development stack deployment
- Database initialization and migrations
- Health checks with timeout
- Service status monitoring
- Log tailing support

**Examples:**
```bash
# Start all services
./deploy-dev.sh up

# Start and follow logs
./deploy-dev.sh up --follow

# Start specific services
./deploy-dev.sh up --service app --service postgres

# Restart services
./deploy-dev.sh restart

# View logs
./deploy-dev.sh logs

# Stop and clean up
./deploy-dev.sh down
```

### 4. deploy-prod.sh - Deploy Production Environment

Production deployment with blue-green support, health checks, and rollback capability.

```bash
./deploy-prod.sh [OPTIONS]
```

**Features:**
- Multiple deployment strategies (rolling, blue-green, recreate)
- Pre-deployment validation and checks
- Automatic backup before deployment
- Health verification and smoke tests
- Auto-rollback on failure
- Deployment state tracking

**Examples:**
```bash
# Deploy with rolling update
./deploy-prod.sh --tag v1.2.3

# Deploy with blue-green strategy
./deploy-prod.sh --tag v1.2.3 --strategy blue-green

# Deploy without backup (not recommended)
./deploy-prod.sh --no-backup

# Dry run
./deploy-prod.sh --dry-run --tag v1.2.3
```

### 5. deploy-k8s.sh - Deploy to Kubernetes

Deploy to Kubernetes using kubectl and kustomize with namespace and secret management.

```bash
./deploy-k8s.sh [OPTIONS]
```

**Features:**
- Namespace creation and labeling
- Secret management from environment files
- Kustomize overlay support
- Image tag updates
- Rollout status verification
- Health checks

**Examples:**
```bash
# Deploy to development
./deploy-k8s.sh --env dev

# Deploy to production
./deploy-k8s.sh --env prod --tag v1.2.3

# Deploy to custom namespace
./deploy-k8s.sh --namespace my-app --env staging

# Dry run
./deploy-k8s.sh --dry-run --env prod
```

### 6. deploy-helm.sh - Deploy with Helm

Deploy to Kubernetes using Helm charts with values file selection and release management.

```bash
./deploy-helm.sh [OPTIONS] [ACTION]
```

**Features:**
- Automatic values file selection by environment
- Chart validation and linting
- Release history and rollback
- Atomic deployments
- Custom values override
- Test execution

**Examples:**
```bash
# Install release
./deploy-helm.sh install

# Upgrade release
./deploy-helm.sh upgrade --tag v1.2.3

# Install to production
./deploy-helm.sh install --env prod --tag v1.0.0

# Rollback to previous version
./deploy-helm.sh rollback

# Check release status
./deploy-helm.sh status

# Run tests
./deploy-helm.sh test
```

### 7. cleanup.sh - Clean Up Resources

Clean up Docker resources including containers, volumes, images, and networks.

```bash
./cleanup.sh [OPTIONS]
```

**Features:**
- Selective cleanup (containers, volumes, images, networks)
- Build cache cleanup
- Confirmation prompts for destructive operations
- Dry-run preview
- Docker system information

**Examples:**
```bash
# Clean containers and networks (safe)
./cleanup.sh

# Clean everything
./cleanup.sh --all

# Clean specific resources
./cleanup.sh --containers --volumes

# Preview cleanup
./cleanup.sh --all --dry-run

# Force cleanup without confirmation
./cleanup.sh --all --force
```

### 8. migrate.sh - Database Migrations

Run database migrations for SQLite and PostgreSQL with rollback support.

```bash
./migrate.sh [OPTIONS] [ACTION]
```

**Features:**
- Support for SQLite and PostgreSQL
- Migration up/down/status
- Step-based rollback
- Migration file creation
- Database reset capability
- Verification and integrity checks

**Examples:**
```bash
# Run pending migrations
./migrate.sh up

# Rollback last migration
./migrate.sh down

# Rollback 3 migrations
./migrate.sh down --steps 3

# Check migration status
./migrate.sh status

# Create new migration
./migrate.sh create add_user_table

# Reset database (DESTRUCTIVE!)
./migrate.sh reset --force
```

### 9. logs.sh - Log Aggregation and Viewing

Aggregate and view logs from Docker containers with filtering and analysis.

```bash
./logs.sh [OPTIONS] [SERVICES...]
```

**Features:**
- Service-specific or aggregate logs
- Follow mode (tail -f)
- Time-based filtering (since, until)
- Pattern filtering (grep)
- Timestamp display
- Log export and analysis

**Examples:**
```bash
# View all logs
./logs.sh

# Follow application logs
./logs.sh --follow app

# View last 50 lines from multiple services
./logs.sh --tail 50 app postgres redis

# Filter logs by pattern
./logs.sh --filter "ERROR" app

# Show logs since 1 hour ago
./logs.sh --since 1h app

# Export logs to file
./logs.sh --tail 1000 > logs.txt
```

### 10. backup.sh - Backup Data and Configurations

Backup databases, volumes, and configurations with S3 upload support.

```bash
./backup.sh [OPTIONS]
```

**Features:**
- Database backup (PostgreSQL, SQLite)
- Docker volume backup
- Configuration backup
- Compression and archiving
- S3 upload support
- Retention policy management
- Backup manifest generation

**Examples:**
```bash
# Full backup
./backup.sh

# Database only backup
./backup.sh --no-volumes --no-config

# Backup with custom tag
./backup.sh --tag pre-deployment

# Backup and upload to S3
./backup.sh --s3 --s3-bucket my-backups

# List existing backups
./backup.sh --list

# Dry run
./backup.sh --dry-run
```

## Prerequisites

### Required Tools

All scripts require:
- **Docker**: Version 20.10 or later
- **Docker Compose**: Version 2.0 or later (or docker-compose v1.29+)
- **Bash**: Version 4.0 or later

### Script-Specific Requirements

**build.sh / push.sh:**
- Docker buildx for multi-arch builds
- Registry credentials configured

**deploy-k8s.sh:**
- kubectl configured with cluster access
- kustomize (standalone or kubectl kustomize)

**deploy-helm.sh:**
- Helm 3.x installed
- kubectl configured

**backup.sh:**
- tar, gzip for compression
- aws CLI for S3 uploads (optional)
- pg_dump for PostgreSQL backups (optional)
- sqlite3 for SQLite backups (optional)

**migrate.sh:**
- psql for PostgreSQL migrations (optional)
- sqlite3 for SQLite migrations (optional)

### Installation

```bash
# Make all scripts executable
chmod +x /workspaces/llm-cost-ops/scripts/docker/*.sh

# Add to PATH (optional)
export PATH="$PATH:/workspaces/llm-cost-ops/scripts/docker"
```

## Quick Start

### Development Workflow

```bash
# 1. Build the image
./build.sh

# 2. Deploy development environment
./deploy-dev.sh up

# 3. Run migrations
./migrate.sh up

# 4. View logs
./logs.sh --follow app

# 5. Stop environment
./deploy-dev.sh down
```

### Production Deployment Workflow

```bash
# 1. Build multi-arch image
./build.sh --platforms linux/amd64,linux/arm64 --tag v1.0.0

# 2. Push to registry
./push.sh --ghcr --version v1.0.0

# 3. Backup current state
./backup.sh --tag pre-v1.0.0

# 4. Deploy to production
./deploy-prod.sh --tag v1.0.0 --strategy rolling

# 5. Verify deployment
./logs.sh --tail 100 --filter "ERROR"
```

### Kubernetes Deployment Workflow

```bash
# Option 1: Using kubectl + kustomize
./deploy-k8s.sh --env prod --tag v1.0.0

# Option 2: Using Helm
./deploy-helm.sh install --env prod --tag v1.0.0
```

## Usage Examples

### CI/CD Integration

#### GitHub Actions

```yaml
- name: Build and push Docker image
  run: |
    ./scripts/docker/build.sh --tag ${{ github.sha }} --push

- name: Deploy to production
  run: |
    ./scripts/docker/deploy-prod.sh --tag ${{ github.sha }}
```

#### GitLab CI

```yaml
deploy:
  script:
    - ./scripts/docker/build.sh --tag $CI_COMMIT_SHA --push
    - ./scripts/docker/deploy-prod.sh --tag $CI_COMMIT_SHA
```

### Multi-Environment Deployment

```bash
# Development
./deploy-dev.sh up

# Staging
./deploy-k8s.sh --env staging --tag v1.0.0-rc1

# Production
./deploy-prod.sh --tag v1.0.0 --strategy blue-green
```

### Disaster Recovery

```bash
# Create backup
./backup.sh --s3 --s3-bucket disaster-recovery

# Stop services
./cleanup.sh --all

# Restore from backup
# (Implement restore functionality as needed)

# Redeploy
./deploy-prod.sh --tag v1.0.0
```

## Environment Variables

All scripts support environment variables for configuration:

### Common Variables

```bash
export DRY_RUN=true              # Enable dry-run mode
export FORCE=true                # Skip confirmation prompts
```

### Build & Push

```bash
export IMAGE_NAME=llm-cost-ops
export REGISTRY=ghcr.io/yourusername
export VERSION=v1.0.0
export PLATFORMS=linux/amd64,linux/arm64
```

### Deployment

```bash
export COMPOSE_FILE=/path/to/docker-compose.yml
export ENV_FILE=/path/to/.env
export NAMESPACE=llm-cost-ops
export ENVIRONMENT=prod
```

### Backup

```bash
export BACKUP_DIR=/path/to/backups
export RETENTION_DAYS=30
export S3_BUCKET=my-backups
export S3_PREFIX=llm-cost-ops/backups
```

### Migration

```bash
export DATABASE_TYPE=postgres
export DATABASE_URL=postgresql://user:pass@host:5432/db
export MIGRATION_DIR=/path/to/migrations
```

## Best Practices

### 1. Always Use Dry Run First

```bash
# Test commands before execution
./deploy-prod.sh --dry-run --tag v1.0.0
./cleanup.sh --all --dry-run
./backup.sh --dry-run
```

### 2. Tag Your Builds Semantically

```bash
# Use semantic versioning
./build.sh --tag v1.2.3

# Or git commit SHAs
./build.sh --tag $(git rev-parse --short HEAD)
```

### 3. Backup Before Major Changes

```bash
# Always backup before production deployments
./backup.sh --tag pre-v2.0.0 --s3
./deploy-prod.sh --tag v2.0.0
```

### 4. Monitor Logs During Deployment

```bash
# In one terminal
./deploy-prod.sh --tag v1.0.0

# In another terminal
./logs.sh --follow app
```

### 5. Use Environment-Specific Configurations

```bash
# Development
./deploy-dev.sh up

# Production
./deploy-prod.sh --tag v1.0.0 --env prod
```

### 6. Verify Deployments

```bash
# Check service health
kubectl get pods -n llm-cost-ops

# Review logs
./logs.sh --tail 100 --filter "ERROR\|WARN"

# Run smoke tests
curl -f http://localhost:8080/health
```

### 7. Regular Cleanup

```bash
# Weekly cleanup of old images
./cleanup.sh --images

# Monthly cleanup of old backups
./backup.sh  # Automatically cleans old backups
```

## Troubleshooting

### Common Issues

#### Build Fails

```bash
# Check Docker daemon
docker info

# Clean build cache
./cleanup.sh --build-cache

# Build with verbose output
./build.sh --no-cache 2>&1 | tee build.log
```

#### Deployment Fails

```bash
# Check service status
./deploy-dev.sh status

# View detailed logs
./logs.sh --tail 200 app

# Check resource usage
docker stats

# Verify environment variables
cat .env
```

#### Database Migration Issues

```bash
# Check migration status
./migrate.sh status

# Verify database connectivity
docker compose exec postgres pg_isready

# Review migration files
ls -la migrations/
```

#### Kubernetes Deployment Issues

```bash
# Check pod status
kubectl get pods -n llm-cost-ops

# Describe problematic pod
kubectl describe pod <pod-name> -n llm-cost-ops

# View pod logs
kubectl logs <pod-name> -n llm-cost-ops

# Check events
kubectl get events -n llm-cost-ops --sort-by='.lastTimestamp'
```

#### Cleanup Issues

```bash
# Force remove containers
docker rm -f $(docker ps -aq --filter "name=llm-cost-ops")

# Prune system
docker system prune -a --volumes -f

# Check disk space
df -h
docker system df
```

### Getting Help

All scripts include comprehensive help:

```bash
./build.sh --help
./deploy-prod.sh --help
./migrate.sh --help
# etc.
```

### Debug Mode

Enable verbose output for troubleshooting:

```bash
# Bash debug mode
bash -x ./build.sh

# Docker debug
export DOCKER_BUILDKIT=0
export COMPOSE_DOCKER_CLI_BUILD=0
```

## Script Architecture

All scripts follow a consistent structure:

1. **Configuration & Defaults**: Environment variable setup
2. **Helper Functions**: Logging, validation, utilities
3. **Prerequisites Check**: Tool availability and connectivity
4. **Core Functions**: Main business logic
5. **Command Line Parsing**: Argument handling
6. **Main Execution**: Orchestration and error handling

### Error Handling

All scripts use `set -euo pipefail` for:
- Exit on error (`-e`)
- Error on undefined variables (`-u`)
- Pipe failure detection (`-o pipefail`)

### Logging

Consistent logging with color-coded output:
- **INFO** (Blue): Informational messages
- **SUCCESS** (Green): Success messages
- **WARN** (Yellow): Warnings
- **ERROR** (Red): Errors

## Contributing

When adding new scripts or modifying existing ones:

1. Follow the existing script structure
2. Include comprehensive help documentation
3. Add dry-run support
4. Implement proper error handling
5. Add logging at key points
6. Test with various scenarios
7. Update this README

## License

These scripts are part of the LLM Cost Ops project and follow the same license.

## Support

For issues or questions:
1. Check the troubleshooting section
2. Review script help: `./script.sh --help`
3. Check logs: `./logs.sh`
4. Open an issue on the project repository
