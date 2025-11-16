# Kubernetes Deployment Guide for LLM Cost Ops

**Enterprise-Grade Container Orchestration**

Version: 1.0.0
Last Updated: 2025-11-16

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Cluster Requirements](#cluster-requirements)
4. [Namespace Setup](#namespace-setup)
5. [ConfigMaps and Secrets](#configmaps-and-secrets)
6. [Storage Configuration](#storage-configuration)
7. [Database Deployment](#database-deployment)
8. [Application Deployment](#application-deployment)
9. [Service Configuration](#service-configuration)
10. [Ingress Setup](#ingress-setup)
11. [Monitoring and Logging](#monitoring-and-logging)
12. [Auto-Scaling](#auto-scaling)
13. [Security](#security)
14. [Production Checklist](#production-checklist)
15. [Troubleshooting](#troubleshooting)

---

## Overview

### Why Kubernetes?

Kubernetes provides:

- **High availability** - Automatic failover and recovery
- **Auto-scaling** - Horizontal and vertical scaling
- **Self-healing** - Automatic container restart
- **Service discovery** - Built-in DNS and load balancing
- **Rolling updates** - Zero-downtime deployments
- **Resource optimization** - Efficient cluster utilization
- **Multi-cloud** - Cloud-agnostic deployments

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         Ingress                             │
│                    (NGINX/Traefik)                          │
└────────────────────┬────────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│   Service: app  │     │ Service: grafana│
│   (ClusterIP)   │     │   (ClusterIP)   │
└────────┬────────┘     └────────┬────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│  Deployment:    │     │  Deployment:    │
│    app          │     │    grafana      │
│  (3 replicas)   │     │  (1 replica)    │
└────────┬────────┘     └────────┬────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│  StatefulSet:   │     │  StatefulSet:   │
│   postgres      │     │   prometheus    │
│  (1 replica)    │     │  (1 replica)    │
└────────┬────────┘     └────────┬────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│      PVC:       │     │      PVC:       │
│  postgres-data  │     │ prometheus-data │
└─────────────────┘     └─────────────────┘
```

---

## Prerequisites

### Required Tools

**kubectl (Kubernetes CLI)**

```bash
# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
chmod +x kubectl
sudo mv kubectl /usr/local/bin/

# Verify
kubectl version --client
```

**Helm (Package Manager)**

```bash
# Install Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Verify
helm version
```

**Optional Tools**

```bash
# k9s - Terminal UI for Kubernetes
brew install k9s

# kubectx/kubens - Context switching
brew install kubectx

# stern - Multi-pod log tailing
brew install stern
```

### Cluster Access

**Local Kubernetes:**

```bash
# Minikube
minikube start --cpus=4 --memory=8192 --disk-size=50g

# Kind (Kubernetes in Docker)
kind create cluster --config kind-config.yaml

# Docker Desktop
# Enable Kubernetes in Docker Desktop settings
```

**Cloud Providers:**

```bash
# Google Kubernetes Engine (GKE)
gcloud container clusters get-credentials llm-cost-ops --zone us-central1-a

# Amazon Elastic Kubernetes Service (EKS)
aws eks update-kubeconfig --name llm-cost-ops --region us-east-1

# Azure Kubernetes Service (AKS)
az aks get-credentials --resource-group llm-cost-ops --name llm-cost-ops
```

### Verify Cluster

```bash
# Check cluster info
kubectl cluster-info

# View nodes
kubectl get nodes

# Check version
kubectl version
```

---

## Cluster Requirements

### Minimum Requirements (Development)

- **Nodes:** 3 worker nodes
- **CPU per node:** 4 cores
- **Memory per node:** 8 GB
- **Storage:** 100 GB SSD
- **Kubernetes version:** 1.25+

### Recommended Requirements (Production)

- **Nodes:** 5+ worker nodes
- **CPU per node:** 8 cores
- **Memory per node:** 16 GB
- **Storage:** 500 GB SSD/NVMe
- **Kubernetes version:** 1.27+
- **Load balancer:** Cloud provider LB or MetalLB
- **Storage class:** SSD-backed with dynamic provisioning

### Required Add-ons

```bash
# Metrics Server (for HPA)
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml

# NGINX Ingress Controller
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/cloud/deploy.yaml

# Certificate Manager (for TLS)
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml
```

---

## Namespace Setup

### Create Namespace

**namespace.yaml:**

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: llm-cost-ops
  labels:
    name: llm-cost-ops
    environment: production
    app: llm-cost-ops

---
apiVersion: v1
kind: Namespace
metadata:
  name: llm-cost-ops-monitoring
  labels:
    name: llm-cost-ops-monitoring
    environment: production
```

```bash
kubectl apply -f namespace.yaml
```

### Set Default Namespace

```bash
# Set context to use namespace
kubectl config set-context --current --namespace=llm-cost-ops

# Verify
kubectl config view --minify | grep namespace
```

### Resource Quotas

**resourcequota.yaml:**

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: llm-cost-ops-quota
  namespace: llm-cost-ops
spec:
  hard:
    requests.cpu: "20"
    requests.memory: 40Gi
    requests.storage: 500Gi
    persistentvolumeclaims: "10"
    services.loadbalancers: "2"
    services.nodeports: "0"
```

### Limit Ranges

**limitrange.yaml:**

```yaml
apiVersion: v1
kind: LimitRange
metadata:
  name: llm-cost-ops-limits
  namespace: llm-cost-ops
spec:
  limits:
  - max:
      cpu: "4"
      memory: 8Gi
    min:
      cpu: 100m
      memory: 128Mi
    default:
      cpu: 500m
      memory: 512Mi
    defaultRequest:
      cpu: 200m
      memory: 256Mi
    type: Container
```

---

## ConfigMaps and Secrets

### Application ConfigMap

**configmap-app.yaml:**

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: llm-cost-ops-config
  namespace: llm-cost-ops
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    workers = 4

    [database]
    max_connections = 50
    min_connections = 10

    [observability]
    metrics_port = 9090
    log_level = "info"

  application.yaml: |
    features:
      compression: true
      rate_limiting: true
      metrics: true
      tracing: true

    cors:
      allowed_origins:
        - "https://app.example.com"
        - "https://api.example.com"
```

### Secrets Management

**Create secrets from files:**

```bash
# Database credentials
kubectl create secret generic postgres-credentials \
  --from-literal=username=postgres \
  --from-literal=password=$(openssl rand -base64 32) \
  -n llm-cost-ops

# JWT secret
kubectl create secret generic jwt-secret \
  --from-literal=secret=$(openssl rand -base64 64) \
  -n llm-cost-ops

# Redis password
kubectl create secret generic redis-credentials \
  --from-literal=password=$(openssl rand -base64 32) \
  -n llm-cost-ops

# TLS certificates
kubectl create secret tls llm-cost-ops-tls \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key \
  -n llm-cost-ops
```

**Using YAML (base64 encoded):**

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
  namespace: llm-cost-ops
type: Opaque
data:
  database-url: cG9zdGdyZXM6Ly8uLi4=  # base64 encoded
  jwt-secret: c3VwZXJzZWNyZXQ=        # base64 encoded
```

**Using External Secrets Operator:**

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: app-secrets
  namespace: llm-cost-ops
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets-manager
    kind: SecretStore
  target:
    name: app-secrets
  data:
  - secretKey: database-password
    remoteRef:
      key: llm-cost-ops/prod/postgres-password
```

---

## Storage Configuration

### Storage Class

**storageclass.yaml:**

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: kubernetes.io/gce-pd  # GCE example
parameters:
  type: pd-ssd
  replication-type: regional-pd
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

### Persistent Volume Claims

**pvc-postgres.yaml:**

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-data
  namespace: llm-cost-ops
  labels:
    app: postgres
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 100Gi
```

**pvc-prometheus.yaml:**

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: prometheus-data
  namespace: llm-cost-ops-monitoring
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 50Gi
```

### Volume Snapshots

**volumesnapshot.yaml:**

```yaml
apiVersion: snapshot.storage.k8s.io/v1
kind: VolumeSnapshot
metadata:
  name: postgres-snapshot-20241116
  namespace: llm-cost-ops
spec:
  volumeSnapshotClassName: csi-snapclass
  source:
    persistentVolumeClaimName: postgres-data
```

---

## Database Deployment

### PostgreSQL StatefulSet

**statefulset-postgres.yaml:**

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: llm-cost-ops
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:16-alpine
        ports:
        - containerPort: 5432
          name: postgres
        env:
        - name: POSTGRES_DB
          value: llm_cost_ops_prod
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: postgres-credentials
              key: username
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-credentials
              key: password
        - name: PGDATA
          value: /var/lib/postgresql/data/pgdata
        volumeMounts:
        - name: postgres-data
          mountPath: /var/lib/postgresql/data
        - name: postgres-config
          mountPath: /etc/postgresql
        resources:
          requests:
            cpu: 1000m
            memory: 2Gi
          limits:
            cpu: 4000m
            memory: 8Gi
        livenessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: postgres-config
        configMap:
          name: postgres-config
  volumeClaimTemplates:
  - metadata:
      name: postgres-data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 100Gi

---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: llm-cost-ops
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
  clusterIP: None  # Headless service for StatefulSet
```

### Redis Deployment

**deployment-redis.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: llm-cost-ops
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
        args:
        - --requirepass
        - $(REDIS_PASSWORD)
        - --maxmemory
        - 2gb
        - --maxmemory-policy
        - allkeys-lru
        env:
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: redis-credentials
              key: password
        resources:
          requests:
            cpu: 200m
            memory: 512Mi
          limits:
            cpu: 1000m
            memory: 2Gi
        livenessProbe:
          exec:
            command:
            - redis-cli
            - ping
          initialDelaySeconds: 30
          periodSeconds: 10

---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: llm-cost-ops
spec:
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
```

---

## Application Deployment

### Application Deployment

**deployment-app.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-cost-ops
  namespace: llm-cost-ops
  labels:
    app: llm-cost-ops
    version: v1.0.0
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: llm-cost-ops
  template:
    metadata:
      labels:
        app: llm-cost-ops
        version: v1.0.0
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: llm-cost-ops
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      initContainers:
      - name: wait-for-postgres
        image: postgres:16-alpine
        command:
        - sh
        - -c
        - |
          until pg_isready -h postgres -U postgres; do
            echo "Waiting for postgres..."
            sleep 2
          done
      - name: run-migrations
        image: llm-cost-ops:v1.0.0
        command: ["cargo", "sqlx", "migrate", "run"]
        env:
        - name: DATABASE_URL
          value: postgres://postgres:$(POSTGRES_PASSWORD)@postgres:5432/llm_cost_ops_prod
        envFrom:
        - secretRef:
            name: postgres-credentials
      containers:
      - name: app
        image: llm-cost-ops:v1.0.0
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        - containerPort: 9090
          name: metrics
          protocol: TCP
        env:
        - name: RUST_LOG
          value: info
        - name: LOG_LEVEL
          value: info
        - name: PORT
          value: "8080"
        - name: METRICS_PORT
          value: "9090"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: app-secrets
              key: database-url
        - name: REDIS_URL
          value: redis://:$(REDIS_PASSWORD)@redis:6379/0
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: jwt-secret
              key: secret
        envFrom:
        - configMapRef:
            name: llm-cost-ops-config
        - secretRef:
            name: redis-credentials
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
      volumes:
      - name: config
        configMap:
          name: llm-cost-ops-config
```

### Service Account

**serviceaccount.yaml:**

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: llm-cost-ops
  namespace: llm-cost-ops
```

---

## Service Configuration

### Application Service

**service-app.yaml:**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: llm-cost-ops
  namespace: llm-cost-ops
  labels:
    app: llm-cost-ops
spec:
  type: ClusterIP
  selector:
    app: llm-cost-ops
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: 9090
    protocol: TCP
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800
```

### Headless Service (StatefulSet)

```yaml
apiVersion: v1
kind: Service
metadata:
  name: postgres-headless
  namespace: llm-cost-ops
spec:
  clusterIP: None
  selector:
    app: postgres
  ports:
  - port: 5432
```

---

## Ingress Setup

### NGINX Ingress

**ingress.yaml:**

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: llm-cost-ops-ingress
  namespace: llm-cost-ops
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/enable-cors: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - api.llm-cost-ops.com
    secretName: llm-cost-ops-tls
  rules:
  - host: api.llm-cost-ops.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: llm-cost-ops
            port:
              number: 80
```

### TLS Certificate

**certificate.yaml:**

```yaml
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: llm-cost-ops-cert
  namespace: llm-cost-ops
spec:
  secretName: llm-cost-ops-tls
  issuerRef:
    name: letsencrypt-prod
    kind: ClusterIssuer
  dnsNames:
  - api.llm-cost-ops.com
  - www.llm-cost-ops.com
```

---

## Monitoring and Logging

### Prometheus Deployment

**deployment-prometheus.yaml:**

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: prometheus
  namespace: llm-cost-ops-monitoring
spec:
  serviceName: prometheus
  replicas: 1
  selector:
    matchLabels:
      app: prometheus
  template:
    metadata:
      labels:
        app: prometheus
    spec:
      containers:
      - name: prometheus
        image: prom/prometheus:v2.48.0
        args:
        - --config.file=/etc/prometheus/prometheus.yml
        - --storage.tsdb.path=/prometheus
        - --storage.tsdb.retention.time=30d
        ports:
        - containerPort: 9090
        volumeMounts:
        - name: prometheus-config
          mountPath: /etc/prometheus
        - name: prometheus-data
          mountPath: /prometheus
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 2000m
            memory: 4Gi
      volumes:
      - name: prometheus-config
        configMap:
          name: prometheus-config
  volumeClaimTemplates:
  - metadata:
      name: prometheus-data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 50Gi
```

### ServiceMonitor

**servicemonitor.yaml:**

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: llm-cost-ops
  namespace: llm-cost-ops-monitoring
spec:
  selector:
    matchLabels:
      app: llm-cost-ops
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

---

## Auto-Scaling

### Horizontal Pod Autoscaler

**hpa.yaml:**

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: llm-cost-ops-hpa
  namespace: llm-cost-ops
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
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
      - type: Pods
        value: 2
        periodSeconds: 30
      selectPolicy: Max
```

### Vertical Pod Autoscaler

**vpa.yaml:**

```yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: llm-cost-ops-vpa
  namespace: llm-cost-ops
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-cost-ops
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: app
      minAllowed:
        cpu: 200m
        memory: 256Mi
      maxAllowed:
        cpu: 4000m
        memory: 8Gi
```

---

## Security

### Network Policies

**networkpolicy.yaml:**

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: llm-cost-ops-network-policy
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
          app: postgres
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
```

### Pod Security Standards

**pod-security.yaml:**

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: llm-cost-ops
  labels:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
```

### RBAC

**rbac.yaml:**

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: llm-cost-ops-role
  namespace: llm-cost-ops
rules:
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list"]
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list", "watch"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: llm-cost-ops-rolebinding
  namespace: llm-cost-ops
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: llm-cost-ops-role
subjects:
- kind: ServiceAccount
  name: llm-cost-ops
  namespace: llm-cost-ops
```

---

## Production Checklist

- [ ] Cluster meets minimum requirements
- [ ] Namespaces created with resource quotas
- [ ] Secrets configured securely
- [ ] Persistent storage provisioned
- [ ] Database backed up
- [ ] Application deployed with replicas
- [ ] Health checks configured
- [ ] Auto-scaling enabled
- [ ] Monitoring deployed
- [ ] Logging configured
- [ ] Ingress with TLS configured
- [ ] Network policies applied
- [ ] RBAC configured
- [ ] Resource limits set
- [ ] Disaster recovery plan documented

---

## Troubleshooting

### Common Issues

**Pod not starting:**

```bash
kubectl describe pod <pod-name> -n llm-cost-ops
kubectl logs <pod-name> -n llm-cost-ops
kubectl logs <pod-name> -n llm-cost-ops --previous
```

**Service unreachable:**

```bash
kubectl get endpoints <service-name> -n llm-cost-ops
kubectl describe service <service-name> -n llm-cost-ops
```

**Storage issues:**

```bash
kubectl get pvc -n llm-cost-ops
kubectl describe pvc <pvc-name> -n llm-cost-ops
```

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0
