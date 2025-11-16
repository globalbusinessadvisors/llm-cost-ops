# Docker Guide for LLM Cost Ops Platform

**Comprehensive Guide to Containerization, Deployment, and Operations**

Version: 1.0.0
Last Updated: 2025-11-16

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Prerequisites](#prerequisites)
4. [Quick Start](#quick-start)
5. [Development Workflow](#development-workflow)
6. [Production Deployment](#production-deployment)
7. [Service Configuration](#service-configuration)
8. [Networking](#networking)
9. [Storage and Volumes](#storage-and-volumes)
10. [Security](#security)
11. [Monitoring and Observability](#monitoring-and-observability)
12. [Scaling](#scaling)
13. [Backup and Recovery](#backup-and-recovery)
14. [Troubleshooting](#troubleshooting)
15. [Best Practices](#best-practices)
16. [Performance Tuning](#performance-tuning)
17. [Migration Guides](#migration-guides)
18. [FAQ](#faq)
19. [Reference](#reference)
20. [Support](#support)

---

## Overview

### What is LLM Cost Ops?

LLM Cost Ops is an enterprise-grade platform for monitoring, tracking, and optimizing costs associated with Large Language Model deployments. The platform provides:

- **Real-time cost tracking** across multiple LLM providers
- **Usage analytics** with detailed breakdowns
- **Budget management** with alerts and thresholds
- **Multi-tenancy support** for enterprise deployments
- **Export and reporting** capabilities
- **SSO integration** with major identity providers
- **Comprehensive audit trails** for compliance

### Docker Benefits

This Docker implementation provides:

- **Consistent environments** across development, staging, and production
- **Easy setup** with single-command deployment
- **Isolated services** for improved security and maintainability
- **Horizontal scaling** capabilities for high-availability
- **Infrastructure as Code** for reproducible deployments
- **Integrated monitoring** with Prometheus, Grafana, and Jaeger
- **Automated backups** and disaster recovery
- **Resource optimization** with configurable limits

### Container Strategy

The platform uses a microservices-oriented architecture with the following components:

```
┌─────────────────────────────────────────────────────────┐
│                    Load Balancer (Nginx)                │
│                    ┌─────────────────┐                  │
│                    │   SSL/TLS       │                  │
│                    │   Rate Limiting │                  │
│                    │   Compression   │                  │
│                    └────────┬────────┘                  │
└─────────────────────────────┼───────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   App Instance  │  │   App Instance  │  │   App Instance  │
│      (Rust)     │  │      (Rust)     │  │      (Rust)     │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   PostgreSQL    │  │     Redis       │  │      NATS       │
│   (Database)    │  │    (Cache)      │  │  (Messaging)    │
└─────────────────┘  └─────────────────┘  └─────────────────┘
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   Prometheus    │  │    Grafana      │  │     Jaeger      │
│   (Metrics)     │  │  (Dashboards)   │  │    (Tracing)    │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

---

## Architecture

### Multi-Stage Build Strategy

The platform uses multi-stage Docker builds to optimize image size and security:

#### Production Image (Dockerfile)

```dockerfile
# Stage 1: Build - Full Rust toolchain
FROM rust:1.75-slim-bullseye AS builder
# Install build dependencies
# Build optimized release binary
# ~1.2GB

# Stage 2: Runtime - Minimal Debian
FROM debian:bullseye-slim AS runtime
# Copy only the binary and runtime dependencies
# Final image size: ~150MB
```

**Benefits:**
- Small final image (150MB vs 1.2GB)
- Fast deployment and startup
- Reduced attack surface
- Lower storage costs

#### Development Image (Dockerfile.dev)

```dockerfile
# Single stage with full toolchain
FROM rust:1.75-bullseye
# Install development tools
# cargo-watch for hot reload
# sqlx-cli for migrations
# Full debugging capabilities
# ~1.8GB
```

**Benefits:**
- Hot reload for rapid development
- Full debugging tools included
- Database migration tools
- Interactive development

### Service Architecture

#### Core Services

**1. Application Service (app)**
- **Image:** Custom Rust application
- **Ports:** 8080 (HTTP), 9090 (Metrics)
- **Resources:** 2 CPU cores, 2GB RAM (dev), scalable in production
- **Purpose:** Main API server handling all business logic

**2. PostgreSQL Database (postgres)**
- **Image:** postgres:16-alpine
- **Port:** 5432
- **Resources:** 1 CPU core, 1GB RAM
- **Purpose:** Primary data store with ACID guarantees

**3. Redis Cache (redis)**
- **Image:** redis:7-alpine
- **Port:** 6379
- **Resources:** 0.5 CPU, 512MB RAM
- **Purpose:** Session storage, caching, rate limiting

**4. NATS Message Broker (nats)**
- **Image:** nats:2.10-alpine
- **Ports:** 4222 (client), 8222 (management), 6222 (cluster)
- **Resources:** 0.5 CPU, 512MB RAM
- **Purpose:** Event streaming, async job processing

#### Observability Services

**5. Prometheus (prometheus)**
- **Image:** prom/prometheus:v2.48.0
- **Port:** 9091
- **Resources:** 1 CPU, 1GB RAM
- **Purpose:** Metrics collection and time-series database

**6. Grafana (grafana)**
- **Image:** grafana/grafana:10.2.2
- **Port:** 3000
- **Resources:** 0.5 CPU, 512MB RAM
- **Purpose:** Metrics visualization and dashboards

**7. Jaeger (jaeger)**
- **Image:** jaegertracing/all-in-one:1.52
- **Ports:** 16686 (UI), 14268 (collector), 6831 (agent)
- **Resources:** 1 CPU, 1GB RAM
- **Purpose:** Distributed tracing and performance analysis

#### Development Support Services

**8. MailHog (mailhog)**
- **Image:** mailhog/mailhog:v1.0.1
- **Ports:** 1025 (SMTP), 8025 (Web UI)
- **Purpose:** Email testing in development

**9. pgAdmin (pgadmin)**
- **Image:** dpage/pgadmin4:8.1
- **Port:** 5050
- **Purpose:** Database administration UI

**10. Redis Commander (redis-commander)**
- **Image:** rediscommander/redis-commander
- **Port:** 8081
- **Purpose:** Redis cache inspection and management

### Network Architecture

```
┌──────────────────────────────────────────────────────┐
│           llm-cost-ops-network (172.28.0.0/16)       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐           │
│  │   App    │  │ Database │  │  Cache   │           │
│  │          │──│          │──│          │           │
│  │  :8080   │  │  :5432   │  │  :6379   │           │
│  └──────────┘  └──────────┘  └──────────┘           │
│                                                       │
│  ┌──────────┐  ┌──────────┐                          │
│  │  NATS    │  │ MailHog  │                          │
│  │  :4222   │  │  :1025   │                          │
│  └──────────┘  └──────────┘                          │
└──────────────────┬───────────────────────────────────┘
                   │
┌──────────────────┴───────────────────────────────────┐
│           monitoring-network (bridge)                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐           │
│  │Prometheus│  │ Grafana  │  │  Jaeger  │           │
│  │  :9091   │  │  :3000   │  │  :16686  │           │
│  └──────────┘  └──────────┘  └──────────┘           │
└──────────────────────────────────────────────────────┘
```

**Network Isolation:**
- Application network: Internal service communication
- Monitoring network: Observability stack isolation
- Shared access: App connects to both networks for metrics export

### Volume Strategy

**Persistent Volumes:**

```yaml
volumes:
  postgres-data:      # Database files - MUST backup
  redis-data:         # Cache data - can rebuild
  nats-data:          # Message queue state
  prometheus-data:    # Metrics history - SHOULD backup
  grafana-data:       # Dashboards and configs - SHOULD backup
  jaeger-data:        # Trace data - optional backup
  app-data:           # Application data and logs
```

**Volume Characteristics:**

| Volume | Size | Growth | Backup Priority | Retention |
|--------|------|--------|----------------|-----------|
| postgres-data | 10-100GB | Linear with data | Critical | 30+ days |
| redis-data | 1-5GB | Stable | Low | 7 days |
| nats-data | 1-10GB | Variable | Medium | 14 days |
| prometheus-data | 10-50GB | Linear with metrics | Medium | 30 days |
| grafana-data | 100MB-1GB | Slow | Medium | 30 days |
| jaeger-data | 5-20GB | Linear with traces | Low | 7 days |
| app-data | 1-10GB | Linear with logs | Medium | 14 days |

---

## Prerequisites

### System Requirements

#### Minimum (Development)
- **CPU:** 4 cores (2.5 GHz or better)
- **RAM:** 8 GB
- **Storage:** 20 GB free space (SSD recommended)
- **OS:** Linux, macOS, or Windows with WSL2

#### Recommended (Development)
- **CPU:** 8 cores (3.0 GHz or better)
- **RAM:** 16 GB
- **Storage:** 50 GB free space (SSD required)
- **OS:** Linux or macOS

#### Production
- **CPU:** 16+ cores
- **RAM:** 32+ GB
- **Storage:** 200+ GB (SSD/NVMe required)
- **Network:** 1 Gbps+ connection
- **OS:** Ubuntu 22.04 LTS, RHEL 8+, or compatible

### Software Requirements

#### Required

**Docker Engine**
- Version: 20.10.0 or newer
- Installation: [docs.docker.com/engine/install](https://docs.docker.com/engine/install/)
- Verify: `docker --version`

**Docker Compose**
- Version: v2.0.0 or newer (Compose V2)
- Usually bundled with Docker Desktop
- Verify: `docker compose version`

#### Optional but Recommended

**Docker Desktop** (for macOS/Windows)
- Includes Docker Engine and Compose
- Provides convenient GUI
- Download: [docker.com/products/docker-desktop](https://www.docker.com/products/docker-desktop)

**Make** (for convenience scripts)
- Install on Ubuntu: `sudo apt-get install build-essential`
- Install on macOS: `xcode-select --install`

**Git** (for version control)
- Version: 2.30+ recommended
- Install: `sudo apt-get install git` (Ubuntu)

**jq** (for JSON processing in scripts)
- Install: `sudo apt-get install jq`

### Verification Steps

Run these commands to verify your environment:

```bash
# Check Docker version
docker --version
# Expected: Docker version 20.10.0 or newer

# Check Docker Compose version
docker compose version
# Expected: Docker Compose version v2.0.0 or newer

# Verify Docker is running
docker info
# Should show server information without errors

# Check available system resources
docker system df
# Shows disk usage

# Test Docker functionality
docker run --rm hello-world
# Should download and run successfully

# Check for running containers
docker ps
# Should list any running containers

# Verify network connectivity
docker network ls
# Should list default networks
```

### Platform-Specific Setup

#### Linux (Ubuntu/Debian)

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Add user to docker group (avoid sudo)
sudo usermod -aG docker $USER
newgrp docker

# Enable Docker service
sudo systemctl enable docker
sudo systemctl start docker

# Install Docker Compose
sudo apt-get update
sudo apt-get install docker-compose-plugin

# Verify installation
docker --version
docker compose version
```

#### macOS

```bash
# Install Docker Desktop from:
# https://docs.docker.com/desktop/mac/install/

# Or via Homebrew
brew install --cask docker

# Start Docker Desktop
open /Applications/Docker.app

# Verify in terminal
docker --version
docker compose version
```

#### Windows (with WSL2)

```powershell
# Install WSL2
wsl --install

# Install Docker Desktop from:
# https://docs.docker.com/desktop/windows/install/

# Enable WSL2 integration in Docker Desktop settings

# In WSL2 terminal, verify:
docker --version
docker compose version
```

---

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/llm-devops/llm-cost-ops.git
cd llm-cost-ops
```

### 2. Configure Environment

```bash
# Copy example environment file
cp .env.example .env

# Edit configuration (optional for development)
nano .env
```

**Key environment variables:**

```bash
# Database
POSTGRES_PASSWORD=postgres          # Change in production
POSTGRES_DB=llm_cost_ops_dev

# Application
JWT_SECRET=dev-secret-change-in-production
LOG_LEVEL=debug
PORT=8080

# Redis
REDIS_PASSWORD=                     # Empty for dev

# Grafana
GF_SECURITY_ADMIN_PASSWORD=admin    # Change in production
```

### 3. Start All Services

```bash
# Start entire development stack
docker compose up -d

# View logs
docker compose logs -f

# Check status
docker compose ps
```

**Expected output:**

```
NAME                          STATUS              PORTS
llm-cost-ops-app              Up (healthy)        0.0.0.0:8080->8080/tcp
llm-cost-ops-postgres         Up (healthy)        0.0.0.0:5432->5432/tcp
llm-cost-ops-redis            Up (healthy)        0.0.0.0:6379->6379/tcp
llm-cost-ops-nats             Up                  0.0.0.0:4222->4222/tcp
llm-cost-ops-prometheus       Up (healthy)        0.0.0.0:9091->9090/tcp
llm-cost-ops-grafana          Up (healthy)        0.0.0.0:3000->3000/tcp
llm-cost-ops-jaeger           Up (healthy)        0.0.0.0:16686->16686/tcp
llm-cost-ops-mailhog          Up                  0.0.0.0:8025->8025/tcp
llm-cost-ops-pgadmin          Up                  0.0.0.0:5050->80/tcp
llm-cost-ops-redis-commander  Up                  0.0.0.0:8081->8081/tcp
```

### 4. Run Database Migrations

```bash
# Apply database schema
docker compose exec app cargo sqlx migrate run

# Verify migrations
docker compose exec postgres psql -U postgres -d llm_cost_ops_dev -c "\dt"
```

### 5. Access Services

Open your browser and access:

| Service | URL | Default Credentials |
|---------|-----|-------------------|
| API Documentation | http://localhost:8080/docs | - |
| API Health Check | http://localhost:8080/health | - |
| Grafana Dashboards | http://localhost:3000 | admin / admin |
| Prometheus | http://localhost:9091 | - |
| Jaeger UI | http://localhost:16686 | - |
| MailHog | http://localhost:8025 | - |
| pgAdmin | http://localhost:5050 | admin@llm-cost-ops.local / admin |
| Redis Commander | http://localhost:8081 | admin / admin |

### 6. Test API

```bash
# Health check
curl http://localhost:8080/health

# Get API version
curl http://localhost:8080/api/v1/version

# View metrics
curl http://localhost:9090/metrics
```

### 7. Stop Services

```bash
# Stop but keep data
docker compose down

# Stop and remove all data (CAUTION!)
docker compose down -v

# Stop specific service
docker compose stop app
```

---

## Development Workflow

### Starting Development

#### Option 1: Full Stack with Hot Reload

```bash
# Start all services
docker compose up -d

# View application logs
docker compose logs -f app

# Code changes trigger automatic rebuild
# (cargo-watch monitors /app/src directory)
```

#### Option 2: Minimal Stack

```bash
# Start only essential services
docker compose up -d postgres redis nats app

# Skip monitoring services to save resources
```

#### Option 3: Custom Configuration

```bash
# Create override file
cat > docker-compose.override.yml << 'EOF'
version: '3.8'
services:
  app:
    environment:
      RUST_LOG: trace
      LOG_LEVEL: trace
    volumes:
      - ./custom-config.toml:/app/config.toml:ro
EOF

# Start with override
docker compose up -d
```

### Development Tasks

#### Running Tests

```bash
# Unit tests
docker compose exec app cargo test

# Integration tests
docker compose exec app cargo test --test '*'

# Specific test
docker compose exec app cargo test test_cost_calculation

# With output
docker compose exec app cargo test -- --nocapture

# Run tests with test compose file
docker compose -f docker-compose.test.yml up --abort-on-container-exit
```

#### Code Quality

```bash
# Format code
docker compose exec app cargo fmt

# Check formatting
docker compose exec app cargo fmt -- --check

# Run linter
docker compose exec app cargo clippy

# Run clippy with strict warnings
docker compose exec app cargo clippy -- -D warnings

# Security audit
docker compose exec app cargo audit
```

#### Database Operations

```bash
# Access PostgreSQL shell
docker compose exec postgres psql -U postgres -d llm_cost_ops_dev

# Create new migration
docker compose exec app cargo sqlx migrate add migration_name

# Run migrations
docker compose exec app cargo sqlx migrate run

# Revert last migration
docker compose exec app cargo sqlx migrate revert

# Check migration status
docker compose exec app cargo sqlx migrate info

# Reset database (CAUTION!)
docker compose exec postgres psql -U postgres -c "DROP DATABASE llm_cost_ops_dev;"
docker compose exec postgres psql -U postgres -c "CREATE DATABASE llm_cost_ops_dev;"
docker compose exec app cargo sqlx migrate run
```

#### Redis Operations

```bash
# Access Redis CLI
docker compose exec redis redis-cli

# Check cache contents
docker compose exec redis redis-cli KEYS '*'

# Clear all cache
docker compose exec redis redis-cli FLUSHALL

# Monitor Redis commands
docker compose exec redis redis-cli MONITOR

# Get cache statistics
docker compose exec redis redis-cli INFO stats
```

#### NATS Operations

```bash
# View NATS status
curl http://localhost:8222/varz | jq

# View connections
curl http://localhost:8222/connz | jq

# View subscriptions
curl http://localhost:8222/subsz | jq

# View routes (cluster)
curl http://localhost:8222/routez | jq
```

### Interactive Development Shell

```bash
# Start interactive shell in app container
docker compose exec app bash

# Inside container, run any command:
cargo build
cargo test
cargo run
cargo doc --open
```

### Debugging

#### Attach Debugger

```bash
# Build with debug symbols
docker compose exec app cargo build

# Run with debugger
docker compose exec app rust-gdb target/debug/llm-cost-ops
```

#### View Application Logs

```bash
# Follow all logs
docker compose logs -f

# Follow specific service
docker compose logs -f app

# Last 100 lines
docker compose logs --tail=100 app

# Filter by time
docker compose logs --since 30m app

# Save logs to file
docker compose logs app > app.log
```

#### Check Resource Usage

```bash
# Real-time stats
docker stats

# Specific container
docker stats llm-cost-ops-app

# Export stats to JSON
docker stats --no-stream --format "{{json .}}" | jq
```

### Hot Reload Configuration

The development container uses `cargo-watch` for automatic rebuilding:

```bash
# Configuration in docker-compose.yml
services:
  app:
    command: cargo watch -x run
    volumes:
      - ./src:/app/src:ro
```

**How it works:**
1. File change detected in `/app/src`
2. `cargo-watch` triggers rebuild
3. Application restarts automatically
4. Logs show rebuild progress

**Customizing watch behavior:**

```bash
# Watch and run tests
docker compose exec app cargo watch -x test

# Watch and check
docker compose exec app cargo watch -x check

# Watch specific files
docker compose exec app cargo watch -w src -w Cargo.toml -x run

# Clear screen on each run
docker compose exec app cargo watch -c -x run
```

### Building Custom Images

#### Build Development Image

```bash
# Build from Dockerfile.dev
docker compose build app

# Build with no cache
docker compose build --no-cache app

# Build with specific Rust version
docker compose build --build-arg RUST_VERSION=1.76 app
```

#### Build Production Image

```bash
# Build production image
docker build -t llm-cost-ops:latest -f Dockerfile .

# Build with metadata
docker build \
  -t llm-cost-ops:latest \
  --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
  --build-arg VCS_REF=$(git rev-parse --short HEAD) \
  -f Dockerfile .

# Multi-platform build
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t llm-cost-ops:latest \
  -f Dockerfile .
```

### Performance Profiling

```bash
# Enable profiling
docker compose exec app cargo build --release

# Run with profiling
docker compose exec app cargo run --release

# Generate flamegraph (requires cargo-flamegraph)
docker compose exec app cargo flamegraph
```

### Managing Dependencies

```bash
# Update dependencies
docker compose exec app cargo update

# Add new dependency
docker compose exec app cargo add tokio

# Remove dependency
docker compose exec app cargo rm tokio

# Check outdated packages
docker compose exec app cargo outdated

# Tree view of dependencies
docker compose exec app cargo tree
```

---

## Production Deployment

### Pre-Deployment Checklist

Before deploying to production, ensure:

- [ ] Environment variables configured securely
- [ ] SSL/TLS certificates obtained
- [ ] Database backups configured
- [ ] Monitoring and alerting set up
- [ ] Resource limits defined
- [ ] Security scanning completed
- [ ] Load testing performed
- [ ] Disaster recovery plan documented
- [ ] Access controls configured
- [ ] Logging aggregation set up

### Production Environment Setup

#### 1. Prepare Production Configuration

```bash
# Copy production template
cp .env.example .env.production

# Edit with secure values
nano .env.production
```

**Critical production settings:**

```bash
# Security
JWT_SECRET=$(openssl rand -base64 64)
POSTGRES_PASSWORD=$(openssl rand -base64 32)
REDIS_PASSWORD=$(openssl rand -base64 32)
GF_SECURITY_ADMIN_PASSWORD=$(openssl rand -base64 16)

# Application
RUST_LOG=info
LOG_LEVEL=info
ENABLE_COMPRESSION=true
ENABLE_RATE_LIMITING=true

# Database
DATABASE_MAX_CONNECTIONS=50
DATABASE_IDLE_TIMEOUT=300

# CORS
CORS_ALLOWED_ORIGINS=https://your-domain.com

# Email
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_FROM=noreply@your-domain.com

# Backup
AWS_ACCESS_KEY_ID=your-key
AWS_SECRET_ACCESS_KEY=your-secret
AWS_S3_BUCKET=llm-cost-ops-backups
AWS_REGION=us-east-1
```

#### 2. SSL/TLS Certificate Setup

```bash
# Option 1: Let's Encrypt (recommended)
mkdir -p docker/nginx/ssl

# Install certbot
sudo apt-get install certbot

# Generate certificate
sudo certbot certonly --standalone -d your-domain.com

# Copy certificates
sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem docker/nginx/ssl/
sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem docker/nginx/ssl/

# Option 2: Self-signed (testing only)
openssl req -x509 -nodes -days 365 -newkey rsa:4096 \
  -keyout docker/nginx/ssl/server.key \
  -out docker/nginx/ssl/server.crt \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=your-domain.com"
```

#### 3. Initialize Production Database

```bash
# Start database first
docker compose -f docker-compose.prod.yml up -d postgres

# Wait for healthy status
docker compose -f docker-compose.prod.yml ps postgres

# Run migrations
docker compose -f docker-compose.prod.yml run --rm app \
  cargo sqlx migrate run

# Create admin user (if applicable)
docker compose -f docker-compose.prod.yml exec postgres \
  psql -U postgres -d llm_cost_ops_prod -c \
  "INSERT INTO users (email, role) VALUES ('admin@your-domain.com', 'admin');"
```

#### 4. Deploy Production Stack

```bash
# Pull latest images
docker compose -f docker-compose.prod.yml pull

# Start all services
docker compose -f docker-compose.prod.yml --env-file .env.production up -d

# Verify health
docker compose -f docker-compose.prod.yml ps

# Check logs
docker compose -f docker-compose.prod.yml logs -f
```

#### 5. Configure Monitoring

```bash
# Import Grafana dashboards
curl -X POST http://localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -u admin:${GF_SECURITY_ADMIN_PASSWORD} \
  -d @docker/grafana/dashboards/llm-cost-ops.json

# Configure Prometheus alerts
docker compose -f docker-compose.prod.yml restart prometheus

# Test alerting
curl -X POST http://localhost:9091/-/reload
```

### Production Compose File

Create `docker-compose.prod.yml`:

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
    environment:
      RUST_LOG: info
      DATABASE_URL: postgres://postgres:${POSTGRES_PASSWORD}@postgres:5432/llm_cost_ops_prod
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379/0
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 30s

  nginx:
    image: nginx:alpine
    restart: always
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./docker/nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./docker/nginx/conf.d:/etc/nginx/conf.d:ro
      - ./docker/nginx/ssl:/etc/nginx/ssl:ro
    depends_on:
      - app
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G

  postgres:
    image: postgres:16-alpine
    restart: always
    environment:
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: llm_cost_ops_prod
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./backups:/backups
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 8G
    command: >
      postgres
      -c max_connections=200
      -c shared_buffers=2GB
      -c effective_cache_size=6GB
      -c maintenance_work_mem=512MB
      -c checkpoint_completion_target=0.9
      -c wal_buffers=16MB
      -c default_statistics_target=100
      -c random_page_cost=1.1
      -c effective_io_concurrency=200
      -c work_mem=10MB
      -c min_wal_size=1GB
      -c max_wal_size=4GB

  redis:
    image: redis:7-alpine
    restart: always
    command: >
      redis-server
      --requirepass ${REDIS_PASSWORD}
      --maxmemory 2gb
      --maxmemory-policy allkeys-lru
      --save 900 1
      --save 300 10
      --save 60 10000
    volumes:
      - redis-data:/data
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G

volumes:
  postgres-data:
  redis-data:
```

### Scaling Services

#### Horizontal Scaling (Multiple App Instances)

```bash
# Scale to 5 app instances
docker compose -f docker-compose.prod.yml up -d --scale app=5

# Verify instances
docker compose -f docker-compose.prod.yml ps app

# Load balancing via nginx upstream
# See docker/nginx/conf.d/default.conf
```

#### Vertical Scaling (Increase Resources)

```bash
# Edit docker-compose.prod.yml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '8.0'
          memory: 16G

# Restart with new limits
docker compose -f docker-compose.prod.yml up -d app
```

### Rolling Updates

```bash
# Build new version
docker build -t llm-cost-ops:v2.0.0 .

# Tag as latest
docker tag llm-cost-ops:v2.0.0 llm-cost-ops:latest

# Update with rolling deployment
docker compose -f docker-compose.prod.yml up -d --no-deps --scale app=6 app

# Wait for health checks
sleep 30

# Scale down old instances
docker compose -f docker-compose.prod.yml up -d --no-deps --scale app=3 app
```

### Health Checks

Production health check configuration:

```yaml
healthcheck:
  test: |
    curl -f http://localhost:8080/health || exit 1
    curl -f http://localhost:9090/metrics || exit 1
  interval: 10s
  timeout: 5s
  retries: 3
  start_period: 30s
```

### Backup Strategy

See [BACKUP_AND_RECOVERY.md](./BACKUP_AND_RECOVERY.md) for detailed procedures.

**Quick backup:**

```bash
# Automated backup via cron
docker compose -f docker-compose.prod.yml exec postgres \
  /backups/backup.sh

# Manual backup
docker compose -f docker-compose.prod.yml exec postgres \
  pg_dump -U postgres llm_cost_ops_prod | gzip > backup-$(date +%Y%m%d-%H%M%S).sql.gz
```

---

## Service Configuration

### Application Configuration

Configuration file: `config.toml`

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4
keep_alive = 75

[database]
url = "postgres://postgres:password@postgres:5432/llm_cost_ops"
max_connections = 20
min_connections = 5
connection_timeout = 30
idle_timeout = 600

[redis]
url = "redis://redis:6379/0"
pool_size = 10
timeout = 5

[nats]
url = "nats://nats:4222"
cluster_id = "llm-cost-ops"
client_id = "llm-cost-ops-1"

[observability]
jaeger_endpoint = "http://jaeger:14268/api/traces"
metrics_port = 9090
log_level = "info"

[security]
jwt_secret = "change-in-production"
jwt_expiry = 3600
cors_origins = ["*"]

[features]
rate_limiting = true
compression = true
metrics = true
tracing = true
```

### PostgreSQL Tuning

Configuration in `docker-compose.yml`:

```yaml
postgres:
  command:
    - postgres
    - -c max_connections=100
    - -c shared_buffers=256MB
    - -c effective_cache_size=1GB
    - -c maintenance_work_mem=64MB
    - -c checkpoint_completion_target=0.9
    - -c wal_buffers=16MB
    - -c default_statistics_target=100
    - -c random_page_cost=1.1  # For SSD
    - -c effective_io_concurrency=200
    - -c work_mem=4MB
    - -c min_wal_size=1GB
    - -c max_wal_size=4GB
    - -c max_worker_processes=4
    - -c max_parallel_workers_per_gather=2
    - -c max_parallel_workers=4
```

### Redis Configuration

Configuration file: `docker/redis/redis.conf`

```conf
# Network
bind 0.0.0.0
protected-mode yes
port 6379

# General
daemonize no
supervised no
loglevel notice

# Snapshotting
save 900 1
save 300 10
save 60 10000

# Replication
# replicaof <masterip> <masterport>

# Security
# requirepass yourpassword

# Limits
maxclients 10000
maxmemory 512mb
maxmemory-policy allkeys-lru

# Append Only File
appendonly yes
appendfilename "appendonly.aof"
appendfsync everysec

# Slow log
slowlog-log-slower-than 10000
slowlog-max-len 128
```

### NATS Configuration

Configuration file: `docker/nats/nats.conf`

```conf
# Server Configuration
port: 4222
http_port: 8222

# Logging
debug: false
trace: false
logtime: true

# Limits
max_connections: 65536
max_payload: 1048576
max_pending: 67108864

# Timeouts
ping_interval: "2m"
ping_max: 3
write_deadline: "10s"

# JetStream
jetstream {
  store_dir: "/data/jetstream"
  max_memory: 1GB
  max_file: 10GB
}

# Clustering
cluster {
  name: llm-cost-ops-cluster
  listen: 0.0.0.0:6222

  routes = [
    nats://nats-1:6222
    nats://nats-2:6222
    nats://nats-3:6222
  ]
}
```

### Prometheus Configuration

Configuration file: `docker/prometheus/prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'llm-cost-ops'
    environment: 'production'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - /etc/prometheus/alerts.yml

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'llm-cost-ops'
    static_configs:
      - targets: ['app:9090']
    metrics_path: '/metrics'
    scrape_interval: 10s

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

  - job_name: 'nats'
    static_configs:
      - targets: ['nats:8222']
    metrics_path: '/metrics'

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
```

### Grafana Configuration

Environment variables in `docker-compose.yml`:

```yaml
grafana:
  environment:
    # Server
    GF_SERVER_ROOT_URL: https://grafana.your-domain.com
    GF_SERVER_DOMAIN: grafana.your-domain.com

    # Security
    GF_SECURITY_ADMIN_USER: admin
    GF_SECURITY_ADMIN_PASSWORD: ${GF_ADMIN_PASSWORD}
    GF_SECURITY_SECRET_KEY: ${GF_SECRET_KEY}
    GF_SECURITY_DISABLE_GRAVATAR: true

    # Database
    GF_DATABASE_TYPE: postgres
    GF_DATABASE_HOST: postgres:5432
    GF_DATABASE_NAME: grafana
    GF_DATABASE_USER: grafana
    GF_DATABASE_PASSWORD: ${GRAFANA_DB_PASSWORD}

    # Auth
    GF_AUTH_DISABLE_LOGIN_FORM: false
    GF_AUTH_DISABLE_SIGNOUT_MENU: false

    # SMTP
    GF_SMTP_ENABLED: true
    GF_SMTP_HOST: ${SMTP_HOST}:${SMTP_PORT}
    GF_SMTP_USER: ${SMTP_USER}
    GF_SMTP_PASSWORD: ${SMTP_PASSWORD}
    GF_SMTP_FROM_ADDRESS: ${SMTP_FROM}

    # Alerting
    GF_ALERTING_ENABLED: true
    GF_UNIFIED_ALERTING_ENABLED: true
```

---

## Networking

### Network Overview

The platform uses two isolated Docker networks:

**1. Application Network (llm-cost-ops-network)**
- Type: Bridge
- Subnet: 172.28.0.0/16
- Purpose: Internal service communication
- Services: app, postgres, redis, nats, mailhog, pgadmin, redis-commander

**2. Monitoring Network (monitoring-network)**
- Type: Bridge
- Subnet: Auto-assigned
- Purpose: Observability stack
- Services: app, prometheus, grafana, jaeger, nats

### Network Inspection

```bash
# List networks
docker network ls

# Inspect network
docker network inspect llm-cost-ops-network

# View connected containers
docker network inspect llm-cost-ops-network \
  --format '{{range .Containers}}{{.Name}} {{.IPv4Address}}{{"\n"}}{{end}}'

# Test connectivity between services
docker compose exec app ping postgres
docker compose exec app curl http://redis:6379
```

### Service Discovery

Services communicate using Docker's built-in DNS:

```bash
# From app container:
ping postgres          # Resolves to postgres container IP
curl http://redis:6379 # Connects to Redis
nc -zv nats 4222      # Test NATS connection
```

### Port Mapping

**Development Ports:**

| Service | Internal Port | External Port | Purpose |
|---------|--------------|---------------|---------|
| app | 8080 | 8080 | HTTP API |
| app | 9090 | 9090 | Metrics |
| postgres | 5432 | 5432 | PostgreSQL |
| redis | 6379 | 6379 | Redis |
| nats | 4222 | 4222 | NATS Client |
| nats | 8222 | 8222 | NATS Management |
| prometheus | 9090 | 9091 | Prometheus |
| grafana | 3000 | 3000 | Grafana |
| jaeger | 16686 | 16686 | Jaeger UI |
| mailhog | 8025 | 8025 | MailHog UI |
| pgadmin | 80 | 5050 | pgAdmin |
| redis-commander | 8081 | 8081 | Redis Commander |

**Production Ports (via Nginx):**

| Service | Internal Port | External Port | Purpose |
|---------|--------------|---------------|---------|
| nginx | 80 | 80 | HTTP (redirects to HTTPS) |
| nginx | 443 | 443 | HTTPS |
| All others | - | Not exposed | Internal only |

### Network Security

**Best practices:**

```yaml
# docker-compose.prod.yml
networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true  # No external access

services:
  app:
    networks:
      - frontend
      - backend

  postgres:
    networks:
      - backend  # Not accessible from outside
```

### Custom Network Configuration

```yaml
networks:
  llm-cost-ops-network:
    driver: bridge
    driver_opts:
      com.docker.network.bridge.name: llm-ops-br0
      com.docker.network.bridge.enable_icc: "true"
      com.docker.network.bridge.enable_ip_masquerade: "true"
    ipam:
      driver: default
      config:
        - subnet: 172.28.0.0/16
          gateway: 172.28.0.1
```

---

## Storage and Volumes

### Volume Types

**1. Named Volumes (Recommended for Production)**

```yaml
volumes:
  postgres-data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/data/postgres
```

**2. Bind Mounts (Development)**

```yaml
volumes:
  - ./src:/app/src:ro
  - ./config.toml:/app/config.toml:ro
```

**3. Tmpfs Mounts (Temporary Data)**

```yaml
tmpfs:
  - /tmp
  - /run
```

### Volume Management

```bash
# List volumes
docker volume ls

# Inspect volume
docker volume inspect llm-cost-ops_postgres-data

# Create volume
docker volume create --name postgres-backup

# Remove unused volumes
docker volume prune

# Backup volume
docker run --rm \
  -v llm-cost-ops_postgres-data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/postgres-data.tar.gz /data

# Restore volume
docker run --rm \
  -v llm-cost-ops_postgres-data:/data \
  -v $(pwd):/backup \
  alpine tar xzf /backup/postgres-data.tar.gz -C /
```

### Storage Monitoring

```bash
# Check volume disk usage
docker system df -v

# Monitor specific volume
docker volume inspect llm-cost-ops_postgres-data \
  --format '{{.Mountpoint}}' | \
  xargs sudo du -sh

# Set up volume size alerts (see MONITORING.md)
```

### Volume Backup Strategy

See detailed guide in [docs/docker/BACKUP_RECOVERY.md]

**Quick backup script:**

```bash
#!/bin/bash
# backup-volumes.sh

BACKUP_DIR="/backups/$(date +%Y%m%d)"
mkdir -p $BACKUP_DIR

# Backup database
docker compose exec postgres pg_dump -U postgres llm_cost_ops_prod | \
  gzip > $BACKUP_DIR/postgres.sql.gz

# Backup Redis
docker compose exec redis redis-cli BGSAVE
docker cp llm-cost-ops-redis:/data/dump.rdb $BACKUP_DIR/

# Backup Grafana
docker cp llm-cost-ops-grafana:/var/lib/grafana $BACKUP_DIR/grafana

# Upload to S3
aws s3 sync $BACKUP_DIR s3://llm-cost-ops-backups/$(date +%Y%m%d)/
```

---

## Security

### Image Security

**Scan images for vulnerabilities:**

```bash
# Using Trivy
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image llm-cost-ops:latest

# Using Snyk
snyk container test llm-cost-ops:latest

# Using Docker Scout
docker scout cves llm-cost-ops:latest
```

### Container Hardening

**Production security settings:**

```yaml
services:
  app:
    # Run as non-root user
    user: "1000:1000"

    # Read-only root filesystem
    read_only: true

    # Drop all capabilities
    cap_drop:
      - ALL

    # Add only required capabilities
    cap_add:
      - NET_BIND_SERVICE

    # No new privileges
    security_opt:
      - no-new-privileges:true

    # Limit resources
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
          pids: 100
```

### Secret Management

**Using Docker Secrets:**

```bash
# Create secrets
echo "superSecretPassword" | docker secret create postgres_password -

# Use in compose
services:
  postgres:
    secrets:
      - postgres_password
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/postgres_password

secrets:
  postgres_password:
    external: true
```

**Using environment files:**

```bash
# .env.production (never commit!)
POSTGRES_PASSWORD=xxxxx
JWT_SECRET=xxxxx

# Load in compose
docker compose --env-file .env.production up -d
```

### Network Security

**Isolate services:**

```yaml
networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true  # No internet access
```

### Access Control

**Implement authentication:**

```bash
# Basic auth for monitoring services
htpasswd -c docker/nginx/.htpasswd admin

# Configure in nginx.conf
location /prometheus {
    auth_basic "Restricted";
    auth_basic_user_file /etc/nginx/.htpasswd;
    proxy_pass http://prometheus:9090;
}
```

For comprehensive security practices, see [SECURITY.md](./SECURITY.md).

---

## Monitoring and Observability

### Metrics Collection

**Application metrics exposed at `/metrics`:**

```
# Counter
http_requests_total{method="GET",path="/api/v1/costs",status="200"} 1523
http_requests_total{method="POST",path="/api/v1/usage",status="201"} 234

# Gauge
process_resident_memory_bytes 125829120
database_connections_active 5

# Histogram
http_request_duration_seconds_bucket{le="0.1"} 450
http_request_duration_seconds_bucket{le="0.5"} 890
http_request_duration_seconds_sum 234.5
http_request_duration_seconds_count 900
```

**View metrics:**

```bash
# Application metrics
curl http://localhost:9090/metrics

# Prometheus metrics
curl http://localhost:9091/api/v1/query?query=up

# Export all metrics
curl http://localhost:9091/api/v1/query?query=up | jq
```

### Logging

**View logs:**

```bash
# All services
docker compose logs -f

# Specific service with timestamps
docker compose logs -f --timestamps app

# JSON formatted logs
docker compose logs app | jq

# Export logs
docker compose logs --no-color > logs.txt
```

**Log aggregation (Loki):**

```yaml
# docker-compose.yml
services:
  loki:
    image: grafana/loki:2.9.0
    ports:
      - "3100:3100"
    volumes:
      - ./docker/loki/loki-config.yml:/etc/loki/local-config.yaml
    command: -config.file=/etc/loki/local-config.yaml

  promtail:
    image: grafana/promtail:2.9.0
    volumes:
      - /var/log:/var/log
      - ./docker/promtail/promtail-config.yml:/etc/promtail/config.yml
    command: -config.file=/etc/promtail/config.yml
```

### Distributed Tracing

**View traces in Jaeger:**

1. Open http://localhost:16686
2. Select service: "llm-cost-ops"
3. Click "Find Traces"

**Trace structure:**

```
POST /api/v1/costs [200ms]
├── database_query [50ms]
│   ├── acquire_connection [5ms]
│   └── execute_query [45ms]
├── cache_lookup [10ms]
└── response_serialization [5ms]
```

### Dashboards

**Import pre-built dashboards:**

```bash
# Application dashboard
docker/grafana/dashboards/llm-cost-ops.json

# PostgreSQL dashboard
docker/grafana/dashboards/postgres.json

# Redis dashboard
docker/grafana/dashboards/redis.json

# NATS dashboard
docker/grafana/dashboards/nats.json
```

### Alerting

**Prometheus alert rules:**

```yaml
# docker/prometheus/alerts.yml
groups:
  - name: llm-cost-ops
    rules:
      - alert: HighErrorRate
        expr: |
          rate(http_requests_total{status=~"5.."}[5m])
          / rate(http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }}"

      - alert: DatabaseDown
        expr: up{job="postgres"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Database is down"

      - alert: HighMemoryUsage
        expr: |
          (container_memory_usage_bytes / container_spec_memory_limit_bytes) > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
```

For detailed monitoring setup, see [MONITORING.md](./MONITORING.md).

---

## Scaling

### Horizontal Scaling

**Scale application instances:**

```bash
# Scale to 5 instances
docker compose up -d --scale app=5

# Verify
docker compose ps app

# View instance distribution
docker compose exec nginx cat /etc/nginx/conf.d/upstream.conf
```

**Nginx load balancing configuration:**

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
        proxy_next_upstream error timeout invalid_header http_500;
        proxy_connect_timeout 5s;
    }
}
```

### Vertical Scaling

**Increase container resources:**

```yaml
# docker-compose.override.yml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 8G
        reservations:
          cpus: '2.0'
          memory: 4G
```

### Database Scaling

**Read replicas:**

```yaml
services:
  postgres-master:
    image: postgres:16-alpine
    environment:
      POSTGRES_REPLICATION_MODE: master

  postgres-replica-1:
    image: postgres:16-alpine
    environment:
      POSTGRES_REPLICATION_MODE: slave
      POSTGRES_MASTER_HOST: postgres-master
      POSTGRES_MASTER_PORT: 5432
```

**Connection pooling:**

```yaml
services:
  pgbouncer:
    image: pgbouncer/pgbouncer:latest
    environment:
      DATABASES_HOST: postgres
      DATABASES_PORT: 5432
      DATABASES_DATABASE: llm_cost_ops_prod
      DATABASES_USER: postgres
      PGBOUNCER_POOL_MODE: transaction
      PGBOUNCER_MAX_CLIENT_CONN: 1000
      PGBOUNCER_DEFAULT_POOL_SIZE: 25
```

### Cache Scaling

**Redis Cluster:**

```yaml
services:
  redis-master:
    image: redis:7-alpine
    command: redis-server --appendonly yes

  redis-replica-1:
    image: redis:7-alpine
    command: redis-server --replicaof redis-master 6379

  redis-sentinel:
    image: redis:7-alpine
    command: redis-sentinel /etc/redis/sentinel.conf
```

### Auto-Scaling

**Docker Swarm mode:**

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-compose.prod.yml llm-cost-ops

# Auto-scale based on CPU
docker service update \
  --replicas-max-per-node 2 \
  --update-parallelism 1 \
  --update-delay 10s \
  llm-cost-ops_app
```

**Kubernetes (see [KUBERNETES.md](./KUBERNETES.md)):**

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: llm-cost-ops-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-cost-ops
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

---

## Backup and Recovery

### Automated Backups

**Backup script (`docker/backup/backup.sh`):**

```bash
#!/bin/bash
set -e

BACKUP_DIR="/backups/$(date +%Y%m%d-%H%M%S)"
S3_BUCKET="s3://llm-cost-ops-backups"

mkdir -p $BACKUP_DIR

echo "Backing up PostgreSQL..."
docker compose exec -T postgres pg_dump -U postgres llm_cost_ops_prod | \
  gzip > $BACKUP_DIR/postgres.sql.gz

echo "Backing up Redis..."
docker compose exec redis redis-cli BGSAVE
docker cp llm-cost-ops-redis:/data/dump.rdb $BACKUP_DIR/

echo "Backing up Grafana..."
docker cp llm-cost-ops-grafana:/var/lib/grafana $BACKUP_DIR/grafana

echo "Backing up Prometheus..."
docker cp llm-cost-ops-prometheus:/prometheus $BACKUP_DIR/prometheus

echo "Uploading to S3..."
aws s3 sync $BACKUP_DIR $S3_BUCKET/$(date +%Y%m%d-%H%M%S)/

echo "Cleaning up old backups..."
find /backups -mtime +30 -delete

echo "Backup complete!"
```

**Schedule with cron:**

```bash
# crontab -e
0 2 * * * /app/docker/backup/backup.sh >> /var/log/backup.log 2>&1
```

### Point-in-Time Recovery

**PostgreSQL WAL archiving:**

```yaml
services:
  postgres:
    command:
      - postgres
      - -c wal_level=replica
      - -c archive_mode=on
      - -c archive_command='cp %p /backups/wal/%f'
    volumes:
      - ./backups/wal:/backups/wal
```

### Disaster Recovery

**Full system restore:**

```bash
#!/bin/bash
# restore.sh

# Stop all services
docker compose down

# Restore PostgreSQL
gunzip -c backup/postgres.sql.gz | \
  docker compose exec -T postgres psql -U postgres llm_cost_ops_prod

# Restore Redis
docker cp backup/dump.rdb llm-cost-ops-redis:/data/

# Restore Grafana
docker cp backup/grafana/. llm-cost-ops-grafana:/var/lib/grafana/

# Restart services
docker compose up -d

# Verify
docker compose ps
docker compose exec app curl http://localhost:8080/health
```

---

## Troubleshooting

### Common Issues

#### Service Won't Start

**Symptoms:**
- Container exits immediately
- "Unhealthy" status
- Connection refused errors

**Diagnosis:**

```bash
# Check container status
docker compose ps

# View logs
docker compose logs app

# Inspect container
docker inspect llm-cost-ops-app

# Check resource usage
docker stats
```

**Solutions:**

```bash
# Restart service
docker compose restart app

# Rebuild image
docker compose build --no-cache app

# Reset everything
docker compose down -v
docker compose up -d
```

#### Database Connection Errors

**Symptoms:**
- "Connection refused"
- "Too many connections"
- Slow queries

**Diagnosis:**

```bash
# Check database status
docker compose exec postgres pg_isready

# View active connections
docker compose exec postgres psql -U postgres -c \
  "SELECT count(*) FROM pg_stat_activity;"

# Check slow queries
docker compose exec postgres psql -U postgres -c \
  "SELECT query, state, wait_event_type FROM pg_stat_activity WHERE state != 'idle';"
```

**Solutions:**

```bash
# Increase max connections
# Edit docker-compose.yml:
command: postgres -c max_connections=200

# Reset connections
docker compose restart postgres

# Check application connection pool
# Edit config.toml:
[database]
max_connections = 50
```

#### High Memory Usage

**Diagnosis:**

```bash
# Monitor memory
docker stats

# Check specific container
docker stats llm-cost-ops-app

# View memory limits
docker inspect llm-cost-ops-app | jq '.[0].HostConfig.Memory'
```

**Solutions:**

```bash
# Increase memory limit
# docker-compose.override.yml:
services:
  app:
    deploy:
      resources:
        limits:
          memory: 4G

# Restart with new limits
docker compose up -d app
```

#### Port Conflicts

**Symptoms:**
- "port is already allocated"
- Cannot bind to port

**Diagnosis:**

```bash
# Check what's using the port
sudo lsof -i :8080
sudo netstat -tulpn | grep 8080
```

**Solutions:**

```bash
# Change port in docker-compose.override.yml:
services:
  app:
    ports:
      - "8081:8080"

# Or stop conflicting service
sudo systemctl stop service-using-port
```

#### Volume Permission Issues

**Symptoms:**
- "Permission denied"
- Cannot write to volume

**Diagnosis:**

```bash
# Check volume ownership
docker volume inspect llm-cost-ops_postgres-data | jq '.[0].Mountpoint'
sudo ls -la /var/lib/docker/volumes/llm-cost-ops_postgres-data/_data
```

**Solutions:**

```bash
# Fix permissions
docker compose down
docker volume rm llm-cost-ops_postgres-data
docker compose up -d

# Or use named user
# docker-compose.yml:
services:
  postgres:
    user: "999:999"  # postgres user
```

### Debug Mode

**Enable debug logging:**

```bash
# docker-compose.override.yml
services:
  app:
    environment:
      RUST_LOG: trace
      RUST_BACKTRACE: full
      LOG_LEVEL: trace
```

### Performance Issues

**Slow API responses:**

```bash
# Check application metrics
curl http://localhost:9090/metrics | grep http_request_duration

# View traces
# Open http://localhost:16686

# Check database performance
docker compose exec postgres psql -U postgres -c \
  "SELECT * FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;"
```

### Health Check Failures

**Diagnosis:**

```bash
# Manual health check
docker compose exec app curl -f http://localhost:8080/health

# Check health check configuration
docker inspect llm-cost-ops-app | jq '.[0].State.Health'
```

### Getting Help

1. Check logs: `docker compose logs -f`
2. Check health: `docker compose ps`
3. Check metrics: http://localhost:9090/metrics
4. Check traces: http://localhost:16686
5. Review documentation
6. Search GitHub issues
7. Ask in community forums

---

## Best Practices

### Development

1. **Use bind mounts for code** - Enable hot reload
2. **Keep dev and prod configs separate** - Use override files
3. **Use health checks** - Ensure services are ready
4. **Monitor resource usage** - Prevent OOM kills
5. **Use named volumes** - Persist data between restarts
6. **Tag images with versions** - Easy rollback
7. **Run security scans** - Catch vulnerabilities early
8. **Use .dockerignore** - Faster builds

### Production

1. **Use multi-stage builds** - Smaller images
2. **Run as non-root user** - Better security
3. **Set resource limits** - Prevent resource exhaustion
4. **Enable read-only filesystem** - Prevent tampering
5. **Use secrets management** - No plaintext passwords
6. **Implement backup strategy** - Disaster recovery
7. **Monitor everything** - Metrics, logs, traces
8. **Automate updates** - Security patches
9. **Use container orchestration** - Kubernetes, Swarm
10. **Document everything** - Runbooks, procedures

### Security

1. **Scan images regularly** - Trivy, Snyk
2. **Update base images** - Latest patches
3. **Minimize attack surface** - Remove unnecessary tools
4. **Use network isolation** - Separate networks
5. **Implement access control** - Authentication, authorization
6. **Encrypt data at rest** - Volume encryption
7. **Encrypt data in transit** - TLS everywhere
8. **Audit access logs** - Track changes
9. **Use security policies** - AppArmor, SELinux
10. **Regular security audits** - Penetration testing

### Performance

1. **Use caching** - Docker layer caching
2. **Optimize images** - Multi-stage builds
3. **Tune database** - Connection pooling
4. **Use CDN** - Static assets
5. **Enable compression** - Reduce bandwidth
6. **Monitor performance** - Identify bottlenecks
7. **Scale horizontally** - Multiple instances
8. **Use async I/O** - Better throughput
9. **Profile regularly** - Find hot paths
10. **Load test** - Before production

---

## Performance Tuning

### Application Optimization

**Rust compilation:**

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

**Runtime configuration:**

```toml
# config.toml
[server]
workers = 8  # Number of CPU cores
keep_alive = 75
max_request_size = "10MB"

[database]
max_connections = 50
min_connections = 10
connection_timeout = 30
```

### Database Tuning

**PostgreSQL configuration:**

```bash
# Calculated for 8GB RAM, 4 CPUs
shared_buffers = 2GB              # 25% of RAM
effective_cache_size = 6GB        # 75% of RAM
maintenance_work_mem = 512MB      # RAM / 16
work_mem = 10MB                   # RAM / (max_connections * 16)
max_connections = 200
random_page_cost = 1.1            # For SSD
effective_io_concurrency = 200    # For SSD
```

**Indexes:**

```sql
-- Add indexes for frequently queried columns
CREATE INDEX idx_costs_user_id ON costs(user_id);
CREATE INDEX idx_costs_created_at ON costs(created_at DESC);
CREATE INDEX idx_usage_model_id ON usage(model_id);
```

### Redis Tuning

```conf
# redis.conf
maxmemory 2gb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
tcp-backlog 511
timeout 300
```

### Nginx Optimization

```nginx
# nginx.conf
worker_processes auto;
worker_rlimit_nofile 65535;

events {
    worker_connections 4096;
    use epoll;
    multi_accept on;
}

http {
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    keepalive_requests 100;

    gzip on;
    gzip_vary on;
    gzip_types text/plain text/css application/json;

    proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=app_cache:10m;
}
```

### Docker Optimization

**Build optimization:**

```dockerfile
# Use build cache effectively
FROM rust:1.75 AS builder

# Cache dependencies separately
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build application
COPY src ./src
RUN cargo build --release
```

**Runtime optimization:**

```yaml
services:
  app:
    # Use host network for better performance (use with caution)
    # network_mode: host

    # Adjust ulimits
    ulimits:
      nofile:
        soft: 65536
        hard: 65536

    # Use tmpfs for temporary files
    tmpfs:
      - /tmp:size=1G,mode=1777
```

---

## Migration Guides

### From Standalone to Docker

1. **Export existing data**

```bash
# Export database
pg_dump llm_cost_ops > database.sql

# Export config
cp /etc/llm-cost-ops/config.toml ./
```

2. **Configure Docker environment**

```bash
cp .env.example .env
# Edit .env with existing configuration
```

3. **Import data**

```bash
# Start containers
docker compose up -d postgres

# Import database
cat database.sql | docker compose exec -T postgres \
  psql -U postgres llm_cost_ops_dev

# Start application
docker compose up -d app
```

### From Docker Compose to Kubernetes

See detailed guide in [KUBERNETES.md](./KUBERNETES.md)

### Version Upgrades

**Upgrading from v1.x to v2.x:**

```bash
# Backup current version
docker compose exec postgres pg_dump -U postgres llm_cost_ops_prod > backup-v1.sql

# Pull new version
docker pull llm-cost-ops:v2.0.0

# Stop current version
docker compose down

# Update docker-compose.yml
# image: llm-cost-ops:v2.0.0

# Start new version
docker compose up -d

# Run migrations
docker compose exec app cargo sqlx migrate run
```

---

## FAQ

**Q: How do I access container logs?**

```bash
docker compose logs -f app
```

**Q: How do I reset the database?**

```bash
docker compose down -v
docker compose up -d
```

**Q: How do I scale to multiple instances?**

```bash
docker compose up -d --scale app=3
```

**Q: How do I update to the latest version?**

```bash
docker compose pull
docker compose up -d
```

**Q: How do I backup my data?**

```bash
docker compose exec postgres pg_dump -U postgres llm_cost_ops_prod | gzip > backup.sql.gz
```

**Q: How do I connect to the database?**

```bash
docker compose exec postgres psql -U postgres -d llm_cost_ops_dev
```

**Q: Why is my container using so much memory?**

Check resource limits and application configuration. See [Performance Tuning](#performance-tuning).

**Q: How do I enable SSL/TLS?**

See [Production Deployment](#production-deployment) section.

**Q: Can I run this on Windows?**

Yes, using Docker Desktop with WSL2. See [Prerequisites](#prerequisites).

**Q: How do I troubleshoot networking issues?**

```bash
docker network inspect llm-cost-ops-network
docker compose exec app ping postgres
```

---

## Reference

### Environment Variables

Complete list of environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| RUST_LOG | info | Logging level |
| DATABASE_URL | postgres://... | Database connection string |
| REDIS_URL | redis://redis:6379 | Redis connection string |
| JWT_SECRET | - | JWT signing secret |
| PORT | 8080 | HTTP port |
| METRICS_PORT | 9090 | Metrics port |
| ENABLE_COMPRESSION | true | Enable gzip compression |
| ENABLE_RATE_LIMITING | true | Enable rate limiting |
| CORS_ALLOWED_ORIGINS | * | CORS allowed origins |

### Docker Commands

Common Docker commands:

```bash
# Images
docker images
docker build -t name:tag .
docker pull image:tag
docker push image:tag
docker rmi image:tag

# Containers
docker ps
docker ps -a
docker run image
docker stop container
docker rm container
docker logs container
docker exec -it container bash

# Compose
docker compose up
docker compose down
docker compose ps
docker compose logs
docker compose build
docker compose pull

# System
docker system df
docker system prune
docker volume ls
docker network ls
```

### File Locations

Important file locations:

```
/workspaces/llm-cost-ops/
├── docker-compose.yml           # Main compose file
├── docker-compose.prod.yml      # Production compose
├── docker-compose.test.yml      # Testing compose
├── Dockerfile                   # Production image
├── Dockerfile.dev               # Development image
├── .env.example                 # Environment template
├── config.toml                  # Application config
├── docker/
│   ├── nginx/                   # Nginx configs
│   ├── postgres/                # PostgreSQL configs
│   ├── redis/                   # Redis configs
│   ├── prometheus/              # Prometheus configs
│   ├── grafana/                 # Grafana configs
│   └── backup/                  # Backup scripts
└── docs/docker/
    ├── README.md                # This file
    ├── DOCKER_COMPOSE.md        # Compose guide
    ├── KUBERNETES.md            # K8s guide
    ├── HELM.md                  # Helm guide
    ├── SECURITY.md              # Security guide
    └── MONITORING.md            # Monitoring guide
```

### Related Documentation

- [Docker Compose Guide](./DOCKER_COMPOSE.md)
- [Kubernetes Deployment](./KUBERNETES.md)
- [Helm Charts](./HELM.md)
- [Security Best Practices](./SECURITY.md)
- [Monitoring Setup](./MONITORING.md)
- [Main README](../../README.md)
- [Quick Start](../../DOCKER_QUICKSTART.md)

---

## Support

### Getting Help

1. **Documentation**: Read this guide and related docs
2. **GitHub Issues**: Search existing issues or create new one
3. **Community Forum**: Ask questions and share experiences
4. **Stack Overflow**: Tag with `llm-cost-ops` and `docker`

### Reporting Bugs

When reporting bugs, include:

1. Docker version: `docker --version`
2. Compose version: `docker compose version`
3. OS and version
4. Steps to reproduce
5. Expected vs actual behavior
6. Relevant logs: `docker compose logs`
7. Container status: `docker compose ps`

### Contributing

Contributions welcome! See [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

## License

Apache 2.0 - See [LICENSE](../../LICENSE)

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0
**Maintainers:** LLM DevOps Team
