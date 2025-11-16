# LLM Cost Ops Helm Chart

A comprehensive Helm chart for deploying the LLM Cost Operations Platform on Kubernetes.

## Overview

LLM Cost Ops is a cost tracking, monitoring, and optimization platform for Large Language Model (LLM) deployments. This Helm chart provides a complete, production-ready deployment with support for:

- Multiple database backends (PostgreSQL, SQLite)
- Horizontal pod autoscaling
- Prometheus metrics integration
- Distributed tracing (Jaeger, OTLP)
- Event streaming (NATS, Redis)
- Rate limiting and authentication
- Network policies and RBAC
- Pod disruption budgets
- Comprehensive health checks

## Prerequisites

- Kubernetes 1.24+
- Helm 3.8+
- PV provisioner support in the underlying infrastructure (for persistence)
- Prometheus Operator (optional, for ServiceMonitor)

## Installation

### Add Helm Repository

```bash
helm repo add llm-cost-ops https://charts.llm-cost-ops.io
helm repo update
```

### Install Chart

```bash
# Install with default configuration (development)
helm install llm-cost-ops llm-cost-ops/llm-cost-ops

# Install with custom values file
helm install llm-cost-ops llm-cost-ops/llm-cost-ops -f values-prod.yaml

# Install with inline overrides
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --set image.tag=v0.1.0 \
  --set postgresql.enabled=true \
  --set autoscaling.enabled=true
```

### Install from Local Chart

```bash
# From the helm directory
helm install llm-cost-ops ./llm-cost-ops

# With environment-specific values
helm install llm-cost-ops ./llm-cost-ops -f ./llm-cost-ops/values-dev.yaml
```

## Configuration

### Environment-Specific Values Files

The chart includes pre-configured values files for different environments:

- `values.yaml` - Default configuration with comprehensive documentation
- `values-dev.yaml` - Development environment (minimal resources, SQLite)
- `values-staging.yaml` - Staging environment (moderate resources, PostgreSQL)
- `values-prod.yaml` - Production environment (high availability, full features)

### Key Configuration Options

#### Application Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |
| `image.registry` | Image registry | `docker.io` |
| `image.repository` | Image repository | `llm-cost-ops/llm-cost-ops` |
| `image.tag` | Image tag | `""` (uses appVersion) |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |

#### Database Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.database.type` | Database type (postgres, sqlite) | `postgres` |
| `config.database.host` | PostgreSQL host | `llm-cost-ops-postgres` |
| `config.database.port` | PostgreSQL port | `5432` |
| `config.database.name` | Database name | `llm_cost_ops` |
| `config.database.user` | Database user | `postgres` |
| `config.database.poolSize` | Connection pool size | `10` |
| `postgresql.enabled` | Enable PostgreSQL deployment | `true` |

#### API Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.api.host` | API bind address | `0.0.0.0` |
| `config.api.port` | API port | `8080` |
| `config.api.enableCors` | Enable CORS | `true` |
| `config.api.requestTimeoutSecs` | Request timeout | `30` |

#### Authentication

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.auth.enabled` | Enable authentication | `true` |
| `config.auth.jwt.secret` | JWT secret | `change-me-in-production` |
| `config.auth.jwt.existingSecret` | Use existing secret for JWT | `""` |
| `config.auth.rbac.enabled` | Enable RBAC | `true` |

#### Autoscaling

| Parameter | Description | Default |
|-----------|-------------|---------|
| `autoscaling.enabled` | Enable HPA | `false` |
| `autoscaling.minReplicas` | Minimum replicas | `1` |
| `autoscaling.maxReplicas` | Maximum replicas | `10` |
| `autoscaling.targetCPUUtilizationPercentage` | Target CPU | `80` |

#### Ingress

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable ingress | `false` |
| `ingress.className` | Ingress class | `nginx` |
| `ingress.hosts` | Ingress hosts | `[llm-cost-ops.local]` |
| `ingress.tls` | TLS configuration | `[]` |

#### Monitoring

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.metrics.enabled` | Enable Prometheus metrics | `true` |
| `config.metrics.port` | Metrics port | `9090` |
| `serviceMonitor.enabled` | Enable ServiceMonitor | `false` |
| `config.tracing.enabled` | Enable distributed tracing | `false` |

#### Resources

| Parameter | Description | Default |
|-----------|-------------|---------|
| `resources.limits.cpu` | CPU limit | `1000m` |
| `resources.limits.memory` | Memory limit | `1Gi` |
| `resources.requests.cpu` | CPU request | `250m` |
| `resources.requests.memory` | Memory request | `256Mi` |

### Complete Configuration

See [values.yaml](values.yaml) for all available configuration options with detailed comments.

## Deployment Examples

### Development Deployment

```bash
# Minimal deployment with SQLite
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-dev.yaml
```

### Staging Deployment

```bash
# Staging with PostgreSQL and Redis
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-staging.yaml \
  --set ingress.hosts[0].host=staging.example.com
```

### Production Deployment

```bash
# High-availability production deployment
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set ingress.hosts[0].host=llm-cost-ops.example.com \
  --set config.auth.jwt.existingSecret=llm-cost-ops-jwt \
  --set config.database.existingSecret=llm-cost-ops-db
```

### With External Database

```bash
# Use external PostgreSQL
helm install llm-cost-ops ./llm-cost-ops \
  --set postgresql.enabled=false \
  --set config.database.host=postgres.example.com \
  --set config.database.port=5432 \
  --set config.database.existingSecret=db-credentials
```

## Upgrade

```bash
# Upgrade with new values
helm upgrade llm-cost-ops ./llm-cost-ops -f values-prod.yaml

# Upgrade with specific overrides
helm upgrade llm-cost-ops ./llm-cost-ops \
  --set image.tag=v0.2.0 \
  --reuse-values
```

## Uninstall

```bash
# Uninstall the chart
helm uninstall llm-cost-ops

# Uninstall and delete PVCs
helm uninstall llm-cost-ops
kubectl delete pvc -l app.kubernetes.io/instance=llm-cost-ops
```

## Testing

```bash
# Run helm tests
helm test llm-cost-ops

# Check deployment status
kubectl get all -l app.kubernetes.io/instance=llm-cost-ops

# View logs
kubectl logs -f -l app.kubernetes.io/name=llm-cost-ops
```

## Security Considerations

### Production Security Checklist

- [ ] Use external secrets management (Vault, Sealed Secrets, External Secrets Operator)
- [ ] Enable TLS for ingress
- [ ] Use strong JWT secrets
- [ ] Enable network policies
- [ ] Configure RBAC appropriately
- [ ] Use read-only root filesystem
- [ ] Enable pod security standards
- [ ] Use separate database credentials
- [ ] Enable audit logging
- [ ] Configure resource limits
- [ ] Use private image registry
- [ ] Enable image scanning
- [ ] Review and customize security contexts

### Using External Secrets

```bash
# Create Kubernetes secret for JWT
kubectl create secret generic llm-cost-ops-jwt \
  --from-literal=jwt-secret=$(openssl rand -base64 32)

# Create secret for database
kubectl create secret generic llm-cost-ops-db \
  --from-literal=password=$(openssl rand -base64 32)

# Install with external secrets
helm install llm-cost-ops ./llm-cost-ops \
  --set config.auth.jwt.existingSecret=llm-cost-ops-jwt \
  --set config.database.existingSecret=llm-cost-ops-db
```

## Troubleshooting

### Common Issues

#### Pods not starting

```bash
# Check pod status
kubectl get pods -l app.kubernetes.io/name=llm-cost-ops

# View pod events
kubectl describe pod <pod-name>

# Check logs
kubectl logs <pod-name>
```

#### Database connection issues

```bash
# Test database connectivity
kubectl run -it --rm debug --image=postgres:16-alpine --restart=Never -- \
  psql -h llm-cost-ops-postgres -U postgres -d llm_cost_ops

# Check database pod
kubectl get pods -l app.kubernetes.io/name=postgresql
kubectl logs -l app.kubernetes.io/name=postgresql
```

#### Ingress not working

```bash
# Check ingress status
kubectl get ingress
kubectl describe ingress llm-cost-ops

# Verify ingress controller
kubectl get pods -n ingress-nginx
```

### Debug Mode

```bash
# Install with debug logging
helm install llm-cost-ops ./llm-cost-ops \
  --set config.logging.level=debug \
  --set env[0].name=RUST_BACKTRACE \
  --set env[0].value=full
```

## Monitoring

### Prometheus Integration

```bash
# Enable ServiceMonitor
helm install llm-cost-ops ./llm-cost-ops \
  --set serviceMonitor.enabled=true \
  --set serviceMonitor.labels.release=prometheus
```

### Grafana Dashboard

Import the included Grafana dashboard from `dashboards/llm-cost-ops.json`.

### Key Metrics

- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request duration
- `cost_calculations_total` - Total cost calculations
- `database_connections` - Active database connections
- `dlq_messages_total` - Dead letter queue messages

## Development

### Local Testing

```bash
# Lint the chart
helm lint ./llm-cost-ops

# Template the chart
helm template llm-cost-ops ./llm-cost-ops -f ./llm-cost-ops/values-dev.yaml

# Dry run installation
helm install llm-cost-ops ./llm-cost-ops --dry-run --debug
```

### Chart Structure

```
llm-cost-ops/
├── Chart.yaml                 # Chart metadata
├── values.yaml                # Default values
├── values-dev.yaml           # Development values
├── values-staging.yaml       # Staging values
├── values-prod.yaml          # Production values
├── templates/
│   ├── _helpers.tpl          # Template helpers
│   ├── deployment.yaml       # Main deployment
│   ├── service.yaml          # Service
│   ├── ingress.yaml          # Ingress
│   ├── configmap.yaml        # Configuration
│   ├── secret.yaml           # Secrets
│   ├── serviceaccount.yaml   # Service account
│   ├── rbac.yaml            # RBAC resources
│   ├── hpa.yaml             # Horizontal Pod Autoscaler
│   ├── pdb.yaml             # Pod Disruption Budget
│   ├── networkpolicy.yaml   # Network policies
│   ├── servicemonitor.yaml  # Prometheus ServiceMonitor
│   ├── statefulset.yaml     # PVC for persistence
│   ├── database/
│   │   ├── postgres-statefulset.yaml
│   │   ├── postgres-service.yaml
│   │   └── postgres-pvc.yaml
│   └── tests/
│       └── test-connection.yaml
└── README.md
```

## Support

- GitHub Issues: https://github.com/llm-devops/llm-cost-ops/issues
- Documentation: https://llm-cost-ops.io/docs
- Email: support@llm-cost-ops.io

## License

Apache License 2.0 - see LICENSE for details.
