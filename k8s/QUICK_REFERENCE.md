# Kubernetes Quick Reference Guide

## Common Commands

### Deployment

```bash
# Deploy to development
kubectl apply -k k8s/overlays/dev/

# Deploy to staging
kubectl apply -k k8s/overlays/staging/

# Deploy to production
kubectl apply -k k8s/overlays/prod/
```

### Validation

```bash
# Validate all manifests
./k8s/validate.sh

# Preview what will be deployed (dry-run)
kubectl kustomize k8s/overlays/dev/
kubectl kustomize k8s/overlays/staging/
kubectl kustomize k8s/overlays/prod/
```

### Viewing Resources

```bash
# List all resources in a namespace
kubectl get all -n llm-cost-ops-dev
kubectl get all -n llm-cost-ops-staging
kubectl get all -n llm-cost-ops-prod

# Get pods
kubectl get pods -n llm-cost-ops-prod

# Get deployments
kubectl get deployments -n llm-cost-ops-prod

# Get services
kubectl get svc -n llm-cost-ops-prod

# Get persistent volumes
kubectl get pvc -n llm-cost-ops-prod
```

### Logs

```bash
# View application logs
kubectl logs -n llm-cost-ops-prod deployment/llm-cost-ops -f

# View specific pod logs
kubectl logs -n llm-cost-ops-prod <pod-name> -f

# View database logs
kubectl logs -n llm-cost-ops-prod postgres-0 -f

# View init container logs
kubectl logs -n llm-cost-ops-prod <pod-name> -c migrate
```

### Debugging

```bash
# Describe a pod
kubectl describe pod <pod-name> -n llm-cost-ops-prod

# Get pod events
kubectl get events -n llm-cost-ops-prod --sort-by='.lastTimestamp'

# Execute commands in a pod
kubectl exec -it -n llm-cost-ops-prod <pod-name> -- /bin/sh

# Port forward to a service
kubectl port-forward -n llm-cost-ops-prod svc/llm-cost-ops 8080:8080
kubectl port-forward -n llm-cost-ops-prod svc/prometheus 9090:9090
kubectl port-forward -n llm-cost-ops-prod svc/grafana 3000:3000
```

### Scaling

```bash
# Manual scaling
kubectl scale deployment llm-cost-ops --replicas=5 -n llm-cost-ops-prod

# View HPA status
kubectl get hpa -n llm-cost-ops-prod

# Describe HPA
kubectl describe hpa llm-cost-ops -n llm-cost-ops-prod
```

### Updates

```bash
# Update image
kubectl set image deployment/llm-cost-ops llm-cost-ops=your-registry/llm-cost-ops:v1.2.0 -n llm-cost-ops-prod

# Rollout status
kubectl rollout status deployment/llm-cost-ops -n llm-cost-ops-prod

# Rollout history
kubectl rollout history deployment/llm-cost-ops -n llm-cost-ops-prod

# Rollback
kubectl rollout undo deployment/llm-cost-ops -n llm-cost-ops-prod
```

### Resource Usage

```bash
# View resource usage
kubectl top pods -n llm-cost-ops-prod
kubectl top nodes

# Describe resource quotas
kubectl describe resourcequota -n llm-cost-ops-prod
```

### Secrets

```bash
# Create secret
kubectl create secret generic llm-cost-ops-secrets \
  --from-literal=DATABASE_URL=postgresql://user:pass@postgres:5432/db \
  -n llm-cost-ops-prod

# View secrets (base64 encoded)
kubectl get secret llm-cost-ops-secrets -n llm-cost-ops-prod -o yaml

# Decode secret
kubectl get secret llm-cost-ops-secrets -n llm-cost-ops-prod -o jsonpath='{.data.DATABASE_URL}' | base64 -d
```

### Database

```bash
# Connect to database
kubectl exec -it -n llm-cost-ops-prod postgres-0 -- psql -U postgres -d llm_cost_ops

# Backup database
kubectl exec -n llm-cost-ops-prod postgres-0 -- pg_dump -U postgres llm_cost_ops > backup.sql

# Restore database
cat backup.sql | kubectl exec -i -n llm-cost-ops-prod postgres-0 -- psql -U postgres llm_cost_ops
```

### Monitoring

```bash
# Access Prometheus
kubectl port-forward -n llm-cost-ops-prod svc/prometheus 9090:9090
# Then visit http://localhost:9090

# Access Grafana
kubectl port-forward -n llm-cost-ops-prod svc/grafana 3000:3000
# Then visit http://localhost:3000
# Default: admin/admin
```

### Cleanup

```bash
# Delete specific environment
kubectl delete -k k8s/overlays/dev/
kubectl delete -k k8s/overlays/staging/
kubectl delete -k k8s/overlays/prod/

# Delete namespace (will delete all resources)
kubectl delete namespace llm-cost-ops-dev
kubectl delete namespace llm-cost-ops-staging
kubectl delete namespace llm-cost-ops-prod
```

## Environment Details

| Environment | Namespace | Replicas | Resources | Storage | HPA Range |
|------------|-----------|----------|-----------|---------|-----------|
| Development | llm-cost-ops-dev | 1 | 256Mi-1Gi / 0.25-1 CPU | 10Gi | 1-3 |
| Staging | llm-cost-ops-staging | 2 | 384Mi-1.5Gi / 0.35-1.5 CPU | 30Gi | 2-10 |
| Production | llm-cost-ops-prod | 5 | 1Gi-4Gi / 1-4 CPU | 100Gi | 5-50 |

## Service Endpoints

### Development
- API: `http://dev-llm-cost-ops.llm-cost-ops-dev.svc.cluster.local:8080`
- Metrics: `http://dev-llm-cost-ops.llm-cost-ops-dev.svc.cluster.local:9090`
- Database: `postgres://dev-postgres.llm-cost-ops-dev.svc.cluster.local:5432`

### Staging
- API: `http://staging-llm-cost-ops.llm-cost-ops-staging.svc.cluster.local:8080`
- Metrics: `http://staging-llm-cost-ops.llm-cost-ops-staging.svc.cluster.local:9090`
- Database: `postgres://staging-postgres.llm-cost-ops-staging.svc.cluster.local:5432`
- Prometheus: `http://staging-prometheus.llm-cost-ops-staging.svc.cluster.local:9090`
- Grafana: `http://staging-grafana.llm-cost-ops-staging.svc.cluster.local:3000`

### Production
- API: `http://prod-llm-cost-ops.llm-cost-ops-prod.svc.cluster.local:8080`
- Metrics: `http://prod-llm-cost-ops.llm-cost-ops-prod.svc.cluster.local:9090`
- Database: `postgres://prod-postgres.llm-cost-ops-prod.svc.cluster.local:5432`
- Prometheus: `http://prod-prometheus.llm-cost-ops-prod.svc.cluster.local:9090`
- Grafana: `http://prod-grafana.llm-cost-ops-prod.svc.cluster.local:3000`

## Health Check Endpoints

```bash
# API Health
curl http://<service-endpoint>:8080/health

# API Readiness
curl http://<service-endpoint>:8080/ready

# API Liveness
curl http://<service-endpoint>:8080/live

# Metrics
curl http://<service-endpoint>:8080/metrics
```

## Troubleshooting Quick Checks

```bash
# Check pod status
kubectl get pods -n llm-cost-ops-prod

# Check pod details
kubectl describe pod <pod-name> -n llm-cost-ops-prod

# Check events
kubectl get events -n llm-cost-ops-prod --sort-by='.lastTimestamp'

# Check logs
kubectl logs -n llm-cost-ops-prod <pod-name> --previous

# Check resource usage
kubectl top pods -n llm-cost-ops-prod

# Check HPA
kubectl get hpa -n llm-cost-ops-prod

# Check network policies
kubectl get networkpolicies -n llm-cost-ops-prod

# Test database connectivity
kubectl run -it --rm debug --image=postgres:16-alpine --restart=Never -n llm-cost-ops-prod -- \
  psql -h postgres -U postgres -d llm_cost_ops
```

## Common Issues

### Pod Stuck in Pending
```bash
# Check events
kubectl describe pod <pod-name> -n llm-cost-ops-prod

# Common causes:
# - Insufficient resources
# - PVC not bound
# - Node selector/affinity issues
```

### CrashLoopBackOff
```bash
# Check logs
kubectl logs <pod-name> -n llm-cost-ops-prod --previous

# Common causes:
# - Application errors
# - Missing environment variables
# - Database connection issues
```

### ImagePullBackOff
```bash
# Check events
kubectl describe pod <pod-name> -n llm-cost-ops-prod

# Common causes:
# - Wrong image name/tag
# - Missing image pull secrets
# - Registry authentication issues
```

## Documentation

- **Full Guide**: `KUBERNETES_DEPLOYMENT_GUIDE.md`
- **Summary**: `MANIFEST_SUMMARY.md`
- **Validation**: Run `./validate.sh`
