# Docker Compose Guide for LLM Cost Ops

**Comprehensive Guide to Service Orchestration**

Version: 1.0.0
Last Updated: 2025-11-16

---

## Table of Contents

1. [Overview](#overview)
2. [Compose File Structure](#compose-file-structure)
3. [Service Descriptions](#service-descriptions)
4. [Network Configuration](#network-configuration)
5. [Volume Management](#volume-management)
6. [Environment Variables](#environment-variables)
7. [Service Dependencies](#service-dependencies)
8. [Health Checks](#health-checks)
9. [Resource Management](#resource-management)
10. [Scaling Services](#scaling-services)
11. [Common Operations](#common-operations)
12. [Development Workflows](#development-workflows)
13. [Production Configurations](#production-configurations)
14. [Troubleshooting](#troubleshooting)
15. [Advanced Topics](#advanced-topics)
16. [Best Practices](#best-practices)
17. [Reference](#reference)

---

## Overview

### What is Docker Compose?

Docker Compose is a tool for defining and running multi-container Docker applications. With Compose, you use a YAML file to configure your application's services, networks, and volumes.

### Benefits

- **Single command deployment** - Start entire stack with `docker compose up`
- **Service orchestration** - Manage dependencies between services
- **Environment flexibility** - Easy switching between dev/staging/prod
- **Reproducible environments** - Same setup across all machines
- **Version control** - Infrastructure as code
- **Easy scaling** - Scale services with a single command

### Compose File Versions

The platform uses Docker Compose V2 (file version 3.8):

```yaml
version: '3.8'  # Latest stable version
```

**Key features:**
- Native Docker integration
- Improved CLI experience
- Better error messages
- Faster performance
- Support for Docker Swarm (optional)

---

## Compose File Structure

### Main Files

The platform uses multiple compose files for different environments:

```
.
├── docker-compose.yml          # Base development configuration
├── docker-compose.override.yml # Local overrides (gitignored)
├── docker-compose.prod.yml     # Production configuration
├── docker-compose.test.yml     # Testing configuration
└── .env.example                # Environment variables template
```

### Base Configuration (docker-compose.yml)

```yaml
version: '3.8'

networks:
  llm-cost-ops-network:
    driver: bridge
  monitoring-network:
    driver: bridge

volumes:
  postgres-data:
  redis-data:
  prometheus-data:
  grafana-data:

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://postgres:postgres@postgres:5432/llm_cost_ops_dev
    depends_on:
      postgres:
        condition: service_healthy
    networks:
      - llm-cost-ops-network

  postgres:
    image: postgres:16-alpine
    # ... configuration
```

### File Composition

Docker Compose automatically merges files:

```bash
# Default: docker-compose.yml + docker-compose.override.yml
docker compose up

# Explicit files
docker compose -f docker-compose.yml -f docker-compose.prod.yml up

# Multiple files (right overrides left)
docker compose \
  -f docker-compose.yml \
  -f docker-compose.prod.yml \
  -f docker-compose.custom.yml \
  up
```

### Override Pattern

**docker-compose.override.yml** (local development):

```yaml
version: '3.8'

services:
  app:
    environment:
      RUST_LOG: trace  # More verbose logging
    volumes:
      - ./custom-config.toml:/app/config.toml:ro
    ports:
      - "8081:8080"  # Different port to avoid conflicts
```

This file should be in `.gitignore` for personal customizations.

---

## Service Descriptions

### Application Service (app)

**Purpose:** Main Rust application serving the API

```yaml
app:
  build:
    context: .
    dockerfile: Dockerfile.dev
    args:
      - RUST_VERSION=1.75
      - BUILD_DATE=${BUILD_DATE:-2024-01-01}

  container_name: llm-cost-ops-app
  hostname: llm-cost-ops-app
  restart: unless-stopped

  ports:
    - "8080:8080"      # HTTP API
    - "9090:9090"      # Prometheus metrics

  networks:
    - llm-cost-ops-network
    - monitoring-network

  volumes:
    - ./src:/app/src:ro
    - ./config.toml:/app/config.toml:ro
    - app-data:/app/data

  environment:
    RUST_LOG: ${RUST_LOG:-debug}
    DATABASE_URL: postgres://postgres:postgres@postgres:5432/llm_cost_ops_dev
    REDIS_URL: redis://redis:6379/0

  depends_on:
    postgres:
      condition: service_healthy
    redis:
      condition: service_healthy

  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
    interval: 30s
    timeout: 10s
    retries: 3
    start_period: 40s

  deploy:
    resources:
      limits:
        cpus: '2.0'
        memory: 2G
      reservations:
        cpus: '0.5'
        memory: 512M
```

**Key features:**
- Hot reload for development (via cargo-watch)
- Health checks for readiness
- Resource limits to prevent OOM
- Multiple network connections
- Volume mounts for code and data

### PostgreSQL Database (postgres)

**Purpose:** Primary relational database for persistent storage

```yaml
postgres:
  image: postgres:16-alpine
  container_name: llm-cost-ops-postgres
  restart: unless-stopped

  ports:
    - "5432:5432"

  networks:
    - llm-cost-ops-network

  volumes:
    - postgres-data:/var/lib/postgresql/data
    - ./docker/postgres/init.sql:/docker-entrypoint-initdb.d/init.sql:ro

  environment:
    POSTGRES_USER: ${POSTGRES_USER:-postgres}
    POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-postgres}
    POSTGRES_DB: ${POSTGRES_DB:-llm_cost_ops_dev}
    POSTGRES_INITDB_ARGS: "--encoding=UTF-8 --lc-collate=en_US.UTF-8"
    PGDATA: /var/lib/postgresql/data/pgdata

  healthcheck:
    test: ["CMD-SHELL", "pg_isready -U postgres -d llm_cost_ops_dev"]
    interval: 10s
    timeout: 5s
    retries: 5

  command:
    - postgres
    - -c shared_buffers=256MB
    - -c effective_cache_size=1GB
    - -c max_connections=100
```

**Key features:**
- Alpine base for smaller image
- Persistent data volume
- Initialization scripts
- Health checks for availability
- Performance tuning via command args

### Redis Cache (redis)

**Purpose:** In-memory data store for caching and session management

```yaml
redis:
  image: redis:7-alpine
  container_name: llm-cost-ops-redis
  restart: unless-stopped

  ports:
    - "6379:6379"

  networks:
    - llm-cost-ops-network

  volumes:
    - redis-data:/data
    - ./docker/redis/redis.conf:/usr/local/etc/redis/redis.conf:ro

  command: redis-server /usr/local/etc/redis/redis.conf

  healthcheck:
    test: ["CMD", "redis-cli", "ping"]
    interval: 10s
    timeout: 3s
    retries: 5
```

**Key features:**
- AOF persistence enabled
- Custom configuration
- Fast health checks
- Minimal resource usage

### NATS Message Broker (nats)

**Purpose:** Lightweight message broker for event streaming

```yaml
nats:
  image: nats:2.10-alpine
  container_name: llm-cost-ops-nats
  restart: unless-stopped

  ports:
    - "4222:4222"      # Client port
    - "8222:8222"      # HTTP monitoring
    - "6222:6222"      # Cluster port

  networks:
    - llm-cost-ops-network
    - monitoring-network

  volumes:
    - nats-data:/data
    - ./docker/nats/nats.conf:/etc/nats/nats.conf:ro

  command: ["-c", "/etc/nats/nats.conf", "-D"]

  healthcheck:
    test: ["CMD", "wget", "--spider", "http://localhost:8222/healthz"]
    interval: 10s
    timeout: 3s
    retries: 5
```

**Key features:**
- JetStream enabled for persistence
- HTTP monitoring endpoint
- Cluster-ready configuration
- Debug mode for development

### Prometheus Metrics (prometheus)

**Purpose:** Time-series database for metrics collection

```yaml
prometheus:
  image: prom/prometheus:v2.48.0
  container_name: llm-cost-ops-prometheus
  restart: unless-stopped

  ports:
    - "9091:9090"

  networks:
    - monitoring-network

  volumes:
    - prometheus-data:/prometheus
    - ./docker/prometheus/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    - ./docker/prometheus/alerts.yml:/etc/prometheus/alerts.yml:ro

  command:
    - '--config.file=/etc/prometheus/prometheus.yml'
    - '--storage.tsdb.path=/prometheus'
    - '--storage.tsdb.retention.time=30d'
    - '--web.enable-lifecycle'

  healthcheck:
    test: ["CMD", "wget", "--spider", "http://localhost:9090/-/healthy"]
    interval: 15s
```

**Key features:**
- 30-day data retention
- Lifecycle API for config reloads
- Alerting rules support
- Persistent storage

### Grafana Dashboards (grafana)

**Purpose:** Visualization and dashboarding platform

```yaml
grafana:
  image: grafana/grafana:10.2.2
  container_name: llm-cost-ops-grafana
  restart: unless-stopped

  ports:
    - "3000:3000"

  networks:
    - monitoring-network

  volumes:
    - grafana-data:/var/lib/grafana
    - ./docker/grafana/provisioning:/etc/grafana/provisioning:ro
    - ./docker/grafana/dashboards:/var/lib/grafana/dashboards:ro

  environment:
    GF_SECURITY_ADMIN_USER: ${GF_SECURITY_ADMIN_USER:-admin}
    GF_SECURITY_ADMIN_PASSWORD: ${GF_SECURITY_ADMIN_PASSWORD:-admin}
    GF_INSTALL_PLUGINS: grafana-piechart-panel

  depends_on:
    prometheus:
      condition: service_healthy
```

**Key features:**
- Provisioned datasources
- Pre-loaded dashboards
- Plugin support
- SQLite database (dev) or PostgreSQL (prod)

### Jaeger Tracing (jaeger)

**Purpose:** Distributed tracing for performance monitoring

```yaml
jaeger:
  image: jaegertracing/all-in-one:1.52
  container_name: llm-cost-ops-jaeger
  restart: unless-stopped

  ports:
    - "16686:16686"    # UI
    - "14268:14268"    # HTTP collector
    - "6831:6831/udp"  # Compact thrift

  networks:
    - llm-cost-ops-network
    - monitoring-network

  environment:
    SPAN_STORAGE_TYPE: badger
    BADGER_EPHEMERAL: false
    BADGER_DIRECTORY_VALUE: /tmp/jaeger/data

  volumes:
    - jaeger-data:/tmp
```

**Key features:**
- All-in-one deployment
- Badger storage backend
- Multiple protocol support
- Web UI included

### Development Support Services

#### MailHog (Email Testing)

```yaml
mailhog:
  image: mailhog/mailhog:v1.0.1
  container_name: llm-cost-ops-mailhog

  ports:
    - "1025:1025"      # SMTP
    - "8025:8025"      # Web UI

  networks:
    - llm-cost-ops-network
```

#### pgAdmin (Database Management)

```yaml
pgadmin:
  image: dpage/pgadmin4:8.1
  container_name: llm-cost-ops-pgadmin

  ports:
    - "5050:80"

  networks:
    - llm-cost-ops-network

  environment:
    PGADMIN_DEFAULT_EMAIL: admin@llm-cost-ops.local
    PGADMIN_DEFAULT_PASSWORD: admin

  depends_on:
    postgres:
      condition: service_healthy
```

#### Redis Commander (Cache Management)

```yaml
redis-commander:
  image: rediscommander/redis-commander:latest
  container_name: llm-cost-ops-redis-commander

  ports:
    - "8081:8081"

  networks:
    - llm-cost-ops-network

  environment:
    REDIS_HOSTS: local:redis:6379
    HTTP_USER: admin
    HTTP_PASSWORD: admin
```

---

## Network Configuration

### Network Types

**Bridge Network (Default)**

```yaml
networks:
  llm-cost-ops-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16
          gateway: 172.28.0.1
```

**Internal Network (Production)**

```yaml
networks:
  backend:
    driver: bridge
    internal: true  # No external access
```

**Host Network (Advanced)**

```yaml
services:
  app:
    network_mode: host  # Use host's network stack
```

### Network Isolation

**Multi-network setup:**

```yaml
networks:
  frontend:    # Public-facing
    driver: bridge
  backend:     # Internal only
    driver: bridge
    internal: true
  monitoring:  # Observability
    driver: bridge

services:
  nginx:
    networks:
      - frontend

  app:
    networks:
      - frontend
      - backend
      - monitoring

  postgres:
    networks:
      - backend  # Not accessible from frontend
```

### Service Discovery

Services automatically discover each other via DNS:

```bash
# From app container:
ping postgres          # Resolves to postgres service
curl http://redis:6379
nc -zv nats 4222
```

### Network Aliases

```yaml
services:
  app:
    networks:
      llm-cost-ops-network:
        aliases:
          - api
          - backend
          - llm-ops
```

Now accessible via multiple names:
- `app`
- `api`
- `backend`
- `llm-ops`

### Network Troubleshooting

```bash
# List networks
docker network ls

# Inspect network
docker network inspect llm-cost-ops_llm-cost-ops-network

# View connected containers
docker network inspect llm-cost-ops_llm-cost-ops-network \
  --format '{{range .Containers}}{{.Name}} {{.IPv4Address}}{{"\n"}}{{end}}'

# Test connectivity
docker compose exec app ping postgres
docker compose exec app telnet redis 6379
docker compose exec app nc -zv nats 4222
```

---

## Volume Management

### Volume Types

**Named Volumes (Recommended)**

```yaml
volumes:
  postgres-data:
    driver: local
    labels:
      com.llm-cost-ops.backup: "required"
```

**Bind Mounts (Development)**

```yaml
volumes:
  - ./src:/app/src:ro           # Read-only
  - ./config.toml:/app/config.toml:ro
  - ./logs:/app/logs:rw         # Read-write
```

**Tmpfs Mounts (Temporary)**

```yaml
tmpfs:
  - /tmp:size=1G,mode=1777
  - /run:size=100M
```

### Volume Configuration

**With driver options:**

```yaml
volumes:
  postgres-data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/data/postgres
```

**External volumes:**

```yaml
volumes:
  postgres-data:
    external: true
    name: llm-cost-ops-postgres-prod
```

### Volume Operations

**Create volume:**

```bash
docker volume create postgres-data
```

**Inspect volume:**

```bash
docker volume inspect llm-cost-ops_postgres-data
```

**Backup volume:**

```bash
docker run --rm \
  -v llm-cost-ops_postgres-data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/postgres-backup.tar.gz /data
```

**Restore volume:**

```bash
docker run --rm \
  -v llm-cost-ops_postgres-data:/data \
  -v $(pwd):/backup \
  alpine tar xzf /backup/postgres-backup.tar.gz -C /
```

**Remove volume:**

```bash
# Stop services first
docker compose down

# Remove specific volume
docker volume rm llm-cost-ops_postgres-data

# Remove all unused volumes
docker volume prune
```

### Volume Best Practices

1. **Use named volumes for production** - Better management
2. **Bind mounts for development** - Live code updates
3. **Label important volumes** - Easy identification
4. **Regular backups** - Disaster recovery
5. **Monitor disk usage** - Prevent out-of-space errors
6. **Use read-only when possible** - Security

---

## Environment Variables

### Configuration Hierarchy

Environment variables can be set in multiple places (highest priority first):

1. Shell environment: `export DATABASE_URL=...`
2. `.env` file in project root
3. `env_file:` in docker-compose.yml
4. `environment:` in docker-compose.yml
5. Default values in application

### .env File

**.env (Development)**

```bash
# Application
RUST_LOG=debug
LOG_LEVEL=debug
PORT=8080
METRICS_PORT=9090

# Database
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=llm_cost_ops_dev

# Redis
REDIS_PASSWORD=

# Security
JWT_SECRET=dev-secret-change-in-production

# Features
ENABLE_COMPRESSION=true
ENABLE_RATE_LIMITING=true
```

**.env.production (Production)**

```bash
# Application
RUST_LOG=info
LOG_LEVEL=info
PORT=8080
METRICS_PORT=9090

# Database
POSTGRES_USER=postgres
POSTGRES_PASSWORD=${SECURE_POSTGRES_PASSWORD}
POSTGRES_DB=llm_cost_ops_prod

# Redis
REDIS_PASSWORD=${SECURE_REDIS_PASSWORD}

# Security
JWT_SECRET=${SECURE_JWT_SECRET}
CORS_ALLOWED_ORIGINS=https://your-domain.com

# Email
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=${SENDGRID_API_KEY}

# Monitoring
GF_SECURITY_ADMIN_PASSWORD=${SECURE_GRAFANA_PASSWORD}
```

### Using Environment Files

**Single file:**

```bash
docker compose --env-file .env.production up
```

**Multiple files:**

```bash
docker compose \
  --env-file .env \
  --env-file .env.local \
  up
```

**Variable substitution:**

```yaml
services:
  app:
    image: llm-cost-ops:${VERSION:-latest}
    environment:
      DATABASE_URL: ${DATABASE_URL}
      JWT_SECRET: ${JWT_SECRET:-default-secret}
```

### Environment Variable Types

**Simple value:**

```yaml
environment:
  LOG_LEVEL: info
```

**Array format:**

```yaml
environment:
  - LOG_LEVEL=info
  - DATABASE_URL=postgres://...
```

**From .env file:**

```yaml
env_file:
  - .env
  - .env.production
```

**With defaults:**

```yaml
environment:
  LOG_LEVEL: ${LOG_LEVEL:-info}
  PORT: ${PORT:-8080}
```

### Secret Management

**Using Docker secrets:**

```yaml
services:
  app:
    secrets:
      - postgres_password
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/postgres_password

secrets:
  postgres_password:
    file: ./secrets/postgres_password.txt
```

**Using external secrets:**

```yaml
secrets:
  postgres_password:
    external: true
```

Create secret:

```bash
echo "supersecret" | docker secret create postgres_password -
```

---

## Service Dependencies

### Dependency Types

**Simple dependency:**

```yaml
services:
  app:
    depends_on:
      - postgres
      - redis
```

Services start in order: postgres → redis → app

**Conditional dependency:**

```yaml
services:
  app:
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
```

**Available conditions:**
- `service_started` - Service started (default)
- `service_healthy` - Health check passing
- `service_completed_successfully` - One-time command succeeded

### Startup Order

**Example dependency graph:**

```yaml
services:
  # 1. Start first (no dependencies)
  postgres:
    image: postgres:16-alpine

  # 2. Start after postgres is healthy
  app:
    depends_on:
      postgres:
        condition: service_healthy

  # 3. Start after app is healthy
  nginx:
    depends_on:
      app:
        condition: service_healthy
```

**Startup sequence:**
1. postgres starts
2. postgres health check passes
3. app starts
4. app health check passes
5. nginx starts

### Circular Dependencies

**Problem:**

```yaml
# DON'T DO THIS
services:
  app:
    depends_on:
      - worker

  worker:
    depends_on:
      - app  # Circular dependency!
```

**Solution:**

```yaml
services:
  app:
    depends_on:
      postgres:
        condition: service_healthy

  worker:
    depends_on:
      postgres:
        condition: service_healthy

  # Both depend on postgres, not each other
```

### Restart Policies

```yaml
services:
  app:
    restart: unless-stopped  # Recommended for development

  postgres:
    restart: always  # Recommended for production

  worker:
    restart: on-failure:3  # Retry 3 times

  migration:
    restart: "no"  # Run once
```

**Options:**
- `no` - Never restart (default)
- `always` - Always restart
- `on-failure` - Restart on non-zero exit
- `on-failure:3` - Retry max 3 times
- `unless-stopped` - Always restart unless explicitly stopped

---

## Health Checks

### Health Check Configuration

**HTTP health check:**

```yaml
services:
  app:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s      # Check every 30 seconds
      timeout: 10s       # Timeout after 10 seconds
      retries: 3         # 3 failed checks = unhealthy
      start_period: 40s  # Grace period before checks start
```

**Shell command health check:**

```yaml
healthcheck:
  test: ["CMD-SHELL", "curl -f http://localhost:8080/health || exit 1"]
```

**PostgreSQL health check:**

```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U postgres -d llm_cost_ops_dev"]
  interval: 10s
  timeout: 5s
  retries: 5
```

**Redis health check:**

```yaml
healthcheck:
  test: ["CMD", "redis-cli", "ping"]
  interval: 10s
  timeout: 3s
  retries: 5
```

### Health Check Best Practices

1. **Use meaningful endpoints** - `/health` should check dependencies
2. **Set appropriate intervals** - Balance responsiveness vs overhead
3. **Include start period** - Allow time for initialization
4. **Test health checks** - Verify they work correctly
5. **Monitor health status** - Alert on unhealthy services

### Viewing Health Status

```bash
# View health status
docker compose ps

# Expected output:
NAME                   STATUS
llm-cost-ops-app       Up (healthy)
llm-cost-ops-postgres  Up (healthy)
llm-cost-ops-redis     Up (healthy)

# Inspect health details
docker inspect llm-cost-ops-app | jq '.[0].State.Health'

# View health logs
docker inspect llm-cost-ops-app | jq '.[0].State.Health.Log'
```

### Custom Health Check Scripts

**Complex health check:**

```yaml
services:
  app:
    healthcheck:
      test: ["CMD", "/app/healthcheck.sh"]
```

**healthcheck.sh:**

```bash
#!/bin/bash
set -e

# Check HTTP endpoint
curl -f http://localhost:8080/health || exit 1

# Check database connection
pg_isready -h postgres -U postgres || exit 1

# Check Redis connection
redis-cli -h redis ping || exit 1

# All checks passed
exit 0
```

---

## Resource Management

### CPU Limits

**Development (flexible):**

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '2.0'
        reservations:
          cpus: '0.5'
```

**Production (strict):**

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '4.0'
        reservations:
          cpus: '2.0'
```

### Memory Limits

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          memory: 2G
        reservations:
          memory: 512M
```

**With swap:**

```yaml
services:
  app:
    mem_limit: 2G
    mem_reservation: 512M
    memswap_limit: 3G  # Total memory + swap
```

### PID Limits

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          pids: 100  # Max processes
```

### Device Limits

```yaml
services:
  app:
    devices:
      - "/dev/sda:/dev/xvda:rwm"

    device_read_bps:
      - path: /dev/sda
        rate: '100mb'

    device_write_bps:
      - path: /dev/sda
        rate: '50mb'
```

### Monitoring Resources

```bash
# Real-time stats
docker stats

# Specific service
docker stats llm-cost-ops-app

# JSON output
docker stats --no-stream --format "{{json .}}" | jq
```

### Resource Best Practices

1. **Set limits in production** - Prevent resource exhaustion
2. **Monitor resource usage** - Adjust limits based on actual usage
3. **Use reservations** - Guarantee minimum resources
4. **Test under load** - Verify limits are appropriate
5. **Leave headroom** - Don't allocate 100% of system resources

---

## Scaling Services

### Horizontal Scaling

**Scale up:**

```bash
# Scale to 5 instances
docker compose up -d --scale app=5

# Verify
docker compose ps app
```

**Scale in compose file:**

```yaml
services:
  app:
    deploy:
      replicas: 3
```

**Scale down:**

```bash
docker compose up -d --scale app=1
```

### Load Balancing

**Nginx upstream configuration:**

```nginx
upstream app_backend {
    least_conn;  # Load balancing algorithm

    server app:8080 max_fails=3 fail_timeout=30s;
    server app:8080 max_fails=3 fail_timeout=30s;
    server app:8080 max_fails=3 fail_timeout=30s;

    keepalive 32;
}

server {
    listen 80;

    location / {
        proxy_pass http://app_backend;
        proxy_next_upstream error timeout http_502 http_503 http_504;
    }
}
```

### Auto-Scaling

**Based on CPU:**

```yaml
services:
  app:
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
```

**Using external tools:**
- Docker Swarm autoscaling
- Kubernetes HPA (see KUBERNETES.md)
- Custom scripts with metrics

### Scaling Limitations

**Cannot scale:**
- Services with `container_name`
- Services with specific port mappings
- Services with unique volumes

**Solution - Remove container_name:**

```yaml
services:
  app:
    # container_name: llm-cost-ops-app  # Remove this
    image: llm-cost-ops:latest
    # Now can scale
```

**Solution - Use port ranges:**

```yaml
services:
  app:
    ports:
      - "8080-8084:8080"  # Maps to 5 instances
```

---

## Common Operations

### Starting Services

```bash
# Start all services
docker compose up

# Start in background
docker compose up -d

# Start specific services
docker compose up app postgres redis

# Start with build
docker compose up --build

# Start with fresh volumes
docker compose up --renew-anon-volumes

# Force recreate
docker compose up --force-recreate
```

### Stopping Services

```bash
# Stop all services
docker compose down

# Stop but keep volumes
docker compose down

# Stop and remove volumes
docker compose down -v

# Stop and remove images
docker compose down --rmi all

# Stop specific service
docker compose stop app
```

### Restarting Services

```bash
# Restart all
docker compose restart

# Restart specific service
docker compose restart app

# Restart with timeout
docker compose restart -t 30 app
```

### Viewing Logs

```bash
# All logs
docker compose logs

# Follow logs
docker compose logs -f

# Specific service
docker compose logs -f app

# Last 100 lines
docker compose logs --tail=100 app

# Since timestamp
docker compose logs --since 2024-01-01T00:00:00

# With timestamps
docker compose logs -f --timestamps app

# Multiple services
docker compose logs -f app postgres redis
```

### Executing Commands

```bash
# Interactive shell
docker compose exec app bash

# Run command
docker compose exec app cargo test

# Run as different user
docker compose exec -u root app apt-get update

# Run without TTY
docker compose exec -T app echo "hello"

# Run in new container
docker compose run --rm app cargo test
```

### Building Images

```bash
# Build all
docker compose build

# Build specific service
docker compose build app

# Build with no cache
docker compose build --no-cache

# Build with build args
docker compose build --build-arg RUST_VERSION=1.76

# Pull latest base images
docker compose build --pull
```

### Pulling Images

```bash
# Pull all images
docker compose pull

# Pull specific service
docker compose pull postgres

# Pull quietly
docker compose pull -q
```

### Viewing Status

```bash
# List services
docker compose ps

# List all (including stopped)
docker compose ps -a

# Service status
docker compose ps app

# Top processes
docker compose top

# Service ports
docker compose port app 8080
```

---

## Development Workflows

### Workflow 1: Full Stack Development

```bash
# 1. Start all services
docker compose up -d

# 2. View logs
docker compose logs -f app

# 3. Make code changes (auto-reloads via cargo-watch)

# 4. Run tests
docker compose exec app cargo test

# 5. Check database
docker compose exec postgres psql -U postgres -d llm_cost_ops_dev

# 6. View metrics
open http://localhost:9090/metrics

# 7. Stop when done
docker compose down
```

### Workflow 2: Minimal Development

```bash
# Start only essential services
docker compose up -d postgres redis nats app

# Skip monitoring to save resources
```

### Workflow 3: Database Migrations

```bash
# 1. Create migration
docker compose exec app cargo sqlx migrate add add_users_table

# 2. Edit migration files

# 3. Run migrations
docker compose exec app cargo sqlx migrate run

# 4. Verify
docker compose exec postgres psql -U postgres -d llm_cost_ops_dev -c "\dt"

# 5. Revert if needed
docker compose exec app cargo sqlx migrate revert
```

### Workflow 4: Testing

```bash
# Run all tests
docker compose -f docker-compose.test.yml up --abort-on-container-exit

# Run specific test suite
docker compose -f docker-compose.test.yml run test-unit

# Run with coverage
docker compose -f docker-compose.test.yml run test-coverage

# Clean up
docker compose -f docker-compose.test.yml down -v
```

### Workflow 5: Debugging

```bash
# 1. Enable debug logging
export RUST_LOG=trace
export RUST_BACKTRACE=full

# 2. Restart with debug
docker compose up -d app

# 3. Attach to logs
docker compose logs -f app

# 4. Check metrics
curl http://localhost:9090/metrics

# 5. View traces
open http://localhost:16686
```

---

## Production Configurations

### Production Compose File

**docker-compose.prod.yml:**

```yaml
version: '3.8'

services:
  app:
    image: llm-cost-ops:${VERSION:-latest}
    restart: always

    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '4.0'
          memory: 4G
        reservations:
          cpus: '1.0'
          memory: 1G
      update_config:
        parallelism: 1
        delay: 10s
        failure_action: rollback
      rollback_config:
        parallelism: 1
        delay: 5s

    environment:
      RUST_LOG: info
      DATABASE_URL: postgres://postgres:${POSTGRES_PASSWORD}@postgres:5432/llm_cost_ops_prod
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379/0

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 3

    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  postgres:
    image: postgres:16-alpine
    restart: always

    environment:
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: llm_cost_ops_prod

    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./backups:/backups

    command:
      - postgres
      - -c max_connections=200
      - -c shared_buffers=2GB
      - -c effective_cache_size=6GB

    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 8G

  nginx:
    image: nginx:alpine
    restart: always

    ports:
      - "80:80"
      - "443:443"

    volumes:
      - ./docker/nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./docker/nginx/ssl:/etc/nginx/ssl:ro

    depends_on:
      - app

volumes:
  postgres-data:
```

### Deployment Commands

```bash
# Deploy production stack
docker compose -f docker-compose.prod.yml --env-file .env.production up -d

# Update to new version
VERSION=v2.0.0 docker compose -f docker-compose.prod.yml up -d

# Rolling update
docker compose -f docker-compose.prod.yml up -d --no-deps --scale app=6 app
sleep 30
docker compose -f docker-compose.prod.yml up -d --no-deps --scale app=3 app

# Rollback
docker compose -f docker-compose.prod.yml rollback
```

### Production Checklist

- [ ] Set strong passwords
- [ ] Configure SSL/TLS
- [ ] Set up backups
- [ ] Configure monitoring
- [ ] Set resource limits
- [ ] Enable health checks
- [ ] Configure logging
- [ ] Set up alerts
- [ ] Document procedures
- [ ] Test disaster recovery

---

## Troubleshooting

### Service Won't Start

```bash
# Check logs
docker compose logs app

# Check service status
docker compose ps

# Inspect container
docker inspect llm-cost-ops-app

# Try recreating
docker compose up -d --force-recreate app
```

### Network Issues

```bash
# List networks
docker network ls

# Inspect network
docker network inspect llm-cost-ops_llm-cost-ops-network

# Test connectivity
docker compose exec app ping postgres

# Recreate networks
docker compose down
docker compose up -d
```

### Volume Issues

```bash
# List volumes
docker volume ls

# Inspect volume
docker volume inspect llm-cost-ops_postgres-data

# Check permissions
docker volume inspect llm-cost-ops_postgres-data | jq '.[0].Mountpoint'
sudo ls -la /var/lib/docker/volumes/llm-cost-ops_postgres-data/_data

# Recreate volume (CAUTION: deletes data)
docker compose down -v
docker compose up -d
```

### Resource Issues

```bash
# Check resource usage
docker stats

# Check disk space
docker system df

# Clean up
docker system prune -a --volumes
```

---

## Advanced Topics

### Extending Services

```yaml
# base.yml
services:
  app:
    image: llm-cost-ops:latest

# docker-compose.yml
version: '3.8'
services:
  app:
    extends:
      file: base.yml
      service: app
    environment:
      LOG_LEVEL: debug
```

### Using Profiles

```yaml
services:
  app:
    profiles: ["production"]

  mailhog:
    profiles: ["development"]
```

```bash
# Activate profile
docker compose --profile production up
docker compose --profile development up
```

### Custom Build Context

```yaml
services:
  app:
    build:
      context: ./app
      dockerfile: ../Dockerfile.custom
      args:
        - VERSION=2.0.0
      cache_from:
        - llm-cost-ops:latest
      target: production
```

---

## Best Practices

1. **Use version control** - Track docker-compose.yml
2. **Environment files** - Don't commit .env files
3. **Health checks** - Always define health checks
4. **Resource limits** - Set in production
5. **Named volumes** - Use for persistent data
6. **Network isolation** - Separate networks
7. **Service dependencies** - Use conditions
8. **Logging** - Configure log rotation
9. **Documentation** - Comment complex configurations
10. **Testing** - Test compose files before production

---

## Reference

### Common Commands

```bash
# Life cycle
docker compose up
docker compose down
docker compose restart

# Logs
docker compose logs -f

# Execute
docker compose exec app bash

# Build
docker compose build

# Scale
docker compose up -d --scale app=3

# Status
docker compose ps
```

### File Locations

```
/workspaces/llm-cost-ops/
├── docker-compose.yml
├── docker-compose.prod.yml
├── docker-compose.test.yml
├── .env.example
└── docker/
    ├── nginx/
    ├── postgres/
    ├── redis/
    └── prometheus/
```

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0
