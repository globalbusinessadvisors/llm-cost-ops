# Docker Quick Start Guide - LLM Cost Ops

## Prerequisites

- Docker Engine 20.10+
- Docker Compose V2
- At least 8GB RAM available
- At least 20GB disk space

## Quick Start - Development

### 1. Copy Environment File

```bash
cp .env.example .env
```

### 2. Start All Services

```bash
docker-compose up -d
```

This will start:
- Application (with hot-reload)
- PostgreSQL database
- Redis cache
- NATS message broker
- Prometheus monitoring
- Grafana dashboards
- Jaeger tracing
- MailHog email testing
- pgAdmin database UI
- Redis Commander cache UI

### 3. Wait for Services to be Healthy

```bash
docker-compose ps
```

All services should show "healthy" status within 1-2 minutes.

### 4. Access Services

| Service | URL | Credentials |
|---------|-----|-------------|
| **Application API** | http://localhost:8080 | - |
| **Application Metrics** | http://localhost:9090 | - |
| **Grafana** | http://localhost:3000 | admin / admin |
| **Prometheus** | http://localhost:9091 | - |
| **Jaeger UI** | http://localhost:16686 | - |
| **MailHog** | http://localhost:8025 | - |
| **pgAdmin** | http://localhost:5050 | admin@llm-cost-ops.local / admin |
| **Redis Commander** | http://localhost:8081 | admin / admin |

### 5. View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f app

# Last 100 lines
docker-compose logs --tail=100 app
```

### 6. Run Database Migrations

```bash
docker-compose exec app cargo sqlx migrate run
```

### 7. Stop Services

```bash
# Stop but keep data
docker-compose down

# Stop and remove all data
docker-compose down -v
```

## Quick Start - Testing

### Run All Tests

```bash
docker-compose -f docker-compose.test.yml up --abort-on-container-exit
```

### Run Specific Tests

```bash
# Unit tests
docker-compose -f docker-compose.test.yml run test-app

# Integration tests
docker-compose -f docker-compose.test.yml run test-integration

# Benchmarks
docker-compose -f docker-compose.test.yml run benchmark

# Security scan
docker-compose -f docker-compose.test.yml run security-scanner

# Lint check
docker-compose -f docker-compose.test.yml run lint-checker

# Format check
docker-compose -f docker-compose.test.yml run format-checker
```

## Quick Start - Production

### 1. Configure Environment

```bash
cp .env.example .env.production
```

Edit `.env.production` and set:
- `POSTGRES_PASSWORD` - Strong database password
- `JWT_SECRET` - Strong JWT secret (min 32 chars)
- `REDIS_PASSWORD` - Strong Redis password
- `GF_SECURITY_ADMIN_PASSWORD` - Grafana admin password
- `SMTP_*` - Email configuration
- `AWS_*` - Backup S3 credentials (optional)

### 2. Generate SSL Certificates

```bash
# Self-signed for testing
mkdir -p docker/nginx/ssl
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout docker/nginx/ssl/server.key \
  -out docker/nginx/ssl/server.crt
```

### 3. Start Production Stack

```bash
docker-compose -f docker-compose.prod.yml --env-file .env.production up -d
```

### 4. Verify All Services

```bash
docker-compose -f docker-compose.prod.yml ps
```

### 5. Access via Nginx

- HTTP: http://your-domain (redirects to HTTPS)
- HTTPS: https://your-domain

## Development Commands

### Access Development Shell

```bash
docker-compose exec dev-shell bash
```

### Run Cargo Commands

```bash
# Inside dev-shell
cargo build
cargo test
cargo clippy
cargo fmt
```

### Access Database

```bash
# Via pgAdmin web UI
open http://localhost:5050

# Or via SQL client
docker-compose exec sql-client psql
```

### Access Redis

```bash
# Via Redis Commander web UI
open http://localhost:8081

# Or via Redis CLI
docker-compose exec redis-cli redis-cli -h redis
```

### Generate Documentation

```bash
docker-compose up docs -d
open http://localhost:8000
```

## Troubleshooting

### Service Won't Start

```bash
# Check logs
docker-compose logs [service-name]

# Restart service
docker-compose restart [service-name]
```

### Database Connection Issues

```bash
# Check database is healthy
docker-compose ps postgres

# Access database directly
docker-compose exec postgres psql -U postgres -d llm_cost_ops_dev

# Reset database
docker-compose down -v
docker-compose up -d postgres
```

### Port Conflicts

If you see port conflict errors, edit `.env` or `docker-compose.override.yml` to change ports.

### Clean Slate Reset

```bash
# Stop everything
docker-compose down -v

# Remove all images
docker-compose down --rmi all

# Start fresh
docker-compose up -d
```

## Monitoring

### View Metrics in Grafana

1. Open http://localhost:3000
2. Login with admin/admin
3. Browse to Dashboards > LLM Cost Ops

### View Traces in Jaeger

1. Open http://localhost:16686
2. Select "llm-cost-ops" service
3. Click "Find Traces"

### Query Prometheus

1. Open http://localhost:9091
2. Try queries like:
   - `http_requests_total`
   - `process_resident_memory_bytes`
   - `rate(http_requests_total[5m])`

## Performance Tips

### For Development

```bash
# Reduce resource usage
export DOCKER_COMPOSE_PARALLEL=1
docker-compose up -d app postgres redis

# Skip unnecessary services
docker-compose up -d app postgres redis nats
```

### For CI/CD

```bash
# Run tests with coverage
docker-compose -f docker-compose.test.yml run test-runner

# Build production image
docker-compose -f docker-compose.prod.yml build
```

## Common Tasks

### Update Application Code

With hot-reload enabled, just edit your code and save. The app will automatically rebuild and restart.

### Add New Database Migration

```bash
# Create migration
docker-compose exec dev-shell bash
cargo sqlx migrate add your_migration_name

# Edit migration files in migrations/
# Then run:
docker-compose exec app cargo sqlx migrate run
```

### Export Database

```bash
docker-compose exec postgres pg_dump -U postgres llm_cost_ops_dev > backup.sql
```

### Import Database

```bash
cat backup.sql | docker-compose exec -T postgres psql -U postgres -d llm_cost_ops_dev
```

### Scale Application Instances

```bash
# Production only
docker-compose -f docker-compose.prod.yml up -d --scale app=3
```

## Next Steps

- Read [docs/docker/README.md](docs/docker/README.md) for detailed documentation
- Configure monitoring alerts in Prometheus/Grafana
- Set up CI/CD pipelines
- Configure SSL certificates for production
- Set up automated backups
- Review security hardening checklist

## Advanced Topics

### Docker Compose Configurations

For detailed service orchestration and configuration options, see:
- [Docker Compose Guide](docs/docker/DOCKER_COMPOSE.md) - Complete guide to service management

### Kubernetes Deployment

For production-scale orchestration with Kubernetes:
- [Kubernetes Guide](docs/docker/KUBERNETES.md) - Complete K8s deployment guide
- [Helm Charts](docs/docker/HELM.md) - Package management with Helm

### Security

For production security best practices:
- [Security Guide](docs/docker/SECURITY.md) - Container security, secrets, RBAC
- Review image scanning procedures
- Implement secret rotation
- Configure network policies

### Monitoring

For comprehensive observability:
- [Monitoring Guide](docs/docker/MONITORING.md) - Metrics, logs, traces, and alerting
- Set up custom dashboards
- Configure alert rules
- Enable distributed tracing

## Documentation Index

**Quick Start:**
- This file - Quick start for development
- [docs/docker/README.md](docs/docker/README.md) - Complete Docker guide (800+ lines)

**Orchestration:**
- [docs/docker/DOCKER_COMPOSE.md](docs/docker/DOCKER_COMPOSE.md) - Docker Compose reference (600+ lines)
- [docs/docker/KUBERNETES.md](docs/docker/KUBERNETES.md) - Kubernetes deployment (700+ lines)
- [docs/docker/HELM.md](docs/docker/HELM.md) - Helm charts guide (500+ lines)

**Operations:**
- [docs/docker/SECURITY.md](docs/docker/SECURITY.md) - Security best practices (400+ lines)
- [docs/docker/MONITORING.md](docs/docker/MONITORING.md) - Monitoring and observability (400+ lines)

## Support

For issues:
1. Check logs: `docker-compose logs -f [service]`
2. Check health: `docker-compose ps`
3. Review [docs/docker/README.md](docs/docker/README.md)
4. Check Grafana dashboards: http://localhost:3000
5. Review Jaeger traces: http://localhost:16686
6. Search documentation in `docs/docker/`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License

Apache 2.0 - See [LICENSE](LICENSE)
