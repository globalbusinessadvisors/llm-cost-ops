# LLM Cost Ops - Kubernetes Deployment Guide

This directory contains comprehensive Kubernetes manifests for deploying the LLM Cost Ops platform across different environments.

## Directory Structure

```
k8s/
├── base/                          # Base Kubernetes resources
│   ├── namespace.yaml             # Namespace definition
│   ├── serviceaccount.yaml        # Service accounts for app, database, monitoring
│   ├── rbac.yaml                  # Role-based access control
│   ├── configmap.yaml             # Application configuration
│   ├── secret.yaml                # Secrets (template with placeholders)
│   ├── deployment.yaml            # Main application deployment
│   ├── service.yaml               # Service definitions
│   ├── ingress.yaml               # Ingress configuration
│   ├── hpa.yaml                   # Horizontal Pod Autoscaler
│   ├── pdb.yaml                   # Pod Disruption Budget
│   ├── networkpolicy.yaml         # Network policies
│   └── servicemonitor.yaml        # Prometheus ServiceMonitor
│
├── database/                      # PostgreSQL database components
│   ├── postgres-configmap.yaml    # PostgreSQL configuration
│   ├── postgres-pvc.yaml          # Persistent volume claims
│   ├── postgres-statefulset.yaml  # PostgreSQL StatefulSet
│   └── postgres-service.yaml      # Database services
│
├── monitoring/                    # Monitoring stack
│   ├── prometheus-configmap.yaml  # Prometheus configuration & alerts
│   ├── prometheus-deployment.yaml # Prometheus deployment
│   ├── prometheus-service.yaml    # Prometheus service
│   ├── grafana-deployment.yaml    # Grafana deployment with dashboards
│   └── grafana-service.yaml       # Grafana service
│
├── overlays/                      # Environment-specific configurations
│   ├── dev/                       # Development environment
│   │   └── kustomization.yaml
│   ├── staging/                   # Staging environment
│   │   └── kustomization.yaml
│   └── prod/                      # Production environment
│       └── kustomization.yaml
│
└── helm/                          # Helm charts (alternative deployment method)
```

## Prerequisites

- Kubernetes cluster (v1.24+)
- kubectl CLI tool
- kustomize (v4.0+) or kubectl with kustomize support
- Persistent volume provisioner
- Ingress controller (nginx, traefik, etc.)
- Optional: Helm 3+ for Helm-based deployment

## Quick Start

### Deploy to Development

```bash
# Deploy using kubectl with kustomize
kubectl apply -k k8s/overlays/dev/

# Or using kustomize standalone
kustomize build k8s/overlays/dev/ | kubectl apply -f -

# Verify deployment
kubectl get all -n llm-cost-ops-dev
```

### Deploy to Staging

```bash
kubectl apply -k k8s/overlays/staging/
kubectl get all -n llm-cost-ops-staging
```

### Deploy to Production

```bash
# IMPORTANT: Update secrets before deploying to production
kubectl apply -k k8s/overlays/prod/
kubectl get all -n llm-cost-ops-prod
```

## Environment Configurations

### Development

- **Namespace**: `llm-cost-ops-dev`
- **Replicas**: 1 API pod
- **Resources**: Minimal (256Mi-1Gi RAM, 250m-1000m CPU)
- **Database**: 10Gi storage, 256Mi-512Mi RAM
- **Monitoring**: Optional (commented out by default to save resources)
- **Scaling**: 1-3 pods via HPA

### Staging

- **Namespace**: `llm-cost-ops-staging`
- **Replicas**: 2 API pods
- **Resources**: Moderate (384Mi-1.5Gi RAM, 350m-1500m CPU)
- **Database**: 30Gi storage, 512Mi-1Gi RAM
- **Monitoring**: Full stack (Prometheus + Grafana)
- **Scaling**: 2-10 pods via HPA

### Production

- **Namespace**: `llm-cost-ops-prod`
- **Replicas**: 5 API pods
- **Resources**: High (1Gi-4Gi RAM, 1000m-4000m CPU)
- **Database**: 100Gi storage, 2Gi-4Gi RAM
- **Monitoring**: Full stack with extended retention
- **Scaling**: 5-50 pods via HPA
- **HA**: PodDisruptionBudget ensures 3+ pods always available

## Configuration Management

### ConfigMaps

Application configuration is managed via ConfigMaps. Each environment has its own settings:

```yaml
# Example: Development
ENVIRONMENT=development
LOG_LEVEL=debug
DATABASE_POOL_SIZE=5

# Example: Production
ENVIRONMENT=production
LOG_LEVEL=warn
DATABASE_POOL_SIZE=20
```

### Secrets Management

**IMPORTANT**: The manifests include placeholder secrets for development only.

For **staging** and **production**, use one of these approaches:

#### Option 1: Sealed Secrets (Recommended)

```bash
# Install sealed-secrets controller
kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/controller.yaml

# Create a secret
kubectl create secret generic llm-cost-ops-secrets \
  --from-literal=DATABASE_URL=postgresql://user:pass@postgres:5432/db \
  --from-literal=database-password=securepassword \
  --dry-run=client -o yaml | \
  kubeseal -o yaml > sealed-secret.yaml

# Apply sealed secret
kubectl apply -f sealed-secret.yaml
```

#### Option 2: External Secrets Operator

```bash
# Install external-secrets
helm repo add external-secrets https://charts.external-secrets.io
helm install external-secrets external-secrets/external-secrets -n external-secrets-system --create-namespace

# Configure SecretStore for your provider (AWS Secrets Manager, GCP Secret Manager, Vault, etc.)
```

#### Option 3: Manual Secret Creation

```bash
# Create secrets manually (not recommended for production)
kubectl create secret generic llm-cost-ops-secrets \
  --from-literal=DATABASE_URL=postgresql://user:pass@postgres:5432/db \
  --from-literal=database-password=securepassword \
  --from-literal=grafana-admin-user=admin \
  --from-literal=grafana-admin-password=securepassword \
  -n llm-cost-ops-prod
```

## Database

### PostgreSQL StatefulSet

The database uses a StatefulSet with:

- Persistent storage via PVCs
- PostgreSQL 16 Alpine
- Optimized configuration for OLTP workloads
- PostgreSQL Exporter for metrics
- Backup volume mount

### Database Backups

```bash
# Manual backup
kubectl exec -n llm-cost-ops-prod postgres-0 -- \
  pg_dump -U postgres llm_cost_ops > backup.sql

# Automated backups (setup CronJob)
# See k8s/jobs/backup-cronjob.yaml (create if needed)
```

### Database Migrations

Migrations run automatically via init container on deployment:

```yaml
initContainers:
- name: migrate
  image: llm-cost-ops:latest
  command: ["sqlx", "migrate", "run"]
```

## Monitoring

### Prometheus

- Scrapes metrics from API pods, database, and Kubernetes
- Stores data for 30 days
- Includes predefined alerts for common issues
- Accessible at: `http://prometheus.<namespace>.svc.cluster.local:9090`

### Grafana

- Pre-configured with Prometheus datasource
- Includes LLM Cost Ops dashboard
- Accessible at: `http://grafana.<namespace>.svc.cluster.local:3000`
- Default credentials: admin/admin (change in production!)

### Custom Metrics

The application exposes Prometheus metrics at `/metrics`:

- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request latency
- `llm_cost_ops_*` - Application-specific metrics

## Scaling

### Horizontal Pod Autoscaling

HPA automatically scales based on CPU and memory:

```yaml
# Production example
minReplicas: 5
maxReplicas: 50
targetCPUUtilizationPercentage: 70
targetMemoryUtilizationPercentage: 80
```

### Manual Scaling

```bash
# Scale deployment manually
kubectl scale deployment llm-cost-ops --replicas=10 -n llm-cost-ops-prod
```

## Security

### Network Policies

Network policies restrict traffic:

- API pods can access database
- Database only accepts connections from API pods
- Monitoring can scrape all pods
- External traffic only via Ingress

### Security Context

All pods run with:

- Non-root user
- Read-only root filesystem (where possible)
- Dropped capabilities
- seccomp profile

### RBAC

Service accounts with minimal required permissions:

- `llm-cost-ops` - Main application
- `llm-cost-ops-postgres` - Database
- `llm-cost-ops-monitoring` - Prometheus/Grafana

## High Availability

### Pod Anti-Affinity

API pods are spread across nodes:

```yaml
podAntiAffinity:
  preferredDuringSchedulingIgnoredDuringExecution:
  - podAffinityTerm:
      topologyKey: kubernetes.io/hostname
```

### Pod Disruption Budget

In production, minimum 3 pods must be available during disruptions:

```yaml
minAvailable: 3
```

## Ingress

Configure ingress in `base/ingress.yaml`:

```yaml
# Update host and TLS settings
spec:
  rules:
  - host: llm-cost-ops.example.com
    http:
      paths:
      - path: /
        backend:
          service:
            name: llm-cost-ops
            port: 8080
  tls:
  - hosts:
    - llm-cost-ops.example.com
    secretName: llm-cost-ops-tls
```

## Troubleshooting

### Check Pod Status

```bash
kubectl get pods -n llm-cost-ops-prod
kubectl describe pod <pod-name> -n llm-cost-ops-prod
```

### View Logs

```bash
# Application logs
kubectl logs -n llm-cost-ops-prod deployment/llm-cost-ops -f

# Database logs
kubectl logs -n llm-cost-ops-prod postgres-0 -f

# Init container logs
kubectl logs -n llm-cost-ops-prod <pod-name> -c migrate
```

### Database Connection Issues

```bash
# Test database connectivity
kubectl run -it --rm debug --image=postgres:16-alpine --restart=Never -n llm-cost-ops-prod -- \
  psql -h postgres -U postgres -d llm_cost_ops
```

### Check Metrics

```bash
# Port-forward Prometheus
kubectl port-forward -n llm-cost-ops-prod svc/prometheus 9090:9090

# Visit http://localhost:9090
```

### Check Resource Usage

```bash
kubectl top pods -n llm-cost-ops-prod
kubectl top nodes
```

## Cleanup

### Remove Environment

```bash
# Development
kubectl delete -k k8s/overlays/dev/

# Staging
kubectl delete -k k8s/overlays/staging/

# Production (BE CAREFUL!)
kubectl delete -k k8s/overlays/prod/
```

### Remove PVCs (Data Loss Warning!)

```bash
# List PVCs
kubectl get pvc -n llm-cost-ops-prod

# Delete specific PVC
kubectl delete pvc <pvc-name> -n llm-cost-ops-prod
```

## Advanced Topics

### Custom Resource Requests

Override in your kustomization overlay:

```yaml
patches:
- patch: |-
    - op: replace
      path: /spec/template/spec/containers/0/resources/requests/memory
      value: "2Gi"
  target:
    kind: Deployment
    name: llm-cost-ops
```

### External Database

To use an external database instead of the in-cluster PostgreSQL:

1. Comment out database resources in overlay kustomization
2. Update DATABASE_URL secret to point to external instance
3. Ensure network connectivity and security groups

### CI/CD Integration

```bash
# Example GitLab CI/CD
deploy:
  stage: deploy
  script:
    - kubectl apply -k k8s/overlays/prod/
  only:
    - main
```

## Support

For issues or questions:

- Check logs: `kubectl logs -n <namespace> <pod-name>`
- Review events: `kubectl get events -n <namespace>`
- Consult documentation: See main README.md
- Open an issue: GitHub repository

## Best Practices

1. **Always use version tags** in production (not `latest`)
2. **Use external secret management** for sensitive data
3. **Enable monitoring** in staging and production
4. **Set up automated backups** for the database
5. **Configure resource limits** appropriate to your workload
6. **Use network policies** to restrict traffic
7. **Enable RBAC** and follow least privilege principle
8. **Test in staging** before deploying to production
9. **Use PodDisruptionBudgets** for high availability
10. **Monitor resource usage** and adjust limits accordingly

## License

Apache License 2.0 - See LICENSE file for details
