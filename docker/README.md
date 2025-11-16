# Docker Configuration for LLM Cost Ops

This directory contains Docker and Docker Compose configurations for the LLM Cost Ops platform.

## Directory Structure

```
docker/
├── alertmanager/       # Alertmanager configuration
├── backup/            # Backup scripts and configuration
├── grafana/           # Grafana provisioning and dashboards
│   ├── provisioning/
│   │   ├── datasources/
│   │   └── dashboards/
│   └── dashboards/
├── loki/              # Loki log aggregation configuration
├── nats/              # NATS message broker configuration
├── nginx/             # Nginx reverse proxy configuration
│   └── conf.d/
├── postgres/          # PostgreSQL initialization scripts
├── prometheus/        # Prometheus monitoring configuration
├── promtail/          # Promtail log collection configuration
└── redis/             # Redis cache configuration
```

## Docker Compose Files

### 1. docker-compose.yml
Main development environment with:
- Application service (hot-reload enabled)
- PostgreSQL database
- Redis cache
- NATS message broker
- Prometheus monitoring
- Grafana dashboards
- Jaeger tracing
- MailHog (email testing)
- pgAdmin (database management)
- Redis Commander (cache management)

**Usage:**
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down

# Stop and remove volumes (clean slate)
docker-compose down -v
```

### 2. docker-compose.prod.yml
Production environment with:
- High-availability setup (2 app instances)
- PostgreSQL with replication (primary + replica)
- Redis with Sentinel (master + replica)
- NATS cluster (3 nodes)
- Nginx reverse proxy with SSL
- Complete monitoring stack (Prometheus, Grafana, Alertmanager)
- Log aggregation (Loki + Promtail)
- Jaeger with Elasticsearch backend
- Automated backup service

**Usage:**
```bash
# Start production stack
docker-compose -f docker-compose.prod.yml up -d

# View all services
docker-compose -f docker-compose.prod.yml ps

# Scale application instances
docker-compose -f docker-compose.prod.yml up -d --scale app=3

# Stop production stack
docker-compose -f docker-compose.prod.yml down
```

### 3. docker-compose.test.yml
Testing environment with:
- Test application
- Test database (PostgreSQL)
- Test cache (Redis)
- Test message broker (NATS)
- Mock server (WireMock)
- Test runner with coverage
- Security scanner (cargo-audit)
- Lint checker (clippy)
- Format checker (rustfmt)
- Load testing (k6)

**Usage:**
```bash
# Run all tests
docker-compose -f docker-compose.test.yml up --abort-on-container-exit

# Run specific test service
docker-compose -f docker-compose.test.yml run test-runner

# Run integration tests
docker-compose -f docker-compose.test.yml run test-integration

# Run benchmarks
docker-compose -f docker-compose.test.yml run benchmark

# Run security scan
docker-compose -f docker-compose.test.yml run security-scanner

# Run lint check
docker-compose -f docker-compose.test.yml run lint-checker
```

### 4. docker-compose.override.yml
Local development overrides (automatically merged with docker-compose.yml):
- Hot-reload with cargo-watch
- Enhanced logging
- Exposed ports for all services
- Development-friendly settings
- Additional development tools
- Documentation generator
- SQL and Redis CLI tools
- Development shell

**Additional Services:**
```bash
# Access development shell
docker-compose exec dev-shell bash

# View generated documentation
open http://localhost:8000

# Access SQL client
docker-compose exec sql-client psql

# Access Redis CLI
docker-compose exec redis-cli redis-cli -h redis
```

## Environment Configuration

1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Edit `.env` and customize:
   - Database passwords
   - JWT secret
   - Redis password
   - Email configuration (production)
   - S3 backup credentials (production)

## Service Access

### Development Environment

| Service | URL | Credentials |
|---------|-----|-------------|
| Application | http://localhost:8080 | - |
| Metrics | http://localhost:9090 | - |
| Grafana | http://localhost:3000 | admin / admin |
| Prometheus | http://localhost:9091 | - |
| Jaeger UI | http://localhost:16686 | - |
| MailHog | http://localhost:8025 | - |
| pgAdmin | http://localhost:5050 | admin@llm-cost-ops.local / admin |
| Redis Commander | http://localhost:8081 | admin / admin |
| PostgreSQL | localhost:5432 | postgres / postgres |
| Redis | localhost:6379 | - |
| NATS | localhost:4222 | - |
| Documentation | http://localhost:8000 | - |

### Production Environment

All production services are internal by default. External access is through Nginx on ports 80/443.

## Networks

### Development
- `llm-cost-ops-network`: Main application network
- `monitoring-network`: Monitoring and observability

### Production
- `frontend`: Public-facing services
- `backend`: Internal application services
- `database`: Database services (isolated)
- `monitoring`: Monitoring stack

## Volumes

All persistent data is stored in named volumes:
- `postgres-data`: PostgreSQL database
- `redis-data`: Redis cache
- `nats-data`: NATS message broker
- `prometheus-data`: Prometheus metrics
- `grafana-data`: Grafana dashboards
- `jaeger-data`: Jaeger traces
- `app-data`: Application data and logs

## Health Checks

All services include health checks:
- Automatic restart on failure
- Dependency ordering with health check conditions
- Health check endpoints exposed for monitoring

## Resource Limits

Resource limits are configured for all services:
- Development: Balanced for local development
- Production: Optimized for production workloads
- Testing: Optimized for CI/CD pipelines

## Monitoring and Observability

### Metrics (Prometheus)
- Application metrics on port 9090
- Service discovery for all components
- Pre-configured alerting rules
- 30-day retention (dev), 90-day retention (prod)

### Dashboards (Grafana)
- Auto-provisioned datasources
- Pre-configured dashboards
- Custom alerting
- User management

### Tracing (Jaeger)
- Full distributed tracing
- Service dependency visualization
- Performance analysis
- 100% sampling (dev), 10% sampling (prod)

### Logging (Loki + Promtail)
- Centralized log aggregation
- Log query and analysis
- Log-based alerting
- 31-day retention

## Backup and Recovery

### Automated Backups (Production)
- Daily PostgreSQL backups at 2 AM
- Grafana configuration backups
- Prometheus metrics backups
- Optional S3 upload
- 30-day retention

### Manual Backup
```bash
# Run backup manually
docker-compose -f docker-compose.prod.yml exec backup /usr/local/bin/backup.sh

# List backups
docker-compose -f docker-compose.prod.yml exec backup ls -lh /backup
```

### Restore
```bash
# Restore PostgreSQL
docker-compose -f docker-compose.prod.yml exec postgres-primary \
  psql -U postgres -d llm_cost_ops < backup.sql
```

## Troubleshooting

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f app

# Last 100 lines
docker-compose logs --tail=100 app
```

### Restart Service
```bash
docker-compose restart app
```

### Check Service Health
```bash
docker-compose ps
```

### Access Service Shell
```bash
docker-compose exec app sh
```

### Database Migration
```bash
docker-compose exec app cargo sqlx migrate run
```

### Clear All Data
```bash
docker-compose down -v
```

## Security Considerations

### Development
- Default passwords (change in production)
- Exposed ports for debugging
- Anonymous access enabled for some services

### Production
- Strong passwords required
- Internal networks for sensitive services
- TLS/SSL encryption
- Rate limiting enabled
- Security headers configured
- Regular security updates

## Performance Tuning

### PostgreSQL
- Shared buffers: 25% of RAM
- Effective cache size: 50-75% of RAM
- Work memory: Based on connections and complexity
- Max connections: Based on application needs

### Redis
- Max memory policy: allkeys-lru
- Persistence: Enabled in production
- Replication: Async replication to replica

### NATS
- JetStream for persistent messaging
- Clustering for high availability
- Message limits based on workload

## CI/CD Integration

### GitHub Actions Example
```yaml
- name: Run tests
  run: docker-compose -f docker-compose.test.yml up --abort-on-container-exit

- name: Build production image
  run: docker-compose -f docker-compose.prod.yml build
```

## Support

For issues or questions:
1. Check service logs: `docker-compose logs -f [service]`
2. Check health status: `docker-compose ps`
3. Review documentation: http://localhost:8000
4. Check Grafana dashboards: http://localhost:3000
5. Review traces in Jaeger: http://localhost:16686

## License

Apache 2.0
