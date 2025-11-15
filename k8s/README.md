# Kubernetes Deployment for LLM Cost Ops

This directory contains Kubernetes manifests and Helm charts for deploying the LLM Cost Ops platform.

## Directory Structure

```
k8s/
├── base/                    # Base Kubernetes manifests
│   ├── deployment.yaml      # Application deployment
│   ├── service.yaml         # Service definitions
│   ├── ingress.yaml         # Ingress configuration
│   ├── configmap.yaml       # Application configuration
│   ├── secret.yaml          # Secrets (template)
│   ├── hpa.yaml             # Horizontal Pod Autoscaler
│   ├── pdb.yaml             # Pod Disruption Budget
│   ├── rbac.yaml            # RBAC configuration
│   ├── networkpolicy.yaml   # Network policies
│   ├── servicemonitor.yaml  # Prometheus ServiceMonitor
│   └── kustomization.yaml   # Kustomize base configuration
├── overlays/                # Environment-specific overlays
│   ├── dev/                 # Development environment
│   ├── staging/             # Staging environment
│   └── prod/                # Production environment
└── helm/                    # Helm charts
    └── llm-cost-ops/        # Main Helm chart
        ├── Chart.yaml       # Chart metadata
        ├── values.yaml      # Default values
        └── templates/       # Kubernetes templates

```

## Deployment Methods

### Method 1: Using Kustomize

#### Deploy to Development
```bash
kubectl apply -k k8s/overlays/dev/
```

#### Deploy to Staging
```bash
kubectl apply -k k8s/overlays/staging/
```

#### Deploy to Production
```bash
kubectl apply -k k8s/overlays/prod/
```

#### View rendered manifests without applying
```bash
kubectl kustomize k8s/overlays/prod/
```

### Method 2: Using Helm

#### Install the chart
```bash
helm install llm-cost-ops ./k8s/helm/llm-cost-ops \
  --namespace llm-cost-ops \
  --create-namespace \
  --values ./k8s/helm/llm-cost-ops/values.yaml
```

#### Upgrade the chart
```bash
helm upgrade llm-cost-ops ./k8s/helm/llm-cost-ops \
  --namespace llm-cost-ops \
  --values ./k8s/helm/llm-cost-ops/values.yaml
```

#### Uninstall the chart
```bash
helm uninstall llm-cost-ops --namespace llm-cost-ops
```

#### Dry run to see what would be deployed
```bash
helm install llm-cost-ops ./k8s/helm/llm-cost-ops \
  --dry-run --debug \
  --values ./k8s/helm/llm-cost-ops/values.yaml
```

## Prerequisites

### Required Components

1. **Kubernetes Cluster** (v1.25+)
   - EKS, GKE, AKS, or self-managed cluster
   - Cluster should have at least 3 nodes for high availability

2. **NGINX Ingress Controller**
   ```bash
   kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/cloud/deploy.yaml
   ```

3. **cert-manager** (for TLS certificates)
   ```bash
   kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml
   ```

4. **Prometheus Operator** (for monitoring)
   ```bash
   helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
   helm install kube-prometheus-stack prometheus-community/kube-prometheus-stack
   ```

5. **Metrics Server** (for HPA)
   ```bash
   kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml
   ```

### Optional Components

- **PostgreSQL** - External database or in-cluster deployment
- **Redis** - Caching layer
- **NATS** - Event streaming
- **OpenTelemetry Collector** - Distributed tracing

## Configuration

### Secrets Management

**WARNING**: The `secret.yaml` file contains example secrets. In production:

1. **Use External Secrets Operator**:
   ```bash
   helm repo add external-secrets https://charts.external-secrets.io
   helm install external-secrets external-secrets/external-secrets
   ```

2. **Use Sealed Secrets**:
   ```bash
   kubectl apply -f https://github.com/bitnami-labs/sealed-secrets/releases/download/v0.24.0/controller.yaml
   ```

3. **Use Cloud Provider Secret Management**:
   - AWS Secrets Manager
   - GCP Secret Manager
   - Azure Key Vault

### Update Secrets

Create a secret from file:
```bash
kubectl create secret generic llm-cost-ops-secrets \
  --from-env-file=.env.production \
  --namespace llm-cost-ops
```

Or manually:
```bash
kubectl create secret generic llm-cost-ops-secrets \
  --from-literal=DATABASE_URL="postgresql://..." \
  --from-literal=JWT_SECRET="your-secret-here" \
  --namespace llm-cost-ops
```

### Configure Ingress

Update `k8s/base/ingress.yaml` with your domain:
```yaml
hosts:
  - host: api.your-domain.com
```

### Resource Tuning

Adjust resources based on your workload in each overlay:

**Development** (1 replica):
- CPU: 250m request, 1000m limit
- Memory: 256Mi request, 1Gi limit

**Staging** (2 replicas):
- CPU: 350m request, 1500m limit
- Memory: 384Mi request, 1.5Gi limit

**Production** (5+ replicas):
- CPU: 1000m request, 4000m limit
- Memory: 1Gi request, 4Gi limit

## Monitoring

### Access Metrics

Prometheus metrics are exposed at:
- Port: 9090
- Path: /metrics

### ServiceMonitor

The ServiceMonitor resource is automatically created when Prometheus Operator is installed.

View metrics:
```bash
kubectl port-forward svc/llm-cost-ops 9090:9090
curl localhost:9090/metrics
```

### Grafana Dashboards

Import the included Grafana dashboard (coming soon).

## Health Checks

The application provides three health endpoints:

1. **Liveness**: `/live` - Is the application alive?
2. **Readiness**: `/ready` - Is the application ready to serve traffic?
3. **Health**: `/health` - Overall health status with component details

Test health endpoints:
```bash
kubectl port-forward svc/llm-cost-ops 8080:80
curl localhost:8080/health
curl localhost:8080/ready
curl localhost:8080/live
```

## Autoscaling

### Horizontal Pod Autoscaler

Automatically scales pods based on:
- CPU utilization (target: 70%)
- Memory utilization (target: 80%)
- Custom metrics (HTTP requests/sec)

View HPA status:
```bash
kubectl get hpa llm-cost-ops
```

### Manual Scaling

Scale deployment manually:
```bash
kubectl scale deployment llm-cost-ops --replicas=10
```

## Security

### Network Policies

Network policies are enabled by default and restrict:
- **Ingress**: Only from NGINX ingress and Prometheus
- **Egress**: Only to database, cache, and external APIs

### Pod Security

- Runs as non-root user (UID 1000)
- Read-only root filesystem
- Drops all capabilities
- Uses seccomp profile

### RBAC

Least-privilege RBAC configuration:
- Service account with minimal permissions
- Role limited to namespace
- ClusterRole only for metrics reading

## Troubleshooting

### View Logs
```bash
kubectl logs -f deployment/llm-cost-ops
kubectl logs -f deployment/llm-cost-ops --previous  # Previous container
```

### Describe Resources
```bash
kubectl describe deployment llm-cost-ops
kubectl describe pod <pod-name>
```

### Check Events
```bash
kubectl get events --sort-by='.lastTimestamp'
```

### Debug Network Policies
```bash
kubectl get networkpolicies
kubectl describe networkpolicy llm-cost-ops
```

### Access Pod Shell
```bash
kubectl exec -it deployment/llm-cost-ops -- /bin/sh
```

### Check Resource Usage
```bash
kubectl top pods
kubectl top nodes
```

## Backup and Disaster Recovery

### Database Backups

Configure automated PostgreSQL backups:
```bash
# Example using pg_dump
kubectl exec -it postgres-pod -- pg_dump -U llm_cost_ops > backup.sql
```

### Configuration Backups

Export all configurations:
```bash
kubectl get configmap,secret,deployment,service,ingress \
  -n llm-cost-ops \
  -o yaml > backup-$(date +%Y%m%d).yaml
```

## Migration

### Zero-Downtime Deployment

The configuration includes:
- Rolling update strategy (maxSurge: 1, maxUnavailable: 0)
- Pod Disruption Budget (minAvailable: 2)
- Readiness probes to prevent traffic to unhealthy pods

### Database Migrations

Migrations run automatically via init container before app starts.

Manual migration:
```bash
kubectl exec -it deployment/llm-cost-ops -- /app/cost-ops migrate
```

## Performance Tuning

### Connection Pooling

Configure in ConfigMap:
```yaml
DB_POOL_MAX_SIZE: "20"
DB_POOL_MIN_IDLE: "5"
DB_POOL_TIMEOUT_SECONDS: "30"
```

### Caching

Adjust cache settings:
```yaml
CACHE_TTL_SECONDS: "3600"
CACHE_MAX_SIZE_MB: "512"
```

### Rate Limiting

Tune rate limits:
```yaml
RATE_LIMIT_REQUESTS_PER_SECOND: "100"
RATE_LIMIT_BURST_SIZE: "200"
```

## Cost Optimization

1. **Right-size resources** based on actual usage
2. **Use spot instances** for non-critical workloads
3. **Enable cluster autoscaler** for node scaling
4. **Use pod disruption budgets** to allow cluster scale-down
5. **Monitor resource requests** vs. actual usage

## Support

For issues and questions:
- GitHub Issues: https://github.com/your-org/llm-cost-ops/issues
- Documentation: https://docs.llm-cost-ops.example.com
- Slack: #llm-cost-ops
