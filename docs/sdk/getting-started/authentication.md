# Authentication Guide

LLM-CostOps supports two authentication methods: API Keys and JWT tokens. This guide covers both methods and best practices.

## Authentication Methods

| Method | Use Case | Lifetime | Revocable |
|--------|----------|----------|-----------|
| API Key | Server-to-server, CLI tools | Long-lived | Yes |
| JWT Token | User sessions, web apps | Short-lived (1h default) | Yes |

## API Key Authentication

API keys are the simplest and most common authentication method for server-to-server communication.

### Creating an API Key

#### Cloud (Web Dashboard)

1. Log in to https://app.llm-cost-ops.dev
2. Navigate to **Settings** → **API Keys**
3. Click **Create API Key**
4. Provide a name (e.g., "Production API Key")
5. Copy the key (it will only be shown once)
6. Store it securely

#### Self-Hosted (CLI)

```bash
cost-ops auth create-key \
  --organization org-123 \
  --name "Production API Key" \
  --permissions "usage:write,costs:read,analytics:read"
```

**Output:**
```
API Key created successfully!
Key ID: key_1234567890abcdef
API Key: llmco_sk_1234567890abcdef1234567890abcdef

⚠️  Store this key securely. It will not be shown again.

Permissions:
  - usage:write
  - costs:read
  - analytics:read
```

#### Via REST API

```bash
curl -X POST https://api.llm-cost-ops.dev/api/v1/auth/keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_MASTER_KEY" \
  -d '{
    "name": "Production API Key",
    "organization_id": "org-123",
    "permissions": ["usage:write", "costs:read", "analytics:read"],
    "expires_at": "2026-01-01T00:00:00Z"
  }'
```

### Using an API Key

Include the API key in the `Authorization` header:

```bash
curl -X GET https://api.llm-cost-ops.dev/api/v1/costs \
  -H "Authorization: Bearer llmco_sk_1234567890abcdef1234567890abcdef"
```

### API Key Format

API keys follow this format:
```
llmco_sk_<32_character_hash>
```

- `llmco` - Product identifier
- `sk` - Secret key type
- `<hash>` - 32-character hexadecimal hash

### API Key Permissions

Control what each API key can access:

| Permission | Description |
|------------|-------------|
| `usage:read` | Read usage records |
| `usage:write` | Submit usage records |
| `costs:read` | Query cost data |
| `pricing:read` | View pricing tables |
| `pricing:write` | Create/update pricing |
| `analytics:read` | Access analytics |
| `forecasts:read` | View forecasts |
| `forecasts:write` | Generate forecasts |
| `budgets:read` | View budgets |
| `budgets:write` | Create/update budgets |
| `admin` | Full access (use sparingly) |

**Example:** Create a read-only key for dashboards:

```bash
cost-ops auth create-key \
  --organization org-123 \
  --name "Dashboard Read-Only" \
  --permissions "usage:read,costs:read,analytics:read"
```

### Rotating API Keys

Regularly rotate API keys for security:

1. Create a new API key
2. Update your application to use the new key
3. Verify the new key works
4. Delete the old key

```bash
# List existing keys
cost-ops auth list-keys --organization org-123

# Delete old key
cost-ops auth delete-key key_old123
```

### Revoking API Keys

If a key is compromised:

```bash
# Revoke immediately
cost-ops auth revoke-key key_1234567890abcdef
```

Or via REST API:

```bash
curl -X DELETE https://api.llm-cost-ops.dev/api/v1/auth/keys/key_1234567890abcdef \
  -H "Authorization: Bearer YOUR_MASTER_KEY"
```

## JWT Authentication

JWT (JSON Web Token) authentication is ideal for user sessions in web applications.

### Obtaining a JWT Token

#### Login with Username/Password

```bash
curl -X POST https://api.llm-cost-ops.dev/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your_password"
  }'
```

**Response:**
```json
{
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": {
      "id": "user-123",
      "email": "user@example.com",
      "organization_id": "org-123",
      "roles": ["admin"]
    }
  }
}
```

#### Using SSO (Enterprise)

```bash
# Initiate SSO flow
curl -X POST https://api.llm-cost-ops.dev/api/v1/auth/sso/initiate \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "okta",
    "organization_id": "org-123"
  }'
```

See [SSO Integration Guide](../guides/sso-integration.md) for details.

### Using JWT Tokens

Include the JWT token in the `Authorization` header:

```bash
curl -X GET https://api.llm-cost-ops.dev/api/v1/costs \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### Token Expiration

- **Access Token**: 1 hour (default)
- **Refresh Token**: 30 days (default)

### Refreshing Tokens

When the access token expires, use the refresh token to get a new one:

```bash
curl -X POST https://api.llm-cost-ops.dev/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }'
```

**Response:**
```json
{
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600
  }
}
```

### JWT Token Structure

JWT tokens contain three parts separated by dots (`.`):

```
header.payload.signature
```

**Decoded Payload Example:**
```json
{
  "sub": "user-123",
  "org": "org-123",
  "roles": ["admin"],
  "permissions": ["usage:write", "costs:read"],
  "iat": 1705320000,
  "exp": 1705323600
}
```

Claims:
- `sub` - Subject (user ID)
- `org` - Organization ID
- `roles` - User roles
- `permissions` - Granted permissions
- `iat` - Issued at (Unix timestamp)
- `exp` - Expiration (Unix timestamp)

## Security Best Practices

### 1. Store Credentials Securely

❌ **Don't:**
```python
# Hardcoded in source code
API_KEY = "llmco_sk_1234567890abcdef1234567890abcdef"
```

✅ **Do:**
```python
# Load from environment variable
import os
API_KEY = os.getenv("LLM_COST_OPS_API_KEY")
```

Or use a secrets manager:
```python
# AWS Secrets Manager
import boto3
secrets_client = boto3.client('secretsmanager')
secret = secrets_client.get_secret_value(SecretId='llm-cost-ops-api-key')
API_KEY = secret['SecretString']
```

### 2. Use Environment-Specific Keys

Maintain separate API keys for each environment:

- **Development**: `llmco_sk_dev_...`
- **Staging**: `llmco_sk_staging_...`
- **Production**: `llmco_sk_prod_...`

### 3. Principle of Least Privilege

Grant only the permissions needed:

```bash
# Good: Limited permissions for frontend
cost-ops auth create-key \
  --name "Frontend Dashboard" \
  --permissions "costs:read,analytics:read"

# Bad: Admin access for everything
cost-ops auth create-key \
  --name "Frontend Dashboard" \
  --permissions "admin"
```

### 4. Set Expiration Dates

Create keys with expiration dates:

```bash
cost-ops auth create-key \
  --name "Temporary Integration Key" \
  --expires-at "2025-03-01T00:00:00Z"
```

### 5. Monitor API Key Usage

Track which keys are being used:

```bash
# View API key activity
cost-ops auth audit-log --key key_1234567890abcdef
```

### 6. Use HTTPS Only

Always use HTTPS in production:

```bash
# ✅ Secure
https://api.llm-cost-ops.dev/api/v1/usage

# ❌ Insecure (development only)
http://localhost:8080/api/v1/usage
```

### 7. Implement Rate Limiting

Protect your API keys with rate limiting:

```python
from llm_cost_ops import CostOpsClient
from llm_cost_ops.middleware import RateLimiter

client = CostOpsClient(
    api_key=os.getenv("LLM_COST_OPS_API_KEY"),
    middleware=[
        RateLimiter(max_requests=100, window_seconds=60)
    ]
)
```

## Role-Based Access Control (RBAC)

LLM-CostOps supports fine-grained RBAC:

### Roles

| Role | Description | Default Permissions |
|------|-------------|---------------------|
| `admin` | Full access to organization | All permissions |
| `analyst` | Read-only access to costs and analytics | `costs:read`, `analytics:read`, `forecasts:read` |
| `developer` | Can submit usage and query costs | `usage:write`, `costs:read` |
| `viewer` | Read-only access to basic data | `usage:read`, `costs:read` |
| `billing` | Manage pricing and budgets | `pricing:write`, `budgets:write`, `costs:read` |

### Assigning Roles

```bash
# Assign role to user
cost-ops rbac assign-role \
  --user user-123 \
  --organization org-123 \
  --role analyst
```

### Custom Roles

Create custom roles with specific permissions:

```bash
cost-ops rbac create-role \
  --name "ci-cd-bot" \
  --permissions "usage:write,costs:read" \
  --description "Role for CI/CD automation"
```

## Multi-Organization Access

If a user belongs to multiple organizations:

### Specify Organization in Requests

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Switch Organizations

```bash
# Get token for different organization
curl -X POST https://api.llm-cost-ops.dev/api/v1/auth/switch-org \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "organization_id": "org-456"
  }'
```

## Audit Logging

All authentication events are logged:

```bash
# View auth audit log
cost-ops auth audit-log \
  --organization org-123 \
  --start-date 2025-01-01 \
  --end-date 2025-01-31
```

**Events tracked:**
- API key creation
- API key deletion
- API key usage
- Login attempts (success/failure)
- Token refresh
- Permission changes
- Role assignments

## Troubleshooting

### Error: "Invalid API key"

- Verify the API key is correct (no extra spaces)
- Check if the key has been revoked
- Ensure the key has not expired

### Error: "Insufficient permissions"

- Check the permissions of your API key
- Verify you have the required role
- Contact your organization admin to grant permissions

### Error: "Token expired"

- Use the refresh token to get a new access token
- If the refresh token is expired, log in again

### Error: "Organization not found"

- Verify the `organization_id` is correct
- Check if you have access to this organization
- Contact support if the issue persists

## Code Examples

### Python

```python
import os
from llm_cost_ops import CostOpsClient

# Using API Key
client = CostOpsClient(api_key=os.getenv("LLM_COST_OPS_API_KEY"))

# Using JWT
client = CostOpsClient(access_token="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")

# Login to get JWT
credentials = client.auth.login(
    email="user@example.com",
    password=os.getenv("PASSWORD")
)
client = CostOpsClient(access_token=credentials.access_token)
```

### TypeScript

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

// Using API Key
const client = new CostOpsClient({
  apiKey: process.env.LLM_COST_OPS_API_KEY
});

// Using JWT
const authClient = new CostOpsClient();
const credentials = await authClient.auth.login({
  email: 'user@example.com',
  password: process.env.PASSWORD
});

const client = new CostOpsClient({
  accessToken: credentials.accessToken
});
```

### Go

```go
import "github.com/llm-devops/llm-cost-ops-go"

// Using API Key
client := costops.NewClient(
    costops.WithAPIKey(os.Getenv("LLM_COST_OPS_API_KEY")),
)

// Using JWT
credentials, err := costops.Login(ctx, "user@example.com", password)
client := costops.NewClient(
    costops.WithAccessToken(credentials.AccessToken),
)
```

## Next Steps

- [Submit Your First Usage](first-usage.md)
- [Query Costs](query-costs.md)
- [SSO Integration Guide](../guides/sso-integration.md)
- [Security Best Practices](../guides/security.md)
