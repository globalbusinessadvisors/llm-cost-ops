---
sidebar_position: 1
title: Installation
---

# Installation

Get LLM-CostOps up and running in your environment. This guide covers installation for all supported platforms and SDKs.

## Server Installation

### Using Docker (Recommended)

The easiest way to run LLM-CostOps is with Docker:

```bash
# Pull the latest image
docker pull ghcr.io/llm-devops/llm-cost-ops:latest

# Run with SQLite (development)
docker run -d \
  --name llm-cost-ops \
  -p 8080:8080 \
  -v $(pwd)/data:/data \
  ghcr.io/llm-devops/llm-cost-ops:latest

# Run with PostgreSQL (production)
docker run -d \
  --name llm-cost-ops \
  -p 8080:8080 \
  -e DATABASE_URL=postgresql://user:pass@postgres:5432/costops \
  -e API_KEY_SECRET=your-secret-key \
  ghcr.io/llm-devops/llm-cost-ops:latest
```

### Using Docker Compose

Create a `docker-compose.yml` file:

```yaml
version: '3.8'

services:
  llm-cost-ops:
    image: ghcr.io/llm-devops/llm-cost-ops:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://costops:password@postgres:5432/costops
      - API_KEY_SECRET=your-secret-key
      - RUST_LOG=info
    depends_on:
      - postgres
    volumes:
      - ./config:/etc/llm-cost-ops

  postgres:
    image: postgres:16-alpine
    environment:
      - POSTGRES_DB=costops
      - POSTGRES_USER=costops
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

volumes:
  postgres_data:
```

Start the services:

```bash
docker-compose up -d
```

### From Source

Requirements:
- Rust 1.91 or higher
- PostgreSQL 14+ (for production) or SQLite 3.35+ (for development)

```bash
# Clone the repository
git clone https://github.com/llm-devops/llm-cost-ops.git
cd llm-cost-ops

# Build the project
cargo build --release

# Run database migrations
./target/release/cost-ops migrate --database-url postgresql://user:pass@localhost/costops

# Start the server
./target/release/cost-ops serve --port 8080
```

### Kubernetes

Deploy with Helm:

```bash
# Add the Helm repository
helm repo add llm-cost-ops https://charts.llm-cost-ops.dev
helm repo update

# Install with default values
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --create-namespace

# Install with custom values
helm install llm-cost-ops llm-cost-ops/llm-cost-ops \
  --namespace llm-cost-ops \
  --create-namespace \
  --values custom-values.yaml
```

Or using kubectl with Kustomize:

```bash
# Deploy to development
kubectl apply -k k8s/overlays/dev/

# Deploy to production
kubectl apply -k k8s/overlays/prod/
```

## SDK Installation

### Python SDK

```bash
# Install from PyPI
pip install llm-cost-ops

# Install with development dependencies
pip install llm-cost-ops[dev]

# Install with metrics support
pip install llm-cost-ops[metrics]

# Install all extras
pip install llm-cost-ops[all]
```

Using Poetry:

```bash
poetry add llm-cost-ops
```

Using Pipenv:

```bash
pipenv install llm-cost-ops
```

### TypeScript/JavaScript SDK

```bash
# Using npm
npm install @llm-cost-ops/sdk

# Using yarn
yarn add @llm-cost-ops/sdk

# Using pnpm
pnpm add @llm-cost-ops/sdk
```

### Go SDK

```bash
go get github.com/llm-devops/llm-cost-ops/sdk/go
```

### Rust SDK

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-cost-ops = "0.1.0"
```

Or use cargo:

```bash
cargo add llm-cost-ops
```

## Verify Installation

### Server Health Check

Test that the server is running:

```bash
curl http://localhost:8080/health
```

Expected response:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "database": "connected",
  "uptime_seconds": 42
}
```

### SDK Verification

Test your SDK installation:

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="python" label="Python">

```python
from llm_cost_ops import CostOpsClient

# Create client
client = CostOpsClient(
    api_key="your-api-key",
    base_url="http://localhost:8080"
)

# Check health
health = client.health()
print(f"Server status: {health.status}")
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

// Create client
const client = new CostOpsClient({
  baseUrl: 'http://localhost:8080',
  apiKey: 'your-api-key',
});

// Check health
const health = await client.health();
console.log('Server status:', health.status);
```

</TabItem>
<TabItem value="go" label="Go">

```go
package main

import (
    "context"
    "fmt"
    "log"

    llmcostops "github.com/llm-devops/llm-cost-ops/sdk/go"
)

func main() {
    client, err := llmcostops.NewClient(
        llmcostops.WithAPIKey("your-api-key"),
        llmcostops.WithBaseURL("http://localhost:8080"),
    )
    if err != nil {
        log.Fatal(err)
    }
    defer client.Close()

    health, err := client.Health.Check(context.Background())
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Server status: %s\n", health.Status)
}
```

</TabItem>
<TabItem value="rust" label="Rust">

```rust
use llm_cost_ops::{CostOpsClient, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClientConfig::builder()
        .base_url("http://localhost:8080")?
        .api_key("your-api-key")
        .build()?;

    let client = CostOpsClient::new(config)?;

    let health = client.health().await?;
    println!("Server status: {}", health.status);

    Ok(())
}
```

</TabItem>
</Tabs>

## Environment Variables

Configure LLM-CostOps using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL or SQLite connection string | `sqlite:cost-ops.db` |
| `API_KEY_SECRET` | Secret for API key hashing | Required |
| `PORT` | Server port | `8080` |
| `HOST` | Server host | `0.0.0.0` |
| `RUST_LOG` | Log level (error, warn, info, debug, trace) | `info` |
| `METRICS_ENABLED` | Enable Prometheus metrics | `true` |
| `TRACING_ENABLED` | Enable OpenTelemetry tracing | `false` |
| `OTLP_ENDPOINT` | OpenTelemetry collector endpoint | - |
| `CORS_ORIGINS` | Allowed CORS origins (comma-separated) | `*` |
| `RATE_LIMIT_REQUESTS` | Requests per minute per IP | `100` |
| `RATE_LIMIT_ENABLED` | Enable rate limiting | `true` |

## Configuration File

Create a `config.toml` file for advanced configuration:

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://user:pass@localhost/costops"
max_connections = 20
min_connections = 5

[auth]
api_key_secret = "your-secret-key"
jwt_secret = "your-jwt-secret"
jwt_expiry_hours = 24

[observability]
log_level = "info"
metrics_enabled = true
tracing_enabled = true
otlp_endpoint = "http://localhost:4317"

[rate_limiting]
enabled = true
requests_per_minute = 100
burst_size = 20

[cors]
origins = ["https://app.example.com", "https://dashboard.example.com"]
methods = ["GET", "POST", "PUT", "DELETE"]
headers = ["Content-Type", "Authorization"]
```

## Next Steps

Now that you have LLM-CostOps installed:

1. [Set up authentication](/docs/getting-started/authentication)
2. [Follow the quick start guide](/docs/getting-started/quick-start)
3. [Choose your SDK](/docs/intro#getting-started)

## Troubleshooting

### Connection Issues

If you can't connect to the server:

1. Check that the server is running:
   ```bash
   curl http://localhost:8080/health
   ```

2. Verify firewall settings allow port 8080

3. Check logs:
   ```bash
   docker logs llm-cost-ops
   ```

### Database Issues

If you see database connection errors:

1. Verify database is running:
   ```bash
   pg_isready -h localhost -p 5432
   ```

2. Check connection string format:
   ```
   postgresql://username:password@host:port/database
   ```

3. Run migrations:
   ```bash
   cost-ops migrate --database-url postgresql://user:pass@localhost/costops
   ```

### Permission Issues

If you see permission errors:

1. Ensure the data directory is writable:
   ```bash
   chmod 755 /path/to/data
   ```

2. Check container user permissions

3. Verify database user has necessary privileges

For more help, see the [Troubleshooting Guide](/docs/guides/troubleshooting) or [open an issue](https://github.com/llm-devops/llm-cost-ops/issues).
