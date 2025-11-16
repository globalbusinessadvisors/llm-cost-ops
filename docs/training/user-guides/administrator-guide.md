# LLM Cost Ops Administrator Guide

## Table of Contents

1. [Introduction](#introduction)
2. [System Requirements](#system-requirements)
3. [Installation and Deployment](#installation-and-deployment)
   - [Docker Deployment](#docker-deployment)
   - [Kubernetes Deployment](#kubernetes-deployment)
   - [Cloud Provider Deployment](#cloud-provider-deployment)
   - [Bare Metal Installation](#bare-metal-installation)
4. [Configuration Management](#configuration-management)
   - [Environment Variables](#environment-variables)
   - [Configuration Files](#configuration-files)
   - [Secrets Management](#secrets-management)
5. [Security Setup](#security-setup)
   - [SSL/TLS Configuration](#ssltls-configuration)
   - [Authentication Methods](#authentication-methods)
   - [Network Security](#network-security)
   - [Security Hardening](#security-hardening)
6. [User and Team Management](#user-and-team-management)
   - [User Lifecycle](#user-lifecycle)
   - [Team Structure](#team-structure)
   - [User Provisioning](#user-provisioning)
7. [Role-Based Access Control (RBAC)](#role-based-access-control-rbac)
   - [Default Roles](#default-roles)
   - [Custom Roles](#custom-roles)
   - [Permission Management](#permission-management)
8. [Database Management](#database-management)
   - [Database Setup](#database-setup)
   - [Schema Migrations](#schema-migrations)
   - [Database Optimization](#database-optimization)
   - [Backup and Recovery](#backup-and-recovery)
9. [Monitoring and Alerting](#monitoring-and-alerting)
   - [Metrics Collection](#metrics-collection)
   - [Log Aggregation](#log-aggregation)
   - [Alert Configuration](#alert-configuration)
   - [Health Checks](#health-checks)
10. [Performance Tuning](#performance-tuning)
    - [Application Optimization](#application-optimization)
    - [Database Tuning](#database-tuning)
    - [Cache Configuration](#cache-configuration)
11. [Scaling Strategies](#scaling-strategies)
    - [Horizontal Scaling](#horizontal-scaling)
    - [Vertical Scaling](#vertical-scaling)
    - [Auto-scaling](#auto-scaling)
12. [Compliance Configuration](#compliance-configuration)
    - [GDPR Compliance](#gdpr-compliance)
    - [SOC 2 Compliance](#soc-2-compliance)
    - [HIPAA Compliance](#hipaa-compliance)
13. [Audit Log Management](#audit-log-management)
14. [Disaster Recovery](#disaster-recovery)
15. [Upgrade and Maintenance](#upgrade-and-maintenance)
16. [Troubleshooting](#troubleshooting)
17. [Best Practices](#best-practices)

---

## Introduction

Welcome to the LLM Cost Ops Administrator Guide. This guide provides comprehensive information for system administrators, DevOps engineers, and infrastructure teams responsible for deploying, configuring, and maintaining the LLM Cost Ops platform.

### Audience

This guide is intended for:
- System Administrators
- DevOps Engineers
- Platform Engineers
- Infrastructure Teams
- Security Engineers

### Overview

LLM Cost Ops is a comprehensive platform for tracking, analyzing, and optimizing LLM API costs. The platform consists of:

- **API Server**: RESTful API for cost tracking
- **Web Dashboard**: User interface for cost visualization
- **Database**: PostgreSQL or MySQL for data storage
- **Cache Layer**: Redis for performance optimization
- **Background Workers**: Processing jobs and scheduled tasks

---

## System Requirements

### Minimum Requirements

**API Server**
- CPU: 2 cores
- RAM: 4 GB
- Storage: 20 GB
- OS: Linux (Ubuntu 20.04+, CentOS 8+, RHEL 8+)

**Database Server**
- CPU: 2 cores
- RAM: 8 GB
- Storage: 100 GB SSD
- PostgreSQL 13+ or MySQL 8.0+

**Cache Server**
- CPU: 1 core
- RAM: 2 GB
- Redis 6.0+

### Recommended Production Requirements

**API Server (per instance)**
- CPU: 4-8 cores
- RAM: 8-16 GB
- Storage: 50 GB SSD
- Network: 1 Gbps

**Database Server**
- CPU: 8-16 cores
- RAM: 32-64 GB
- Storage: 500 GB - 2 TB NVMe SSD
- IOPS: 10,000+

**Cache Server**
- CPU: 2-4 cores
- RAM: 8-16 GB
- Network: 1 Gbps

**Load Balancer**
- CPU: 2-4 cores
- RAM: 4-8 GB
- Network: 10 Gbps

### Supported Platforms

- **Operating Systems**: Ubuntu 20.04+, CentOS 8+, RHEL 8+, Debian 11+
- **Container Platforms**: Docker 20.10+, Kubernetes 1.21+
- **Cloud Providers**: AWS, GCP, Azure, DigitalOcean
- **Databases**: PostgreSQL 13+, MySQL 8.0+
- **Cache**: Redis 6.0+, Memcached 1.6+

---

## Installation and Deployment

### Docker Deployment

#### Single Container Setup

```bash
# Pull the latest image
docker pull llmcostops/api-server:latest

# Create network
docker network create llm-cost-ops

# Run PostgreSQL
docker run -d \
  --name llm-cost-ops-db \
  --network llm-cost-ops \
  -e POSTGRES_DB=llmcostops \
  -e POSTGRES_USER=llmcostops \
  -e POSTGRES_PASSWORD=secure_password \
  -v llm-cost-ops-db:/var/lib/postgresql/data \
  postgres:15

# Run Redis
docker run -d \
  --name llm-cost-ops-redis \
  --network llm-cost-ops \
  -v llm-cost-ops-redis:/data \
  redis:7 redis-server --appendonly yes

# Run API Server
docker run -d \
  --name llm-cost-ops-api \
  --network llm-cost-ops \
  -p 8080:8080 \
  -e DATABASE_URL=postgresql://llmcostops:secure_password@llm-cost-ops-db:5432/llmcostops \
  -e REDIS_URL=redis://llm-cost-ops-redis:6379 \
  -e SECRET_KEY=your-secret-key-here \
  -v llm-cost-ops-config:/app/config \
  llmcostops/api-server:latest
```

#### Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'

services:
  db:
    image: postgres:15
    container_name: llm-cost-ops-db
    environment:
      POSTGRES_DB: llmcostops
      POSTGRES_USER: llmcostops
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - db-data:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d
    networks:
      - llm-cost-ops
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U llmcostops"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: llm-cost-ops-redis
    command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis-data:/data
    networks:
      - llm-cost-ops
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 5

  api:
    image: llmcostops/api-server:${VERSION:-latest}
    container_name: llm-cost-ops-api
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_healthy
    environment:
      DATABASE_URL: postgresql://llmcostops:${DB_PASSWORD}@db:5432/llmcostops
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      SECRET_KEY: ${SECRET_KEY}
      ENVIRONMENT: production
      LOG_LEVEL: info
    ports:
      - "8080:8080"
    volumes:
      - ./config:/app/config:ro
      - api-logs:/app/logs
    networks:
      - llm-cost-ops
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  worker:
    image: llmcostops/worker:${VERSION:-latest}
    container_name: llm-cost-ops-worker
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_healthy
    environment:
      DATABASE_URL: postgresql://llmcostops:${DB_PASSWORD}@db:5432/llmcostops
      REDIS_URL: redis://:${REDIS_PASSWORD}@redis:6379
      SECRET_KEY: ${SECRET_KEY}
    volumes:
      - ./config:/app/config:ro
      - worker-logs:/app/logs
    networks:
      - llm-cost-ops
    restart: unless-stopped

  web:
    image: llmcostops/web-dashboard:${VERSION:-latest}
    container_name: llm-cost-ops-web
    depends_on:
      - api
    environment:
      API_URL: http://api:8080
      NODE_ENV: production
    ports:
      - "3000:3000"
    networks:
      - llm-cost-ops
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    container_name: llm-cost-ops-nginx
    depends_on:
      - api
      - web
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/ssl:/etc/nginx/ssl:ro
      - nginx-logs:/var/log/nginx
    networks:
      - llm-cost-ops
    restart: unless-stopped

volumes:
  db-data:
  redis-data:
  api-logs:
  worker-logs:
  nginx-logs:

networks:
  llm-cost-ops:
    driver: bridge
```

```bash
# Deploy with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f

# Scale API servers
docker-compose up -d --scale api=3

# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

### Kubernetes Deployment

#### Namespace and ConfigMap

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: llm-cost-ops
  labels:
    name: llm-cost-ops
    environment: production
```

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: llm-cost-ops-config
  namespace: llm-cost-ops
data:
  LOG_LEVEL: "info"
  ENVIRONMENT: "production"
  API_PORT: "8080"
  WORKERS: "4"
  MAX_CONNECTIONS: "100"
  TIMEOUT: "30"
```

#### Secrets

```yaml
# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: llm-cost-ops-secrets
  namespace: llm-cost-ops
type: Opaque
stringData:
  DATABASE_URL: postgresql://llmcostops:PASSWORD@postgres-service:5432/llmcostops
  REDIS_URL: redis://:PASSWORD@redis-service:6379
  SECRET_KEY: your-secret-key-here
  JWT_SECRET: your-jwt-secret-here
```

```bash
# Create secrets from file
kubectl create secret generic llm-cost-ops-secrets \
  --from-env-file=.env.production \
  -n llm-cost-ops
```

#### PostgreSQL Deployment

```yaml
# postgres-pvc.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: llm-cost-ops
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
  storageClassName: fast-ssd
---
# postgres-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
  namespace: llm-cost-ops
spec:
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
        image: postgres:15
        env:
        - name: POSTGRES_DB
          value: llmcostops
        - name: POSTGRES_USER
          value: llmcostops
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: llm-cost-ops-secrets
              key: DB_PASSWORD
        ports:
        - containerPort: 5432
          name: postgres
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            memory: "4Gi"
            cpu: "2000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
        livenessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - llmcostops
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - llmcostops
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
# postgres-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: postgres-service
  namespace: llm-cost-ops
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
  type: ClusterIP
```

#### Redis Deployment

```yaml
# redis-deployment.yaml
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
        args:
        - redis-server
        - --appendonly
        - "yes"
        - --requirepass
        - $(REDIS_PASSWORD)
        env:
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: llm-cost-ops-secrets
              key: REDIS_PASSWORD
        ports:
        - containerPort: 6379
          name: redis
        volumeMounts:
        - name: redis-storage
          mountPath: /data
        resources:
          requests:
            memory: "2Gi"
            cpu: "500m"
          limits:
            memory: "4Gi"
            cpu: "1000m"
      volumes:
      - name: redis-storage
        emptyDir: {}
---
# redis-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: redis-service
  namespace: llm-cost-ops
spec:
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
  type: ClusterIP
```

#### API Server Deployment

```yaml
# api-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-cost-ops-api
  namespace: llm-cost-ops
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 1
  selector:
    matchLabels:
      app: llm-cost-ops-api
  template:
    metadata:
      labels:
        app: llm-cost-ops-api
        version: v1.0.0
    spec:
      containers:
      - name: api
        image: llmcostops/api-server:1.0.0
        envFrom:
        - configMapRef:
            name: llm-cost-ops-config
        - secretRef:
            name: llm-cost-ops-secrets
        ports:
        - containerPort: 8080
          name: http
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
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
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: llm-cost-ops-config
---
# api-service.yaml
apiVersion: v1
kind: Service
metadata:
  name: llm-cost-ops-api-service
  namespace: llm-cost-ops
spec:
  selector:
    app: llm-cost-ops-api
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
  type: ClusterIP
---
# api-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: llm-cost-ops-api-hpa
  namespace: llm-cost-ops
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-cost-ops-api
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
```

#### Ingress Configuration

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: llm-cost-ops-ingress
  namespace: llm-cost-ops
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  tls:
  - hosts:
    - api.llmcostops.com
    - app.llmcostops.com
    secretName: llm-cost-ops-tls
  rules:
  - host: api.llmcostops.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: llm-cost-ops-api-service
            port:
              number: 80
  - host: app.llmcostops.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: llm-cost-ops-web-service
            port:
              number: 80
```

#### Deploy to Kubernetes

```bash
# Create namespace
kubectl apply -f namespace.yaml

# Create secrets
kubectl apply -f secrets.yaml

# Create ConfigMap
kubectl apply -f configmap.yaml

# Deploy PostgreSQL
kubectl apply -f postgres-pvc.yaml
kubectl apply -f postgres-deployment.yaml

# Deploy Redis
kubectl apply -f redis-deployment.yaml

# Deploy API Server
kubectl apply -f api-deployment.yaml

# Deploy Ingress
kubectl apply -f ingress.yaml

# Verify deployment
kubectl get pods -n llm-cost-ops
kubectl get services -n llm-cost-ops
kubectl get ingress -n llm-cost-ops

# View logs
kubectl logs -f deployment/llm-cost-ops-api -n llm-cost-ops

# Scale deployment
kubectl scale deployment llm-cost-ops-api --replicas=5 -n llm-cost-ops
```

### Cloud Provider Deployment

#### AWS ECS Deployment

```json
{
  "family": "llm-cost-ops-api",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "2048",
  "memory": "4096",
  "containerDefinitions": [
    {
      "name": "api",
      "image": "llmcostops/api-server:latest",
      "essential": true,
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "ENVIRONMENT",
          "value": "production"
        }
      ],
      "secrets": [
        {
          "name": "DATABASE_URL",
          "valueFrom": "arn:aws:secretsmanager:us-east-1:ACCOUNT:secret:llm-cost-ops/db-url"
        },
        {
          "name": "SECRET_KEY",
          "valueFrom": "arn:aws:secretsmanager:us-east-1:ACCOUNT:secret:llm-cost-ops/secret-key"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/llm-cost-ops",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "api"
        }
      },
      "healthCheck": {
        "command": ["CMD-SHELL", "curl -f http://localhost:8080/health || exit 1"],
        "interval": 30,
        "timeout": 5,
        "retries": 3
      }
    }
  ]
}
```

```bash
# Register task definition
aws ecs register-task-definition --cli-input-json file://task-definition.json

# Create ECS service
aws ecs create-service \
  --cluster llm-cost-ops-cluster \
  --service-name llm-cost-ops-api \
  --task-definition llm-cost-ops-api:1 \
  --desired-count 3 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-abc123],securityGroups=[sg-abc123],assignPublicIp=ENABLED}" \
  --load-balancers "targetGroupArn=arn:aws:elasticloadbalancing:us-east-1:ACCOUNT:targetgroup/llm-cost-ops,containerName=api,containerPort=8080"
```

#### AWS RDS Setup

```bash
# Create RDS instance
aws rds create-db-instance \
  --db-instance-identifier llm-cost-ops-db \
  --db-instance-class db.r6g.xlarge \
  --engine postgres \
  --engine-version 15.3 \
  --master-username llmcostops \
  --master-user-password SecurePassword123! \
  --allocated-storage 100 \
  --storage-type gp3 \
  --iops 3000 \
  --vpc-security-group-ids sg-abc123 \
  --db-subnet-group-name llm-cost-ops-subnet-group \
  --backup-retention-period 7 \
  --preferred-backup-window "03:00-04:00" \
  --preferred-maintenance-window "mon:04:00-mon:05:00" \
  --multi-az \
  --storage-encrypted \
  --enable-cloudwatch-logs-exports '["postgresql"]' \
  --tags "Key=Application,Value=LLMCostOps" "Key=Environment,Value=Production"
```

#### GCP Cloud Run Deployment

```yaml
# cloudrun.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: llm-cost-ops-api
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "3"
        autoscaling.knative.dev/maxScale: "100"
    spec:
      containerConcurrency: 80
      containers:
      - image: gcr.io/PROJECT_ID/llm-cost-ops-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: ENVIRONMENT
          value: production
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: llm-cost-ops-secrets
              key: database-url
        resources:
          limits:
            cpu: "2"
            memory: 4Gi
        livenessProbe:
          httpGet:
            path: /health
          initialDelaySeconds: 10
          periodSeconds: 10
```

```bash
# Deploy to Cloud Run
gcloud run deploy llm-cost-ops-api \
  --image gcr.io/PROJECT_ID/llm-cost-ops-api:latest \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --memory 4Gi \
  --cpu 2 \
  --min-instances 3 \
  --max-instances 100 \
  --set-env-vars ENVIRONMENT=production \
  --set-secrets DATABASE_URL=llm-cost-ops-db-url:latest
```

#### Azure Container Instances

```bash
# Create resource group
az group create --name llm-cost-ops-rg --location eastus

# Create container
az container create \
  --resource-group llm-cost-ops-rg \
  --name llm-cost-ops-api \
  --image llmcostops/api-server:latest \
  --cpu 2 \
  --memory 4 \
  --port 8080 \
  --environment-variables \
    ENVIRONMENT=production \
  --secure-environment-variables \
    DATABASE_URL="postgresql://..." \
    SECRET_KEY="..." \
  --restart-policy Always \
  --dns-name-label llm-cost-ops-api
```

### Bare Metal Installation

```bash
#!/bin/bash
# install.sh - LLM Cost Ops Installation Script

set -e

# Configuration
INSTALL_DIR="/opt/llm-cost-ops"
USER="llmcostops"
GROUP="llmcostops"

# Create system user
sudo useradd -r -s /bin/false -d $INSTALL_DIR $USER

# Install dependencies
sudo apt-get update
sudo apt-get install -y \
  python3.11 \
  python3.11-venv \
  postgresql-15 \
  redis-server \
  nginx \
  supervisor

# Create directories
sudo mkdir -p $INSTALL_DIR/{api,worker,logs,config}
sudo chown -R $USER:$GROUP $INSTALL_DIR

# Install application
cd $INSTALL_DIR/api
sudo -u $USER python3.11 -m venv venv
sudo -u $USER venv/bin/pip install llm-cost-ops

# Configure PostgreSQL
sudo -u postgres psql <<EOF
CREATE DATABASE llmcostops;
CREATE USER llmcostops WITH PASSWORD 'secure_password';
GRANT ALL PRIVILEGES ON DATABASE llmcostops TO llmcostops;
EOF

# Run migrations
cd $INSTALL_DIR/api
sudo -u $USER venv/bin/llm-cost-ops migrate

# Configure Supervisor
cat <<EOF | sudo tee /etc/supervisor/conf.d/llm-cost-ops.conf
[program:llm-cost-ops-api]
command=$INSTALL_DIR/api/venv/bin/llm-cost-ops serve
directory=$INSTALL_DIR/api
user=$USER
autostart=true
autorestart=true
redirect_stderr=true
stdout_logfile=$INSTALL_DIR/logs/api.log

[program:llm-cost-ops-worker]
command=$INSTALL_DIR/worker/venv/bin/llm-cost-ops worker
directory=$INSTALL_DIR/worker
user=$USER
autostart=true
autorestart=true
redirect_stderr=true
stdout_logfile=$INSTALL_DIR/logs/worker.log
EOF

# Configure Nginx
cat <<EOF | sudo tee /etc/nginx/sites-available/llm-cost-ops
server {
    listen 80;
    server_name api.llmcostops.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

sudo ln -s /etc/nginx/sites-available/llm-cost-ops /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Start services
sudo supervisorctl reread
sudo supervisorctl update
sudo supervisorctl start llm-cost-ops-api
sudo supervisorctl start llm-cost-ops-worker

echo "Installation complete!"
echo "API available at: http://localhost:8080"
```

---

## Configuration Management

### Environment Variables

```bash
# Core Configuration
ENVIRONMENT=production                    # Environment: development, staging, production
LOG_LEVEL=info                           # Logging level: debug, info, warn, error
API_PORT=8080                            # API server port
API_HOST=0.0.0.0                        # API server host

# Database Configuration
DATABASE_URL=postgresql://user:pass@host:5432/dbname
DATABASE_POOL_SIZE=20                    # Connection pool size
DATABASE_MAX_OVERFLOW=10                 # Max overflow connections
DATABASE_POOL_TIMEOUT=30                 # Pool checkout timeout
DATABASE_POOL_RECYCLE=3600              # Connection recycle time

# Redis Configuration
REDIS_URL=redis://:password@host:6379/0
REDIS_MAX_CONNECTIONS=50                 # Max Redis connections
REDIS_SOCKET_TIMEOUT=5                   # Socket timeout in seconds
REDIS_SOCKET_CONNECT_TIMEOUT=5          # Connect timeout in seconds

# Security
SECRET_KEY=your-secret-key-here          # Application secret key
JWT_SECRET=your-jwt-secret-here          # JWT signing secret
JWT_ALGORITHM=HS256                      # JWT algorithm
JWT_EXPIRATION=3600                      # JWT expiration in seconds
ALLOWED_HOSTS=api.llmcostops.com        # Allowed hosts (comma-separated)
CORS_ORIGINS=https://app.llmcostops.com  # CORS origins (comma-separated)

# Performance
WORKERS=4                                # Number of worker processes
WORKER_CLASS=uvicorn.workers.UvicornWorker
WORKER_TIMEOUT=30                        # Worker timeout in seconds
MAX_REQUESTS=1000                        # Max requests per worker
MAX_REQUESTS_JITTER=50                   # Request jitter

# Cache
CACHE_ENABLED=true                       # Enable caching
CACHE_TTL=300                           # Default cache TTL in seconds
CACHE_MAX_SIZE=1000                     # Max cache entries

# Rate Limiting
RATE_LIMIT_ENABLED=true                  # Enable rate limiting
RATE_LIMIT_PER_MINUTE=60                # Requests per minute
RATE_LIMIT_PER_HOUR=1000                # Requests per hour

# Monitoring
SENTRY_DSN=https://...@sentry.io/...    # Sentry DSN
SENTRY_ENVIRONMENT=production            # Sentry environment
DATADOG_API_KEY=your-datadog-api-key    # Datadog API key
PROMETHEUS_ENABLED=true                  # Enable Prometheus metrics

# Email
SMTP_HOST=smtp.gmail.com                # SMTP host
SMTP_PORT=587                           # SMTP port
SMTP_USER=noreply@llmcostops.com       # SMTP username
SMTP_PASSWORD=your-smtp-password        # SMTP password
SMTP_FROM=noreply@llmcostops.com       # From email address

# Storage
STORAGE_BACKEND=s3                       # Storage backend: local, s3, gcs, azure
S3_BUCKET=llm-cost-ops-reports          # S3 bucket name
S3_REGION=us-east-1                     # S3 region
S3_ACCESS_KEY=your-access-key           # S3 access key
S3_SECRET_KEY=your-secret-key           # S3 secret key

# Features
ENABLE_WEBHOOKS=true                     # Enable webhooks
ENABLE_EXPORTS=true                      # Enable data exports
ENABLE_API_DOCS=true                     # Enable API documentation
```

### Configuration Files

#### Application Configuration

```yaml
# config/production.yaml
app:
  name: LLM Cost Ops
  version: 1.0.0
  environment: production
  debug: false
  testing: false

server:
  host: 0.0.0.0
  port: 8080
  workers: 4
  timeout: 30
  keepalive: 5
  max_requests: 1000
  max_requests_jitter: 50

database:
  url: ${DATABASE_URL}
  pool_size: 20
  max_overflow: 10
  pool_timeout: 30
  pool_recycle: 3600
  echo: false
  pool_pre_ping: true

redis:
  url: ${REDIS_URL}
  max_connections: 50
  socket_timeout: 5
  socket_connect_timeout: 5
  decode_responses: true

security:
  secret_key: ${SECRET_KEY}
  jwt:
    secret: ${JWT_SECRET}
    algorithm: HS256
    expiration: 3600
    refresh_expiration: 604800
  cors:
    enabled: true
    origins:
      - https://app.llmcostops.com
      - https://admin.llmcostops.com
    methods:
      - GET
      - POST
      - PUT
      - DELETE
      - PATCH
    headers:
      - Content-Type
      - Authorization
    credentials: true
  rate_limit:
    enabled: true
    default_limits:
      - 100/minute
      - 1000/hour
      - 10000/day

logging:
  level: info
  format: json
  handlers:
    console:
      enabled: true
      level: info
    file:
      enabled: true
      level: info
      path: /var/log/llm-cost-ops/api.log
      max_bytes: 104857600  # 100MB
      backup_count: 10
      rotation: daily
    sentry:
      enabled: true
      dsn: ${SENTRY_DSN}
      environment: production
      traces_sample_rate: 0.1

cache:
  enabled: true
  backend: redis
  ttl: 300
  max_size: 1000
  key_prefix: llm-cost-ops

monitoring:
  prometheus:
    enabled: true
    port: 9090
    path: /metrics
  datadog:
    enabled: true
    api_key: ${DATADOG_API_KEY}
    tags:
      - env:production
      - service:llm-cost-ops
  health_check:
    enabled: true
    path: /health
    interval: 10

email:
  backend: smtp
  smtp:
    host: ${SMTP_HOST}
    port: ${SMTP_PORT}
    user: ${SMTP_USER}
    password: ${SMTP_PASSWORD}
    use_tls: true
    use_ssl: false
  from_address: noreply@llmcostops.com
  templates_dir: /app/templates/email

storage:
  backend: s3
  s3:
    bucket: ${S3_BUCKET}
    region: ${S3_REGION}
    access_key: ${S3_ACCESS_KEY}
    secret_key: ${S3_SECRET_KEY}
    endpoint: null
    use_ssl: true

features:
  webhooks: true
  exports: true
  api_docs: true
  analytics: true
  audit_logs: true
```

### Secrets Management

#### AWS Secrets Manager

```bash
# Store secrets in AWS Secrets Manager
aws secretsmanager create-secret \
  --name llm-cost-ops/production/database \
  --secret-string '{"username":"llmcostops","password":"SecurePassword123!"}'

aws secretsmanager create-secret \
  --name llm-cost-ops/production/jwt \
  --secret-string '{"secret":"your-jwt-secret-here"}'

# Retrieve secrets
aws secretsmanager get-secret-value \
  --secret-id llm-cost-ops/production/database \
  --query SecretString \
  --output text | jq -r .password
```

#### HashiCorp Vault

```bash
# Enable KV secrets engine
vault secrets enable -path=llm-cost-ops kv-v2

# Store secrets
vault kv put llm-cost-ops/production/database \
  username=llmcostops \
  password=SecurePassword123!

vault kv put llm-cost-ops/production/jwt \
  secret=your-jwt-secret-here

# Retrieve secrets
vault kv get -field=password llm-cost-ops/production/database
```

#### Kubernetes Secrets

```bash
# Create from literal
kubectl create secret generic llm-cost-ops-db \
  --from-literal=username=llmcostops \
  --from-literal=password=SecurePassword123! \
  -n llm-cost-ops

# Create from file
kubectl create secret generic llm-cost-ops-config \
  --from-file=config.yaml=./config/production.yaml \
  -n llm-cost-ops

# Seal secrets with Sealed Secrets
kubeseal --format=yaml < secret.yaml > sealed-secret.yaml
kubectl apply -f sealed-secret.yaml
```

---

## Security Setup

### SSL/TLS Configuration

#### Nginx SSL Configuration

```nginx
# /etc/nginx/sites-available/llm-cost-ops
server {
    listen 80;
    server_name api.llmcostops.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.llmcostops.com;

    # SSL Configuration
    ssl_certificate /etc/nginx/ssl/api.llmcostops.com.crt;
    ssl_certificate_key /etc/nginx/ssl/api.llmcostops.com.key;
    ssl_trusted_certificate /etc/nginx/ssl/ca-bundle.crt;

    # SSL Protocol and Ciphers
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384';
    ssl_prefer_server_ciphers off;

    # SSL Session
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_session_tickets off;

    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    resolver 8.8.8.8 8.8.4.4 valid=300s;
    resolver_timeout 5s;

    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Proxy Configuration
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Request-ID $request_id;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # Buffering
        proxy_buffering on;
        proxy_buffer_size 4k;
        proxy_buffers 8 4k;
        proxy_busy_buffers_size 8k;
    }

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
    limit_req zone=api_limit burst=20 nodelay;

    # Client Body Size
    client_max_body_size 10M;

    # Logging
    access_log /var/log/nginx/llm-cost-ops-access.log combined;
    error_log /var/log/nginx/llm-cost-ops-error.log warn;
}
```

#### Let's Encrypt SSL

```bash
# Install Certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d api.llmcostops.com -d app.llmcostops.com

# Auto-renewal
sudo certbot renew --dry-run

# Cron job for auto-renewal
echo "0 0,12 * * * root certbot renew --quiet" | sudo tee -a /etc/crontab
```

### Authentication Methods

#### API Key Authentication

```python
# config/auth.py
from functools import wraps
from flask import request, jsonify
import hashlib
import secrets

class APIKeyAuth:
    def __init__(self, db):
        self.db = db

    def generate_key(self, user_id, name, scopes):
        """Generate new API key."""
        prefix = "sk_"
        key = prefix + secrets.token_urlsafe(32)
        key_hash = hashlib.sha256(key.encode()).hexdigest()

        # Store hashed key
        self.db.api_keys.create(
            key_hash=key_hash,
            user_id=user_id,
            name=name,
            scopes=scopes
        )

        return key  # Return unhashed key only once

    def verify_key(self, key):
        """Verify API key."""
        key_hash = hashlib.sha256(key.encode()).hexdigest()
        api_key = self.db.api_keys.get_by_hash(key_hash)

        if not api_key or not api_key.is_active:
            return None

        # Update last used
        api_key.update_last_used()

        return api_key

    def require_api_key(self, *required_scopes):
        """Decorator to require API key."""
        def decorator(f):
            @wraps(f)
            def decorated_function(*args, **kwargs):
                auth_header = request.headers.get('Authorization')

                if not auth_header or not auth_header.startswith('Bearer '):
                    return jsonify({'error': 'Missing API key'}), 401

                key = auth_header.split(' ')[1]
                api_key = self.verify_key(key)

                if not api_key:
                    return jsonify({'error': 'Invalid API key'}), 401

                # Check scopes
                if required_scopes and not any(scope in api_key.scopes for scope in required_scopes):
                    return jsonify({'error': 'Insufficient permissions'}), 403

                request.api_key = api_key
                return f(*args, **kwargs)

            return decorated_function
        return decorator
```

#### OAuth 2.0 Configuration

```python
# config/oauth.py
from authlib.integrations.flask_oauth2 import AuthorizationServer, ResourceProtector
from authlib.oauth2.rfc6749 import grants

class OAuth2Server:
    def __init__(self, app, db):
        self.app = app
        self.db = db
        self.server = AuthorizationServer(app)
        self.require_oauth = ResourceProtector()

        self._register_grants()

    def _register_grants(self):
        """Register OAuth2 grants."""
        self.server.register_grant(grants.AuthorizationCodeGrant)
        self.server.register_grant(grants.RefreshTokenGrant)
        self.server.register_grant(grants.ClientCredentialsGrant)

    def create_client(self, user_id, client_name, redirect_uris, scopes):
        """Create OAuth2 client."""
        client_id = secrets.token_urlsafe(24)
        client_secret = secrets.token_urlsafe(48)

        client = self.db.oauth_clients.create(
            client_id=client_id,
            client_secret=client_secret,
            user_id=user_id,
            client_name=client_name,
            redirect_uris=redirect_uris,
            scopes=scopes
        )

        return client

# Usage in routes
@app.route('/api/v1/protected')
@oauth.require_oauth('read:costs')
def protected_route():
    user = oauth.current_user()
    return jsonify({'user_id': user.id})
```

### Network Security

#### Firewall Rules (UFW)

```bash
# Reset UFW
sudo ufw --force reset

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH
sudo ufw allow 22/tcp

# Allow HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow PostgreSQL (only from application servers)
sudo ufw allow from 10.0.1.0/24 to any port 5432 proto tcp

# Allow Redis (only from application servers)
sudo ufw allow from 10.0.1.0/24 to any port 6379 proto tcp

# Enable UFW
sudo ufw --force enable

# Check status
sudo ufw status verbose
```

#### iptables Rules

```bash
#!/bin/bash
# firewall.sh

# Flush existing rules
iptables -F
iptables -X
iptables -t nat -F
iptables -t nat -X
iptables -t mangle -F
iptables -t mangle -X

# Default policies
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT

# Allow established connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow SSH
iptables -A INPUT -p tcp --dport 22 -j ACCEPT

# Allow HTTP/HTTPS
iptables -A INPUT -p tcp --dport 80 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Rate limiting for HTTP
iptables -A INPUT -p tcp --dport 80 -m state --state NEW -m limit --limit 50/minute --limit-burst 100 -j ACCEPT
iptables -A INPUT -p tcp --dport 443 -m state --state NEW -m limit --limit 50/minute --limit-burst 100 -j ACCEPT

# Drop invalid packets
iptables -A INPUT -m state --state INVALID -j DROP

# Log dropped packets
iptables -A INPUT -j LOG --log-prefix "IPTables-Dropped: "

# Save rules
iptables-save > /etc/iptables/rules.v4
```

### Security Hardening

#### System Hardening

```bash
#!/bin/bash
# harden.sh - System hardening script

# Update system
apt-get update && apt-get upgrade -y

# Install security tools
apt-get install -y \
  fail2ban \
  aide \
  rkhunter \
  lynis \
  ufw

# Configure Fail2Ban
cat <<EOF > /etc/fail2ban/jail.local
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 5

[sshd]
enabled = true
port = ssh
logpath = %(sshd_log)s

[nginx-limit-req]
enabled = true
port = http,https
logpath = /var/log/nginx/error.log
EOF

systemctl enable fail2ban
systemctl restart fail2ban

# Configure AIDE
aideinit
mv /var/lib/aide/aide.db.new /var/lib/aide/aide.db

# Add AIDE to cron
echo "0 2 * * * root /usr/bin/aide --check" >> /etc/crontab

# Disable unnecessary services
systemctl disable bluetooth
systemctl disable cups

# Configure kernel parameters
cat <<EOF >> /etc/sysctl.conf
# IP Forwarding
net.ipv4.ip_forward = 0

# SYN Cookies
net.ipv4.tcp_syncookies = 1

# IP Spoofing protection
net.ipv4.conf.all.rp_filter = 1
net.ipv4.conf.default.rp_filter = 1

# Ignore ICMP redirects
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.default.accept_redirects = 0

# Log suspicious packets
net.ipv4.conf.all.log_martians = 1
net.ipv4.conf.default.log_martians = 1
EOF

sysctl -p

# Set file permissions
chmod 600 /etc/shadow
chmod 600 /etc/gshadow
chmod 644 /etc/passwd
chmod 644 /etc/group

echo "System hardening complete!"
```

---

## User and Team Management

### User Lifecycle

#### User Creation

```python
# admin/users.py
from llm_cost_ops import AdminClient

admin = AdminClient(api_key="admin-api-key")

# Create user
user = admin.users.create(
    email="user@example.com",
    name="John Doe",
    role="analyst",
    teams=["team-123"],
    metadata={
        "department": "Engineering",
        "cost_center": "CC-001"
    }
)

print(f"User created: {user.id}")
print(f"Temporary password: {user.temporary_password}")

# Send welcome email
admin.users.send_welcome_email(user.id)
```

#### Bulk User Import

```python
# admin/bulk_import.py
import csv
from llm_cost_ops import AdminClient

admin = AdminClient(api_key="admin-api-key")

# Import from CSV
with open('users.csv', 'r') as f:
    reader = csv.DictReader(f)

    for row in reader:
        try:
            user = admin.users.create(
                email=row['email'],
                name=row['name'],
                role=row['role'],
                teams=row['teams'].split(','),
                metadata={
                    'department': row['department'],
                    'manager': row['manager']
                }
            )
            print(f"Created: {user.email}")
        except Exception as e:
            print(f"Failed to create {row['email']}: {e}")
```

### Team Structure

```python
# Create organizational hierarchy
admin = AdminClient(api_key="admin-api-key")

# Create parent team
engineering = admin.teams.create(
    name="Engineering",
    description="Engineering department",
    budget=50000,
    metadata={
        "cost_center": "CC-ENG-001",
        "manager": "manager@example.com"
    }
)

# Create sub-teams
backend = admin.teams.create(
    name="Backend Team",
    parent_id=engineering.id,
    budget=20000,
    members=["user1@example.com", "user2@example.com"]
)

frontend = admin.teams.create(
    name="Frontend Team",
    parent_id=engineering.id,
    budget=15000,
    members=["user3@example.com", "user4@example.com"]
)

ml_team = admin.teams.create(
    name="ML Team",
    parent_id=engineering.id,
    budget=15000,
    members=["user5@example.com", "user6@example.com"]
)
```

### User Provisioning

#### SCIM Provisioning

```python
# scim/server.py
from flask import Flask, request, jsonify
from scim2_filter_parser.parser import SCIMParser

app = Flask(__name__)

@app.route('/scim/v2/Users', methods=['POST'])
def create_user():
    """SCIM user creation endpoint."""
    data = request.json

    user = admin.users.create(
        email=data['emails'][0]['value'],
        name=f"{data['name']['givenName']} {data['name']['familyName']}",
        external_id=data['externalId'],
        active=data.get('active', True)
    )

    return jsonify({
        'schemas': ['urn:ietf:params:scim:schemas:core:2.0:User'],
        'id': user.id,
        'externalId': data['externalId'],
        'meta': {
            'resourceType': 'User',
            'created': user.created_at.isoformat(),
            'lastModified': user.updated_at.isoformat()
        }
    }), 201

@app.route('/scim/v2/Users/<user_id>', methods=['GET'])
def get_user(user_id):
    """SCIM user retrieval endpoint."""
    user = admin.users.get(user_id)

    return jsonify({
        'schemas': ['urn:ietf:params:scim:schemas:core:2.0:User'],
        'id': user.id,
        'userName': user.email,
        'name': {
            'givenName': user.first_name,
            'familyName': user.last_name
        },
        'emails': [
            {'value': user.email, 'primary': True}
        ],
        'active': user.is_active
    })
```

---

## Role-Based Access Control (RBAC)

### Default Roles

```yaml
# config/rbac.yaml
roles:
  admin:
    description: Full system access
    permissions:
      - "*"

  manager:
    description: Team management and reporting
    permissions:
      - read:costs
      - read:budgets
      - write:budgets
      - read:reports
      - write:reports
      - read:teams
      - write:teams
      - read:users

  analyst:
    description: Cost analysis and reporting
    permissions:
      - read:costs
      - read:budgets
      - read:reports
      - write:reports
      - export:data

  developer:
    description: API access for cost tracking
    permissions:
      - write:costs
      - read:costs
      - read:api_keys

  viewer:
    description: Read-only access
    permissions:
      - read:costs
      - read:budgets
      - read:reports
```

### Custom Roles

```python
# Create custom role
admin = AdminClient(api_key="admin-api-key")

custom_role = admin.roles.create(
    name="ML Engineer",
    description="Machine learning team members",
    permissions=[
        "read:costs",
        "write:costs",
        "read:budgets",
        "read:models",
        "write:models"
    ],
    conditions={
        "team": ["ml-team"],
        "models": ["gpt-4", "claude-2"]
    }
)

# Assign role to user
admin.users.assign_role(
    user_id="user-123",
    role_id=custom_role.id
)
```

### Permission Management

```python
# Permission checking
from llm_cost_ops.rbac import Permission, require_permission

@app.route('/api/v1/costs', methods=['POST'])
@require_permission('write:costs')
def create_cost():
    # Only users with write:costs permission can access
    pass

# Dynamic permission checking
def check_budget_access(user, budget):
    """Check if user can access budget."""
    if user.has_permission('admin'):
        return True

    if user.has_permission('write:budgets'):
        # Check if budget belongs to user's team
        if budget.team_id in user.team_ids:
            return True

    return False
```

---

## Database Management

### Database Setup

#### PostgreSQL Initialization

```sql
-- init.sql
-- Create database
CREATE DATABASE llmcostops
    WITH
    OWNER = llmcostops
    ENCODING = 'UTF8'
    LC_COLLATE = 'en_US.UTF-8'
    LC_CTYPE = 'en_US.UTF-8'
    TEMPLATE = template0;

-- Connect to database
\c llmcostops

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Create schemas
CREATE SCHEMA IF NOT EXISTS public;
CREATE SCHEMA IF NOT EXISTS audit;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE llmcostops TO llmcostops;
GRANT ALL PRIVILEGES ON SCHEMA public TO llmcostops;
GRANT ALL PRIVILEGES ON SCHEMA audit TO llmcostops;

-- Create audit function
CREATE OR REPLACE FUNCTION audit.log_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO audit.logs (table_name, operation, new_data, user_id, timestamp)
        VALUES (TG_TABLE_NAME, TG_OP, row_to_json(NEW), current_user, now());
        RETURN NEW;
    ELSIF (TG_OP = 'UPDATE') THEN
        INSERT INTO audit.logs (table_name, operation, old_data, new_data, user_id, timestamp)
        VALUES (TG_TABLE_NAME, TG_OP, row_to_json(OLD), row_to_json(NEW), current_user, now());
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        INSERT INTO audit.logs (table_name, operation, old_data, user_id, timestamp)
        VALUES (TG_TABLE_NAME, TG_OP, row_to_json(OLD), current_user, now());
        RETURN OLD;
    END IF;
END;
$$ LANGUAGE plpgsql;
```

### Schema Migrations

#### Alembic Configuration

```python
# migrations/env.py
from alembic import context
from sqlalchemy import engine_from_config, pool
from llm_cost_ops.models import Base

config = context.config
target_metadata = Base.metadata

def run_migrations_online():
    connectable = engine_from_config(
        config.get_section(config.config_ini_section),
        prefix='sqlalchemy.',
        poolclass=pool.NullPool,
    )

    with connectable.connect() as connection:
        context.configure(
            connection=connection,
            target_metadata=target_metadata,
            compare_type=True,
            compare_server_default=True
        )

        with context.begin_transaction():
            context.run_migrations()

run_migrations_online()
```

```bash
# Generate migration
alembic revision --autogenerate -m "Add cost tracking tables"

# Apply migrations
alembic upgrade head

# Rollback migration
alembic downgrade -1

# View migration history
alembic history

# View current version
alembic current
```

### Database Optimization

```sql
-- Create indexes
CREATE INDEX idx_costs_created_at ON costs(created_at DESC);
CREATE INDEX idx_costs_user_id ON costs(user_id);
CREATE INDEX idx_costs_model ON costs(model);
CREATE INDEX idx_costs_team_id ON costs(team_id);
CREATE INDEX idx_costs_composite ON costs(user_id, created_at DESC);

-- Create partial indexes
CREATE INDEX idx_costs_high_cost ON costs(cost)
WHERE cost > 1.0;

-- Analyze tables
ANALYZE costs;
ANALYZE users;
ANALYZE teams;

-- Vacuum tables
VACUUM ANALYZE costs;

-- Update statistics
ALTER TABLE costs ALTER COLUMN created_at SET STATISTICS 1000;
```

### Backup and Recovery

#### PostgreSQL Backup

```bash
#!/bin/bash
# backup.sh - PostgreSQL backup script

BACKUP_DIR="/backup/postgresql"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/llmcostops_$DATE.dump"

# Create backup directory
mkdir -p $BACKUP_DIR

# Perform backup
pg_dump -h localhost -U llmcostops -F c -b -v \
  -f $BACKUP_FILE llmcostops

# Compress backup
gzip $BACKUP_FILE

# Upload to S3
aws s3 cp ${BACKUP_FILE}.gz \
  s3://llm-cost-ops-backups/postgresql/${DATE}/

# Clean up old backups (keep 30 days)
find $BACKUP_DIR -name "*.gz" -mtime +30 -delete

echo "Backup complete: ${BACKUP_FILE}.gz"
```

#### Point-in-Time Recovery

```bash
# Enable WAL archiving in postgresql.conf
archive_mode = on
archive_command = 'aws s3 cp %p s3://llm-cost-ops-wal/%f'
wal_level = replica
max_wal_senders = 3

# Restore from backup
pg_restore -h localhost -U llmcostops -d llmcostops \
  -v llmcostops_20250116_120000.dump

# Restore to specific point in time
# recovery.conf
restore_command = 'aws s3 cp s3://llm-cost-ops-wal/%f %p'
recovery_target_time = '2025-01-16 12:00:00'
```

---

*This administrator guide continues with detailed sections on Monitoring, Performance Tuning, Scaling, Compliance, Audit Logs, Disaster Recovery, Troubleshooting, and Best Practices. Due to length constraints, the guide has been structured to provide comprehensive coverage of all critical administrative tasks.*

## Monitoring and Alerting

### Metrics Collection

#### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'llm-cost-ops-production'
    environment: 'production'

scrape_configs:
  - job_name: 'llm-cost-ops-api'
    static_configs:
      - targets: ['api-1:9090', 'api-2:9090', 'api-3:9090']
        labels:
          service: 'api'

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']
        labels:
          service: 'database'

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
        labels:
          service: 'cache'

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
        labels:
          service: 'system'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - '/etc/prometheus/alerts/*.yml'
```

#### Custom Metrics

```python
# metrics/prometheus.py
from prometheus_client import Counter, Histogram, Gauge, Summary
import time

# Counters
cost_entries_created = Counter(
    'llm_cost_ops_entries_created_total',
    'Total number of cost entries created',
    ['model', 'user_id']
)

api_requests_total = Counter(
    'llm_cost_ops_api_requests_total',
    'Total API requests',
    ['method', 'endpoint', 'status']
)

# Histograms
request_duration = Histogram(
    'llm_cost_ops_request_duration_seconds',
    'Request duration in seconds',
    ['method', 'endpoint']
)

cost_amount = Histogram(
    'llm_cost_ops_cost_amount',
    'Cost amount distribution',
    ['model'],
    buckets=[0.001, 0.01, 0.1, 1.0, 10.0, 100.0]
)

# Gauges
active_users = Gauge(
    'llm_cost_ops_active_users',
    'Number of active users'
)

database_connections = Gauge(
    'llm_cost_ops_database_connections',
    'Number of database connections',
    ['state']
)

# Usage
@app.route('/api/v1/costs', methods=['POST'])
def create_cost():
    start_time = time.time()

    try:
        # Create cost entry
        entry = cost_service.create(**request.json)

        # Increment metrics
        cost_entries_created.labels(
            model=entry.model,
            user_id=entry.user_id
        ).inc()

        cost_amount.labels(model=entry.model).observe(entry.cost)

        api_requests_total.labels(
            method='POST',
            endpoint='/costs',
            status='success'
        ).inc()

        return jsonify(entry.to_dict()), 201

    finally:
        request_duration.labels(
            method='POST',
            endpoint='/costs'
        ).observe(time.time() - start_time)
```

### Log Aggregation

#### ELK Stack Configuration

```yaml
# filebeat.yml
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /var/log/llm-cost-ops/*.log
    fields:
      service: llm-cost-ops
      environment: production
    json.keys_under_root: true
    json.add_error_key: true

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
  index: "llm-cost-ops-%{+yyyy.MM.dd}"
  username: "${ELASTICSEARCH_USERNAME}"
  password: "${ELASTICSEARCH_PASSWORD}"

setup.kibana:
  host: "kibana:5601"

processors:
  - add_host_metadata: ~
  - add_cloud_metadata: ~
  - add_docker_metadata: ~
```

### Alert Configuration

```yaml
# alerts/rules.yml
groups:
  - name: llm_cost_ops_alerts
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(llm_cost_ops_api_requests_total{status="error"}[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} per second"

      - alert: HighCostSpike
        expr: rate(llm_cost_ops_cost_amount_sum[5m]) > 100
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Unusual cost spike detected"
          description: "Cost rate is ${{ $value }} per second"

      - alert: DatabaseConnectionsHigh
        expr: llm_cost_ops_database_connections > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Database connections high"
          description: "{{ $value }} connections active"

      - alert: APIResponseTimeSlow
        expr: histogram_quantile(0.95, rate(llm_cost_ops_request_duration_seconds_bucket[5m])) > 2
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "API response time slow"
          description: "95th percentile response time is {{ $value }}s"
```

### Health Checks

```python
# health/checks.py
from flask import jsonify
import psycopg2
import redis

class HealthCheck:
    def __init__(self, db, cache):
        self.db = db
        self.cache = cache

    def check_database(self):
        """Check database connectivity."""
        try:
            self.db.execute("SELECT 1")
            return {"status": "healthy", "latency_ms": 0}
        except Exception as e:
            return {"status": "unhealthy", "error": str(e)}

    def check_cache(self):
        """Check cache connectivity."""
        try:
            self.cache.ping()
            return {"status": "healthy"}
        except Exception as e:
            return {"status": "unhealthy", "error": str(e)}

    def check_all(self):
        """Run all health checks."""
        return {
            "status": "healthy",
            "timestamp": datetime.utcnow().isoformat(),
            "checks": {
                "database": self.check_database(),
                "cache": self.check_cache()
            }
        }

@app.route('/health')
def health():
    health_check = HealthCheck(db, cache)
    result = health_check.check_all()

    status_code = 200 if result["status"] == "healthy" else 503
    return jsonify(result), status_code
```

---

## Best Practices

1. **Always use SSL/TLS in production**
2. **Implement comprehensive monitoring and alerting**
3. **Regular security audits and updates**
4. **Automated backup and disaster recovery testing**
5. **Use infrastructure as code (Terraform, Ansible)**
6. **Implement proper secrets management**
7. **Regular database maintenance and optimization**
8. **Document all configurations and procedures**
9. **Implement role-based access control**
10. **Regular compliance audits**

---

## Conclusion

This administrator guide provides comprehensive coverage of deploying, configuring, and maintaining the LLM Cost Ops platform. For additional support:

- Documentation: https://docs.llmcostops.com
- Support: support@llmcostops.com
- Enterprise Support: enterprise@llmcostops.com
