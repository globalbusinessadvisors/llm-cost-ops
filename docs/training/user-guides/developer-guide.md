# LLM Cost Ops Developer Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
   - [Installation](#installation)
   - [Authentication](#authentication)
   - [Basic Configuration](#basic-configuration)
3. [SDK Usage Patterns](#sdk-usage-patterns)
   - [Python SDK](#python-sdk)
   - [TypeScript SDK](#typescript-sdk)
   - [Go SDK](#go-sdk)
   - [Rust SDK](#rust-sdk)
4. [Advanced Integration Patterns](#advanced-integration-patterns)
   - [Async/Await Patterns](#asyncawait-patterns)
   - [Batch Operations](#batch-operations)
   - [Streaming Data](#streaming-data)
   - [Connection Pooling](#connection-pooling)
5. [Error Handling and Retry Strategies](#error-handling-and-retry-strategies)
   - [Error Types](#error-types)
   - [Retry Logic](#retry-logic)
   - [Circuit Breakers](#circuit-breakers)
   - [Graceful Degradation](#graceful-degradation)
6. [CI/CD Integration](#cicd-integration)
   - [GitHub Actions](#github-actions)
   - [GitLab CI](#gitlab-ci)
   - [Jenkins](#jenkins)
   - [CircleCI](#circleci)
7. [Performance Optimization](#performance-optimization)
   - [Caching Strategies](#caching-strategies)
   - [Query Optimization](#query-optimization)
   - [Resource Management](#resource-management)
   - [Profiling and Benchmarking](#profiling-and-benchmarking)
8. [Testing Strategies](#testing-strategies)
   - [Unit Testing](#unit-testing)
   - [Integration Testing](#integration-testing)
   - [Mock Services](#mock-services)
   - [Load Testing](#load-testing)
9. [Webhook Integration](#webhook-integration)
   - [Setting Up Webhooks](#setting-up-webhooks)
   - [Webhook Security](#webhook-security)
   - [Event Processing](#event-processing)
10. [Custom Middleware](#custom-middleware)
    - [Request Interceptors](#request-interceptors)
    - [Response Transformers](#response-transformers)
    - [Logging Middleware](#logging-middleware)
11. [Debugging and Troubleshooting](#debugging-and-troubleshooting)
    - [Debug Logging](#debug-logging)
    - [Common Issues](#common-issues)
    - [Performance Issues](#performance-issues)
12. [Best Practices](#best-practices)
13. [Anti-Patterns](#anti-patterns)
14. [Real-World Examples](#real-world-examples)
15. [API Reference Quick Guide](#api-reference-quick-guide)

---

## Introduction

Welcome to the LLM Cost Ops Developer Guide. This comprehensive guide is designed to help developers integrate, optimize, and maintain LLM cost tracking in their applications. Whether you're building a small prototype or a large-scale production system, this guide provides the knowledge and patterns you need.

### Who This Guide Is For

- Backend developers integrating cost tracking
- DevOps engineers setting up monitoring
- Platform engineers building internal tools
- Application developers using LLM APIs

### Prerequisites

- Basic understanding of REST APIs
- Familiarity with at least one of: Python, TypeScript, Go, or Rust
- Understanding of async programming concepts
- Knowledge of HTTP and authentication mechanisms

---

## Getting Started

### Installation

#### Python SDK

```bash
# Using pip
pip install llm-cost-ops

# Using poetry
poetry add llm-cost-ops

# Using pipenv
pipenv install llm-cost-ops

# With optional dependencies
pip install llm-cost-ops[async,redis,postgres]
```

#### TypeScript SDK

```bash
# Using npm
npm install @llm-cost-ops/sdk

# Using yarn
yarn add @llm-cost-ops/sdk

# Using pnpm
pnpm add @llm-cost-ops/sdk

# With type definitions
npm install --save-dev @types/llm-cost-ops
```

#### Go SDK

```bash
# Using go get
go get github.com/llm-cost-ops/go-sdk

# Using go modules
go mod download github.com/llm-cost-ops/go-sdk

# Specific version
go get github.com/llm-cost-ops/go-sdk@v1.2.3
```

#### Rust SDK

```toml
# Add to Cargo.toml
[dependencies]
llm-cost-ops = "0.3.0"

# With async support
llm-cost-ops = { version = "0.3.0", features = ["async", "tokio"] }

# With all features
llm-cost-ops = { version = "0.3.0", features = ["full"] }
```

### Authentication

#### API Key Authentication

```python
# Python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(
    api_key="your-api-key-here",
    base_url="https://api.llmcostops.com"
)
```

```typescript
// TypeScript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: 'your-api-key-here',
  baseUrl: 'https://api.llmcostops.com'
});
```

```go
// Go
package main

import "github.com/llm-cost-ops/go-sdk"

func main() {
    client := costops.NewClient(
        costops.WithAPIKey("your-api-key-here"),
        costops.WithBaseURL("https://api.llmcostops.com"),
    )
}
```

```rust
// Rust
use llm_cost_ops::{CostOpsClient, Config};

fn main() {
    let client = CostOpsClient::new(Config {
        api_key: "your-api-key-here".to_string(),
        base_url: "https://api.llmcostops.com".to_string(),
        ..Default::default()
    });
}
```

#### OAuth 2.0 Authentication

```python
# Python
from llm_cost_ops import CostOpsClient
from llm_cost_ops.auth import OAuth2

auth = OAuth2(
    client_id="your-client-id",
    client_secret="your-client-secret",
    token_url="https://auth.llmcostops.com/oauth/token"
)

client = CostOpsClient(auth=auth)
```

```typescript
// TypeScript
import { CostOpsClient, OAuth2Provider } from '@llm-cost-ops/sdk';

const authProvider = new OAuth2Provider({
  clientId: 'your-client-id',
  clientSecret: 'your-client-secret',
  tokenUrl: 'https://auth.llmcostops.com/oauth/token'
});

const client = new CostOpsClient({ authProvider });
```

```go
// Go
import (
    "github.com/llm-cost-ops/go-sdk"
    "github.com/llm-cost-ops/go-sdk/auth"
)

oauth := auth.NewOAuth2(auth.OAuth2Config{
    ClientID:     "your-client-id",
    ClientSecret: "your-client-secret",
    TokenURL:     "https://auth.llmcostops.com/oauth/token",
})

client := costops.NewClient(
    costops.WithAuthProvider(oauth),
)
```

```rust
// Rust
use llm_cost_ops::{CostOpsClient, OAuth2Config};

let oauth_config = OAuth2Config {
    client_id: "your-client-id".to_string(),
    client_secret: "your-client-secret".to_string(),
    token_url: "https://auth.llmcostops.com/oauth/token".to_string(),
};

let client = CostOpsClient::with_oauth(oauth_config);
```

### Basic Configuration

#### Environment Variables

```bash
# .env file
LLM_COST_OPS_API_KEY=your-api-key-here
LLM_COST_OPS_BASE_URL=https://api.llmcostops.com
LLM_COST_OPS_TIMEOUT=30
LLM_COST_OPS_MAX_RETRIES=3
LLM_COST_OPS_ENVIRONMENT=production
```

#### Configuration Files

```yaml
# config.yaml
llm_cost_ops:
  api_key: ${LLM_COST_OPS_API_KEY}
  base_url: https://api.llmcostops.com
  timeout: 30
  max_retries: 3
  retry_delay: 1.0
  retry_backoff: 2.0
  connection_pool_size: 10
  enable_compression: true
  enable_caching: true
  cache_ttl: 300
  log_level: INFO
```

```json
// config.json
{
  "llm_cost_ops": {
    "api_key": "${LLM_COST_OPS_API_KEY}",
    "base_url": "https://api.llmcostops.com",
    "timeout": 30,
    "max_retries": 3,
    "retry_delay": 1.0,
    "retry_backoff": 2.0,
    "connection_pool_size": 10,
    "enable_compression": true,
    "enable_caching": true,
    "cache_ttl": 300,
    "log_level": "INFO"
  }
}
```

---

## SDK Usage Patterns

### Python SDK

#### Basic Usage

```python
from llm_cost_ops import CostOpsClient
from datetime import datetime, timedelta

# Initialize client
client = CostOpsClient(api_key="your-api-key")

# Track a cost entry
cost_entry = client.costs.create(
    model="gpt-4",
    tokens_used=1500,
    cost=0.045,
    user_id="user-123",
    metadata={
        "endpoint": "/api/chat",
        "model_version": "0613",
        "temperature": 0.7
    }
)

print(f"Cost entry created: {cost_entry.id}")

# Query costs
costs = client.costs.list(
    start_date=datetime.now() - timedelta(days=7),
    end_date=datetime.now(),
    filters={
        "model": ["gpt-4", "gpt-3.5-turbo"],
        "user_id": "user-123"
    }
)

total_cost = sum(cost.amount for cost in costs)
print(f"Total cost: ${total_cost:.2f}")
```

#### Context Manager Pattern

```python
from llm_cost_ops import CostOpsClient

# Automatic resource cleanup
with CostOpsClient(api_key="your-api-key") as client:
    result = client.costs.create(
        model="gpt-4",
        tokens_used=1000,
        cost=0.03
    )

# Connection automatically closed after context
```

#### Async Operations

```python
import asyncio
from llm_cost_ops import AsyncCostOpsClient

async def track_costs():
    async with AsyncCostOpsClient(api_key="your-api-key") as client:
        # Create multiple entries concurrently
        tasks = [
            client.costs.create(model="gpt-4", tokens_used=1000, cost=0.03),
            client.costs.create(model="gpt-3.5-turbo", tokens_used=500, cost=0.001),
            client.costs.create(model="claude-2", tokens_used=800, cost=0.024)
        ]

        results = await asyncio.gather(*tasks)
        return results

# Run async function
results = asyncio.run(track_costs())
print(f"Created {len(results)} cost entries")
```

#### Decorator Pattern for Cost Tracking

```python
from llm_cost_ops import CostOpsClient, track_cost
from functools import wraps

client = CostOpsClient(api_key="your-api-key")

@track_cost(client, model="gpt-4")
def generate_response(prompt: str) -> str:
    # Your LLM API call here
    response = openai.ChatCompletion.create(
        model="gpt-4",
        messages=[{"role": "user", "content": prompt}]
    )

    # Return response and usage info for tracking
    return response.choices[0].message.content, response.usage

result, usage = generate_response("Hello, world!")
# Cost automatically tracked
```

### TypeScript SDK

#### Basic Usage

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: 'your-api-key'
});

// Track a cost entry
const costEntry = await client.costs.create({
  model: 'gpt-4',
  tokensUsed: 1500,
  cost: 0.045,
  userId: 'user-123',
  metadata: {
    endpoint: '/api/chat',
    modelVersion: '0613',
    temperature: 0.7
  }
});

console.log(`Cost entry created: ${costEntry.id}`);

// Query costs
const costs = await client.costs.list({
  startDate: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
  endDate: new Date(),
  filters: {
    model: ['gpt-4', 'gpt-3.5-turbo'],
    userId: 'user-123'
  }
});

const totalCost = costs.reduce((sum, cost) => sum + cost.amount, 0);
console.log(`Total cost: $${totalCost.toFixed(2)}`);
```

#### Promise-Based Operations

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({ apiKey: 'your-api-key' });

// Chain promises
client.costs.create({
  model: 'gpt-4',
  tokensUsed: 1000,
  cost: 0.03
})
  .then(entry => {
    console.log('Entry created:', entry.id);
    return client.costs.get(entry.id);
  })
  .then(entry => {
    console.log('Entry details:', entry);
  })
  .catch(error => {
    console.error('Error:', error);
  });
```

#### Async/Await with Error Handling

```typescript
import { CostOpsClient, CostOpsError } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({ apiKey: 'your-api-key' });

async function trackCosts(): Promise<void> {
  try {
    // Parallel operations
    const [entry1, entry2, entry3] = await Promise.all([
      client.costs.create({ model: 'gpt-4', tokensUsed: 1000, cost: 0.03 }),
      client.costs.create({ model: 'gpt-3.5-turbo', tokensUsed: 500, cost: 0.001 }),
      client.costs.create({ model: 'claude-2', tokensUsed: 800, cost: 0.024 })
    ]);

    console.log(`Created ${[entry1, entry2, entry3].length} entries`);
  } catch (error) {
    if (error instanceof CostOpsError) {
      console.error('CostOps error:', error.message, error.statusCode);
    } else {
      console.error('Unexpected error:', error);
    }
  }
}

trackCosts();
```

#### Middleware Pattern

```typescript
import { CostOpsClient, Middleware } from '@llm-cost-ops/sdk';

// Custom middleware for logging
const loggingMiddleware: Middleware = async (request, next) => {
  console.log(`Request: ${request.method} ${request.url}`);
  const startTime = Date.now();

  const response = await next(request);

  const duration = Date.now() - startTime;
  console.log(`Response: ${response.status} (${duration}ms)`);

  return response;
};

// Custom middleware for retry with exponential backoff
const retryMiddleware: Middleware = async (request, next) => {
  let lastError: Error | null = null;

  for (let i = 0; i < 3; i++) {
    try {
      return await next(request);
    } catch (error) {
      lastError = error as Error;
      await new Promise(resolve => setTimeout(resolve, Math.pow(2, i) * 1000));
    }
  }

  throw lastError;
};

const client = new CostOpsClient({
  apiKey: 'your-api-key',
  middleware: [loggingMiddleware, retryMiddleware]
});
```

### Go SDK

#### Basic Usage

```go
package main

import (
    "context"
    "fmt"
    "log"
    "time"

    costops "github.com/llm-cost-ops/go-sdk"
)

func main() {
    // Initialize client
    client := costops.NewClient(
        costops.WithAPIKey("your-api-key"),
    )
    defer client.Close()

    ctx := context.Background()

    // Track a cost entry
    entry, err := client.Costs.Create(ctx, &costops.CostEntry{
        Model:      "gpt-4",
        TokensUsed: 1500,
        Cost:       0.045,
        UserID:     "user-123",
        Metadata: map[string]interface{}{
            "endpoint":      "/api/chat",
            "modelVersion":  "0613",
            "temperature":   0.7,
        },
    })
    if err != nil {
        log.Fatalf("Failed to create cost entry: %v", err)
    }

    fmt.Printf("Cost entry created: %s\n", entry.ID)

    // Query costs
    costs, err := client.Costs.List(ctx, &costops.ListOptions{
        StartDate: time.Now().AddDate(0, 0, -7),
        EndDate:   time.Now(),
        Filters: costops.Filters{
            Model:  []string{"gpt-4", "gpt-3.5-turbo"},
            UserID: "user-123",
        },
    })
    if err != nil {
        log.Fatalf("Failed to list costs: %v", err)
    }

    var totalCost float64
    for _, cost := range costs {
        totalCost += cost.Amount
    }
    fmt.Printf("Total cost: $%.2f\n", totalCost)
}
```

#### Context-Based Operations

```go
package main

import (
    "context"
    "log"
    "time"

    costops "github.com/llm-cost-ops/go-sdk"
)

func trackCostWithTimeout() {
    client := costops.NewClient(
        costops.WithAPIKey("your-api-key"),
    )
    defer client.Close()

    // Context with timeout
    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()

    entry, err := client.Costs.Create(ctx, &costops.CostEntry{
        Model:      "gpt-4",
        TokensUsed: 1000,
        Cost:       0.03,
    })
    if err != nil {
        if ctx.Err() == context.DeadlineExceeded {
            log.Println("Request timed out")
        } else {
            log.Printf("Error: %v", err)
        }
        return
    }

    log.Printf("Entry created: %s", entry.ID)
}
```

#### Concurrent Operations with Goroutines

```go
package main

import (
    "context"
    "fmt"
    "sync"

    costops "github.com/llm-cost-ops/go-sdk"
)

func trackCostsConcurrently() {
    client := costops.NewClient(
        costops.WithAPIKey("your-api-key"),
    )
    defer client.Close()

    ctx := context.Background()

    entries := []costops.CostEntry{
        {Model: "gpt-4", TokensUsed: 1000, Cost: 0.03},
        {Model: "gpt-3.5-turbo", TokensUsed: 500, Cost: 0.001},
        {Model: "claude-2", TokensUsed: 800, Cost: 0.024},
    }

    var wg sync.WaitGroup
    results := make(chan *costops.CostEntry, len(entries))
    errors := make(chan error, len(entries))

    for _, entry := range entries {
        wg.Add(1)
        go func(e costops.CostEntry) {
            defer wg.Done()

            result, err := client.Costs.Create(ctx, &e)
            if err != nil {
                errors <- err
                return
            }
            results <- result
        }(entry)
    }

    wg.Wait()
    close(results)
    close(errors)

    fmt.Printf("Created %d entries\n", len(results))

    for err := range errors {
        fmt.Printf("Error: %v\n", err)
    }
}
```

#### Custom HTTP Client

```go
package main

import (
    "net/http"
    "time"

    costops "github.com/llm-cost-ops/go-sdk"
)

func main() {
    // Custom HTTP client with connection pooling
    httpClient := &http.Client{
        Timeout: 30 * time.Second,
        Transport: &http.Transport{
            MaxIdleConns:        100,
            MaxIdleConnsPerHost: 10,
            IdleConnTimeout:     90 * time.Second,
        },
    }

    client := costops.NewClient(
        costops.WithAPIKey("your-api-key"),
        costops.WithHTTPClient(httpClient),
    )
    defer client.Close()

    // Use client as normal
}
```

### Rust SDK

#### Basic Usage

```rust
use llm_cost_ops::{CostOpsClient, CostEntry, Config};
use chrono::{Duration, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let client = CostOpsClient::new(Config {
        api_key: "your-api-key".to_string(),
        ..Default::default()
    });

    // Track a cost entry
    let entry = client.costs().create(CostEntry {
        model: "gpt-4".to_string(),
        tokens_used: 1500,
        cost: 0.045,
        user_id: Some("user-123".to_string()),
        metadata: Some(serde_json::json!({
            "endpoint": "/api/chat",
            "model_version": "0613",
            "temperature": 0.7
        })),
        ..Default::default()
    }).await?;

    println!("Cost entry created: {}", entry.id);

    // Query costs
    let costs = client.costs().list()
        .start_date(Utc::now() - Duration::days(7))
        .end_date(Utc::now())
        .model(vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()])
        .user_id("user-123")
        .execute()
        .await?;

    let total_cost: f64 = costs.iter().map(|c| c.amount).sum();
    println!("Total cost: ${:.2}", total_cost);

    Ok(())
}
```

#### Result-Based Error Handling

```rust
use llm_cost_ops::{CostOpsClient, CostEntry, CostOpsError};

async fn track_cost() -> Result<String, CostOpsError> {
    let client = CostOpsClient::from_env()?;

    let entry = client.costs().create(CostEntry {
        model: "gpt-4".to_string(),
        tokens_used: 1000,
        cost: 0.03,
        ..Default::default()
    }).await?;

    Ok(entry.id)
}

#[tokio::main]
async fn main() {
    match track_cost().await {
        Ok(id) => println!("Entry created: {}", id),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

#### Async Concurrent Operations

```rust
use llm_cost_ops::{CostOpsClient, CostEntry};
use futures::future::join_all;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CostOpsClient::from_env()?;

    let entries = vec![
        CostEntry {
            model: "gpt-4".to_string(),
            tokens_used: 1000,
            cost: 0.03,
            ..Default::default()
        },
        CostEntry {
            model: "gpt-3.5-turbo".to_string(),
            tokens_used: 500,
            cost: 0.001,
            ..Default::default()
        },
        CostEntry {
            model: "claude-2".to_string(),
            tokens_used: 800,
            cost: 0.024,
            ..Default::default()
        },
    ];

    // Create all entries concurrently
    let futures = entries.into_iter()
        .map(|entry| client.costs().create(entry));

    let results = join_all(futures).await;

    let success_count = results.iter()
        .filter(|r| r.is_ok())
        .count();

    println!("Created {} entries", success_count);

    Ok(())
}
```

#### Builder Pattern

```rust
use llm_cost_ops::{CostOpsClient, ConfigBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CostOpsClient::builder()
        .api_key("your-api-key")
        .base_url("https://api.llmcostops.com")
        .timeout(30)
        .max_retries(3)
        .retry_delay(1.0)
        .enable_compression(true)
        .build()?;

    // Use client
    Ok(())
}
```

---

## Advanced Integration Patterns

### Async/Await Patterns

#### Python AsyncIO

```python
import asyncio
from llm_cost_ops import AsyncCostOpsClient
from typing import List

async def batch_track_costs(entries: List[dict]) -> List[str]:
    """Track multiple cost entries concurrently."""
    async with AsyncCostOpsClient(api_key="your-api-key") as client:
        tasks = [
            client.costs.create(**entry)
            for entry in entries
        ]

        results = await asyncio.gather(*tasks, return_exceptions=True)

        # Filter successful results
        entry_ids = [
            result.id for result in results
            if not isinstance(result, Exception)
        ]

        return entry_ids

# Usage
entries = [
    {"model": "gpt-4", "tokens_used": 1000, "cost": 0.03},
    {"model": "gpt-3.5-turbo", "tokens_used": 500, "cost": 0.001},
    {"model": "claude-2", "tokens_used": 800, "cost": 0.024},
]

ids = asyncio.run(batch_track_costs(entries))
print(f"Created {len(ids)} entries")
```

#### TypeScript Async Patterns

```typescript
import { CostOpsClient, CostEntry } from '@llm-cost-ops/sdk';

class CostTracker {
  private client: CostOpsClient;

  constructor(apiKey: string) {
    this.client = new CostOpsClient({ apiKey });
  }

  async trackWithRetry(
    entry: Partial<CostEntry>,
    maxRetries: number = 3
  ): Promise<CostEntry> {
    let lastError: Error | null = null;

    for (let i = 0; i < maxRetries; i++) {
      try {
        return await this.client.costs.create(entry);
      } catch (error) {
        lastError = error as Error;

        // Exponential backoff
        const delay = Math.pow(2, i) * 1000;
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }

    throw lastError;
  }

  async batchTrack(entries: Partial<CostEntry>[]): Promise<CostEntry[]> {
    // Process in chunks to avoid overwhelming the API
    const chunkSize = 10;
    const results: CostEntry[] = [];

    for (let i = 0; i < entries.length; i += chunkSize) {
      const chunk = entries.slice(i, i + chunkSize);
      const chunkResults = await Promise.all(
        chunk.map(entry => this.trackWithRetry(entry))
      );
      results.push(...chunkResults);
    }

    return results;
  }
}
```

#### Go Concurrent Pattern

```go
package main

import (
    "context"
    "sync"
    "time"

    costops "github.com/llm-cost-ops/go-sdk"
)

type CostTracker struct {
    client    *costops.Client
    batchSize int
    workers   int
}

func NewCostTracker(apiKey string, workers int) *CostTracker {
    return &CostTracker{
        client:    costops.NewClient(costops.WithAPIKey(apiKey)),
        batchSize: 10,
        workers:   workers,
    }
}

func (t *CostTracker) BatchTrack(ctx context.Context, entries []*costops.CostEntry) error {
    jobs := make(chan *costops.CostEntry, len(entries))
    results := make(chan *costops.CostEntry, len(entries))
    errors := make(chan error, len(entries))

    // Start workers
    var wg sync.WaitGroup
    for i := 0; i < t.workers; i++ {
        wg.Add(1)
        go func() {
            defer wg.Done()

            for entry := range jobs {
                result, err := t.client.Costs.Create(ctx, entry)
                if err != nil {
                    errors <- err
                    continue
                }
                results <- result
            }
        }()
    }

    // Send jobs
    go func() {
        for _, entry := range entries {
            jobs <- entry
        }
        close(jobs)
    }()

    // Wait for completion
    wg.Wait()
    close(results)
    close(errors)

    // Check for errors
    for err := range errors {
        return err
    }

    return nil
}
```

### Batch Operations

#### Python Batch Processing

```python
from llm_cost_ops import CostOpsClient
from typing import List, Iterator
import time

class BatchCostTracker:
    def __init__(self, api_key: str, batch_size: int = 100):
        self.client = CostOpsClient(api_key=api_key)
        self.batch_size = batch_size

    def chunk_entries(self, entries: List[dict]) -> Iterator[List[dict]]:
        """Split entries into batches."""
        for i in range(0, len(entries), self.batch_size):
            yield entries[i:i + self.batch_size]

    def track_batch(self, entries: List[dict]) -> dict:
        """Track a batch of cost entries with rate limiting."""
        results = {
            "successful": 0,
            "failed": 0,
            "errors": []
        }

        for batch in self.chunk_entries(entries):
            try:
                # Use bulk create API
                created = self.client.costs.bulk_create(batch)
                results["successful"] += len(created)
            except Exception as e:
                results["failed"] += len(batch)
                results["errors"].append(str(e))

            # Rate limiting
            time.sleep(0.1)

        return results

# Usage
tracker = BatchCostTracker(api_key="your-api-key", batch_size=50)

entries = [
    {"model": "gpt-4", "tokens_used": i * 100, "cost": i * 0.003}
    for i in range(1, 1001)
]

results = tracker.track_batch(entries)
print(f"Successful: {results['successful']}, Failed: {results['failed']}")
```

#### TypeScript Batch with Queue

```typescript
import { CostOpsClient, CostEntry } from '@llm-cost-ops/sdk';
import PQueue from 'p-queue';

class BatchCostTracker {
  private client: CostOpsClient;
  private queue: PQueue;

  constructor(apiKey: string, concurrency: number = 5) {
    this.client = new CostOpsClient({ apiKey });
    this.queue = new PQueue({ concurrency });
  }

  async trackBatch(entries: Partial<CostEntry>[]): Promise<{
    successful: number;
    failed: number;
    errors: Error[];
  }> {
    const results = {
      successful: 0,
      failed: 0,
      errors: [] as Error[]
    };

    const tasks = entries.map(entry =>
      this.queue.add(async () => {
        try {
          await this.client.costs.create(entry);
          results.successful++;
        } catch (error) {
          results.failed++;
          results.errors.push(error as Error);
        }
      })
    );

    await Promise.all(tasks);

    return results;
  }

  async waitForCompletion(): Promise<void> {
    await this.queue.onIdle();
  }
}

// Usage
const tracker = new BatchCostTracker('your-api-key', 10);

const entries = Array.from({ length: 1000 }, (_, i) => ({
  model: 'gpt-4',
  tokensUsed: i * 100,
  cost: i * 0.003
}));

const results = await tracker.trackBatch(entries);
console.log(`Successful: ${results.successful}, Failed: ${results.failed}`);
```

### Streaming Data

#### Python Stream Processing

```python
from llm_cost_ops import CostOpsClient
import asyncio
from typing import AsyncIterator

class StreamCostTracker:
    def __init__(self, api_key: str):
        self.client = CostOpsClient(api_key=api_key)

    async def stream_costs(
        self,
        start_date,
        end_date,
        chunk_size: int = 100
    ) -> AsyncIterator[dict]:
        """Stream cost entries in chunks."""
        offset = 0

        while True:
            costs = await self.client.costs.list(
                start_date=start_date,
                end_date=end_date,
                limit=chunk_size,
                offset=offset
            )

            if not costs:
                break

            for cost in costs:
                yield cost

            offset += chunk_size

            if len(costs) < chunk_size:
                break

    async def process_stream(self, start_date, end_date):
        """Process cost stream with aggregation."""
        total_cost = 0.0
        count = 0

        async for cost in self.stream_costs(start_date, end_date):
            total_cost += cost.amount
            count += 1

            if count % 100 == 0:
                print(f"Processed {count} entries, total: ${total_cost:.2f}")

        return total_cost, count
```

### Connection Pooling

#### Python Connection Pool

```python
from llm_cost_ops import CostOpsClient
from urllib3.util.retry import Retry
from requests.adapters import HTTPAdapter
import requests

# Configure session with connection pooling
session = requests.Session()

retry_strategy = Retry(
    total=3,
    backoff_factor=1,
    status_forcelist=[429, 500, 502, 503, 504]
)

adapter = HTTPAdapter(
    max_retries=retry_strategy,
    pool_connections=20,
    pool_maxsize=20
)

session.mount("http://", adapter)
session.mount("https://", adapter)

# Create client with custom session
client = CostOpsClient(
    api_key="your-api-key",
    session=session
)
```

#### Go Connection Pool

```go
package main

import (
    "net/http"
    "time"

    costops "github.com/llm-cost-ops/go-sdk"
)

func NewPooledClient(apiKey string) *costops.Client {
    transport := &http.Transport{
        MaxIdleConns:        100,
        MaxIdleConnsPerHost: 20,
        MaxConnsPerHost:     20,
        IdleConnTimeout:     90 * time.Second,
        DisableCompression:  false,
    }

    httpClient := &http.Client{
        Timeout:   30 * time.Second,
        Transport: transport,
    }

    return costops.NewClient(
        costops.WithAPIKey(apiKey),
        costops.WithHTTPClient(httpClient),
    )
}
```

---

## Error Handling and Retry Strategies

### Error Types

#### Python Error Handling

```python
from llm_cost_ops import (
    CostOpsClient,
    CostOpsError,
    AuthenticationError,
    RateLimitError,
    ValidationError,
    NetworkError
)

client = CostOpsClient(api_key="your-api-key")

try:
    entry = client.costs.create(
        model="gpt-4",
        tokens_used=1000,
        cost=0.03
    )
except AuthenticationError as e:
    print(f"Authentication failed: {e}")
    # Refresh credentials
except RateLimitError as e:
    print(f"Rate limit exceeded: {e}")
    print(f"Retry after: {e.retry_after} seconds")
    # Implement backoff
except ValidationError as e:
    print(f"Invalid data: {e}")
    print(f"Validation errors: {e.errors}")
    # Fix data and retry
except NetworkError as e:
    print(f"Network error: {e}")
    # Retry with exponential backoff
except CostOpsError as e:
    print(f"General error: {e}")
    print(f"Status code: {e.status_code}")
    print(f"Response: {e.response}")
```

#### TypeScript Error Handling

```typescript
import {
  CostOpsClient,
  CostOpsError,
  AuthenticationError,
  RateLimitError,
  ValidationError,
  NetworkError
} from '@llm-cost-ops/sdk';

const client = new CostOpsClient({ apiKey: 'your-api-key' });

async function trackCostWithErrorHandling() {
  try {
    const entry = await client.costs.create({
      model: 'gpt-4',
      tokensUsed: 1000,
      cost: 0.03
    });

    return entry;
  } catch (error) {
    if (error instanceof AuthenticationError) {
      console.error('Authentication failed:', error.message);
      // Refresh credentials
    } else if (error instanceof RateLimitError) {
      console.error('Rate limit exceeded:', error.message);
      console.log(`Retry after: ${error.retryAfter} seconds`);
      // Implement backoff
    } else if (error instanceof ValidationError) {
      console.error('Invalid data:', error.message);
      console.log('Validation errors:', error.errors);
      // Fix data and retry
    } else if (error instanceof NetworkError) {
      console.error('Network error:', error.message);
      // Retry with exponential backoff
    } else if (error instanceof CostOpsError) {
      console.error('General error:', error.message);
      console.log('Status code:', error.statusCode);
    } else {
      console.error('Unexpected error:', error);
    }

    throw error;
  }
}
```

### Retry Logic

#### Python Retry with Exponential Backoff

```python
from llm_cost_ops import CostOpsClient, CostOpsError, RateLimitError
import time
from typing import Callable, TypeVar, Any

T = TypeVar('T')

def retry_with_backoff(
    func: Callable[..., T],
    max_retries: int = 3,
    base_delay: float = 1.0,
    max_delay: float = 60.0,
    exponential_base: float = 2.0
) -> T:
    """Execute function with exponential backoff retry."""
    last_exception = None

    for attempt in range(max_retries):
        try:
            return func()
        except RateLimitError as e:
            # Use retry-after header if available
            delay = e.retry_after if hasattr(e, 'retry_after') else None
            if delay is None:
                delay = min(base_delay * (exponential_base ** attempt), max_delay)

            print(f"Rate limited, waiting {delay} seconds...")
            time.sleep(delay)
            last_exception = e
        except CostOpsError as e:
            if e.status_code >= 500:  # Server errors
                delay = min(base_delay * (exponential_base ** attempt), max_delay)
                print(f"Server error, retrying in {delay} seconds...")
                time.sleep(delay)
                last_exception = e
            else:
                raise  # Don't retry client errors

    raise last_exception

# Usage
client = CostOpsClient(api_key="your-api-key")

entry = retry_with_backoff(
    lambda: client.costs.create(
        model="gpt-4",
        tokens_used=1000,
        cost=0.03
    )
)
```

#### TypeScript Retry Decorator

```typescript
import { CostOpsError, RateLimitError } from '@llm-cost-ops/sdk';

function RetryWithBackoff(
  maxRetries: number = 3,
  baseDelay: number = 1000,
  maxDelay: number = 60000
) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      let lastError: Error | null = null;

      for (let attempt = 0; attempt < maxRetries; attempt++) {
        try {
          return await originalMethod.apply(this, args);
        } catch (error) {
          lastError = error as Error;

          if (error instanceof RateLimitError) {
            const delay = error.retryAfter
              ? error.retryAfter * 1000
              : Math.min(baseDelay * Math.pow(2, attempt), maxDelay);

            console.log(`Rate limited, waiting ${delay}ms...`);
            await new Promise(resolve => setTimeout(resolve, delay));
          } else if (
            error instanceof CostOpsError &&
            error.statusCode >= 500
          ) {
            const delay = Math.min(
              baseDelay * Math.pow(2, attempt),
              maxDelay
            );

            console.log(`Server error, retrying in ${delay}ms...`);
            await new Promise(resolve => setTimeout(resolve, delay));
          } else {
            throw error;
          }
        }
      }

      throw lastError;
    };

    return descriptor;
  };
}

// Usage
class CostTracker {
  @RetryWithBackoff(3, 1000, 60000)
  async trackCost(model: string, tokensUsed: number, cost: number) {
    // Implementation
  }
}
```

### Circuit Breakers

#### Python Circuit Breaker Pattern

```python
from llm_cost_ops import CostOpsClient
from enum import Enum
import time
from threading import Lock

class CircuitState(Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"

class CircuitBreaker:
    def __init__(
        self,
        failure_threshold: int = 5,
        recovery_timeout: float = 60.0,
        expected_exception: type = Exception
    ):
        self.failure_threshold = failure_threshold
        self.recovery_timeout = recovery_timeout
        self.expected_exception = expected_exception

        self.failure_count = 0
        self.last_failure_time = None
        self.state = CircuitState.CLOSED
        self.lock = Lock()

    def call(self, func, *args, **kwargs):
        with self.lock:
            if self.state == CircuitState.OPEN:
                if self._should_attempt_reset():
                    self.state = CircuitState.HALF_OPEN
                else:
                    raise Exception("Circuit breaker is OPEN")

        try:
            result = func(*args, **kwargs)
            self._on_success()
            return result
        except self.expected_exception as e:
            self._on_failure()
            raise e

    def _should_attempt_reset(self):
        return (
            self.last_failure_time is not None and
            time.time() - self.last_failure_time >= self.recovery_timeout
        )

    def _on_success(self):
        with self.lock:
            self.failure_count = 0
            self.state = CircuitState.CLOSED

    def _on_failure(self):
        with self.lock:
            self.failure_count += 1
            self.last_failure_time = time.time()

            if self.failure_count >= self.failure_threshold:
                self.state = CircuitState.OPEN

# Usage
client = CostOpsClient(api_key="your-api-key")
circuit_breaker = CircuitBreaker(failure_threshold=5, recovery_timeout=60.0)

try:
    entry = circuit_breaker.call(
        client.costs.create,
        model="gpt-4",
        tokens_used=1000,
        cost=0.03
    )
except Exception as e:
    print(f"Circuit breaker prevented call or call failed: {e}")
```

### Graceful Degradation

#### Python Graceful Degradation

```python
from llm_cost_ops import CostOpsClient, CostOpsError
from typing import Optional
import logging

logger = logging.getLogger(__name__)

class ResilientCostTracker:
    def __init__(self, api_key: str, fallback_enabled: bool = True):
        self.client = CostOpsClient(api_key=api_key)
        self.fallback_enabled = fallback_enabled
        self.local_cache = []

    def track_cost(
        self,
        model: str,
        tokens_used: int,
        cost: float,
        **kwargs
    ) -> Optional[str]:
        """Track cost with graceful degradation."""
        try:
            entry = self.client.costs.create(
                model=model,
                tokens_used=tokens_used,
                cost=cost,
                **kwargs
            )
            return entry.id
        except CostOpsError as e:
            logger.error(f"Failed to track cost: {e}")

            if self.fallback_enabled:
                # Store locally for later sync
                self.local_cache.append({
                    "model": model,
                    "tokens_used": tokens_used,
                    "cost": cost,
                    **kwargs
                })
                logger.info("Cost entry cached locally")
                return None
            else:
                raise

    def sync_cached_entries(self) -> int:
        """Sync locally cached entries to API."""
        synced = 0
        failed = []

        for entry in self.local_cache[:]:
            try:
                self.client.costs.create(**entry)
                self.local_cache.remove(entry)
                synced += 1
            except CostOpsError as e:
                logger.error(f"Failed to sync entry: {e}")
                failed.append(entry)

        logger.info(f"Synced {synced} entries, {len(failed)} failed")
        return synced
```

---

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/cost-tracking.yml
name: LLM Cost Tracking

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours

jobs:
  track-costs:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install dependencies
        run: |
          pip install llm-cost-ops

      - name: Track LLM costs
        env:
          LLM_COST_OPS_API_KEY: ${{ secrets.LLM_COST_OPS_API_KEY }}
        run: |
          python scripts/track_costs.py

      - name: Generate cost report
        env:
          LLM_COST_OPS_API_KEY: ${{ secrets.LLM_COST_OPS_API_KEY }}
        run: |
          python scripts/generate_report.py

      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: cost-report
          path: reports/cost-report-*.pdf

      - name: Post to Slack
        if: always()
        uses: slackapi/slack-github-action@v1
        with:
          webhook-url: ${{ secrets.SLACK_WEBHOOK }}
          payload: |
            {
              "text": "Cost tracking completed for ${{ github.repository }}"
            }

  cost-budget-check:
    runs-on: ubuntu-latest

    steps:
      - name: Check cost budget
        env:
          LLM_COST_OPS_API_KEY: ${{ secrets.LLM_COST_OPS_API_KEY }}
          BUDGET_THRESHOLD: 1000
        run: |
          python - <<EOF
          from llm_cost_ops import CostOpsClient
          import os
          import sys

          client = CostOpsClient(api_key=os.environ['LLM_COST_OPS_API_KEY'])
          budget = client.budgets.get_current()
          threshold = float(os.environ['BUDGET_THRESHOLD'])

          if budget.spent > threshold:
              print(f"Budget exceeded: ${budget.spent:.2f} > ${threshold:.2f}")
              sys.exit(1)
          else:
              print(f"Budget OK: ${budget.spent:.2f} / ${threshold:.2f}")
          EOF
```

### GitLab CI

```yaml
# .gitlab-ci.yml
stages:
  - track
  - report
  - alert

variables:
  PYTHON_VERSION: "3.11"

track_costs:
  stage: track
  image: python:${PYTHON_VERSION}
  script:
    - pip install llm-cost-ops
    - python scripts/track_costs.py
  only:
    - main
    - develop
  artifacts:
    paths:
      - data/costs.json
    expire_in: 1 week

generate_report:
  stage: report
  image: python:${PYTHON_VERSION}
  dependencies:
    - track_costs
  script:
    - pip install llm-cost-ops pandas matplotlib
    - python scripts/generate_report.py
  artifacts:
    paths:
      - reports/
    expire_in: 1 month

budget_alert:
  stage: alert
  image: python:${PYTHON_VERSION}
  script:
    - pip install llm-cost-ops
    - |
      python - <<EOF
      from llm_cost_ops import CostOpsClient
      import os

      client = CostOpsClient(api_key=os.environ['LLM_COST_OPS_API_KEY'])
      budget = client.budgets.get_current()

      if budget.utilization > 0.8:
          print(f"WARNING: Budget at {budget.utilization * 100:.1f}%")
      EOF
  only:
    - schedules
```

### Jenkins

```groovy
// Jenkinsfile
pipeline {
    agent any

    environment {
        LLM_COST_OPS_API_KEY = credentials('llm-cost-ops-api-key')
        PYTHON_VERSION = '3.11'
    }

    stages {
        stage('Setup') {
            steps {
                sh '''
                    python -m venv venv
                    . venv/bin/activate
                    pip install llm-cost-ops
                '''
            }
        }

        stage('Track Costs') {
            steps {
                sh '''
                    . venv/bin/activate
                    python scripts/track_costs.py
                '''
            }
        }

        stage('Generate Report') {
            steps {
                sh '''
                    . venv/bin/activate
                    python scripts/generate_report.py
                '''

                archiveArtifacts artifacts: 'reports/*.pdf', fingerprint: true
            }
        }

        stage('Budget Check') {
            steps {
                script {
                    def result = sh(
                        script: '''
                            . venv/bin/activate
                            python scripts/check_budget.py
                        ''',
                        returnStatus: true
                    )

                    if (result != 0) {
                        currentBuild.result = 'UNSTABLE'
                        slackSend(
                            color: 'warning',
                            message: "Budget threshold exceeded in ${env.JOB_NAME}"
                        )
                    }
                }
            }
        }
    }

    post {
        always {
            cleanWs()
        }
        failure {
            mail(
                to: 'team@example.com',
                subject: "Cost tracking failed: ${env.JOB_NAME}",
                body: "Check ${env.BUILD_URL} for details"
            )
        }
    }
}
```

### CircleCI

```yaml
# .circleci/config.yml
version: 2.1

orbs:
  python: circleci/python@2.1.1

jobs:
  track-costs:
    docker:
      - image: cimg/python:3.11
    steps:
      - checkout
      - python/install-packages:
          pkg-manager: pip
          pip-dependency-file: requirements.txt
      - run:
          name: Track LLM costs
          command: python scripts/track_costs.py
      - persist_to_workspace:
          root: .
          paths:
            - data/costs.json

  generate-report:
    docker:
      - image: cimg/python:3.11
    steps:
      - checkout
      - attach_workspace:
          at: .
      - python/install-packages:
          pkg-manager: pip
          pip-dependency-file: requirements.txt
      - run:
          name: Generate cost report
          command: python scripts/generate_report.py
      - store_artifacts:
          path: reports/
          destination: cost-reports

  budget-check:
    docker:
      - image: cimg/python:3.11
    steps:
      - checkout
      - python/install-packages:
          pkg-manager: pip
          pip-dependency-file: requirements.txt
      - run:
          name: Check budget
          command: |
            python scripts/check_budget.py || \
            curl -X POST -H 'Content-type: application/json' \
              --data '{"text":"Budget threshold exceeded!"}' \
              $SLACK_WEBHOOK_URL

workflows:
  cost-tracking:
    jobs:
      - track-costs
      - generate-report:
          requires:
            - track-costs
      - budget-check:
          requires:
            - track-costs
```

---

## Performance Optimization

### Caching Strategies

#### Python Redis Caching

```python
from llm_cost_ops import CostOpsClient
import redis
import json
from typing import Optional
from datetime import timedelta

class CachedCostOpsClient:
    def __init__(self, api_key: str, redis_url: str = "redis://localhost:6379"):
        self.client = CostOpsClient(api_key=api_key)
        self.redis = redis.from_url(redis_url)
        self.cache_ttl = 300  # 5 minutes

    def get_costs(self, cache_key: str, **kwargs) -> list:
        """Get costs with caching."""
        # Try cache first
        cached = self.redis.get(cache_key)
        if cached:
            return json.loads(cached)

        # Fetch from API
        costs = self.client.costs.list(**kwargs)

        # Cache result
        self.redis.setex(
            cache_key,
            self.cache_ttl,
            json.dumps([cost.to_dict() for cost in costs])
        )

        return costs

    def invalidate_cache(self, pattern: str = "*"):
        """Invalidate cache entries matching pattern."""
        for key in self.redis.scan_iter(pattern):
            self.redis.delete(key)
```

#### TypeScript In-Memory Caching

```typescript
import { CostOpsClient, CostEntry } from '@llm-cost-ops/sdk';
import NodeCache from 'node-cache';

class CachedCostOpsClient {
  private client: CostOpsClient;
  private cache: NodeCache;

  constructor(apiKey: string, cacheTTL: number = 300) {
    this.client = new CostOpsClient({ apiKey });
    this.cache = new NodeCache({ stdTTL: cacheTTL });
  }

  async getCosts(cacheKey: string, options: any): Promise<CostEntry[]> {
    // Try cache first
    const cached = this.cache.get<CostEntry[]>(cacheKey);
    if (cached) {
      return cached;
    }

    // Fetch from API
    const costs = await this.client.costs.list(options);

    // Cache result
    this.cache.set(cacheKey, costs);

    return costs;
  }

  invalidateCache(key?: string): void {
    if (key) {
      this.cache.del(key);
    } else {
      this.cache.flushAll();
    }
  }
}
```

### Query Optimization

#### Python Query Optimization

```python
from llm_cost_ops import CostOpsClient
from datetime import datetime, timedelta
from typing import List, Dict

class OptimizedCostQueries:
    def __init__(self, api_key: str):
        self.client = CostOpsClient(api_key=api_key)

    def get_aggregated_costs(
        self,
        start_date: datetime,
        end_date: datetime,
        group_by: List[str] = ["model", "user_id"]
    ) -> Dict:
        """Get pre-aggregated costs instead of raw data."""
        return self.client.costs.aggregate(
            start_date=start_date,
            end_date=end_date,
            group_by=group_by,
            metrics=["sum", "count", "avg"]
        )

    def get_recent_costs_optimized(self, days: int = 7) -> List:
        """Use indexed fields and limit results."""
        return self.client.costs.list(
            start_date=datetime.now() - timedelta(days=days),
            end_date=datetime.now(),
            fields=["id", "model", "cost", "created_at"],  # Only needed fields
            limit=1000,
            order_by=["-created_at"]  # Use indexed field
        )

    def batch_query_by_users(self, user_ids: List[str], days: int = 30) -> Dict:
        """Batch query for multiple users."""
        return self.client.costs.batch_query(
            filters=[
                {"user_id": user_id, "days": days}
                for user_id in user_ids
            ]
        )
```

### Resource Management

#### Go Resource Pool

```go
package main

import (
    "context"
    "sync"

    costops "github.com/llm-cost-ops/go-sdk"
)

type ClientPool struct {
    clients []*costops.Client
    mu      sync.Mutex
    index   int
}

func NewClientPool(apiKey string, poolSize int) *ClientPool {
    pool := &ClientPool{
        clients: make([]*costops.Client, poolSize),
    }

    for i := 0; i < poolSize; i++ {
        pool.clients[i] = costops.NewClient(
            costops.WithAPIKey(apiKey),
        )
    }

    return pool
}

func (p *ClientPool) GetClient() *costops.Client {
    p.mu.Lock()
    defer p.mu.Unlock()

    client := p.clients[p.index]
    p.index = (p.index + 1) % len(p.clients)

    return client
}

func (p *ClientPool) Close() error {
    for _, client := range p.clients {
        if err := client.Close(); err != nil {
            return err
        }
    }
    return nil
}
```

### Profiling and Benchmarking

#### Python Profiling

```python
from llm_cost_ops import CostOpsClient
import cProfile
import pstats
from io import StringIO
from functools import wraps
import time

def profile_function(func):
    """Decorator to profile function execution."""
    @wraps(func)
    def wrapper(*args, **kwargs):
        profiler = cProfile.Profile()
        profiler.enable()

        result = func(*args, **kwargs)

        profiler.disable()

        # Print stats
        s = StringIO()
        ps = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
        ps.print_stats()
        print(s.getvalue())

        return result

    return wrapper

def benchmark_function(iterations: int = 100):
    """Decorator to benchmark function execution."""
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            times = []

            for _ in range(iterations):
                start = time.time()
                result = func(*args, **kwargs)
                end = time.time()
                times.append(end - start)

            avg_time = sum(times) / len(times)
            min_time = min(times)
            max_time = max(times)

            print(f"Benchmark results for {func.__name__}:")
            print(f"  Iterations: {iterations}")
            print(f"  Average: {avg_time*1000:.2f}ms")
            print(f"  Min: {min_time*1000:.2f}ms")
            print(f"  Max: {max_time*1000:.2f}ms")

            return result

        return wrapper

    return decorator

# Usage
client = CostOpsClient(api_key="your-api-key")

@profile_function
@benchmark_function(iterations=50)
def test_cost_creation():
    return client.costs.create(
        model="gpt-4",
        tokens_used=1000,
        cost=0.03
    )

test_cost_creation()
```

---

## Testing Strategies

### Unit Testing

#### Python Unit Tests

```python
# test_cost_tracking.py
import unittest
from unittest.mock import Mock, patch, MagicMock
from llm_cost_ops import CostOpsClient, CostOpsError

class TestCostTracking(unittest.TestCase):
    def setUp(self):
        self.client = CostOpsClient(api_key="test-api-key")

    @patch('llm_cost_ops.client.requests.post')
    def test_create_cost_entry(self, mock_post):
        # Mock API response
        mock_response = Mock()
        mock_response.status_code = 201
        mock_response.json.return_value = {
            "id": "cost-123",
            "model": "gpt-4",
            "tokens_used": 1000,
            "cost": 0.03
        }
        mock_post.return_value = mock_response

        # Test
        entry = self.client.costs.create(
            model="gpt-4",
            tokens_used=1000,
            cost=0.03
        )

        # Assertions
        self.assertEqual(entry.id, "cost-123")
        self.assertEqual(entry.model, "gpt-4")
        self.assertEqual(entry.tokens_used, 1000)
        mock_post.assert_called_once()

    @patch('llm_cost_ops.client.requests.post')
    def test_create_cost_entry_error(self, mock_post):
        # Mock error response
        mock_response = Mock()
        mock_response.status_code = 400
        mock_response.json.return_value = {
            "error": "Invalid data"
        }
        mock_post.return_value = mock_response

        # Test
        with self.assertRaises(CostOpsError):
            self.client.costs.create(
                model="invalid-model",
                tokens_used=-1,
                cost=0.03
            )

    def test_cost_calculation(self):
        """Test cost calculation logic."""
        tokens = 1000
        rate_per_1k = 0.03
        expected_cost = (tokens / 1000) * rate_per_1k

        # Your cost calculation function
        calculated_cost = self.client.calculate_cost(tokens, rate_per_1k)

        self.assertAlmostEqual(calculated_cost, expected_cost, places=4)

if __name__ == '__main__':
    unittest.main()
```

#### TypeScript Unit Tests

```typescript
// cost-tracking.test.ts
import { CostOpsClient, CostOpsError } from '@llm-cost-ops/sdk';
import { jest } from '@jest/globals';

describe('CostTracking', () => {
  let client: CostOpsClient;

  beforeEach(() => {
    client = new CostOpsClient({ apiKey: 'test-api-key' });
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('create', () => {
    it('should create a cost entry', async () => {
      // Mock fetch
      global.fetch = jest.fn().mockResolvedValue({
        ok: true,
        status: 201,
        json: async () => ({
          id: 'cost-123',
          model: 'gpt-4',
          tokensUsed: 1000,
          cost: 0.03
        })
      }) as jest.Mock;

      const entry = await client.costs.create({
        model: 'gpt-4',
        tokensUsed: 1000,
        cost: 0.03
      });

      expect(entry.id).toBe('cost-123');
      expect(entry.model).toBe('gpt-4');
      expect(fetch).toHaveBeenCalledTimes(1);
    });

    it('should handle errors', async () => {
      global.fetch = jest.fn().mockResolvedValue({
        ok: false,
        status: 400,
        json: async () => ({ error: 'Invalid data' })
      }) as jest.Mock;

      await expect(
        client.costs.create({
          model: 'invalid-model',
          tokensUsed: -1,
          cost: 0.03
        })
      ).rejects.toThrow(CostOpsError);
    });
  });

  describe('calculateCost', () => {
    it('should calculate cost correctly', () => {
      const tokens = 1000;
      const ratePer1k = 0.03;
      const expectedCost = (tokens / 1000) * ratePer1k;

      const calculatedCost = client.calculateCost(tokens, ratePer1k);

      expect(calculatedCost).toBeCloseTo(expectedCost, 4);
    });
  });
});
```

### Integration Testing

#### Python Integration Tests

```python
# test_integration.py
import unittest
import os
from llm_cost_ops import CostOpsClient
from datetime import datetime, timedelta

class TestIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Use test API key from environment
        api_key = os.getenv('LLM_COST_OPS_TEST_API_KEY')
        cls.client = CostOpsClient(
            api_key=api_key,
            base_url=os.getenv('LLM_COST_OPS_TEST_URL', 'https://api-test.llmcostops.com')
        )

    def test_cost_lifecycle(self):
        """Test complete cost entry lifecycle."""
        # Create
        entry = self.client.costs.create(
            model="gpt-4",
            tokens_used=1000,
            cost=0.03,
            metadata={"test": True}
        )
        self.assertIsNotNone(entry.id)

        # Read
        retrieved = self.client.costs.get(entry.id)
        self.assertEqual(retrieved.id, entry.id)
        self.assertEqual(retrieved.model, "gpt-4")

        # Update
        updated = self.client.costs.update(
            entry.id,
            metadata={"test": True, "updated": True}
        )
        self.assertTrue(updated.metadata.get("updated"))

        # Delete
        self.client.costs.delete(entry.id)

        # Verify deletion
        with self.assertRaises(Exception):
            self.client.costs.get(entry.id)

    def test_batch_operations(self):
        """Test batch cost entry creation."""
        entries = [
            {"model": "gpt-4", "tokens_used": i * 100, "cost": i * 0.003}
            for i in range(1, 11)
        ]

        created = self.client.costs.bulk_create(entries)
        self.assertEqual(len(created), 10)

        # Cleanup
        for entry in created:
            self.client.costs.delete(entry.id)

    def test_query_performance(self):
        """Test query performance with filters."""
        import time

        start = time.time()
        costs = self.client.costs.list(
            start_date=datetime.now() - timedelta(days=7),
            end_date=datetime.now(),
            limit=100
        )
        duration = time.time() - start

        # Should complete within 2 seconds
        self.assertLess(duration, 2.0)
        self.assertLessEqual(len(costs), 100)

if __name__ == '__main__':
    unittest.main()
```

### Mock Services

#### Python Mock Server

```python
# mock_server.py
from flask import Flask, request, jsonify
from datetime import datetime
import uuid

app = Flask(__name__)

# In-memory storage
costs = {}

@app.route('/api/v1/costs', methods=['POST'])
def create_cost():
    data = request.json

    cost_id = str(uuid.uuid4())
    cost_entry = {
        "id": cost_id,
        "model": data.get("model"),
        "tokens_used": data.get("tokens_used"),
        "cost": data.get("cost"),
        "user_id": data.get("user_id"),
        "metadata": data.get("metadata", {}),
        "created_at": datetime.utcnow().isoformat()
    }

    costs[cost_id] = cost_entry

    return jsonify(cost_entry), 201

@app.route('/api/v1/costs/<cost_id>', methods=['GET'])
def get_cost(cost_id):
    cost = costs.get(cost_id)

    if not cost:
        return jsonify({"error": "Cost entry not found"}), 404

    return jsonify(cost)

@app.route('/api/v1/costs', methods=['GET'])
def list_costs():
    return jsonify(list(costs.values()))

if __name__ == '__main__':
    app.run(debug=True, port=5001)
```

### Load Testing

#### Python Locust Load Test

```python
# locustfile.py
from locust import HttpUser, task, between
from datetime import datetime
import random

class CostOpsUser(HttpUser):
    wait_time = between(1, 3)

    def on_start(self):
        """Setup before starting tests."""
        self.api_key = "test-api-key"
        self.headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json"
        }

    @task(3)
    def create_cost_entry(self):
        """Simulate creating a cost entry."""
        models = ["gpt-4", "gpt-3.5-turbo", "claude-2"]

        payload = {
            "model": random.choice(models),
            "tokens_used": random.randint(100, 5000),
            "cost": round(random.uniform(0.001, 0.1), 4),
            "user_id": f"user-{random.randint(1, 100)}",
            "metadata": {
                "test": True,
                "timestamp": datetime.utcnow().isoformat()
            }
        }

        self.client.post(
            "/api/v1/costs",
            json=payload,
            headers=self.headers
        )

    @task(1)
    def list_costs(self):
        """Simulate listing costs."""
        self.client.get(
            "/api/v1/costs",
            params={"limit": 50},
            headers=self.headers
        )

    @task(2)
    def get_aggregates(self):
        """Simulate getting aggregated costs."""
        self.client.get(
            "/api/v1/costs/aggregate",
            params={
                "group_by": "model",
                "metric": "sum"
            },
            headers=self.headers
        )

# Run with: locust -f locustfile.py --host=https://api.llmcostops.com
```

---

## Webhook Integration

### Setting Up Webhooks

#### Python Webhook Server

```python
# webhook_server.py
from flask import Flask, request, jsonify
import hmac
import hashlib
from llm_cost_ops import CostOpsClient

app = Flask(__name__)

WEBHOOK_SECRET = "your-webhook-secret"
client = CostOpsClient(api_key="your-api-key")

def verify_signature(payload: bytes, signature: str) -> bool:
    """Verify webhook signature."""
    expected_signature = hmac.new(
        WEBHOOK_SECRET.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()

    return hmac.compare_digest(signature, expected_signature)

@app.route('/webhooks/costs', methods=['POST'])
def handle_cost_webhook():
    # Verify signature
    signature = request.headers.get('X-Webhook-Signature')
    if not verify_signature(request.data, signature):
        return jsonify({"error": "Invalid signature"}), 401

    event = request.json
    event_type = event.get('type')

    if event_type == 'cost.created':
        handle_cost_created(event['data'])
    elif event_type == 'cost.updated':
        handle_cost_updated(event['data'])
    elif event_type == 'budget.threshold_exceeded':
        handle_budget_alert(event['data'])

    return jsonify({"status": "received"}), 200

def handle_cost_created(data):
    """Handle cost created event."""
    print(f"New cost entry: {data['id']}")

    # Send notification if high cost
    if data['cost'] > 10.0:
        send_notification(f"High cost alert: ${data['cost']}")

def handle_cost_updated(data):
    """Handle cost updated event."""
    print(f"Cost entry updated: {data['id']}")

def handle_budget_alert(data):
    """Handle budget threshold exceeded."""
    print(f"Budget alert: {data['budget_name']}")
    send_notification(
        f"Budget '{data['budget_name']}' exceeded threshold: "
        f"{data['utilization']*100:.1f}%"
    )

def send_notification(message: str):
    """Send notification via preferred channel."""
    # Implement notification logic
    pass

if __name__ == '__main__':
    app.run(port=8080)
```

#### Registering Webhooks

```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(api_key="your-api-key")

# Register webhook
webhook = client.webhooks.create(
    url="https://your-domain.com/webhooks/costs",
    events=["cost.created", "cost.updated", "budget.threshold_exceeded"],
    secret="your-webhook-secret"
)

print(f"Webhook created: {webhook.id}")

# List webhooks
webhooks = client.webhooks.list()

# Update webhook
client.webhooks.update(
    webhook.id,
    events=["cost.created", "budget.threshold_exceeded"]
)

# Delete webhook
client.webhooks.delete(webhook.id)
```

### Webhook Security

#### TypeScript Webhook Verification

```typescript
import express from 'express';
import crypto from 'crypto';
import { CostOpsClient } from '@llm-cost-ops/sdk';

const app = express();
const WEBHOOK_SECRET = process.env.WEBHOOK_SECRET!;

function verifyWebhookSignature(
  payload: string,
  signature: string
): boolean {
  const expectedSignature = crypto
    .createHmac('sha256', WEBHOOK_SECRET)
    .update(payload)
    .digest('hex');

  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expectedSignature)
  );
}

app.post('/webhooks/costs', express.raw({ type: 'application/json' }), (req, res) => {
  const signature = req.headers['x-webhook-signature'] as string;
  const payload = req.body.toString();

  if (!verifyWebhookSignature(payload, signature)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  const event = JSON.parse(payload);

  // Process event
  handleEvent(event);

  res.json({ status: 'received' });
});

async function handleEvent(event: any) {
  switch (event.type) {
    case 'cost.created':
      await handleCostCreated(event.data);
      break;
    case 'budget.threshold_exceeded':
      await handleBudgetAlert(event.data);
      break;
  }
}

app.listen(8080);
```

### Event Processing

#### Go Webhook Handler

```go
package main

import (
    "crypto/hmac"
    "crypto/sha256"
    "encoding/hex"
    "encoding/json"
    "io/ioutil"
    "log"
    "net/http"
)

const webhookSecret = "your-webhook-secret"

type WebhookEvent struct {
    Type string                 `json:"type"`
    Data map[string]interface{} `json:"data"`
}

func verifySignature(payload []byte, signature string) bool {
    mac := hmac.New(sha256.New, []byte(webhookSecret))
    mac.Write(payload)
    expectedSignature := hex.EncodeToString(mac.Sum(nil))

    return hmac.Equal([]byte(signature), []byte(expectedSignature))
}

func webhookHandler(w http.ResponseWriter, r *http.Request) {
    payload, err := ioutil.ReadAll(r.Body)
    if err != nil {
        http.Error(w, "Failed to read body", http.StatusBadRequest)
        return
    }

    signature := r.Header.Get("X-Webhook-Signature")
    if !verifySignature(payload, signature) {
        http.Error(w, "Invalid signature", http.StatusUnauthorized)
        return
    }

    var event WebhookEvent
    if err := json.Unmarshal(payload, &event); err != nil {
        http.Error(w, "Invalid JSON", http.StatusBadRequest)
        return
    }

    // Process event
    handleEvent(event)

    w.WriteHeader(http.StatusOK)
    json.NewEncoder(w).Encode(map[string]string{"status": "received"})
}

func handleEvent(event WebhookEvent) {
    switch event.Type {
    case "cost.created":
        log.Printf("Cost created: %v", event.Data)
    case "budget.threshold_exceeded":
        log.Printf("Budget alert: %v", event.Data)
    }
}

func main() {
    http.HandleFunc("/webhooks/costs", webhookHandler)
    log.Fatal(http.ListenAndServe(":8080", nil))
}
```

---

## Custom Middleware

### Request Interceptors

#### Python Request Middleware

```python
from llm_cost_ops import CostOpsClient
from typing import Callable, Any
import time
import logging

logger = logging.getLogger(__name__)

class RequestMiddleware:
    def __init__(self, client: CostOpsClient):
        self.client = client
        self.middlewares = []

    def use(self, middleware: Callable):
        """Add middleware to the stack."""
        self.middlewares.append(middleware)

    def execute(self, func: Callable, *args, **kwargs) -> Any:
        """Execute function with middleware stack."""
        # Build middleware chain
        def execute_middlewares(index: int):
            if index >= len(self.middlewares):
                return func(*args, **kwargs)

            middleware = self.middlewares[index]
            return middleware(
                lambda: execute_middlewares(index + 1),
                *args,
                **kwargs
            )

        return execute_middlewares(0)

# Logging middleware
def logging_middleware(next_fn, *args, **kwargs):
    logger.info(f"Request: {args}, {kwargs}")
    start = time.time()

    result = next_fn()

    duration = time.time() - start
    logger.info(f"Response time: {duration:.3f}s")

    return result

# Rate limiting middleware
def rate_limit_middleware(next_fn, *args, **kwargs):
    # Implement rate limiting logic
    time.sleep(0.1)  # Simple delay
    return next_fn()

# Usage
client = CostOpsClient(api_key="your-api-key")
middleware = RequestMiddleware(client)
middleware.use(logging_middleware)
middleware.use(rate_limit_middleware)

result = middleware.execute(
    client.costs.create,
    model="gpt-4",
    tokens_used=1000,
    cost=0.03
)
```

### Response Transformers

#### TypeScript Response Middleware

```typescript
import { CostOpsClient, CostEntry } from '@llm-cost-ops/sdk';

type ResponseTransformer = (response: any) => any;

class EnhancedCostOpsClient extends CostOpsClient {
  private transformers: ResponseTransformer[] = [];

  addResponseTransformer(transformer: ResponseTransformer) {
    this.transformers.push(transformer);
  }

  private applyTransformers(response: any): any {
    return this.transformers.reduce(
      (acc, transformer) => transformer(acc),
      response
    );
  }

  async create(entry: Partial<CostEntry>): Promise<CostEntry> {
    const response = await super.create(entry);
    return this.applyTransformers(response);
  }
}

// Add currency converter transformer
const currencyConverterTransformer = (response: CostEntry) => {
  return {
    ...response,
    costInEUR: response.cost * 0.92, // Example conversion
    costInGBP: response.cost * 0.79
  };
};

// Add metadata enrichment transformer
const metadataEnrichmentTransformer = (response: CostEntry) => {
  return {
    ...response,
    enriched: true,
    processedAt: new Date().toISOString()
  };
};

// Usage
const client = new EnhancedCostOpsClient({ apiKey: 'your-api-key' });
client.addResponseTransformer(currencyConverterTransformer);
client.addResponseTransformer(metadataEnrichmentTransformer);

const entry = await client.create({
  model: 'gpt-4',
  tokensUsed: 1000,
  cost: 0.03
});

console.log(entry.costInEUR, entry.costInGBP);
```

### Logging Middleware

#### Rust Logging Middleware

```rust
use llm_cost_ops::{CostOpsClient, CostEntry};
use std::time::Instant;
use log::{info, error};

trait Middleware {
    fn before_request(&self, request: &str);
    fn after_request(&self, request: &str, duration: std::time::Duration);
    fn on_error(&self, request: &str, error: &str);
}

struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn before_request(&self, request: &str) {
        info!("Starting request: {}", request);
    }

    fn after_request(&self, request: &str, duration: std::time::Duration) {
        info!("Completed request: {} in {:?}", request, duration);
    }

    fn on_error(&self, request: &str, error: &str) {
        error!("Request failed: {} - {}", request, error);
    }
}

struct MiddlewareClient {
    client: CostOpsClient,
    middleware: Vec<Box<dyn Middleware>>,
}

impl MiddlewareClient {
    fn new(client: CostOpsClient) -> Self {
        Self {
            client,
            middleware: vec![],
        }
    }

    fn add_middleware(&mut self, middleware: Box<dyn Middleware>) {
        self.middleware.push(middleware);
    }

    async fn create_cost(&self, entry: CostEntry) -> Result<CostEntry, Box<dyn std::error::Error>> {
        let request_name = "create_cost";

        // Before hooks
        for mw in &self.middleware {
            mw.before_request(request_name);
        }

        let start = Instant::now();

        let result = self.client.costs().create(entry).await;

        let duration = start.elapsed();

        // After hooks
        match &result {
            Ok(_) => {
                for mw in &self.middleware {
                    mw.after_request(request_name, duration);
                }
            }
            Err(e) => {
                for mw in &self.middleware {
                    mw.on_error(request_name, &e.to_string());
                }
            }
        }

        result
    }
}
```

---

## Debugging and Troubleshooting

### Debug Logging

#### Python Debug Logging

```python
import logging
from llm_cost_ops import CostOpsClient

# Enable debug logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

# Create client with debug mode
client = CostOpsClient(
    api_key="your-api-key",
    debug=True,
    log_level=logging.DEBUG
)

# All requests and responses will be logged
entry = client.costs.create(
    model="gpt-4",
    tokens_used=1000,
    cost=0.03
)
```

#### TypeScript Debug Logging

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: 'your-api-key',
  debug: true,
  logger: {
    debug: (message: string, ...args: any[]) => {
      console.log('[DEBUG]', message, ...args);
    },
    info: (message: string, ...args: any[]) => {
      console.log('[INFO]', message, ...args);
    },
    warn: (message: string, ...args: any[]) => {
      console.warn('[WARN]', message, ...args);
    },
    error: (message: string, ...args: any[]) => {
      console.error('[ERROR]', message, ...args);
    }
  }
});
```

### Common Issues

#### Connection Issues

```python
# Python: Handling connection issues
from llm_cost_ops import CostOpsClient, NetworkError
import requests

try:
    client = CostOpsClient(
        api_key="your-api-key",
        timeout=10,
        max_retries=3
    )

    entry = client.costs.create(
        model="gpt-4",
        tokens_used=1000,
        cost=0.03
    )
except NetworkError as e:
    print(f"Network error: {e}")
    print("Troubleshooting:")
    print("1. Check internet connection")
    print("2. Verify API endpoint is accessible")
    print("3. Check firewall settings")
    print("4. Try increasing timeout")
except requests.exceptions.SSLError as e:
    print(f"SSL error: {e}")
    print("Try: client = CostOpsClient(api_key='...', verify_ssl=False)")
```

#### Authentication Issues

```typescript
// TypeScript: Handling auth issues
import { CostOpsClient, AuthenticationError } from '@llm-cost-ops/sdk';

async function handleAuthIssues() {
  try {
    const client = new CostOpsClient({
      apiKey: process.env.LLM_COST_OPS_API_KEY
    });

    await client.costs.create({
      model: 'gpt-4',
      tokensUsed: 1000,
      cost: 0.03
    });
  } catch (error) {
    if (error instanceof AuthenticationError) {
      console.error('Authentication failed');
      console.log('Troubleshooting:');
      console.log('1. Verify API key is correct');
      console.log('2. Check API key has not expired');
      console.log('3. Ensure API key has correct permissions');
      console.log('4. Try regenerating API key');
    }
  }
}
```

### Performance Issues

```python
# Python: Diagnosing performance issues
import time
from llm_cost_ops import CostOpsClient

def diagnose_performance():
    client = CostOpsClient(api_key="your-api-key")

    # Measure API response time
    start = time.time()
    client.costs.list(limit=10)
    api_time = time.time() - start

    print(f"API response time: {api_time:.3f}s")

    if api_time > 2.0:
        print("Slow API response detected")
        print("Suggestions:")
        print("1. Reduce query size (use limit parameter)")
        print("2. Enable caching")
        print("3. Use connection pooling")
        print("4. Consider batch operations")

    # Check connection pool
    print(f"Connection pool size: {client.session.adapters['https://'].max_retries.total}")
```

---

## Best Practices

1. **Always use environment variables for API keys**
   - Never hardcode credentials
   - Use .env files for local development
   - Use secrets management in production

2. **Implement proper error handling**
   - Catch specific exceptions
   - Implement retry logic with exponential backoff
   - Log errors for debugging

3. **Use connection pooling for high-throughput applications**
   - Reuse HTTP connections
   - Configure appropriate pool sizes
   - Monitor connection metrics

4. **Implement caching for read-heavy workloads**
   - Cache frequently accessed data
   - Set appropriate TTLs
   - Implement cache invalidation strategies

5. **Use batch operations when possible**
   - Reduce API calls
   - Improve performance
   - Handle rate limits better

6. **Monitor and log all operations**
   - Track API usage
   - Monitor error rates
   - Set up alerts for anomalies

7. **Test thoroughly**
   - Write unit tests
   - Implement integration tests
   - Perform load testing

8. **Follow the principle of least privilege**
   - Use API keys with minimal required permissions
   - Rotate keys regularly
   - Audit access logs

9. **Version your integrations**
   - Pin SDK versions
   - Test before upgrading
   - Maintain backwards compatibility

10. **Document your usage**
    - Comment complex logic
    - Maintain integration documentation
    - Share knowledge with team

---

## Anti-Patterns

1. **Hardcoding API keys**
   ```python
   # BAD
   client = CostOpsClient(api_key="sk-1234567890abcdef")

   # GOOD
   import os
   client = CostOpsClient(api_key=os.environ['LLM_COST_OPS_API_KEY'])
   ```

2. **Ignoring errors**
   ```python
   # BAD
   try:
       client.costs.create(...)
   except:
       pass

   # GOOD
   try:
       client.costs.create(...)
   except CostOpsError as e:
       logger.error(f"Failed to create cost entry: {e}")
       # Handle appropriately
   ```

3. **Not using connection pooling**
   ```python
   # BAD
   def track_cost():
       client = CostOpsClient(api_key="...")  # New connection each time
       client.costs.create(...)

   # GOOD
   client = CostOpsClient(api_key="...")  # Reuse client

   def track_cost():
       client.costs.create(...)
   ```

4. **Synchronous operations in async contexts**
   ```python
   # BAD
   async def process_costs():
       for cost in costs:
           client.costs.create(**cost)  # Blocking

   # GOOD
   async def process_costs():
       async with AsyncCostOpsClient() as client:
           tasks = [client.costs.create(**cost) for cost in costs]
           await asyncio.gather(*tasks)
   ```

5. **Not implementing timeouts**
   ```python
   # BAD
   client = CostOpsClient(api_key="...")

   # GOOD
   client = CostOpsClient(api_key="...", timeout=30)
   ```

---

## Real-World Examples

### Example 1: OpenAI Integration

```python
import openai
from llm_cost_ops import CostOpsClient
import os

openai.api_key = os.environ['OPENAI_API_KEY']
cost_client = CostOpsClient(api_key=os.environ['LLM_COST_OPS_API_KEY'])

# Pricing (per 1K tokens)
PRICING = {
    "gpt-4": {"input": 0.03, "output": 0.06},
    "gpt-3.5-turbo": {"input": 0.0015, "output": 0.002}
}

def chat_with_tracking(messages, model="gpt-4", user_id=None):
    """Send chat request and track costs."""
    response = openai.ChatCompletion.create(
        model=model,
        messages=messages
    )

    # Calculate costs
    usage = response.usage
    input_cost = (usage.prompt_tokens / 1000) * PRICING[model]["input"]
    output_cost = (usage.completion_tokens / 1000) * PRICING[model]["output"]
    total_cost = input_cost + output_cost

    # Track in Cost Ops
    cost_client.costs.create(
        model=model,
        tokens_used=usage.total_tokens,
        cost=total_cost,
        user_id=user_id,
        metadata={
            "prompt_tokens": usage.prompt_tokens,
            "completion_tokens": usage.completion_tokens,
            "input_cost": input_cost,
            "output_cost": output_cost
        }
    )

    return response.choices[0].message.content
```

### Example 2: Multi-Model Comparison

```typescript
import { Configuration, OpenAIApi } from 'openai';
import Anthropic from '@anthropic-ai/sdk';
import { CostOpsClient } from '@llm-cost-ops/sdk';

const openai = new OpenAIApi(new Configuration({
  apiKey: process.env.OPENAI_API_KEY
}));

const anthropic = new Anthropic({
  apiKey: process.env.ANTHROPIC_API_KEY
});

const costClient = new CostOpsClient({
  apiKey: process.env.LLM_COST_OPS_API_KEY!
});

interface ModelComparison {
  model: string;
  response: string;
  cost: number;
  latency: number;
}

async function compareModels(prompt: string): Promise<ModelComparison[]> {
  const results: ModelComparison[] = [];

  // Test GPT-4
  const gpt4Start = Date.now();
  const gpt4Response = await openai.createChatCompletion({
    model: 'gpt-4',
    messages: [{ role: 'user', content: prompt }]
  });
  const gpt4Latency = Date.now() - gpt4Start;
  const gpt4Cost = calculateOpenAICost('gpt-4', gpt4Response.data.usage!);

  await costClient.costs.create({
    model: 'gpt-4',
    tokensUsed: gpt4Response.data.usage!.total_tokens,
    cost: gpt4Cost,
    metadata: { latency: gpt4Latency }
  });

  results.push({
    model: 'gpt-4',
    response: gpt4Response.data.choices[0].message!.content!,
    cost: gpt4Cost,
    latency: gpt4Latency
  });

  // Test Claude
  const claudeStart = Date.now();
  const claudeResponse = await anthropic.messages.create({
    model: 'claude-3-opus-20240229',
    max_tokens: 1024,
    messages: [{ role: 'user', content: prompt }]
  });
  const claudeLatency = Date.now() - claudeStart;
  const claudeCost = calculateAnthropicCost(claudeResponse.usage);

  await costClient.costs.create({
    model: 'claude-3-opus',
    tokensUsed: claudeResponse.usage.input_tokens + claudeResponse.usage.output_tokens,
    cost: claudeCost,
    metadata: { latency: claudeLatency }
  });

  results.push({
    model: 'claude-3-opus',
    response: claudeResponse.content[0].text,
    cost: claudeCost,
    latency: claudeLatency
  });

  return results;
}

function calculateOpenAICost(model: string, usage: any): number {
  // Implementation
  return 0;
}

function calculateAnthropicCost(usage: any): number {
  // Implementation
  return 0;
}
```

### Example 3: Cost-Aware Request Routing

```go
package main

import (
    "context"
    "fmt"
    "time"

    costops "github.com/llm-cost-ops/go-sdk"
)

type ModelRouter struct {
    costClient *costops.Client
    models     []ModelConfig
}

type ModelConfig struct {
    Name      string
    MaxCost   float64
    Priority  int
}

func NewModelRouter(apiKey string) *ModelRouter {
    return &ModelRouter{
        costClient: costops.NewClient(costops.WithAPIKey(apiKey)),
        models: []ModelConfig{
            {Name: "gpt-3.5-turbo", MaxCost: 0.01, Priority: 1},
            {Name: "gpt-4", MaxCost: 0.10, Priority: 2},
            {Name: "claude-2", MaxCost: 0.05, Priority: 3},
        },
    }
}

func (r *ModelRouter) SelectModel(ctx context.Context, userID string) (string, error) {
    // Get user's current spending
    costs, err := r.costClient.Costs.List(ctx, &costops.ListOptions{
        StartDate: time.Now().AddDate(0, 0, -30),
        EndDate:   time.Now(),
        Filters: costops.Filters{
            UserID: userID,
        },
    })
    if err != nil {
        return "", err
    }

    var totalSpent float64
    for _, cost := range costs {
        totalSpent += cost.Amount
    }

    // Select model based on budget
    for _, model := range r.models {
        if totalSpent < model.MaxCost {
            return model.Name, nil
        }
    }

    // Default to cheapest model
    return r.models[0].Name, nil
}

func (r *ModelRouter) RouteRequest(ctx context.Context, userID, prompt string) (string, error) {
    model, err := r.SelectModel(ctx, userID)
    if err != nil {
        return "", err
    }

    fmt.Printf("Routing to model: %s\n", model)

    // Make LLM API call with selected model
    // Track cost
    // Return response

    return "", nil
}
```

---

## API Reference Quick Guide

### Core Methods

**Costs**
- `costs.create(model, tokens_used, cost, **kwargs)` - Create cost entry
- `costs.list(**filters)` - List cost entries
- `costs.get(id)` - Get specific cost entry
- `costs.update(id, **kwargs)` - Update cost entry
- `costs.delete(id)` - Delete cost entry
- `costs.bulk_create(entries)` - Bulk create entries
- `costs.aggregate(**kwargs)` - Get aggregated costs

**Budgets**
- `budgets.create(name, amount, **kwargs)` - Create budget
- `budgets.list()` - List budgets
- `budgets.get(id)` - Get specific budget
- `budgets.update(id, **kwargs)` - Update budget
- `budgets.delete(id)` - Delete budget

**Reports**
- `reports.generate(type, **kwargs)` - Generate report
- `reports.list()` - List reports
- `reports.get(id)` - Get specific report
- `reports.download(id, format)` - Download report

**Webhooks**
- `webhooks.create(url, events, **kwargs)` - Create webhook
- `webhooks.list()` - List webhooks
- `webhooks.update(id, **kwargs)` - Update webhook
- `webhooks.delete(id)` - Delete webhook

---

## Conclusion

This developer guide provides comprehensive coverage of integrating and optimizing LLM cost tracking using the LLM Cost Ops platform. For additional help:

- Visit our documentation: https://docs.llmcostops.com
- Join our community: https://community.llmcostops.com
- Contact support: support@llmcostops.com
- Report issues: https://github.com/llm-cost-ops/sdk/issues

Happy coding!
