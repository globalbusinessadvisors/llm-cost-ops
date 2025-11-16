/*
Package llmcostops provides an enterprise-grade Go SDK for the LLM Cost Operations platform.

The SDK offers idiomatic Go interfaces for tracking, analyzing, and optimizing costs
across multiple LLM providers with production-ready accuracy.

# Installation

Install the SDK using go get:

	go get github.com/llm-devops/llm-cost-ops/sdk/go

# Quick Start

Create a client and start tracking costs:

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

		// Get cost summary
		summary, err := client.Costs.Summary(ctx, &llmcostops.CostSummaryParams{
			Range: llmcostops.RangeLast30Days,
		})
	}

# Enterprise Features

The SDK includes production-ready features:

  - Automatic retry with exponential backoff
  - Rate limiting with token bucket algorithm
  - Structured logging with go.uber.org/zap
  - Connection pooling for HTTP requests
  - Metrics hooks for instrumentation
  - Full context.Context support
  - Goroutine-safe operations
  - Comprehensive error handling with sentinel errors

# Configuration

Configure the client with functional options:

	client, err := llmcostops.NewClient(
		llmcostops.WithAPIKey("your-api-key"),
		llmcostops.WithBaseURL("https://api.costops.example.com"),
		llmcostops.WithTimeout(30 * time.Second),
		llmcostops.WithMaxRetries(3),
		llmcostops.WithRetryDelay(time.Second),
		llmcostops.WithRateLimit(100), // requests per second
		llmcostops.WithLogger(logger),
		llmcostops.WithHTTPClient(httpClient),
		llmcostops.WithMetrics(metricsCollector),
	)

# Error Handling

The SDK provides sentinel errors for common error cases:

	import "errors"

	_, err := client.Usage.Get(ctx, "invalid-id")
	if errors.Is(err, llmcostops.ErrNotFound) {
		// Handle not found
	}

Available sentinel errors:

  - ErrInvalidConfig: Invalid client configuration
  - ErrUnauthorized: Authentication failure
  - ErrNotFound: Resource not found
  - ErrRateLimited: Rate limit exceeded
  - ErrServerError: Server-side error
  - ErrBadRequest: Invalid request parameters
  - ErrContextCanceled: Context was canceled

# Context Support

All API methods accept context.Context for cancellation and timeouts:

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

# Services

The client provides access to multiple services:

Pricing Service - Manage pricing information:

	client.Pricing.Add(ctx, params)
	client.Pricing.Get(ctx, id)
	client.Pricing.List(ctx, params)
	client.Pricing.GetActive(ctx, provider, model)
	client.Pricing.Delete(ctx, id)

Usage Service - Track usage records:

	client.Usage.Ingest(ctx, params)
	client.Usage.Get(ctx, id)
	client.Usage.List(ctx, params)
	client.Usage.Stats(ctx, params)
	client.Usage.Delete(ctx, id)

Cost Service - Analyze costs:

	client.Costs.Get(ctx, id)
	client.Costs.List(ctx, params)
	client.Costs.Summary(ctx, params)
	client.Costs.Analytics(ctx, params)
	client.Costs.ByProvider(ctx, params)
	client.Costs.ByModel(ctx, params)

Export Service - Export and schedule reports:

	client.Export.Export(ctx, params)
	client.Export.ExportToWriter(ctx, params, writer)
	client.Export.ScheduleReport(ctx, params)
	client.Export.ListScheduledReports(ctx)
	client.Export.TriggerScheduledReport(ctx, id)

Health Service - Monitor service health:

	client.Health.Check(ctx)
	client.Health.Live(ctx)
	client.Health.Ready(ctx)

# Testing

The SDK is thoroughly tested with table-driven tests and provides
high test coverage. Run tests with:

	go test -v ./...

Run tests with race detector:

	go test -race ./...

Generate coverage report:

	go test -coverprofile=coverage.txt ./...
	go tool cover -html=coverage.txt

# Performance

The SDK is optimized for production use:

  - Goroutine-safe: All operations are safe for concurrent use
  - Connection pooling: HTTP client uses connection pooling by default
  - Rate limiting: Built-in token bucket rate limiter
  - Retry logic: Automatic retry with exponential backoff
  - Zero allocations: Optimized for minimal allocations in hot paths

# Best Practices

 1. Reuse Client: Create one client and reuse it across your application
 2. Use Context: Always pass context for cancellation and timeouts
 3. Close Client: Call client.Close() when done to release resources
 4. Error Handling: Use sentinel errors for common cases
 5. Structured Logging: Configure a structured logger for production
 6. Metrics: Implement MetricsCollector for observability

# Requirements

# Go 1.21 or higher

# License

Apache 2.0 / MIT dual-licensed.
*/
package llmcostops
