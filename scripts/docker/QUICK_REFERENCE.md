# Docker Scripts - Quick Reference Guide

## One-Liners for Common Tasks

### Development

```bash
# Start development environment
./deploy-dev.sh up --follow

# Rebuild and restart
./build.sh && ./deploy-dev.sh restart

# View application logs
./logs.sh --follow app

# Run migrations
./migrate.sh up
```

### Building & Pushing

```bash
# Build for current platform
./build.sh

# Build multi-arch for production
./build.sh --platforms linux/amd64,linux/arm64 --tag v1.0.0 --push

# Push to GitHub Container Registry
./push.sh --ghcr --version v1.0.0

# Push to multiple registries
./push.sh --docker --ghcr --ecr
```

### Production Deployment

```bash
# Full production deployment workflow
./backup.sh --tag pre-deploy && \
./build.sh --tag v1.0.0 --push && \
./deploy-prod.sh --tag v1.0.0

# Blue-green deployment
./deploy-prod.sh --tag v1.0.0 --strategy blue-green

# Dry run deployment
./deploy-prod.sh --dry-run --tag v1.0.0
```

### Kubernetes

```bash
# Deploy to development
./deploy-k8s.sh --env dev

# Deploy to production
./deploy-k8s.sh --env prod --tag v1.0.0

# Deploy with Helm
./deploy-helm.sh upgrade --env prod --tag v1.0.0

# Check deployment status
kubectl get pods -n llm-cost-ops
```

### Database Operations

```bash
# Run migrations
./migrate.sh up

# Rollback last migration
./migrate.sh down

# Check migration status
./migrate.sh status

# Create new migration
./migrate.sh create add_new_feature
```

### Logs & Monitoring

```bash
# Follow all logs
./logs.sh --follow

# View errors only
./logs.sh --filter "ERROR"

# Last 100 lines from app
./logs.sh --tail 100 app

# Export logs to file
./logs.sh --tail 1000 > logs-$(date +%Y%m%d).txt
```

### Backup & Restore

```bash
# Full backup
./backup.sh

# Backup to S3
./backup.sh --s3 --s3-bucket my-backups --tag daily

# Database only backup
./backup.sh --no-volumes --no-config

# List backups
./backup.sh --list
```

### Cleanup

```bash
# Safe cleanup (containers & networks)
./cleanup.sh

# Full cleanup (DESTRUCTIVE!)
./cleanup.sh --all --force

# Clean old images only
./cleanup.sh --images

# Preview cleanup
./cleanup.sh --all --dry-run
```

## Common Workflows

### CI/CD Pipeline

```bash
#!/bin/bash
set -e

# 1. Build and tag
./build.sh --tag ${CI_COMMIT_SHA} --tag latest

# 2. Run tests (add your test command)
# docker run --rm llm-cost-ops:${CI_COMMIT_SHA} cargo test

# 3. Push to registry
./push.sh --ghcr --version ${CI_COMMIT_SHA}

# 4. Deploy (if main branch)
if [ "$CI_COMMIT_BRANCH" = "main" ]; then
  ./deploy-prod.sh --tag ${CI_COMMIT_SHA}
fi
```

### Daily Backup Job

```bash
#!/bin/bash
# Add to crontab: 0 2 * * * /path/to/daily-backup.sh

./backup.sh \
  --tag daily-$(date +%Y%m%d) \
  --s3 \
  --s3-bucket production-backups \
  --retention 30
```

### Health Check Script

```bash
#!/bin/bash

# Check if services are healthy
if ./logs.sh --since 1m --filter "ERROR" | grep -q "ERROR"; then
  echo "Errors detected in last minute!"
  ./logs.sh --since 1m --filter "ERROR"
  exit 1
fi

echo "All services healthy"
```

### Rolling Deployment Script

```bash
#!/bin/bash
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

# Pre-deployment
echo "==> Creating backup..."
./backup.sh --tag pre-${VERSION}

# Build
echo "==> Building version ${VERSION}..."
./build.sh --tag ${VERSION} --platforms linux/amd64,linux/arm64

# Push
echo "==> Pushing to registry..."
./push.sh --ghcr --version ${VERSION}

# Deploy
echo "==> Deploying version ${VERSION}..."
./deploy-prod.sh --tag ${VERSION} --strategy rolling

# Verify
echo "==> Verifying deployment..."
sleep 10
./logs.sh --tail 50 --filter "ERROR"

echo "==> Deployment complete!"
```

## Environment-Specific Commands

### Development
```bash
export COMPOSE_FILE=docker-compose.yml
export ENV_FILE=.env
./deploy-dev.sh up
```

### Staging
```bash
export COMPOSE_FILE=docker-compose.staging.yml
export ENV_FILE=.env.staging
./deploy-dev.sh up
```

### Production
```bash
export COMPOSE_FILE=docker-compose.prod.yml
export ENV_FILE=.env.prod
./deploy-prod.sh --tag v1.0.0
```

## Emergency Procedures

### Rollback Production

```bash
# Immediate rollback
./deploy-prod.sh --tag v0.9.0  # Previous stable version

# Or using Helm
./deploy-helm.sh rollback
```

### Restore from Backup

```bash
# 1. Stop services
./cleanup.sh --containers

# 2. Restore database
# (Add specific restore commands based on backup format)

# 3. Restart services
./deploy-prod.sh --tag v1.0.0
```

### Fix Stuck Migration

```bash
# Check status
./migrate.sh status

# Rollback problematic migration
./migrate.sh down --steps 1

# Fix migration file
# vim migrations/XXXXXX_problematic.up.sql

# Re-run
./migrate.sh up
```

### Clear Everything and Start Fresh

```bash
# WARNING: DESTRUCTIVE!
./cleanup.sh --all --force
./deploy-dev.sh up
./migrate.sh up
```

## Debugging Commands

### Build Issues
```bash
# Verbose build
./build.sh --no-cache 2>&1 | tee build.log

# Check Docker
docker info
docker buildx ls
```

### Deployment Issues
```bash
# Check services
./deploy-dev.sh status

# View all logs
./logs.sh --tail 200

# Check resource usage
docker stats

# Verify environment
cat .env | grep -v '^#' | grep -v '^$'
```

### Database Issues
```bash
# Connect to database
docker compose exec postgres psql -U postgres llm_cost_ops_dev

# Check migration table
docker compose exec postgres psql -U postgres llm_cost_ops_dev \
  -c "SELECT * FROM schema_migrations;"

# Verify database size
docker compose exec postgres psql -U postgres llm_cost_ops_dev \
  -c "SELECT pg_size_pretty(pg_database_size('llm_cost_ops_dev'));"
```

### Kubernetes Issues
```bash
# Check pods
kubectl get pods -n llm-cost-ops

# Describe pod
kubectl describe pod <pod-name> -n llm-cost-ops

# View logs
kubectl logs -f <pod-name> -n llm-cost-ops

# Execute in pod
kubectl exec -it <pod-name> -n llm-cost-ops -- /bin/bash

# Check resources
kubectl top pods -n llm-cost-ops
```

## Performance Optimization

### Build Performance
```bash
# Use registry cache
./build.sh --cache-type registry

# Use local cache
./build.sh --cache-type local

# Build specific platform only
./build.sh --platforms linux/amd64
```

### Deployment Performance
```bash
# Skip health checks for faster deployment (not recommended)
./deploy-dev.sh up --no-health-check

# Reduce wait timeout
./deploy-k8s.sh --wait-timeout 60s
```

## Monitoring & Alerts

### Log Monitoring
```bash
# Monitor for errors in real-time
./logs.sh --follow --filter "ERROR|FATAL|CRITICAL"

# Count errors in last hour
./logs.sh --since 1h --filter "ERROR" | wc -l

# Find most common errors
./logs.sh --since 1h --filter "ERROR" | \
  sort | uniq -c | sort -rn | head -10
```

### Resource Monitoring
```bash
# Watch container resources
watch docker stats

# Check disk usage
docker system df

# Monitor specific service
docker stats llm-cost-ops-app
```

## Cheat Sheet

| Task | Command |
|------|---------|
| Start dev | `./deploy-dev.sh up` |
| Stop dev | `./deploy-dev.sh down` |
| View logs | `./logs.sh --follow app` |
| Run migrations | `./migrate.sh up` |
| Backup | `./backup.sh` |
| Build | `./build.sh` |
| Deploy prod | `./deploy-prod.sh --tag v1.0.0` |
| Cleanup | `./cleanup.sh` |
| Push image | `./push.sh --ghcr` |
| K8s deploy | `./deploy-k8s.sh --env prod` |
| Helm deploy | `./deploy-helm.sh upgrade` |

## Tips & Tricks

1. **Always dry-run first**: Add `--dry-run` to test commands
2. **Use tags**: Tag builds with versions for easier rollback
3. **Monitor logs**: Keep logs running during deployments
4. **Backup often**: Automate backups before deployments
5. **Test locally**: Use dev environment to test changes
6. **Document changes**: Add comments to migration files
7. **Check resources**: Monitor disk and memory usage
8. **Use environment files**: Separate configs per environment
9. **Validate configs**: Use dry-run to validate before applying
10. **Keep it simple**: Use defaults when possible

## Getting Help

```bash
# Script-specific help
./script.sh --help

# List all available options
./script.sh -h

# View examples in README
cat README.md | grep -A 20 "Examples"
```
