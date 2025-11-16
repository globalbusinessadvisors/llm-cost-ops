---
sidebar_position: 3
title: Authentication
---

# Authentication

LLM-CostOps supports multiple authentication methods to secure your API access. Choose the method that best fits your use case.

## Authentication Methods

### API Keys (Recommended for SDKs)

API keys are the simplest and most common way to authenticate with LLM-CostOps.

#### Creating an API Key

Using the CLI:

```bash
# Create a new API key
cost-ops api-key create \
  --name "Production API Key" \
  --organization org-123 \
  --permissions read,write

# Output:
# API Key created successfully
# Key ID: key-abc123
# Secret: sk_live_1234567890abcdef...
#
# Save this secret securely - it will not be shown again!
```

Using the API:

```bash
curl -X POST https://api.llm-cost-ops.example.com/api/v1/api-keys \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Production API Key",
    "organization_id": "org-123",
    "permissions": ["read", "write"]
  }'
```

#### Using API Keys

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="python" label="Python">

```python
from llm_cost_ops import CostOpsClient

# Direct API key
client = CostOpsClient(
    api_key="sk_live_1234567890abcdef",
    base_url="https://api.llm-cost-ops.example.com"
)

# From environment variable
import os
client = CostOpsClient(
    api_key=os.getenv("COST_OPS_API_KEY"),
    base_url="https://api.llm-cost-ops.example.com"
)
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

// Direct API key
const client = new CostOpsClient({
  apiKey: 'sk_live_1234567890abcdef',
  baseUrl: 'https://api.llm-cost-ops.example.com',
});

// From environment variable
const client = new CostOpsClient({
  apiKey: process.env.COST_OPS_API_KEY,
  baseUrl: 'https://api.llm-cost-ops.example.com',
});
```

</TabItem>
<TabItem value="go" label="Go">

```go
import (
    "os"
    llmcostops "github.com/llm-devops/llm-cost-ops/sdk/go"
)

// Direct API key
client, err := llmcostops.NewClient(
    llmcostops.WithAPIKey("sk_live_1234567890abcdef"),
    llmcostops.WithBaseURL("https://api.llm-cost-ops.example.com"),
)

// From environment variable
client, err := llmcostops.NewClient(
    llmcostops.WithAPIKey(os.Getenv("COST_OPS_API_KEY")),
    llmcostops.WithBaseURL("https://api.llm-cost-ops.example.com"),
)
```

</TabItem>
<TabItem value="rust" label="Rust">

```rust
use llm_cost_ops::{CostOpsClient, ClientConfig};

// Direct API key
let config = ClientConfig::builder()
    .base_url("https://api.llm-cost-ops.example.com")?
    .api_key("sk_live_1234567890abcdef")
    .build()?;

let client = CostOpsClient::new(config)?;

// From environment variable
let config = ClientConfig::builder()
    .base_url("https://api.llm-cost-ops.example.com")?
    .api_key(std::env::var("COST_OPS_API_KEY")?)
    .build()?;

let client = CostOpsClient::new(config)?;
```

</TabItem>
</Tabs>

### JWT Tokens (Recommended for Web Apps)

JWT tokens are ideal for user-facing applications where you need fine-grained access control.

#### Obtaining a JWT Token

```bash
curl -X POST https://api.llm-cost-ops.example.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your-password"
  }'
```

Response:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

#### Using JWT Tokens

```bash
# Use the access token in requests
curl -X GET https://api.llm-cost-ops.example.com/api/v1/costs \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

<Tabs>
<TabItem value="python" label="Python">

```python
from llm_cost_ops import CostOpsClient

# Login and get token
response = requests.post(
    "https://api.llm-cost-ops.example.com/api/v1/auth/login",
    json={"email": "user@example.com", "password": "your-password"}
)
token = response.json()["access_token"]

# Use token with client
client = CostOpsClient(
    api_key=token,  # JWT token works as API key
    base_url="https://api.llm-cost-ops.example.com"
)
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

// Login and get token
const response = await fetch('https://api.llm-cost-ops.example.com/api/v1/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ email: 'user@example.com', password: 'your-password' }),
});
const { access_token } = await response.json();

// Use token with client
const client = new CostOpsClient({
  apiKey: access_token, // JWT token works as API key
  baseUrl: 'https://api.llm-cost-ops.example.com',
});
```

</TabItem>
</Tabs>

#### Refreshing Tokens

JWT access tokens expire after a period (default: 1 hour). Use the refresh token to obtain a new access token:

```bash
curl -X POST https://api.llm-cost-ops.example.com/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }'
```

### OAuth 2.0 (Enterprise)

For enterprise customers, LLM-CostOps supports OAuth 2.0 with SSO providers.

#### Supported Providers

- Google Workspace
- Microsoft Azure AD / Entra ID
- Okta
- Auth0
- Generic OIDC

#### Configuration

Configure OAuth in your `config.toml`:

```toml
[auth.oauth]
enabled = true
provider = "google"
client_id = "your-client-id"
client_secret = "your-client-secret"
redirect_uri = "https://your-app.com/auth/callback"
scopes = ["openid", "email", "profile"]
```

#### OAuth Flow

```mermaid
sequenceDiagram
    participant User
    participant App
    participant LLM-CostOps
    participant OAuth Provider

    User->>App: Click "Login"
    App->>LLM-CostOps: Initiate OAuth
    LLM-CostOps->>OAuth Provider: Redirect to provider
    User->>OAuth Provider: Enter credentials
    OAuth Provider->>LLM-CostOps: Return auth code
    LLM-CostOps->>OAuth Provider: Exchange code for token
    OAuth Provider->>LLM-CostOps: Return access token
    LLM-CostOps->>App: Return JWT token
    App->>User: Logged in
```

## API Key Management

### Listing API Keys

```bash
cost-ops api-key list --organization org-123
```

Output:

```
ID          Name                Created              Last Used            Status
key-abc123  Production API Key  2024-01-15 10:00:00  2024-01-20 14:30:00  Active
key-def456  Staging API Key     2024-01-10 09:00:00  2024-01-19 08:15:00  Active
key-ghi789  Test Key            2024-01-01 12:00:00  Never                Revoked
```

### Revoking API Keys

```bash
# Revoke a specific key
cost-ops api-key revoke key-abc123

# Revoke all keys for an organization
cost-ops api-key revoke-all --organization org-123
```

### Key Rotation

Best practice: Rotate API keys regularly (every 90 days):

```bash
# Create new key
cost-ops api-key create --name "Production API Key (2024-Q2)" --organization org-123

# Update your applications with new key

# Revoke old key after verification
cost-ops api-key revoke key-old123
```

## Permissions and Scopes

API keys can be scoped to specific permissions:

| Permission | Description |
|------------|-------------|
| `read` | Read cost data, usage records, and analytics |
| `write` | Submit usage data |
| `admin` | Manage API keys, users, and settings |
| `billing` | Access billing and payment information |
| `reports` | Generate and download reports |

### Creating Scoped Keys

```bash
# Read-only key for monitoring
cost-ops api-key create \
  --name "Monitoring Key" \
  --organization org-123 \
  --permissions read

# Write-only key for data ingestion
cost-ops api-key create \
  --name "Ingestion Key" \
  --organization org-123 \
  --permissions write

# Admin key for management
cost-ops api-key create \
  --name "Admin Key" \
  --organization org-123 \
  --permissions read,write,admin
```

## Security Best Practices

### 1. Store Keys Securely

Never commit API keys to version control. Use environment variables or secret management:

```bash
# .env file (add to .gitignore)
COST_OPS_API_KEY=sk_live_1234567890abcdef
COST_OPS_BASE_URL=https://api.llm-cost-ops.example.com
```

### 2. Use Different Keys per Environment

Create separate keys for development, staging, and production:

```bash
# Development
COST_OPS_API_KEY_DEV=sk_test_dev123...

# Staging
COST_OPS_API_KEY_STAGING=sk_test_staging456...

# Production
COST_OPS_API_KEY_PROD=sk_live_prod789...
```

### 3. Rotate Keys Regularly

Set up automated key rotation:

```python
from datetime import datetime, timedelta

def should_rotate_key(key_created_at):
    """Rotate keys older than 90 days"""
    rotation_period = timedelta(days=90)
    return datetime.now() - key_created_at > rotation_period

if should_rotate_key(key.created_at):
    # Create new key
    new_key = client.api_keys.create(...)
    # Update configuration
    # Revoke old key
    client.api_keys.revoke(old_key.id)
```

### 4. Monitor Key Usage

Set up alerts for unusual API key activity:

```bash
# Get API key usage stats
cost-ops api-key stats key-abc123

# Output:
# Total Requests: 15,234
# Last Used: 2024-01-20 14:30:00
# Average Requests/Day: 507
# Unusual Activity: None
```

### 5. Use IP Whitelisting

Restrict API key usage to specific IP addresses:

```bash
cost-ops api-key create \
  --name "Production API Key" \
  --organization org-123 \
  --allowed-ips "203.0.113.0/24,198.51.100.0/24"
```

## Environment-Specific Configuration

### Development

```bash
# .env.development
COST_OPS_API_KEY=sk_test_dev123...
COST_OPS_BASE_URL=http://localhost:8080
```

### Staging

```bash
# .env.staging
COST_OPS_API_KEY=sk_test_staging456...
COST_OPS_BASE_URL=https://staging-api.llm-cost-ops.example.com
```

### Production

```bash
# .env.production
COST_OPS_API_KEY=sk_live_prod789...
COST_OPS_BASE_URL=https://api.llm-cost-ops.example.com
```

## Troubleshooting

### Invalid API Key

**Error:** `401 Unauthorized - Invalid API key`

**Solutions:**
1. Verify the API key is correct
2. Check if the key has been revoked
3. Ensure the key has the required permissions

### Expired JWT Token

**Error:** `401 Unauthorized - Token expired`

**Solutions:**
1. Use the refresh token to get a new access token
2. Re-authenticate if refresh token is also expired

### Insufficient Permissions

**Error:** `403 Forbidden - Insufficient permissions`

**Solutions:**
1. Check the API key permissions
2. Request admin to grant required permissions
3. Use a key with appropriate scopes

### Rate Limiting

**Error:** `429 Too Many Requests`

**Solutions:**
1. Implement exponential backoff
2. Use the `Retry-After` header
3. Request a higher rate limit

## Next Steps

- [Quick Start Guide](/docs/getting-started/quick-start) - Start using the SDKs
- [API Reference](/docs/api/) - Detailed API documentation
- [Security Guide](/docs/guides/security) - Advanced security practices

## Support

For authentication issues:
- Check the [troubleshooting guide](/docs/guides/troubleshooting)
- Contact support at support@llm-cost-ops.dev
- [Open an issue](https://github.com/llm-devops/llm-cost-ops/issues)
