# LLM Cost Ops Helm Chart - Usage Examples

This document provides real-world examples for deploying LLM Cost Ops in different scenarios.

## Table of Contents

1. [Development Setup](#development-setup)
2. [CI/CD Integration](#cicd-integration)
3. [Multi-Tenancy](#multi-tenancy)
4. [High Availability](#high-availability)
5. [Cost Optimization](#cost-optimization)
6. [Monitoring & Observability](#monitoring--observability)
7. [Security Hardening](#security-hardening)
8. [Disaster Recovery](#disaster-recovery)

---

## Development Setup

### Local Kubernetes (minikube, kind, k3s)

```bash
# Start minikube with sufficient resources
minikube start --cpus=4 --memory=8192

# Install with dev values
helm install llm-cost-ops ./llm-cost-ops -f ./llm-cost-ops/values-dev.yaml

# Port forward for local access
kubectl port-forward svc/llm-cost-ops 8080:8080

# Access at http://localhost:8080
```

### Development with Hot Reload

```bash
# Use latest dev image
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-dev.yaml \
  --set image.tag=dev \
  --set image.pullPolicy=Always \
  --set config.logging.level=debug
```

---

## CI/CD Integration

### GitHub Actions Deployment

```yaml
# .github/workflows/deploy.yml
name: Deploy to Staging
on:
  push:
    branches: [staging]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Configure kubectl
        uses: azure/k8s-set-context@v3
        with:
          kubeconfig: ${{ secrets.KUBE_CONFIG }}

      - name: Create secrets
        run: |
          kubectl create secret generic llm-cost-ops-db-secret \
            --from-literal=password=${{ secrets.DB_PASSWORD }} \
            --dry-run=client -o yaml | kubectl apply -f -

      - name: Deploy with Helm
        run: |
          helm upgrade --install llm-cost-ops ./helm/llm-cost-ops \
            -f ./helm/llm-cost-ops/values-staging.yaml \
            --set image.tag=${{ github.sha }} \
            --set ingress.hosts[0].host=staging.example.com \
            --wait --timeout=5m
```

### GitLab CI/CD

```yaml
# .gitlab-ci.yml
deploy:staging:
  stage: deploy
  image: alpine/helm:latest
  script:
    - kubectl config use-context staging
    - helm upgrade --install llm-cost-ops ./helm/llm-cost-ops
        -f ./helm/llm-cost-ops/values-staging.yaml
        --set image.tag=$CI_COMMIT_SHORT_SHA
  only:
    - staging
```

---

## Multi-Tenancy

### Deploy Multiple Instances

```bash
# Tenant 1
helm install llm-cost-ops-tenant1 ./llm-cost-ops \
  --create-namespace -n tenant1 \
  --set postgresql.auth.database=tenant1_db \
  --set config.database.name=tenant1_db

# Tenant 2
helm install llm-cost-ops-tenant2 ./llm-cost-ops \
  --create-namespace -n tenant2 \
  --set postgresql.auth.database=tenant2_db \
  --set config.database.name=tenant2_db
```

### Shared Database, Isolated Namespaces

```bash
# Create external DB secret in each namespace
for ns in tenant1 tenant2 tenant3; do
  kubectl create namespace $ns
  kubectl create secret generic db-secret \
    --from-literal=password=$DB_PASSWORD \
    -n $ns

  helm install llm-cost-ops ./llm-cost-ops \
    -n $ns \
    --set postgresql.enabled=false \
    --set config.database.host=shared-postgres.db.svc \
    --set config.database.name=${ns}_db \
    --set config.database.existingSecret=db-secret
done
```

---

## High Availability

### Production HA Setup

```bash
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set replicaCount=3 \
  --set autoscaling.enabled=true \
  --set autoscaling.minReplicas=3 \
  --set autoscaling.maxReplicas=20 \
  --set podDisruptionBudget.enabled=true \
  --set podDisruptionBudget.minAvailable=2 \
  --set affinity.podAntiAffinity.requiredDuringSchedulingIgnoredDuringExecution[0].topologyKey=kubernetes.io/hostname \
  --set topologySpreadConstraints[0].maxSkew=1 \
  --set topologySpreadConstraints[0].topologyKey=topology.kubernetes.io/zone
```

### Multi-Region Deployment

```bash
# Region 1 (us-east-1)
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set ingress.hosts[0].host=us-east.llm-cost-ops.com \
  --set nodeSelector.topology\.kubernetes\.io/region=us-east-1

# Region 2 (us-west-1)
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set ingress.hosts[0].host=us-west.llm-cost-ops.com \
  --set nodeSelector.topology\.kubernetes\.io/region=us-west-1
```

---

## Cost Optimization

### Spot Instances / Preemptible Nodes

```bash
helm install llm-cost-ops ./llm-cost-ops \
  --set nodeSelector.node\.kubernetes\.io/instance-type=spot \
  --set tolerations[0].key=spot \
  --set tolerations[0].operator=Equal \
  --set tolerations[0].value=true \
  --set tolerations[0].effect=NoSchedule \
  --set podDisruptionBudget.enabled=true \
  --set podDisruptionBudget.minAvailable=1
```

### Resource-Optimized Deployment

```bash
helm install llm-cost-ops ./llm-cost-ops \
  --set resources.requests.cpu=100m \
  --set resources.requests.memory=128Mi \
  --set resources.limits.cpu=500m \
  --set resources.limits.memory=512Mi \
  --set autoscaling.enabled=true \
  --set autoscaling.targetCPUUtilizationPercentage=70
```

---

## Monitoring & Observability

### Full Observability Stack

```bash
# Prerequisites: Install Prometheus Operator
helm install prometheus prometheus-community/kube-prometheus-stack

# Install with monitoring enabled
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set serviceMonitor.enabled=true \
  --set serviceMonitor.labels.release=prometheus \
  --set config.tracing.enabled=true \
  --set config.tracing.backend=otlp \
  --set config.tracing.otlp.endpoint=http://tempo.monitoring:4317
```

### Jaeger Tracing Integration

```bash
# Prerequisites: Install Jaeger
kubectl create namespace observability
helm install jaeger jaegertracing/jaeger -n observability

# Install with Jaeger tracing
helm install llm-cost-ops ./llm-cost-ops \
  --set config.tracing.enabled=true \
  --set config.tracing.backend=jaeger \
  --set config.tracing.jaeger.endpoint=http://jaeger-collector.observability:14268/api/traces \
  --set config.tracing.samplingRate=0.1
```

### Grafana Dashboard

```bash
# Install Grafana if not present
helm install grafana grafana/grafana

# Import dashboard (in Grafana UI):
# Dashboard ID: Import from file
# File: dashboards/llm-cost-ops.json (create this based on metrics)
```

---

## Security Hardening

### Maximum Security Configuration

```bash
# Create secrets
kubectl create secret generic llm-cost-ops-jwt \
  --from-literal=jwt-secret=$(openssl rand -base64 64)

kubectl create secret generic llm-cost-ops-db \
  --from-literal=password=$(openssl rand -base64 32)

# Deploy with security hardening
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set config.auth.enabled=true \
  --set config.auth.jwt.existingSecret=llm-cost-ops-jwt \
  --set config.database.existingSecret=llm-cost-ops-db \
  --set networkPolicy.enabled=true \
  --set podSecurityContext.runAsNonRoot=true \
  --set podSecurityContext.runAsUser=1000 \
  --set securityContext.readOnlyRootFilesystem=true \
  --set securityContext.allowPrivilegeEscalation=false \
  --set ingress.tls[0].secretName=llm-cost-ops-tls \
  --set ingress.tls[0].hosts[0]=secure.llm-cost-ops.com
```

### With HashiCorp Vault

```bash
# Prerequisites: Vault installed and configured
# Create policy and role in Vault

# Deploy with Vault integration
helm install llm-cost-ops ./llm-cost-ops \
  --set serviceAccount.annotations.vault\.hashicorp\.com/agent-inject=true \
  --set serviceAccount.annotations.vault\.hashicorp\.com/role=llm-cost-ops \
  --set serviceAccount.annotations.vault\.hashicorp\.com/agent-inject-secret-db=secret/data/database/creds
```

### With cert-manager for TLS

```bash
# Prerequisites: cert-manager installed
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: llm-cost-ops-tls
spec:
  secretName: llm-cost-ops-tls
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames:
  - llm-cost-ops.example.com
EOF

# Install with automatic TLS
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set ingress.tls[0].secretName=llm-cost-ops-tls \
  --set ingress.tls[0].hosts[0]=llm-cost-ops.example.com
```

---

## Disaster Recovery

### Backup Configuration

```bash
# Install Velero for cluster backups
velero install --provider aws --bucket llm-cost-ops-backups

# Create backup schedule
velero schedule create llm-cost-ops-daily \
  --schedule="0 2 * * *" \
  --include-namespaces llm-cost-ops \
  --ttl 720h0m0s
```

### Database Backup

```bash
# PostgreSQL backup CronJob
kubectl apply -f - <<EOF
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
spec:
  schedule: "0 2 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:16-alpine
            env:
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: llm-cost-ops-db-secret
                  key: password
            command:
            - /bin/sh
            - -c
            - |
              pg_dump -h llm-cost-ops-postgres -U postgres llm_cost_ops | \
              gzip > /backup/llm-cost-ops-\$(date +%Y%m%d-%H%M%S).sql.gz
            volumeMounts:
            - name: backup
              mountPath: /backup
          volumes:
          - name: backup
            persistentVolumeClaim:
              claimName: postgres-backup
          restartPolicy: OnFailure
EOF
```

### Restore from Backup

```bash
# Restore using Velero
velero restore create --from-backup llm-cost-ops-daily-20240101

# Restore database manually
kubectl exec -it llm-cost-ops-postgres-0 -- \
  psql -U postgres -c "CREATE DATABASE llm_cost_ops_restore"

kubectl cp backup.sql.gz llm-cost-ops-postgres-0:/tmp/
kubectl exec -it llm-cost-ops-postgres-0 -- \
  gunzip < /tmp/backup.sql.gz | psql -U postgres -d llm_cost_ops_restore
```

---

## Advanced Scenarios

### Blue-Green Deployment

```bash
# Deploy blue version
helm install llm-cost-ops-blue ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set image.tag=v1.0.0 \
  --set service.selector.version=blue

# Deploy green version
helm install llm-cost-ops-green ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml \
  --set image.tag=v2.0.0 \
  --set service.selector.version=green

# Switch traffic (update ingress)
kubectl patch ingress llm-cost-ops -p '{"spec":{"rules":[{"host":"llm-cost-ops.com","http":{"paths":[{"backend":{"service":{"name":"llm-cost-ops-green"}}}]}}]}}'
```

### Canary Deployment

```bash
# Install with Flagger for canary
helm install llm-cost-ops ./llm-cost-ops \
  -f ./llm-cost-ops/values-prod.yaml

# Create Canary resource
kubectl apply -f - <<EOF
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: llm-cost-ops
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-cost-ops
  service:
    port: 8080
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    metrics:
    - name: request-success-rate
      thresholdRange:
        min: 99
EOF
```

---

## Troubleshooting Commands

```bash
# Check all resources
kubectl get all -l app.kubernetes.io/instance=llm-cost-ops

# Describe problematic pods
kubectl describe pod -l app.kubernetes.io/name=llm-cost-ops

# View logs
kubectl logs -f -l app.kubernetes.io/name=llm-cost-ops --tail=100

# Debug with ephemeral container (K8s 1.23+)
kubectl debug -it llm-cost-ops-pod --image=busybox --target=llm-cost-ops

# Port forward for debugging
kubectl port-forward svc/llm-cost-ops 8080:8080

# Execute commands in pod
kubectl exec -it llm-cost-ops-pod -- /bin/sh

# View Helm release history
helm history llm-cost-ops

# Rollback to previous version
helm rollback llm-cost-ops
```
