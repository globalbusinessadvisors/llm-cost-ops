---
sidebar_position: 3
title: Go SDK
---

# Go SDK

Enterprise-grade Go SDK for the LLM Cost Operations platform. Track, analyze, and optimize costs across multiple LLM providers with idiomatic Go code.

## Features

- **Idiomatic Go**: Following Go best practices and conventions
- **Context-Aware**: Full `context.Context` support for cancellation and timeouts
- **Type-Safe**: Strongly typed API with compile-time safety
- **Goroutine-Safe**: Safe for concurrent use by multiple goroutines
- **Production-Ready**: Enterprise features including:
  - Automatic retry with exponential backoff
  - Rate limiting with token bucket algorithm
  - Structured logging (zap)
  - Connection pooling
  - Metrics hooks for instrumentation
  - Comprehensive error handling with sentinel errors

## Quick Example

```go
package main

import (
    "context"
    "log"

    llmcostops "github.com/llm-devops/llm-cost-ops/sdk/go"
)

func main() {
    // Create client
    client, err := llmcostops.NewClient(
        llmcostops.WithAPIKey("your-api-key"),
        llmcostops.WithBaseURL("https://api.costops.example.com"),
    )
    if err != nil {
        log.Fatal(err)
    }
    defer client.Close()

    ctx := context.Background()

    // Submit usage
    err = client.Usage.Submit(ctx, &llmcostops.UsageSubmitParams{
        OrganizationID:   "org-123",
        Provider:         "openai",
        ModelID:          "gpt-4",
        InputTokens:      1000,
        OutputTokens:     500,
        TotalTokens:      1500,
    })
    if err != nil {
        log.Fatal(err)
    }

    // Get cost summary
    summary, err := client.Costs.Summary(ctx, &llmcostops.CostSummaryParams{
        OrganizationID: "org-123",
        StartDate:      time.Now().AddDate(0, 0, -30),
        EndDate:        time.Now(),
    })
    if err != nil {
        log.Fatal(err)
    }

    log.Printf("Total cost: %s", summary.TotalCost)
}
```

## Installation

```bash
go get github.com/llm-devops/llm-cost-ops/sdk/go
```

## Requirements

- Go 1.21 or higher

## Package Structure

```
github.com/llm-devops/llm-cost-ops/sdk/go/
├── client.go          # Main client
├── config.go          # Configuration
├── errors.go          # Error types
├── models.go          # Data models
├── usage.go           # Usage service
├── costs.go           # Cost service
├── pricing.go         # Pricing service
├── export.go          # Export service
└── health.go          # Health service
```

## Core Concepts

### Client Configuration

```go
import (
    "time"
    llmcostops "github.com/llm-devops/llm-cost-ops/sdk/go"
)

client, err := llmcostops.NewClient(
    // Required
    llmcostops.WithAPIKey("your-api-key"),

    // Optional
    llmcostops.WithBaseURL("https://api.costops.example.com"),
    llmcostops.WithTimeout(30 * time.Second),
    llmcostops.WithMaxRetries(3),
    llmcostops.WithRetryDelay(time.Second),
    llmcostops.WithRateLimit(100), // requests per second
    llmcostops.WithLogger(logger),
    llmcostops.WithHTTPClient(httpClient),
    llmcostops.WithMetrics(metricsCollector),
)
```

### Context Support

All API methods accept `context.Context`:

```go
// With timeout
ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
defer cancel()

usage, err := client.Usage.List(ctx, params)

// With cancellation
ctx, cancel := context.WithCancel(context.Background())
go func() {
    // Cancel after some condition
    cancel()
}()

summary, err := client.Costs.Summary(ctx, params)
```

### Error Handling

The SDK provides sentinel errors:

```go
import (
    "errors"
    llmcostops "github.com/llm-devops/llm-cost-ops/sdk/go"
)

_, err := client.Usage.Get(ctx, "invalid-id")
if errors.Is(err, llmcostops.ErrNotFound) {
    // Handle not found
}

// Available sentinel errors:
// - ErrInvalidConfig
// - ErrUnauthorized
// - ErrNotFound
// - ErrRateLimited
// - ErrServerError
// - ErrBadRequest
// - ErrContextCanceled
```

## Performance

- **Goroutine-Safe**: All operations are safe for concurrent use
- **Connection Pooling**: HTTP client uses connection pooling by default
- **Rate Limiting**: Built-in token bucket rate limiter
- **Retry Logic**: Automatic retry with exponential backoff
- **Zero Allocations**: Optimized for minimal allocations in hot paths

## Next Steps

- [Installation Guide](/docs/sdks/go/installation)
- [Quick Start](/docs/sdks/go/quick-start)
- [API Reference](/docs/sdks/go/api-reference)
- [Examples](/docs/sdks/go/examples)
- [Troubleshooting](/docs/sdks/go/troubleshooting)

## Resources

- [pkg.go.dev](https://pkg.go.dev/github.com/llm-devops/llm-cost-ops/sdk/go)
- [Source Code](https://github.com/llm-devops/llm-cost-ops/tree/main/sdk/go)
- [Examples](https://github.com/llm-devops/llm-cost-ops/tree/main/sdk/go/examples)
- [Go Report Card](https://goreportcard.com/report/github.com/llm-devops/llm-cost-ops/sdk/go)

## Support

- [GitHub Issues](https://github.com/llm-devops/llm-cost-ops/issues)
- [Discord Community](https://discord.gg/llm-cost-ops)
- Email: support@llm-cost-ops.dev
