# Kubernetes Deployment Guide

## Quick Start

### Deploy to Development
```bash
kubectl apply -k k8s/overlays/dev/
```

### Deploy to Production with Helm
```bash
helm install llm-cost-ops ./k8s/helm/llm-cost-ops \
  --namespace llm-cost-ops-prod \
  --create-namespace \
  --values ./k8s/helm/llm-cost-ops/values-prod.yaml
```

## Architecture Overview

### High Availability
- **Minimum 3 replicas** in production (configurable via HPA)
- **Pod Disruption Budget** ensures at least 2 pods available during updates
- **Rolling updates** with zero downtime (maxUnavailable: 0)
- **Anti-affinity rules** spread pods across nodes

### Security
- **Non-root containers** (UID 1000)
- **Read-only filesystem**
- **Dropped capabilities** (ALL)
- **Network policies** restrict ingress/egress
- **Seccomp profile** (RuntimeDefault)
- **RBAC** with least-privilege access

### Observability
- **Prometheus metrics** exposed on port 9090
- **ServiceMonitor** for automatic scraping
- **Health endpoints**: /health, /ready, /live
- **Structured logging** in JSON format
- **Distributed tracing** ready (OTLP)

### Scalability
- **Horizontal autoscaling** based on CPU, memory, and custom metrics
- **Resource limits** prevent resource exhaustion
- **Connection pooling** for database
- **Redis caching** layer

## Deployment Checklist

### Prerequisites
- [ ] Kubernetes cluster (v1.25+)
- [ ] NGINX Ingress Controller
- [ ] cert-manager for TLS
- [ ] Prometheus Operator
- [ ] Metrics Server for HPA

### Configuration
- [ ] Update domain in ingress.yaml
- [ ] Configure database credentials in secrets
- [ ] Set JWT secret
- [ ] Configure OTLP endpoint (if using)
- [ ] Set resource limits based on load testing

### Post-Deployment
- [ ] Verify all pods are running
- [ ] Check health endpoints
- [ ] Verify metrics are being scraped
- [ ] Test autoscaling
- [ ] Verify TLS certificates
- [ ] Test network policies

## Monitoring

### Key Metrics
- `llm_cost_ops_http_requests_total` - Total HTTP requests
- `llm_cost_ops_http_request_duration_seconds` - Request latency
- `llm_cost_ops_cost_calculations_total` - Cost calculations performed
- `llm_cost_ops_db_queries_total` - Database query count
- `llm_cost_ops_cache_hits_total` - Cache hit rate

### Alerts (Prometheus)
```yaml
groups:
- name: llm-cost-ops
  rules:
  - alert: HighErrorRate
    expr: rate(llm_cost_ops_http_requests_total{status=~"5.."}[5m]) > 0.05
    for: 5m
    annotations:
      summary: "High error rate detected"
  
  - alert: HighLatency
    expr: histogram_quantile(0.95, llm_cost_ops_http_request_duration_seconds) > 1
    for: 5m
    annotations:
      summary: "95th percentile latency > 1s"
```

## Troubleshooting

### Common Issues

**Pods not starting**
```bash
kubectl describe pod <pod-name>
kubectl logs <pod-name> --previous
```

**Network connectivity issues**
```bash
kubectl get networkpolicies
kubectl describe networkpolicy llm-cost-ops
```

**Autoscaling not working**
```bash
kubectl get hpa
kubectl describe hpa llm-cost-ops
kubectl top pods
```

**Certificate issues**
```bash
kubectl get certificate
kubectl describe certificate llm-cost-ops-tls
```

## Performance Tuning

### Database
- Tune connection pool size based on load
- Enable query caching
- Use read replicas for analytics

### Application
- Adjust worker threads: `TOKIO_WORKER_THREADS`
- Tune rate limits
- Configure cache TTL appropriately

### Kubernetes
- Right-size resource requests/limits
- Tune HPA thresholds
- Adjust PDB based on cluster size

## Security Hardening

### Secrets Management
Use external secret management:
```bash
# Example with External Secrets Operator
kubectl apply -f - <<EOF
apiVersion: external-secrets.io/v1beta1
kind: SecretStore
metadata:
  name: aws-secrets-manager
spec:
  provider:
    aws:
      service: SecretsManager
      region: us-west-2
---
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: llm-cost-ops-secrets
spec:
  secretStoreRef:
    name: aws-secrets-manager
  target:
    name: llm-cost-ops-secrets
  data:
  - secretKey: DATABASE_URL
    remoteRef:
      key: llm-cost-ops/database-url
