# Video 08: Enterprise Deployment on Kubernetes

## Metadata

- **Duration**: 32-35 minutes
- **Level**: Advanced
- **Prerequisites**: Videos 01-04
- **Target Audience**: DevOps engineers, platform engineers, SREs
- **Video ID**: LLMCO-V08-ENTERPRISE
- **Version**: 1.0.0

## Learning Objectives

- Deploy LLM Cost Ops to Kubernetes with high availability
- Configure auto-scaling and resource management
- Implement monitoring and observability
- Set up multi-region deployments
- Configure security and compliance controls
- Manage database migrations and backups
- Implement disaster recovery procedures

## Scene Breakdown

### Scene 1: Architecture Overview
**Duration**: 0:00-3:00

**Visual**: Production architecture diagram

**Narration**:
"Welcome to enterprise deployment! Today we're deploying LLM Cost Ops to production Kubernetes with high availability, auto-scaling, monitoring, and security. This is everything you need for enterprise-grade deployment."

**Architecture Components**:
- API servers (horizontally scaled)
- PostgreSQL (managed RDS or Cloud SQL)
- Redis cluster
- Ingress controller
- Monitoring stack (Prometheus, Grafana)

---

### Scene 2: Helm Chart Installation
**Duration**: 3:00-8:00

**Code/Demo**:
```bash
# Add Helm repository
helm repo add llm-cost-ops https://charts.llm-cost-ops.dev
helm repo update

# Create namespace
kubectl create namespace llm-cost-ops

# Install with production values
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --values production-values.yaml
```

**production-values.yaml**:
```yaml
# High availability configuration
replicaCount: 3

image:
  repository: llm-cost-ops/api-server
  tag: "1.0.0"
  pullPolicy: IfNotPresent

# Resource limits
resources:
  requests:
    cpu: 1000m
    memory: 2Gi
  limits:
    cpu: 2000m
    memory: 4Gi

# Autoscaling
autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

# Database (external managed PostgreSQL)
postgresql:
  enabled: false  # Using AWS RDS
  external:
    host: postgres.abc123.us-east-1.rds.amazonaws.com
    port: 5432
    database: llm_cost_ops
    username: llm_cost_ops
    passwordSecret: postgres-credentials
    sslMode: require

# Redis (clustered)
redis:
  enabled: true
  cluster:
    enabled: true
    nodes: 6
    replicas: 1
  master:
    persistence:
      enabled: true
      size: 10Gi

# Ingress
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
  hosts:
    - host: llm-cost-ops.company.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: llm-cost-ops-tls
      hosts:
        - llm-cost-ops.company.com

# Monitoring
metrics:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 30s

# Security
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  fsGroup: 1000
  capabilities:
    drop:
      - ALL

# Network policies
networkPolicy:
  enabled: true
  policyTypes:
    - Ingress
    - Egress

# Pod disruption budget
podDisruptionBudget:
  enabled: true
  minAvailable: 2
```

**Highlight**: "Helm chart • High availability • Auto-scaling"

---

### Scene 3: Database Setup & Migrations
**Duration**: 8:00-12:00

**Code/Demo**:
```bash
# Create database migration job
kubectl apply -f - <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: llm-cost-ops-migrate
  namespace: llm-cost-ops
spec:
  template:
    spec:
      restartPolicy: OnFailure
      containers:
      - name: migrate
        image: llm-cost-ops/api-server:1.0.0
        command: ["./migrate"]
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: postgres-credentials
              key: url
        - name: RUN_MIGRATIONS
          value: "true"
EOF

# Watch migration progress
kubectl logs -f job/llm-cost-ops-migrate -n llm-cost-ops

# Verify migration
kubectl exec -it deployment/llm-cost-ops -n llm-cost-ops -- \
  ./llm-cost-ops db status

# Output:
# Current version: 20250115_add_cost_forecasting
# Pending migrations: 0
# Database status: healthy
```

**Backup Configuration**:
```yaml
# CronJob for automated backups
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: llm-cost-ops
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: backup
            image: postgres:15
            command:
            - /bin/sh
            - -c
            - |
              pg_dump $DATABASE_URL | \
              gzip | \
              aws s3 cp - s3://backups/llm-cost-ops/$(date +%Y%m%d).sql.gz
            env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: postgres-credentials
                  key: url
            - name: AWS_ACCESS_KEY_ID
              valueFrom:
                secretKeyRef:
                  name: aws-credentials
                  key: access-key-id
            - name: AWS_SECRET_ACCESS_KEY
              valueFrom:
                secretKeyRef:
                  name: aws-credentials
                  key: secret-access-key
          restartPolicy: OnFailure
```

**Highlight**: "Automated migrations • Backup strategy • Zero-downtime"

---

### Scene 4: Monitoring & Observability
**Duration**: 12:00-17:00

**Code/Demo**:
```yaml
# Prometheus ServiceMonitor
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: llm-cost-ops
  namespace: llm-cost-ops
spec:
  selector:
    matchLabels:
      app: llm-cost-ops
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics

# Custom metrics exposed
# - llm_cost_ops_requests_total
# - llm_cost_ops_request_duration_seconds
# - llm_cost_ops_tracking_errors_total
# - llm_cost_ops_active_projects
# - llm_cost_ops_monthly_cost_usd
```

**Grafana Dashboard:**
```json
{
  "dashboard": {
    "title": "LLM Cost Ops - Production",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [{
          "expr": "rate(llm_cost_ops_requests_total[5m])"
        }]
      },
      {
        "title": "Error Rate",
        "targets": [{
          "expr": "rate(llm_cost_ops_tracking_errors_total[5m])"
        }]
      },
      {
        "title": "P95 Latency",
        "targets": [{
          "expr": "histogram_quantile(0.95, llm_cost_ops_request_duration_seconds_bucket)"
        }]
      },
      {
        "title": "Monthly Cost Trend",
        "targets": [{
          "expr": "llm_cost_ops_monthly_cost_usd"
        }]
      }
    ]
  }
}
```

**Alerts:**
```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: llm-cost-ops-alerts
  namespace: llm-cost-ops
spec:
  groups:
  - name: llm-cost-ops
    interval: 30s
    rules:
    - alert: HighErrorRate
      expr: |
        rate(llm_cost_ops_tracking_errors_total[5m]) > 0.05
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: High tracking error rate
        description: "Error rate is {{ $value }} errors/sec"

    - alert: HighLatency
      expr: |
        histogram_quantile(0.95,
          rate(llm_cost_ops_request_duration_seconds_bucket[5m])
        ) > 1.0
      for: 5m
      labels:
        severity: warning
      annotations:
        summary: High request latency
        description: "P95 latency is {{ $value }}s"

    - alert: DatabaseConnectionFailure
      expr: |
        llm_cost_ops_database_up == 0
      for: 1m
      labels:
        severity: critical
      annotations:
        summary: Database connection failed
```

**Highlight**: "Prometheus metrics • Grafana dashboards • Automated alerts"

---

### Scene 5: Security & Compliance
**Duration**: 17:00-22:00

**Code/Demo**:
```yaml
# Pod Security Policy
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: llm-cost-ops-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'

# Network Policy (zero trust)
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: llm-cost-ops-netpol
  namespace: llm-cost-ops
spec:
  podSelector:
    matchLabels:
      app: llm-cost-ops
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgresql
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
  - to:  # Allow external LLM provider APIs
    - namespaceSelector: {}
    ports:
    - protocol: TCP
      port: 443

# Secret management with External Secrets Operator
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: llm-cost-ops-secrets
  namespace: llm-cost-ops
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets-manager
    kind: SecretStore
  target:
    name: llm-cost-ops-secrets
    creationPolicy: Owner
  data:
  - secretKey: database-url
    remoteRef:
      key: prod/llm-cost-ops/database-url
  - secretKey: jwt-secret
    remoteRef:
      key: prod/llm-cost-ops/jwt-secret
  - secretKey: encryption-key
    remoteRef:
      key: prod/llm-cost-ops/encryption-key

# Audit logging
apiVersion: v1
kind: ConfigMap
metadata:
  name: audit-policy
  namespace: llm-cost-ops
data:
  policy.yaml: |
    apiVersion: audit.k8s.io/v1
    kind: Policy
    rules:
    - level: RequestResponse
      resources:
      - group: ""
        resources: ["secrets"]
    - level: Metadata
      resources:
      - group: ""
        resources: ["pods", "services"]
```

**Highlight**: "Pod security • Network isolation • Secrets management • Audit logging"

---

### Scene 6: Multi-Region Deployment
**Duration**: 22:00-27:00

**Code/Demo**:
```yaml
# Global load balancer configuration
apiVersion: networking.gke.io/v1
kind: MultiClusterIngress
metadata:
  name: llm-cost-ops-global
  namespace: llm-cost-ops
spec:
  template:
    spec:
      backend:
        serviceName: llm-cost-ops
        servicePort: 80
      rules:
      - host: llm-cost-ops.company.com
        http:
          paths:
          - backend:
              serviceName: llm-cost-ops
              servicePort: 80

# Database replication (PostgreSQL)
# Primary in us-east-1, replicas in eu-west-1 and ap-southeast-1
```

**Terraform for multi-region:**
```hcl
# Deploy to multiple regions
module "llm_cost_ops_us_east" {
  source = "./modules/llm-cost-ops"

  region = "us-east-1"
  cluster_name = "prod-us-east-1"
  database_endpoint = aws_rds_cluster.primary.endpoint
  redis_endpoint = aws_elasticache_replication_group.us_east.primary_endpoint_address
}

module "llm_cost_ops_eu_west" {
  source = "./modules/llm-cost-ops"

  region = "eu-west-1"
  cluster_name = "prod-eu-west-1"
  database_endpoint = aws_rds_cluster.replica_eu.endpoint
  redis_endpoint = aws_elasticache_replication_group.eu_west.primary_endpoint_address
}

# Global traffic routing
resource "aws_route53_record" "global" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "llm-cost-ops.company.com"
  type    = "A"

  latency_routing_policy {
    region = "us-east-1"
  }

  alias {
    name    = module.llm_cost_ops_us_east.load_balancer_dns
    zone_id = module.llm_cost_ops_us_east.load_balancer_zone_id
  }
}
```

**Highlight**: "Multi-region HA • Global load balancing • Read replicas"

---

### Scene 7: Disaster Recovery
**Duration**: 27:00-30:00

**Code/Demo**:
```bash
# Disaster recovery procedure

# 1. Regular backups (automated)
# - Database snapshots every 6 hours
# - Point-in-time recovery enabled
# - Cross-region backup replication

# 2. Recovery procedure
cat > disaster-recovery.sh << 'EOF'
#!/bin/bash
set -e

BACKUP_DATE=${1:-$(date +%Y%m%d)}
REGION=${2:-us-east-1}

echo "Starting disaster recovery from backup: $BACKUP_DATE"

# Restore database
aws rds restore-db-cluster-from-snapshot \
  --db-cluster-identifier llm-cost-ops-recovered \
  --snapshot-identifier llm-cost-ops-$BACKUP_DATE \
  --engine postgres

# Wait for database
aws rds wait db-cluster-available \
  --db-cluster-identifier llm-cost-ops-recovered

# Update Kubernetes secrets
kubectl create secret generic postgres-credentials \
  --from-literal=url="postgresql://..." \
  --dry-run=client -o yaml | kubectl apply -f -

# Restart deployments
kubectl rollout restart deployment/llm-cost-ops -n llm-cost-ops

# Verify
kubectl wait --for=condition=ready pod \
  -l app=llm-cost-ops \
  -n llm-cost-ops \
  --timeout=300s

echo "Recovery complete!"
EOF

# Test recovery quarterly
./disaster-recovery.sh 20250115 us-west-2
```

**RTO/RPO Targets:**
- RTO (Recovery Time Objective): < 1 hour
- RPO (Recovery Point Objective): < 15 minutes

**Highlight**: "Automated backups • DR procedures • <1hr recovery"

---

### Scene 8: Performance Tuning
**Duration**: 30:00-32:00

**Code/Demo**:
```yaml
# Connection pooling
env:
- name: DB_POOL_SIZE
  value: "50"
- name: DB_MAX_OVERFLOW
  value: "20"
- name: DB_POOL_TIMEOUT
  value: "30"

# Caching configuration
- name: REDIS_CACHE_TTL
  value: "3600"
- name: REDIS_MAX_CONNECTIONS
  value: "100"

# API rate limiting
- name: RATE_LIMIT_REQUESTS
  value: "1000"
- name: RATE_LIMIT_PERIOD
  value: "60"

# Worker configuration
- name: WORKER_CONCURRENCY
  value: "10"
- name: WORKER_QUEUE_SIZE
  value: "1000"
```

**Load testing:**
```bash
# Run load test
kubectl run load-test --rm -it --image=grafana/k6 -- run - <<EOF
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 100,
  duration: '5m',
};

export default function () {
  let res = http.post('https://llm-cost-ops.company.com/api/track', {
    /* payload */
  });

  check(res, {
    'status is 200': (r) => r.status === 200,
    'latency < 200ms': (r) => r.timings.duration < 200,
  });
}
EOF
```

**Highlight**: "Connection pooling • Load testing • Performance tuning"

---

### Scene 9: Recap
**Duration**: 32:00-33:00

**Narration**:
"You're now ready for enterprise deployment! We covered Kubernetes installation, HA setup, monitoring, security, multi-region deployment, and disaster recovery. Next video: security and compliance deep dive!"

**On-Screen Text**:
- "Production Checklist:"
  - "✅ Helm chart deployment"
  - "✅ High availability (3+ replicas)"
  - "✅ Auto-scaling configured"
  - "✅ Monitoring & alerts"
  - "✅ Security policies"
  - "✅ Backup & DR procedures"
- "Next: Video 09 - Security & Compliance"

---

## Post-Production Notes

### Chapter Markers
- 0:00 - Architecture
- 3:00 - Helm Installation
- 8:00 - Database & Migrations
- 12:00 - Monitoring
- 17:00 - Security
- 22:00 - Multi-Region
- 27:00 - Disaster Recovery
- 30:00 - Performance
- 32:00 - Recap

**Script Version**: 1.0.0
**Last Updated**: 2025-11-16
