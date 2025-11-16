# Docker Security Guide for LLM Cost Ops

**Container Security Best Practices**

Version: 1.0.0
Last Updated: 2025-11-16

---

## Table of Contents

1. [Overview](#overview)
2. [Image Security](#image-security)
3. [Secret Management](#secret-management)
4. [Network Security](#network-security)
5. [Container Hardening](#container-hardening)
6. [RBAC and Access Control](#rbac-and-access-control)
7. [Compliance](#compliance)
8. [Security Scanning](#security-scanning)
9. [Audit and Logging](#audit-and-logging)
10. [Incident Response](#incident-response)
11. [Security Checklist](#security-checklist)

---

## Overview

### Security Principles

The platform follows these security principles:

- **Defense in depth** - Multiple layers of security
- **Least privilege** - Minimal permissions required
- **Secure by default** - Security-first configuration
- **Immutability** - Read-only containers where possible
- **Zero trust** - Verify all connections
- **Encryption** - Data encrypted at rest and in transit

### Threat Model

**Assets:**
- User data in PostgreSQL
- API credentials and tokens
- LLM provider API keys
- Application secrets
- Audit logs

**Threats:**
- Unauthorized access to data
- Container escape
- Network sniffing
- Supply chain attacks
- Denial of service
- Data leaks

---

## Image Security

### Base Image Selection

**Use official images:**

```dockerfile
# Good: Official minimal image
FROM debian:bullseye-slim

# Better: Distroless for runtime
FROM gcr.io/distroless/base-debian11

# Avoid: Unverified images
# FROM random/unknown-image
```

### Multi-Stage Builds

**Minimize attack surface:**

```dockerfile
# Stage 1: Build (large, many tools)
FROM rust:1.75-slim-bullseye AS builder
# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release

# Stage 2: Runtime (minimal)
FROM debian:bullseye-slim
# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates libpq5 libssl1.1 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 1000 app
COPY --from=builder /app/target/release/llm-cost-ops /app/
USER app
CMD ["/app/llm-cost-ops"]
```

### Image Scanning

**Trivy (recommended):**

```bash
# Scan image
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image llm-cost-ops:latest

# Critical and high only
trivy image --severity HIGH,CRITICAL llm-cost-ops:latest

# Exit on vulnerabilities
trivy image --exit-code 1 --severity CRITICAL llm-cost-ops:latest

# Generate report
trivy image -f json -o report.json llm-cost-ops:latest
```

**Snyk:**

```bash
# Scan image
snyk container test llm-cost-ops:latest

# Monitor image
snyk container monitor llm-cost-ops:latest
```

**Docker Scout:**

```bash
# Scan with Docker Scout
docker scout cves llm-cost-ops:latest

# Compare with previous version
docker scout compare llm-cost-ops:v1.0.0 llm-cost-ops:v1.1.0
```

### Image Signing

**Docker Content Trust:**

```bash
# Enable content trust
export DOCKER_CONTENT_TRUST=1

# Push signed image
docker push llm-cost-ops:v1.0.0

# Verify signature on pull
docker pull llm-cost-ops:v1.0.0
```

**Cosign (Sigstore):**

```bash
# Generate key pair
cosign generate-key-pair

# Sign image
cosign sign --key cosign.key llm-cost-ops:v1.0.0

# Verify signature
cosign verify --key cosign.pub llm-cost-ops:v1.0.0
```

### Image Best Practices

1. **Use specific tags** - Avoid `latest`
2. **Scan regularly** - Automated scanning in CI/CD
3. **Update base images** - Regular security patches
4. **Minimize layers** - Fewer attack vectors
5. **Remove unnecessary tools** - Reduce attack surface
6. **Sign images** - Verify authenticity

---

## Secret Management

### Environment Variables (Development Only)

```yaml
# docker-compose.yml (development)
services:
  app:
    environment:
      - JWT_SECRET=${JWT_SECRET}  # From .env file
```

**Never commit secrets:**

```bash
# .gitignore
.env
.env.local
.env.production
secrets/
*.key
*.pem
```

### Docker Secrets (Swarm)

```bash
# Create secret from file
echo "supersecret" | docker secret create postgres_password -

# Use in service
docker service create \
  --name app \
  --secret postgres_password \
  llm-cost-ops:latest

# Access in container
cat /run/secrets/postgres_password
```

**docker-compose.yml:**

```yaml
version: '3.8'

services:
  app:
    image: llm-cost-ops:latest
    secrets:
      - postgres_password
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/postgres_password

secrets:
  postgres_password:
    external: true
```

### Kubernetes Secrets

```bash
# Create secret
kubectl create secret generic app-secrets \
  --from-literal=jwt-secret=$(openssl rand -base64 64) \
  --from-literal=db-password=$(openssl rand -base64 32) \
  -n llm-cost-ops

# Use in pod
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: app
spec:
  containers:
  - name: app
    image: llm-cost-ops:latest
    env:
    - name: JWT_SECRET
      valueFrom:
        secretKeyRef:
          name: app-secrets
          key: jwt-secret
EOF
```

### External Secret Management

**HashiCorp Vault:**

```yaml
# docker-compose.yml
services:
  app:
    image: llm-cost-ops:latest
    environment:
      VAULT_ADDR: http://vault:8200
      VAULT_TOKEN: ${VAULT_TOKEN}
    command: >
      sh -c '
        export JWT_SECRET=$(vault kv get -field=jwt_secret secret/llm-cost-ops)
        /app/llm-cost-ops
      '
```

**AWS Secrets Manager:**

```dockerfile
# Dockerfile
FROM llm-cost-ops:latest
RUN apt-get update && apt-get install -y aws-cli
ENTRYPOINT ["/app/entrypoint.sh"]
```

```bash
# entrypoint.sh
#!/bin/bash
export JWT_SECRET=$(aws secretsmanager get-secret-value \
  --secret-id llm-cost-ops/jwt-secret \
  --query SecretString \
  --output text)
exec /app/llm-cost-ops
```

### Secret Rotation

```bash
# Rotate PostgreSQL password
NEW_PASSWORD=$(openssl rand -base64 32)

# Update secret
kubectl create secret generic postgres-credentials \
  --from-literal=password=$NEW_PASSWORD \
  --dry-run=client -o yaml | kubectl apply -f -

# Restart pods to pick up new secret
kubectl rollout restart deployment/postgres -n llm-cost-ops
```

---

## Network Security

### Network Isolation

**Docker Compose:**

```yaml
networks:
  frontend:
    driver: bridge
  backend:
    driver: bridge
    internal: true  # No internet access

services:
  app:
    networks:
      - frontend
      - backend

  postgres:
    networks:
      - backend  # Not accessible from frontend
```

### Kubernetes Network Policies

**Deny all ingress:**

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
  namespace: llm-cost-ops
spec:
  podSelector: {}
  policyTypes:
  - Ingress
```

**Allow specific traffic:**

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: app-network-policy
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
  - to:  # DNS
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: UDP
      port: 53
```

### TLS/SSL Configuration

**Nginx reverse proxy:**

```nginx
server {
    listen 443 ssl http2;
    server_name api.llm-cost-ops.com;

    ssl_certificate /etc/nginx/ssl/fullchain.pem;
    ssl_certificate_key /etc/nginx/ssl/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # HSTS
    add_header Strict-Transport-Security "max-age=31536000" always;

    location / {
        proxy_pass http://app:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Service Mesh (Advanced)

**Istio mTLS:**

```yaml
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default
  namespace: llm-cost-ops
spec:
  mtls:
    mode: STRICT  # Enforce mTLS
```

---

## Container Hardening

### Run as Non-Root

**Dockerfile:**

```dockerfile
FROM debian:bullseye-slim

# Create non-root user
RUN groupadd -r app && useradd -r -g app -u 1000 app

# Create app directory with correct ownership
RUN mkdir -p /app && chown -R app:app /app

# Switch to non-root user
USER app

WORKDIR /app
COPY --chown=app:app . .

CMD ["/app/llm-cost-ops"]
```

**docker-compose.yml:**

```yaml
services:
  app:
    user: "1000:1000"
```

### Read-Only Filesystem

**docker-compose.yml:**

```yaml
services:
  app:
    read_only: true
    tmpfs:
      - /tmp:size=100M,mode=1777
      - /var/run:size=10M,mode=755
```

**Kubernetes:**

```yaml
securityContext:
  readOnlyRootFilesystem: true

volumeMounts:
- name: tmp
  mountPath: /tmp
- name: var-run
  mountPath: /var/run

volumes:
- name: tmp
  emptyDir: {}
- name: var-run
  emptyDir: {}
```

### Drop Capabilities

**docker-compose.yml:**

```yaml
services:
  app:
    cap_drop:
      - ALL
    cap_add:
      - NET_BIND_SERVICE  # Only if binding to ports < 1024
```

**Kubernetes:**

```yaml
securityContext:
  allowPrivilegeEscalation: false
  capabilities:
    drop:
    - ALL
```

### Security Options

**docker-compose.yml:**

```yaml
services:
  app:
    security_opt:
      - no-new-privileges:true
      - apparmor=docker-default
```

### Resource Limits

```yaml
services:
  app:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
          pids: 100  # Prevent fork bombs
```

---

## RBAC and Access Control

### Kubernetes RBAC

**ServiceAccount:**

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: llm-cost-ops
  namespace: llm-cost-ops
```

**Role:**

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
  resourceNames: ["llm-cost-ops-config", "app-secrets"]
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list"]
```

**RoleBinding:**

```yaml
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

### Authentication

**Basic Auth (Nginx):**

```bash
# Create htpasswd file
htpasswd -c .htpasswd admin

# Use in nginx
location /admin {
    auth_basic "Restricted Area";
    auth_basic_user_file /etc/nginx/.htpasswd;
}
```

**OAuth2 Proxy:**

```yaml
services:
  oauth2-proxy:
    image: quay.io/oauth2-proxy/oauth2-proxy:latest
    command:
      - --provider=google
      - --email-domain=example.com
      - --upstream=http://app:8080
      - --http-address=0.0.0.0:4180
    environment:
      OAUTH2_PROXY_CLIENT_ID: your-client-id
      OAUTH2_PROXY_CLIENT_SECRET: your-client-secret
      OAUTH2_PROXY_COOKIE_SECRET: cookie-secret
```

---

## Compliance

### SOC 2 Requirements

**Audit logging:**

```yaml
services:
  app:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "10"
        labels: "app,environment"
        env: "USER,HOSTNAME"
```

**Access controls:**
- RBAC configured
- Secrets encrypted
- Network policies enforced
- Regular access reviews

**Data encryption:**
- TLS for data in transit
- Encrypted volumes for data at rest
- Key rotation procedures

### HIPAA Compliance

**Required:**
- Encrypted storage (dm-crypt, LUKS)
- Encrypted network (TLS 1.2+)
- Access audit logs
- BAA with cloud provider
- Physical security controls

**Storage encryption:**

```yaml
volumes:
  postgres-data:
    driver: local
    driver_opts:
      type: "none"
      o: "bind,encryption=aes-xts-plain64"
      device: "/encrypted/postgres"
```

### GDPR Compliance

**Data protection:**
- Right to erasure (data deletion)
- Data portability (export features)
- Consent management
- Data retention policies
- Privacy by design

**Implementation:**

```sql
-- Data retention policy
DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '2 years';

-- User data export
SELECT * FROM users WHERE user_id = ?
INTO OUTFILE '/tmp/user_data.json';

-- User data deletion
DELETE FROM users WHERE user_id = ?;
DELETE FROM usage WHERE user_id = ?;
DELETE FROM costs WHERE user_id = ?;
```

---

## Security Scanning

### Container Scanning

**CI/CD Pipeline:**

```yaml
# .github/workflows/security.yml
name: Security Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build image
        run: docker build -t llm-cost-ops:${{ github.sha }} .

      - name: Run Trivy
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: llm-cost-ops:${{ github.sha }}
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Upload results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'
```

### Runtime Security

**Falco (Kubernetes):**

```yaml
# Install Falco
helm repo add falcosecurity https://falcosecurity.github.io/charts
helm install falco falcosecurity/falco \
  --namespace falco \
  --create-namespace

# Custom rules
- rule: Unauthorized Process
  desc: Detect unauthorized process execution
  condition: spawned_process and container.id != "" and not proc.name in (allowed_processes)
  output: Unauthorized process started (user=%user.name command=%proc.cmdline container=%container.id)
  priority: WARNING
```

---

## Audit and Logging

### Application Logging

```rust
// Structured logging
use tracing::{info, warn, error};

#[instrument]
async fn handle_request(req: Request) -> Result<Response> {
    info!(
        user_id = %req.user_id,
        action = "api_request",
        path = %req.path,
        "Processing request"
    );

    // ... process request

    info!(
        user_id = %req.user_id,
        action = "api_response",
        status = 200,
        duration_ms = 45,
        "Request completed"
    );

    Ok(response)
}
```

### Centralized Logging

**ELK Stack:**

```yaml
services:
  elasticsearch:
    image: elasticsearch:8.11.0

  logstash:
    image: logstash:8.11.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf

  kibana:
    image: kibana:8.11.0
```

**Loki + Grafana:**

```yaml
services:
  loki:
    image: grafana/loki:2.9.0

  promtail:
    image: grafana/promtail:2.9.0
    volumes:
      - /var/log:/var/log

  grafana:
    image: grafana/grafana:10.2.0
```

---

## Incident Response

### Security Incident Plan

1. **Detection** - Monitor alerts
2. **Containment** - Isolate affected systems
3. **Eradication** - Remove threat
4. **Recovery** - Restore services
5. **Lessons Learned** - Post-incident review

### Incident Response Commands

```bash
# 1. Isolate container
docker network disconnect llm-cost-ops-network llm-cost-ops-app

# 2. Inspect for compromise
docker inspect llm-cost-ops-app
docker diff llm-cost-ops-app

# 3. Capture forensics
docker export llm-cost-ops-app > forensics.tar
docker logs llm-cost-ops-app > forensics.log

# 4. Rebuild and redeploy
docker compose down
docker compose pull
docker compose up -d
```

---

## Security Checklist

**Image Security:**
- [ ] Use official base images
- [ ] Multi-stage builds
- [ ] No secrets in images
- [ ] Regular vulnerability scans
- [ ] Image signing enabled

**Secret Management:**
- [ ] Secrets not in environment variables
- [ ] External secret management configured
- [ ] Regular secret rotation
- [ ] Encrypted secret storage

**Network Security:**
- [ ] Network policies enforced
- [ ] TLS/SSL configured
- [ ] Internal networks isolated
- [ ] Rate limiting enabled

**Container Hardening:**
- [ ] Run as non-root
- [ ] Read-only filesystem
- [ ] Capabilities dropped
- [ ] Resource limits set

**Access Control:**
- [ ] RBAC configured
- [ ] Least privilege applied
- [ ] Authentication required
- [ ] Regular access reviews

**Compliance:**
- [ ] Audit logging enabled
- [ ] Data encryption configured
- [ ] Retention policies defined
- [ ] Compliance requirements met

**Monitoring:**
- [ ] Security alerts configured
- [ ] Log aggregation enabled
- [ ] Anomaly detection active
- [ ] Incident response plan documented

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0
