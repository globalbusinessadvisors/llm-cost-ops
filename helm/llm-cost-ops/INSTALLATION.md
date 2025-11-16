# LLM Cost Ops - Quick Installation Guide

## Quick Start

### 1. Development (Local/Testing)

```bash
# Install with SQLite and minimal resources
helm install llm-cost-ops ./llm-cost-ops -f ./llm-cost-ops/values-dev.yaml

# Access the application
kubectl port-forward svc/llm-cost-ops 8080:8080
# Visit http://localhost:8080
```

### 2. Staging Environment

```bash
# Create namespace
kubectl create namespace llm-cost-ops-staging

# Create secrets
kubectl create secret generic llm-cost-ops-db-secret \
  --from-literal=password=$(openssl rand -base64 32) \
  -n llm-cost-ops-staging

kubectl create secret generic llm-cost-ops-jwt-secret \
  --from-literal=jwt-secret=$(openssl rand -base64 32) \
  -n llm-cost-ops-staging

kubectl create secret generic llm-cost-ops-redis-secret \
  --from-literal=password=$(openssl rand -base64 32) \
  -n llm-cost-ops-staging

# Install chart
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-staging.yaml \
  --set ingress.hosts[0].host=staging.yourdomain.com \
  -n llm-cost-ops-staging
```

### 3. Production Environment

```bash
# Create namespace
kubectl create namespace llm-cost-ops

# Create secrets
kubectl create secret generic llm-cost-ops-db-secret \
  --from-literal=password=$(openssl rand -base64 32) \
  -n llm-cost-ops

kubectl create secret generic llm-cost-ops-jwt-secret \
  --from-literal=jwt-secret=$(openssl rand -base64 32) \
  -n llm-cost-ops

kubectl create secret generic llm-cost-ops-redis-secret \
  --from-literal=password=$(openssl rand -base64 32) \
  -n llm-cost-ops

kubectl create secret generic llm-cost-ops-smtp-secret \
  --from-literal=smtp-password=your-smtp-password \
  -n llm-cost-ops

# Install chart
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set ingress.hosts[0].host=llm-cost-ops.yourdomain.com \
  --set config.export.email.smtp.host=smtp.yourprovider.com \
  --set config.export.email.smtp.user=noreply@yourdomain.com \
  -n llm-cost-ops
```

## Verification

```bash
# Check pod status
kubectl get pods -n llm-cost-ops

# View logs
kubectl logs -f -l app.kubernetes.io/name=llm-cost-ops -n llm-cost-ops

# Run tests
helm test llm-cost-ops -n llm-cost-ops

# Check all resources
kubectl get all -l app.kubernetes.io/instance=llm-cost-ops -n llm-cost-ops
```

## Common Configurations

### Using External PostgreSQL

```bash
helm install llm-cost-ops ./llm-cost-ops \
  --set postgresql.enabled=false \
  --set config.database.host=external-postgres.example.com \
  --set config.database.port=5432 \
  --set config.database.name=llm_cost_ops \
  --set config.database.user=llmcostops \
  --set config.database.existingSecret=external-db-secret
```

### Enabling NATS Streaming

```bash
helm install llm-cost-ops ./llm-cost-ops \
  --set nats.enabled=true \
  --set config.streaming.enabled=true \
  --set config.streaming.backend=nats
```

### Enabling Tracing with Jaeger

```bash
helm install llm-cost-ops ./llm-cost-ops \
  --set config.tracing.enabled=true \
  --set config.tracing.backend=jaeger \
  --set config.tracing.jaeger.endpoint=http://jaeger-collector.monitoring:14268/api/traces
```

### High Availability Setup

```bash
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set replicaCount=3 \
  --set autoscaling.enabled=true \
  --set autoscaling.minReplicas=3 \
  --set autoscaling.maxReplicas=20 \
  --set podDisruptionBudget.enabled=true \
  --set podDisruptionBudget.minAvailable=2
```

## Upgrade

```bash
# Upgrade to new version
helm upgrade llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set image.tag=v0.2.0

# Rollback if needed
helm rollback llm-cost-ops
```

## Troubleshooting

### Check Helm Release Status

```bash
helm status llm-cost-ops -n llm-cost-ops
helm get values llm-cost-ops -n llm-cost-ops
```

### View Rendered Templates

```bash
helm template llm-cost-ops ./llm-cost-ops -f ./llm-cost-ops/values-prod.yaml
```

### Debug Installation

```bash
helm install llm-cost-ops ./llm-cost-ops --dry-run --debug
```

## Uninstallation

```bash
# Uninstall release
helm uninstall llm-cost-ops -n llm-cost-ops

# Delete PVCs (if needed)
kubectl delete pvc -l app.kubernetes.io/instance=llm-cost-ops -n llm-cost-ops

# Delete namespace (if needed)
kubectl delete namespace llm-cost-ops
```
