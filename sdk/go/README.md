# LLM Cost Ops Go SDK

[![Go Reference](https://pkg.go.dev/badge/github.com/llm-devops/llm-cost-ops/sdk/go.svg)](https://pkg.go.dev/github.com/llm-devops/llm-cost-ops/sdk/go)
[![Go Report Card](https://goreportcard.com/badge/github.com/llm-devops/llm-cost-ops/sdk/go)](https://goreportcard.com/report/github.com/llm-devops/llm-cost-ops/sdk/go)

Enterprise-grade Go SDK for the LLM Cost Operations platform. Track, analyze, and optimize costs across multiple LLM providers with production-ready accuracy.

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

## Installation

```bash
go get github.com/llm-devops/llm-cost-ops/sdk/go
```

## Quick Start

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

    // Add pricing
    pricing, err := client.Pricing.Add(ctx, &llmcostops.PricingAddParams{
        Provider:              llmcostops.ProviderOpenAI,
        Model:                 "gpt-4",
        InputPricePerMillion:  10.0,
        OutputPricePerMillion: 30.0,
    })
    if err != nil {
        log.Fatal(err)
    }

    // Ingest usage
    err = client.Usage.Ingest(ctx, &llmcostops.UsageIngestParams{
        Records: []llmcostops.UsageRecord{
            {
                Provider:         llmcostops.ProviderOpenAI,
                Model:            llmcostops.Model{Name: "gpt-4"},
                OrganizationID:   "org-123",
                PromptTokens:     1000,
                CompletionTokens: 500,
                TotalTokens:      1500,
            },
        },
    })
    if err != nil {
        log.Fatal(err)
    }

    // Get cost summary
    summary, err := client.Costs.Summary(ctx, &llmcostops.CostSummaryParams{
        Range: llmcostops.RangeLast30Days,
    })
    if err != nil {
        log.Fatal(err)
    }

    log.Printf("Total cost: %s", summary.TotalCost)
}
```

## Configuration

### Client Options

```go
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

### Custom HTTP Client

```go
httpClient := &http.Client{
    Timeout: 60 * time.Second,
    Transport: &http.Transport{
        MaxIdleConns:        100,
        MaxIdleConnsPerHost: 10,
        IdleConnTimeout:     90 * time.Second,
    },
}

client, err := llmcostops.NewClient(
    llmcostops.WithAPIKey("your-api-key"),
    llmcostops.WithHTTPClient(httpClient),
)
```

### Structured Logging

```go
import "go.uber.org/zap"

logger, _ := zap.NewProduction()
client, err := llmcostops.NewClient(
    llmcostops.WithAPIKey("your-api-key"),
    llmcostops.WithLogger(logger),
)
```

### Custom Metrics

```go
type MyMetrics struct{}

func (m *MyMetrics) RecordRequest(method string, statusCode int, duration time.Duration) {
    // Record metrics in your monitoring system
}

func (m *MyMetrics) RecordError(operation string, err error) {
    // Record errors
}

client, err := llmcostops.NewClient(
    llmcostops.WithAPIKey("your-api-key"),
    llmcostops.WithMetrics(&MyMetrics{}),
)
```

## API Reference

### Pricing Service

```go
// Add pricing
pricing, err := client.Pricing.Add(ctx, &llmcostops.PricingAddParams{
    Provider:              llmcostops.ProviderOpenAI,
    Model:                 "gpt-4",
    InputPricePerMillion:  10.0,
    OutputPricePerMillion: 30.0,
    CachedInputDiscount:   floatPtr(0.5),
})

// Get pricing by ID
pricing, err := client.Pricing.Get(ctx, "price-123")

// List pricing
pricings, err := client.Pricing.List(ctx, &llmcostops.PricingListParams{
    Provider: llmcostops.ProviderOpenAI,
    Active:   boolPtr(true),
})

// Get active pricing
pricing, err := client.Pricing.GetActive(ctx, llmcostops.ProviderOpenAI, "gpt-4")

// Delete pricing
err := client.Pricing.Delete(ctx, "price-123")
```

### Usage Service

```go
// Ingest usage records
err := client.Usage.Ingest(ctx, &llmcostops.UsageIngestParams{
    Records: []llmcostops.UsageRecord{...},
})

// Get usage record
usage, err := client.Usage.Get(ctx, "usage-123")

// List usage records
usages, err := client.Usage.List(ctx, &llmcostops.UsageListParams{
    Range:          llmcostops.RangeLast24Hours,
    OrganizationID: "org-123",
    Provider:       llmcostops.ProviderOpenAI,
})

// Get usage statistics
stats, err := client.Usage.Stats(ctx, &llmcostops.UsageStatsParams{
    Range:   llmcostops.RangeLast7Days,
    GroupBy: []string{"provider", "model"},
})

// Delete usage record
err := client.Usage.Delete(ctx, "usage-123")
```

### Cost Service

```go
// Get cost record
cost, err := client.Costs.Get(ctx, "cost-123")

// List cost records
costs, err := client.Costs.List(ctx, &llmcostops.CostListParams{
    Range:          llmcostops.RangeLast30Days,
    OrganizationID: "org-123",
    MinCost:        floatPtr(1.0),
})

// Get cost summary
summary, err := client.Costs.Summary(ctx, &llmcostops.CostSummaryParams{
    Range:   llmcostops.RangeLast30Days,
    GroupBy: []string{"provider", "model", "project"},
})

// Get cost analytics
analytics, err := client.Costs.Analytics(ctx, &llmcostops.CostAnalyticsParams{
    Range:       llmcostops.RangeLast90Days,
    Granularity: "day",
})

// Get costs by provider
byProvider, err := client.Costs.ByProvider(ctx, params)

// Get costs by model
byModel, err := client.Costs.ByModel(ctx, params)
```

### Export Service

```go
// Export data
data, err := client.Export.Export(ctx, &llmcostops.ExportParams{
    Format:         llmcostops.FormatCSV,
    Range:          llmcostops.RangeLast7Days,
    IncludeHeaders: true,
})

// Export to writer
file, _ := os.Create("export.csv")
defer file.Close()
err := client.Export.ExportToWriter(ctx, params, file)

// Schedule report
report, err := client.Export.ScheduleReport(ctx, &llmcostops.ReportScheduleParams{
    Name:           "Daily Cost Report",
    Schedule:       "0 9 * * *",
    Format:         llmcostops.FormatExcel,
    ReportType:     "cost",
    DeliveryMethod: "email",
    Enabled:        true,
})

// List scheduled reports
reports, err := client.Export.ListScheduledReports(ctx)

// Trigger scheduled report manually
err := client.Export.TriggerScheduledReport(ctx, "report-123")
```

### Health Service

```go
// Check overall health
health, err := client.Health.Check(ctx)

// Liveness probe
err := client.Health.Live(ctx)

// Readiness probe
err := client.Health.Ready(ctx)
```

## Error Handling

The SDK provides sentinel errors for common error cases:

```go
import "errors"

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

## Context Support

All API methods accept `context.Context` for cancellation and timeouts:

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

## Testing

Run tests:

```bash
go test -v ./...
```

Run tests with coverage:

```bash
go test -v -race -coverprofile=coverage.txt -covermode=atomic ./...
go tool cover -html=coverage.txt
```

Run benchmarks:

```bash
go test -bench=. -benchmem ./...
```

## Examples

See the [examples](./examples) directory for complete examples:

- [basic](./examples/basic/main.go) - Basic usage with all main features
- [advanced](./examples/advanced/main.go) - Advanced usage with custom configuration

## Performance

- **Goroutine-Safe**: All operations are safe for concurrent use
- **Connection Pooling**: HTTP client uses connection pooling by default
- **Rate Limiting**: Built-in token bucket rate limiter
- **Retry Logic**: Automatic retry with exponential backoff
- **Zero Allocations**: Optimized for minimal allocations in hot paths

## Best Practices

1. **Reuse Client**: Create one client and reuse it across your application
2. **Use Context**: Always pass context for cancellation and timeouts
3. **Close Client**: Call `client.Close()` when done to release resources
4. **Error Handling**: Use sentinel errors for common cases
5. **Structured Logging**: Configure a structured logger for production
6. **Metrics**: Implement MetricsCollector for observability

## Requirements

- Go 1.21 or higher

## Dependencies

- [go.uber.org/zap](https://github.com/uber-go/zap) - Structured logging
- [golang.org/x/time](https://golang.org/x/time) - Rate limiting

## License

Apache 2.0 / MIT dual-licensed. See LICENSE-APACHE and LICENSE-MIT for details.

## Support

- Documentation: https://docs.example.com/llm-cost-ops/sdk/go
- Issues: https://github.com/llm-devops/llm-cost-ops/issues
- API Reference: https://pkg.go.dev/github.com/llm-devops/llm-cost-ops/sdk/go
