# Deployment Configuration

This directory contains all Docker and deployment-related configuration files for the LLM Cost Ops platform.

## Contents

### Docker Files

- **`Dockerfile`** - Production-ready multi-stage build
- **`Dockerfile.dev`** - Development environment with hot-reload

### Docker Compose Files

- **`docker-compose.yml`** - Local development stack
- **`docker-compose.prod.yml`** - Production deployment configuration
- **`docker-compose.test.yml`** - Testing environment
- **`docker-compose.override.yml`** - Local overrides (not tracked in production)

## Quick Start

### Development

```bash
# Start development environment
cd deployment
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Production

```bash
# Deploy production stack
docker-compose -f deployment/docker-compose.prod.yml up -d

# Scale services
docker-compose -f deployment/docker-compose.prod.yml up -d --scale api=3
```

### Testing

```bash
# Run test environment
docker-compose -f deployment/docker-compose.test.yml up -d

# Run tests
docker-compose -f deployment/docker-compose.test.yml exec api cargo test
```

## Building Images

### Production Image

```bash
# Build from project root
docker build -f deployment/Dockerfile -t llm-cost-ops:latest .

# With build args
docker build -f deployment/Dockerfile \
  --build-arg RUST_VERSION=1.91 \
  -t llm-cost-ops:v1.0.0 .
```

### Development Image

```bash
docker build -f deployment/Dockerfile.dev -t llm-cost-ops:dev .
```

## Environment Variables

See the main [README.md](../README.md#environment-variables) for required environment variables.

## Kubernetes Deployment

For Kubernetes deployments, see the [k8s/](../k8s/) directory.

## CI/CD Integration

GitHub Actions workflows automatically build and push Docker images on:
- Push to `main` branch
- Pull request creation
- Release tags

See [.github/workflows/deploy.yml](../.github/workflows/deploy.yml) for details.
