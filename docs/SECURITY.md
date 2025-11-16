# Security Policy

## Supported Versions

We actively support the following versions of LLM-CostOps with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

We take the security of LLM-CostOps seriously. If you discover a security vulnerability, please follow these steps:

### 1. Submit a Report

Send details to: **[security@example.com](mailto:security@example.com)**

Include in your report:

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact and severity
- Any suggested fixes or mitigations
- Your contact information

### 2. Response Timeline

- **Initial Response**: Within 48 hours of submission
- **Status Update**: Within 7 days with assessment and plan
- **Fix Timeline**: Varies by severity (see below)
- **Public Disclosure**: After fix is released and deployed

### 3. Severity Levels

We use the following severity classifications:

#### Critical (CVSS 9.0-10.0)

- **Response Time**: 24-48 hours
- **Fix Timeline**: 7-14 days
- **Examples**: Remote code execution, authentication bypass, data breach

#### High (CVSS 7.0-8.9)

- **Response Time**: 48-72 hours
- **Fix Timeline**: 14-30 days
- **Examples**: Privilege escalation, SQL injection, XSS

#### Medium (CVSS 4.0-6.9)

- **Response Time**: 1 week
- **Fix Timeline**: 30-60 days
- **Examples**: Information disclosure, CSRF, rate limiting bypass

#### Low (CVSS 0.1-3.9)

- **Response Time**: 2 weeks
- **Fix Timeline**: Next regular release
- **Examples**: Minor information leaks, outdated dependencies

### 4. What to Expect

After reporting a vulnerability:

1. **Acknowledgment**: We'll confirm receipt of your report
2. **Assessment**: We'll evaluate the severity and impact
3. **Communication**: We'll keep you updated on progress
4. **Fix Development**: We'll develop and test a fix
5. **Release**: We'll release a patch and security advisory
6. **Credit**: We'll acknowledge your contribution (unless you prefer to remain anonymous)

## Security Best Practices

### For Users

#### Database Security

```bash
# Use strong passwords
export DB_PASSWORD=$(openssl rand -base64 32)

# Enable SSL/TLS for PostgreSQL connections
export DATABASE_URL="postgresql://user:pass@localhost/db?sslmode=require"

# Restrict database access
# PostgreSQL pg_hba.conf:
hostssl llm_costops costops 10.0.0.0/8 scram-sha-256
```

#### API Key Management

```bash
# Generate secure API keys
llm-costops api-key generate --scopes read,write

# Rotate keys regularly (every 90 days recommended)
llm-costops api-key rotate --key-id <key-id>

# Store keys securely
# Use environment variables or secret management systems
export COSTOPS_API_KEY=$(cat /run/secrets/api_key)
```

#### Network Security

```yaml
# Enable TLS for API endpoints
api:
  tls:
    enabled: true
    cert_path: /etc/llm-costops/certs/server.crt
    key_path: /etc/llm-costops/certs/server.key
    min_tls_version: "1.3"
```

#### Authentication

```yaml
# Enable JWT authentication
auth:
  jwt:
    enabled: true
    secret_key_path: /run/secrets/jwt_secret
    token_expiry: 3600  # 1 hour
    refresh_token_expiry: 604800  # 7 days
```

#### RBAC Configuration

```yaml
# Implement least privilege
rbac:
  roles:
    - name: analyst
      permissions:
        - cost:read
        - usage:read
        - forecast:read
    - name: admin
      permissions:
        - "*"
```

### For Kubernetes Deployments

#### Security Context

```yaml
# k8s/base/deployment.yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  runAsGroup: 1000
  fsGroup: 1000
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL
```

#### Network Policies

```yaml
# k8s/base/network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: llm-costops-network-policy
spec:
  podSelector:
    matchLabels:
      app: llm-costops
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              name: ingress
      ports:
        - protocol: TCP
          port: 3000
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              name: database
      ports:
        - protocol: TCP
          port: 5432
```

#### Secrets Management

```yaml
# Use Kubernetes Secrets or external secret managers
apiVersion: v1
kind: Secret
metadata:
  name: llm-costops-secrets
type: Opaque
stringData:
  database-password: "${DB_PASSWORD}"
  jwt-secret: "${JWT_SECRET}"
  api-key-salt: "${API_KEY_SALT}"
```

### Data Protection

#### Encryption at Rest

```yaml
# Encrypt sensitive database fields
storage:
  encryption:
    enabled: true
    key_path: /run/secrets/encryption_key
    algorithm: AES-256-GCM
```

#### Encryption in Transit

```yaml
# Enforce TLS for all connections
tls:
  enabled: true
  min_version: "1.3"
  cipher_suites:
    - TLS_AES_256_GCM_SHA384
    - TLS_AES_128_GCM_SHA256
    - TLS_CHACHA20_POLY1305_SHA256
```

#### Data Retention

```sql
-- Automatic data retention policies
SELECT add_retention_policy('usage_records', INTERVAL '2 years');
SELECT add_retention_policy('cost_records', INTERVAL '7 years');  -- For compliance
```

### Audit Logging

```yaml
# Enable comprehensive audit logging
audit:
  enabled: true
  log_level: info
  events:
    - api_access
    - authentication
    - authorization
    - data_modification
    - configuration_changes
  storage:
    type: file
    path: /var/log/llm-costops/audit.log
    rotation: daily
    retention: 90  # days
```

### Rate Limiting

```yaml
# Prevent abuse and DoS attacks
rate_limiting:
  enabled: true
  global:
    requests_per_minute: 1000
  per_api_key:
    requests_per_minute: 100
  per_ip:
    requests_per_minute: 60
```

## Security Checklist for Deployments

### Pre-Deployment

- [ ] Update to latest version
- [ ] Review security configurations
- [ ] Generate secure credentials
- [ ] Configure TLS certificates
- [ ] Set up network policies
- [ ] Enable audit logging
- [ ] Configure rate limiting
- [ ] Review RBAC policies

### Post-Deployment

- [ ] Verify TLS is working
- [ ] Test authentication
- [ ] Review audit logs
- [ ] Monitor for anomalies
- [ ] Test backup and recovery
- [ ] Document security procedures
- [ ] Train team on security practices

### Regular Maintenance

- [ ] Rotate credentials (every 90 days)
- [ ] Update dependencies (monthly)
- [ ] Review audit logs (weekly)
- [ ] Scan for vulnerabilities (monthly)
- [ ] Test disaster recovery (quarterly)
- [ ] Review and update policies (annually)

## Known Security Considerations

### API Rate Limiting

LLM-CostOps includes built-in rate limiting to prevent abuse. Ensure it's properly configured for your deployment:

```yaml
rate_limiting:
  enabled: true
  strategy: token_bucket
  redis_url: redis://localhost:6379
```

### Database Connection Security

Always use connection pooling with appropriate limits to prevent connection exhaustion:

```yaml
database:
  max_connections: 20
  min_connections: 5
  connection_timeout: 30
  idle_timeout: 600
```

### Secrets in Configuration

Never commit secrets to version control. Use environment variables or secret management systems:

```bash
# Bad
database_url: "postgresql://user:password@localhost/db"

# Good
database_url: "${DATABASE_URL}"
```

## Vulnerability Disclosure Policy

### Coordinated Disclosure

We follow a coordinated disclosure process:

1. **Private Reporting**: Report vulnerability privately
2. **Acknowledgment**: We confirm receipt and begin investigation
3. **Fix Development**: We develop and test a fix
4. **Fix Release**: We release a patched version
5. **Public Disclosure**: We publish a security advisory
6. **Recognition**: We credit the reporter (optional)

### Timeline

- **Day 0**: Vulnerability reported
- **Day 1-2**: Initial assessment and acknowledgment
- **Day 3-7**: Detailed analysis and fix development
- **Day 7-30**: Testing and release preparation
- **Day 30**: Public disclosure (may be extended for complex issues)

### Security Advisories

Published security advisories will include:

- CVE identifier (when applicable)
- Affected versions
- Severity rating (CVSS score)
- Description of the vulnerability
- Impact assessment
- Mitigation steps
- Fixed version
- Credit to reporter

## Security Team

Our security team is responsible for:

- Reviewing security reports
- Coordinating vulnerability responses
- Maintaining security documentation
- Conducting security audits
- Managing security releases

**Contact**: [security@example.com](mailto:security@example.com)

## Hall of Fame

We recognize security researchers who have responsibly disclosed vulnerabilities:

<!-- Security researchers who have contributed will be listed here -->

To be listed, you must:

- Report a valid security vulnerability
- Follow responsible disclosure practices
- Provide sufficient detail to reproduce the issue
- Agree to public acknowledgment

## Additional Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Kubernetes Security Best Practices](https://kubernetes.io/docs/concepts/security/)

## Changes to This Policy

We may update this security policy from time to time. We will notify the community of any material changes through:

- GitHub repository announcements
- Security mailing list
- Project blog

Last updated: 2025-01-15
