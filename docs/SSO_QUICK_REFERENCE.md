# Enterprise SSO Quick Reference Guide

**Version:** 1.0.0
**Date:** 2025-11-15

---

## Document Navigation

1. **[SSO_IMPLEMENTATION_SUMMARY.md](./SSO_IMPLEMENTATION_SUMMARY.md)** - Executive summary, roadmap, and key decisions
2. **[SSO_ENTERPRISE_RESEARCH.md](./SSO_ENTERPRISE_RESEARCH.md)** - Complete technical research (200+ pages)

---

## Quick Facts

### Protocols Required
- **SAML 2.0** - Enterprise SSO standard (Priority: P0)
- **OAuth 2.0 + OpenID Connect** - Modern cloud auth (Priority: P0)
- **SCIM 2.0** - User provisioning (Priority: P1)

### Identity Provider Integrations (Priority Order)
1. Okta (SAML + OIDC + SCIM) - Market leader
2. Azure AD / Entra ID (SAML + OIDC) - Microsoft ecosystem
3. Google Workspace (OIDC) - Growing adoption
4. Auth0 (OIDC) - Developer favorite
5. GitHub Enterprise (OAuth 2.0) - Developer identity

### Timeline
- **Phase 1:** Core SSO (4 weeks)
- **Phase 2:** Multi-IdP (4 weeks)
- **Phase 3:** Enterprise features (4 weeks)
- **Phase 4:** Testing & hardening (4 weeks)
- **Total:** 16 weeks (4 months)

---

## Essential Security Checklist

### CSRF Protection
- [ ] State parameter generation (128+ bits entropy)
- [ ] Server-side state storage
- [ ] State expiration (10 minutes)
- [ ] Constant-time comparison

### Token Validation
- [ ] JWT signature verification (RSA/ECDSA)
- [ ] JWKS fetching and caching
- [ ] Expiration validation (exp, nbf, iat)
- [ ] Issuer and audience validation
- [ ] Nonce validation (OIDC)

### Redirect URI Validation
- [ ] Exact match only (no wildcards)
- [ ] HTTPS enforcement (production)
- [ ] Whitelist management

### Replay Prevention
- [ ] SAML assertion ID tracking
- [ ] Nonce validation (OIDC)
- [ ] Timestamp validation

---

## Attribute Mapping Examples

### Standard Mappings

```yaml
# SAML/OIDC → LLM-CostOps
email          → email (primary identifier)
firstName      → first_name
lastName       → last_name
department     → department
organization   → organization_id
groups[]       → roles[] (via mapping)
```

### Group to Role Mappings

```yaml
# IdP Group              → LLM-CostOps Role
LLM-CostOps-Admins      → org_admin
LLM-CostOps-Billing     → billing
LLM-CostOps-ReadOnly    → read_only
LLM-CostOps-Auditors    → auditor
```

---

## SAML 2.0 Checklist

### Service Provider (SP) Configuration
- [ ] Entity ID: `llm-cost-ops`
- [ ] ACS URL: `https://llm-cost-ops.example.com/saml/acs`
- [ ] Single Logout URL: `https://llm-cost-ops.example.com/saml/sls`
- [ ] NameID format: Email address
- [ ] Sign requests: Yes
- [ ] Want assertions signed: Yes

### Required Features
- [ ] SP-initiated SSO flow
- [ ] HTTP POST binding
- [ ] Assertion signature validation
- [ ] Timestamp validation
- [ ] Metadata exchange

### Optional but Recommended
- [ ] IdP-initiated SSO
- [ ] Single Logout (SLO)
- [ ] Assertion encryption
- [ ] Artifact binding

---

## OpenID Connect Checklist

### Client Configuration
- [ ] Client ID from IdP
- [ ] Client Secret (confidential client)
- [ ] Redirect URI: `https://llm-cost-ops.example.com/oauth/callback`
- [ ] Scopes: `openid profile email offline_access`
- [ ] Response type: `code`
- [ ] Grant type: `authorization_code`

### Required Features
- [ ] Authorization Code flow with PKCE
- [ ] ID token validation (signature, iss, aud, exp, nonce)
- [ ] UserInfo endpoint integration
- [ ] JWKS fetching and caching
- [ ] Token refresh

---

## SCIM 2.0 Checklist

### Endpoints Required
- [ ] `GET /scim/v2/Users` - List users
- [ ] `GET /scim/v2/Users/{id}` - Get user
- [ ] `POST /scim/v2/Users` - Create user
- [ ] `PUT /scim/v2/Users/{id}` - Replace user
- [ ] `PATCH /scim/v2/Users/{id}` - Update user
- [ ] `DELETE /scim/v2/Users/{id}` - Delete user
- [ ] `GET /scim/v2/Groups` - List groups
- [ ] `POST /scim/v2/Groups` - Create group

### Features
- [ ] Filtering (e.g., `filter=userName eq "user@example.com"`)
- [ ] Pagination (startIndex, count)
- [ ] Sorting (sortBy, sortOrder)
- [ ] Error handling (SCIM error format)
- [ ] Authentication (OAuth 2.0 Bearer token)

---

## Session Management

### Redis Session Configuration

```yaml
redis:
  host: "redis-cluster.example.com"
  port: 6379
  db: 0
  password: "${REDIS_PASSWORD}"
  tls: true

session:
  prefix: "llm_cost_ops_session"
  default_ttl: 28800      # 8 hours
  idle_timeout: 1800       # 30 minutes
  max_lifetime: 86400      # 24 hours
```

### Session Security
- [ ] Secure cookie flag (HTTPS only)
- [ ] HttpOnly flag (prevent XSS)
- [ ] SameSite=Lax (CSRF protection)
- [ ] Session ID regeneration after login
- [ ] Session revocation on logout

---

## Compliance Quick Checks

### SOC 2 Requirements
- [ ] MFA enforcement for privileged roles
- [ ] Session timeout policies
- [ ] User verification before provisioning
- [ ] Automated access removal
- [ ] Comprehensive audit logging

### GDPR Requirements
- [ ] Data export functionality (Article 15)
- [ ] Data deletion functionality (Article 17)
- [ ] Data minimization (only essential PII)
- [ ] Pseudonymization for retained data
- [ ] Audit trail for all data access

---

## Testing Checklist

### Unit Tests
- [ ] SAML assertion parsing
- [ ] SAML signature verification
- [ ] JWT token validation
- [ ] PKCE code challenge generation
- [ ] State parameter validation
- [ ] Attribute mapping

### Integration Tests
- [ ] Full SAML login flow
- [ ] Full OIDC login flow
- [ ] JIT user provisioning
- [ ] SCIM user creation
- [ ] SCIM user update
- [ ] Session creation/validation

### Security Tests
- [ ] CSRF attack prevention
- [ ] Replay attack prevention
- [ ] Token tampering detection
- [ ] Open redirect prevention
- [ ] Session fixation prevention

### Performance Tests
- [ ] SAML login < 2s (p95)
- [ ] OIDC login < 2s (p95)
- [ ] Token validation < 50ms (p95)
- [ ] Session lookup < 10ms (p95)
- [ ] Concurrent users: 1000+

---

## Common Issues & Solutions

### Issue: SAML Signature Verification Failed
**Cause:** Certificate mismatch, incorrect algorithm, or clock skew
**Solution:**
1. Verify certificate from IdP metadata
2. Check algorithm (RS256 recommended)
3. Ensure server clock synchronized (NTP)
4. Allow 60s clock skew tolerance

### Issue: OIDC Token Expired
**Cause:** Token TTL too short or clock skew
**Solution:**
1. Implement token refresh flow
2. Check exp claim vs current time
3. Allow clock skew tolerance
4. Cache tokens until near expiry

### Issue: Redirect URI Mismatch
**Cause:** URI not whitelisted or slight variation
**Solution:**
1. Ensure exact match (no trailing slash differences)
2. Check protocol (http vs https)
3. Verify port number
4. Update IdP configuration

### Issue: SCIM User Already Exists
**Cause:** Duplicate userName or externalId
**Solution:**
1. Return HTTP 409 Conflict
2. Include scimType: "uniqueness" in error
3. Check for soft-deleted users
4. Implement idempotent updates

---

## Recommended Rust Crates

```toml
[dependencies]
# SAML
samael = "0.0.12"

# OAuth 2.0 / OIDC
oauth2 = "4.4"
openidconnect = "3.3"

# JWT (already in use)
jsonwebtoken = "9.2"

# Session Management
tower-sessions = "0.8"
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# HTTP Client
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Crypto
rand = "0.8"
sha2 = "0.10"
base64 = "0.21"

# Utilities
url = "2.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## Configuration Template

### Minimal SSO Configuration

```yaml
# config/sso.yaml
sso:
  enabled: true

  # Okta SAML
  okta_saml:
    entity_id: "llm-cost-ops"
    acs_url: "${BASE_URL}/saml/acs"
    metadata_url: "https://your-tenant.okta.com/app/abc123/sso/saml/metadata"

  # JIT Provisioning
  provisioning:
    jit_enabled: true
    default_role: "read_only"

  # Session
  session:
    lifetime_secs: 28800  # 8 hours

  # Security
  security:
    mfa_required: true
    mfa_required_roles: ["super_admin", "org_admin"]
```

---

## Monitoring & Alerts

### Key Metrics to Track

```yaml
metrics:
  # Success Rates
  - sso_login_success_rate       # Target: > 99.5%
  - saml_assertion_valid_rate    # Target: > 99.9%
  - oidc_token_valid_rate        # Target: > 99.9%

  # Latency
  - sso_login_duration_p95       # Target: < 2s
  - token_validation_duration_p95 # Target: < 50ms
  - session_lookup_duration_p95   # Target: < 10ms

  # Security
  - csrf_attack_attempts         # Alert: > 0
  - replay_attack_attempts       # Alert: > 0
  - failed_login_attempts        # Alert: > 100/hour

  # Business
  - active_sso_sessions          # Track: growth
  - jit_provisioned_users        # Track: daily count
  - scim_sync_operations         # Track: frequency
```

### Alert Rules

```yaml
alerts:
  - name: SSO Login Failure Rate High
    condition: sso_login_success_rate < 95%
    severity: critical
    notification: pagerduty

  - name: Token Validation Latency High
    condition: token_validation_duration_p95 > 100ms
    severity: warning
    notification: slack

  - name: Security Attack Detected
    condition: csrf_attack_attempts > 0 OR replay_attack_attempts > 0
    severity: critical
    notification: security_team
```

---

## Command Reference

### Generate SAML Signing Certificate

```bash
# Generate private key
openssl genrsa -out saml-sp.key 2048

# Generate certificate (valid 2 years)
openssl req -new -x509 -key saml-sp.key -out saml-sp.crt -days 730 \
  -subj "/CN=llm-cost-ops/O=YourOrg/C=US"

# Convert to PEM format (if needed)
openssl x509 -in saml-sp.crt -out saml-sp.pem -outform PEM
```

### Test SAML Assertion

```bash
# Decode SAML assertion
echo "<base64-encoded-assertion>" | base64 -d | xmllint --format -

# Verify signature
xmlsec1 --verify --pubkey-cert-pem idp-cert.pem assertion.xml
```

### Test JWT Token

```bash
# Decode JWT (without verification)
jwt decode <token>

# Verify JWT signature
jwt verify <token> --key publickey.pem --alg RS256
```

---

## Support & Resources

### Internal Documentation
- Full Research: `/workspaces/llm-cost-ops/docs/SSO_ENTERPRISE_RESEARCH.md`
- Summary: `/workspaces/llm-cost-ops/docs/SSO_IMPLEMENTATION_SUMMARY.md`
- This Guide: `/workspaces/llm-cost-ops/docs/SSO_QUICK_REFERENCE.md`

### External Links
- SAML 2.0 Spec: https://docs.oasis-open.org/security/saml/v2.0/
- OIDC Spec: https://openid.net/specs/openid-connect-core-1_0.html
- SCIM 2.0 RFC: https://datatracker.ietf.org/doc/html/rfc7644
- OWASP Auth Guide: https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html

### Tools
- SAML Tracer (Firefox/Chrome extension) - Debug SAML flows
- jwt.io - Decode and verify JWT tokens
- Postman - Test OAuth/OIDC flows
- SSOCircle - Free SAML testing IdP

---

**Last Updated:** 2025-11-15
**Maintained By:** SSO Standards & Research Specialist
