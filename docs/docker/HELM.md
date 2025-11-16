# Helm Chart Guide for LLM Cost Ops

**Kubernetes Package Management**

Version: 1.0.0
Last Updated: 2025-11-16

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Chart Structure](#chart-structure)
4. [Installation](#installation)
5. [Configuration](#configuration)
6. [Values Reference](#values-reference)
7. [Customization](#customization)
8. [Upgrading](#upgrading)
9. [Rollback](#rollback)
10. [Uninstallation](#uninstallation)
11. [Troubleshooting](#troubleshooting)
12. [Best Practices](#best-practices)

---

## Overview

### What is Helm?

Helm is the package manager for Kubernetes, providing:

- **Templating** - Reusable Kubernetes manifests
- **Version control** - Track chart versions
- **Easy upgrades** - Simple deployment updates
- **Rollback** - Quick recovery from failed deployments
- **Dependencies** - Manage chart dependencies
- **Values** - Configurable deployments

### Chart Benefits

The LLM Cost Ops Helm chart provides:

- One-command deployment
- Environment-specific configurations
- Automated secret generation
- Database initialization
- Monitoring stack inclusion
- Production-ready defaults

---

## Prerequisites

### Install Helm

```bash
# Install Helm 3
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Verify installation
helm version

# Add common repositories
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update
```

### Kubernetes Cluster

```bash
# Verify cluster access
kubectl cluster-info

# Create namespace
kubectl create namespace llm-cost-ops
```

---

## Chart Structure

### Directory Layout

```
helm/llm-cost-ops/
├── Chart.yaml              # Chart metadata
├── values.yaml             # Default values
├── values-dev.yaml         # Development overrides
├── values-staging.yaml     # Staging overrides
├── values-prod.yaml        # Production overrides
├── README.md               # Chart documentation
├── templates/              # Kubernetes manifests
│   ├── NOTES.txt          # Post-install notes
│   ├── _helpers.tpl       # Template helpers
│   ├── deployment.yaml    # App deployment
│   ├── service.yaml       # App service
│   ├── ingress.yaml       # Ingress rules
│   ├── configmap.yaml     # Configuration
│   ├── secret.yaml        # Secrets
│   ├── hpa.yaml           # Auto-scaling
│   ├── pdb.yaml           # Pod disruption budget
│   ├── serviceaccount.yaml
│   ├── statefulset-postgres.yaml
│   ├── statefulset-redis.yaml
│   └── servicemonitor.yaml
├── charts/                # Sub-charts
└── crds/                  # Custom Resource Definitions
```

### Chart.yaml

```yaml
apiVersion: v2
name: llm-cost-ops
description: Enterprise cost operations platform for LLM deployments
type: application
version: 1.0.0
appVersion: "1.0.0"

keywords:
  - llm
  - cost-tracking
  - monitoring
  - operations

home: https://github.com/llm-devops/llm-cost-ops
sources:
  - https://github.com/llm-devops/llm-cost-ops

maintainers:
  - name: LLM DevOps Team
    email: devops@llm-cost-ops.com

icon: https://llm-cost-ops.com/icon.png

dependencies:
  - name: postgresql
    version: 12.x.x
    repository: https://charts.bitnami.com/bitnami
    condition: postgresql.enabled
  - name: redis
    version: 18.x.x
    repository: https://charts.bitnami.com/bitnami
    condition: redis.enabled
  - name: prometheus
    version: 25.x.x
    repository: https://prometheus-community.github.io/helm-charts
    condition: prometheus.enabled
```

---

## Installation

### Quick Start

```bash
# Add chart repository
helm repo add llm-cost-ops https://charts.llm-cost-ops.com
helm repo update

# Install with default values
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --create-namespace

# Install with custom values
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --values values-prod.yaml

# Install from local chart
helm install llm-cost-ops ./helm/llm-cost-ops \
  --namespace llm-cost-ops
```

### Installation with Options

```bash
# Set specific values
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --set image.tag=v1.0.0 \
  --set replicaCount=3 \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=api.example.com

# Dry run (validate)
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --dry-run --debug

# Generate manifests
helm template llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --values values-prod.yaml > manifests.yaml
```

### Development Installation

```bash
# Development values
helm install llm-cost-ops ./helm/llm-cost-ops \
  --namespace llm-cost-ops-dev \
  --create-namespace \
  --values values-dev.yaml \
  --set global.environment=development
```

### Production Installation

```bash
# Production values with secrets
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops-prod \
  --create-namespace \
  --values values-prod.yaml \
  --set-file postgresql.auth.password=./secrets/postgres-password.txt \
  --set-file jwt.secret=./secrets/jwt-secret.txt \
  --set ingress.tls[0].secretName=llm-cost-ops-tls \
  --wait \
  --timeout 10m
```

---

## Configuration

### Default Values (values.yaml)

```yaml
# Global settings
global:
  environment: production
  imageRegistry: docker.io
  storageClass: standard

# Application
replicaCount: 3
image:
  repository: llm-cost-ops
  tag: latest
  pullPolicy: IfNotPresent

imagePullSecrets: []
nameOverride: ""
fullnameOverride: ""

# Service Account
serviceAccount:
  create: true
  annotations: {}
  name: ""

# Pod Security
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 1000
  fsGroup: 1000

securityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  capabilities:
    drop:
    - ALL

# Service
service:
  type: ClusterIP
  port: 80
  targetPort: 8080
  annotations: {}

# Ingress
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
  hosts:
    - host: api.llm-cost-ops.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: llm-cost-ops-tls
      hosts:
        - api.llm-cost-ops.com

# Resources
resources:
  limits:
    cpu: 2000m
    memory: 2Gi
  requests:
    cpu: 500m
    memory: 512Mi

# Auto-scaling
autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

# Pod Disruption Budget
podDisruptionBudget:
  enabled: true
  minAvailable: 2

# Health checks
livenessProbe:
  httpGet:
    path: /health
    port: http
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health
    port: http
  initialDelaySeconds: 10
  periodSeconds: 5

# PostgreSQL
postgresql:
  enabled: true
  auth:
    username: postgres
    database: llm_cost_ops_prod
    existingSecret: ""
  primary:
    persistence:
      enabled: true
      size: 100Gi
      storageClass: fast-ssd
    resources:
      limits:
        cpu: 4000m
        memory: 8Gi
      requests:
        cpu: 1000m
        memory: 2Gi

# Redis
redis:
  enabled: true
  auth:
    enabled: true
    existingSecret: ""
  master:
    persistence:
      enabled: true
      size: 10Gi
    resources:
      limits:
        cpu: 1000m
        memory: 2Gi
      requests:
        cpu: 200m
        memory: 512Mi

# Monitoring
prometheus:
  enabled: true

grafana:
  enabled: true
  adminPassword: changeme

# Application configuration
config:
  logLevel: info
  features:
    compression: true
    rateLimiting: true
    metrics: true
    tracing: true

# Secrets (use external secret management in production)
secrets:
  jwtSecret: ""
  smtpPassword: ""
```

---

## Values Reference

### Image Configuration

```yaml
image:
  repository: llm-cost-ops        # Image repository
  tag: v1.0.0                     # Image tag
  pullPolicy: IfNotPresent        # Pull policy
  pullSecrets: []                 # Image pull secrets
```

### Replica Configuration

```yaml
replicaCount: 3                   # Number of replicas

autoscaling:
  enabled: true                   # Enable HPA
  minReplicas: 3                  # Minimum replicas
  maxReplicas: 10                 # Maximum replicas
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80
```

### Resource Limits

```yaml
resources:
  limits:
    cpu: 2000m                    # Max CPU
    memory: 2Gi                   # Max memory
  requests:
    cpu: 500m                     # Requested CPU
    memory: 512Mi                 # Requested memory
```

### Database Configuration

```yaml
postgresql:
  enabled: true                   # Enable PostgreSQL
  auth:
    username: postgres
    password: ""                  # Set via --set or existingSecret
    database: llm_cost_ops_prod
    existingSecret: postgres-secret
  primary:
    persistence:
      enabled: true
      size: 100Gi
      storageClass: fast-ssd
```

### Ingress Configuration

```yaml
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
  hosts:
    - host: api.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: llm-cost-ops-tls
      hosts:
        - api.example.com
```

---

## Customization

### Environment-Specific Values

**values-dev.yaml:**

```yaml
global:
  environment: development

replicaCount: 1

image:
  tag: latest
  pullPolicy: Always

ingress:
  enabled: false

autoscaling:
  enabled: false

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 200m
    memory: 256Mi

postgresql:
  primary:
    persistence:
      size: 10Gi
    resources:
      limits:
        cpu: 1000m
        memory: 2Gi

config:
  logLevel: debug
```

**values-staging.yaml:**

```yaml
global:
  environment: staging

replicaCount: 2

image:
  tag: staging
  pullPolicy: Always

ingress:
  enabled: true
  hosts:
    - host: staging-api.example.com

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5

postgresql:
  primary:
    persistence:
      size: 50Gi
```

**values-prod.yaml:**

```yaml
global:
  environment: production

replicaCount: 3

image:
  tag: v1.0.0
  pullPolicy: IfNotPresent

ingress:
  enabled: true
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: api.example.com
  tls:
    - secretName: llm-cost-ops-tls

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10

postgresql:
  primary:
    persistence:
      size: 100Gi
      storageClass: fast-ssd
    resources:
      limits:
        cpu: 4000m
        memory: 8Gi

podDisruptionBudget:
  enabled: true
  minAvailable: 2

config:
  logLevel: info
```

### Using Multiple Values Files

```bash
# Merge multiple values files
helm install llm-cost-ops ./helm/llm-cost-ops \
  --values values.yaml \
  --values values-prod.yaml \
  --values values-custom.yaml
```

### Template Customization

**Custom template (_helpers.tpl):**

```yaml
{{/*
Expand the name of the chart.
*/}}
{{- define "llm-cost-ops.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "llm-cost-ops.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "llm-cost-ops.labels" -}}
helm.sh/chart: {{ include "llm-cost-ops.chart" . }}
{{ include "llm-cost-ops.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
environment: {{ .Values.global.environment }}
{{- end }}
```

---

## Upgrading

### Upgrade Release

```bash
# Upgrade with new values
helm upgrade llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --values values-prod.yaml \
  --set image.tag=v1.1.0

# Upgrade with wait
helm upgrade llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --values values-prod.yaml \
  --wait \
  --timeout 10m

# Upgrade with dry-run
helm upgrade llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --values values-prod.yaml \
  --dry-run --debug
```

### Upgrade Strategy

**Rolling update:**

```yaml
# deployment.yaml
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
```

**Blue-Green deployment:**

```bash
# Install new version
helm install llm-cost-ops-v2 llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --set image.tag=v2.0.0

# Switch traffic (update ingress)
kubectl patch ingress llm-cost-ops \
  -n llm-cost-ops \
  --type=json \
  -p='[{"op": "replace", "path": "/spec/rules/0/http/paths/0/backend/service/name", "value":"llm-cost-ops-v2"}]'

# Remove old version
helm uninstall llm-cost-ops -n llm-cost-ops
```

### Viewing History

```bash
# List all releases
helm list -n llm-cost-ops

# Show release history
helm history llm-cost-ops -n llm-cost-ops

# Get release values
helm get values llm-cost-ops -n llm-cost-ops

# Get release manifest
helm get manifest llm-cost-ops -n llm-cost-ops
```

---

## Rollback

### Rollback to Previous Version

```bash
# Rollback to previous revision
helm rollback llm-cost-ops -n llm-cost-ops

# Rollback to specific revision
helm rollback llm-cost-ops 3 -n llm-cost-ops

# Rollback with wait
helm rollback llm-cost-ops -n llm-cost-ops --wait

# Dry-run rollback
helm rollback llm-cost-ops -n llm-cost-ops --dry-run
```

### Automatic Rollback

```bash
# Upgrade with automatic rollback on failure
helm upgrade llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --values values-prod.yaml \
  --atomic \
  --timeout 5m
```

---

## Uninstallation

### Uninstall Release

```bash
# Uninstall release
helm uninstall llm-cost-ops -n llm-cost-ops

# Keep history
helm uninstall llm-cost-ops -n llm-cost-ops --keep-history

# Dry-run uninstall
helm uninstall llm-cost-ops -n llm-cost-ops --dry-run
```

### Clean Up

```bash
# Delete namespace
kubectl delete namespace llm-cost-ops

# Delete PVCs (if needed)
kubectl delete pvc -n llm-cost-ops --all

# Delete secrets
kubectl delete secret -n llm-cost-ops --all
```

---

## Troubleshooting

### Common Issues

**Chart not found:**

```bash
helm repo update
helm search repo llm-cost-ops
```

**Values not applying:**

```bash
# Check merged values
helm get values llm-cost-ops -n llm-cost-ops --all

# Validate template
helm template llm-cost-ops ./helm/llm-cost-ops --debug
```

**Failed installation:**

```bash
# View release status
helm status llm-cost-ops -n llm-cost-ops

# View logs
kubectl logs -n llm-cost-ops -l app.kubernetes.io/name=llm-cost-ops

# Delete and retry
helm uninstall llm-cost-ops -n llm-cost-ops
helm install llm-cost-ops llm-cost-ops/llm-cost-ops -n llm-cost-ops
```

### Debugging

```bash
# Lint chart
helm lint ./helm/llm-cost-ops

# Template with debug
helm template llm-cost-ops ./helm/llm-cost-ops --debug

# Install with debug
helm install llm-cost-ops ./helm/llm-cost-ops \
  --debug \
  --dry-run \
  --namespace llm-cost-ops
```

---

## Best Practices

1. **Version control** - Track values files in git
2. **Use environments** - Separate values per environment
3. **Secrets management** - Use external secret stores
4. **Testing** - Test with `--dry-run` before deployment
5. **Documentation** - Document custom values
6. **Backup** - Backup release values
7. **Monitoring** - Monitor chart deployments
8. **Dependencies** - Pin dependency versions
9. **Rollback plan** - Test rollback procedures
10. **CI/CD** - Automate chart deployments

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0
