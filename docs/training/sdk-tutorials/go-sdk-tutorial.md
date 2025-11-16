# Go SDK Tutorial

## Table of Contents
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Module Setup](#module-setup)
- [Basic Usage](#basic-usage)
- [Context Usage Patterns](#context-usage-patterns)
- [Service Implementations](#service-implementations)
- [Error Handling](#error-handling)
- [Functional Options Pattern](#functional-options-pattern)
- [Retry and Circuit Breaker](#retry-and-circuit-breaker)
- [Concurrent Requests](#concurrent-requests)
- [Testing](#testing)
- [Graceful Shutdown](#graceful-shutdown)
- [Performance Optimization](#performance-optimization)
- [Advanced Patterns](#advanced-patterns)

## Prerequisites

Before getting started with the Go SDK, ensure you have:

- Go 1.19 or higher
- Basic understanding of Go modules
- API key from LLM Cost Ops platform
- Familiarity with Go's context package
- Understanding of goroutines and channels

## Installation

### Using go get

```bash
go get github.com/llmcostops/go-sdk
```

### Verify Installation

```bash
go list -m github.com/llmcostops/go-sdk
```

## Module Setup

### Initialize Go Module

```bash
mkdir llm-cost-tracker
cd llm-cost-tracker
go mod init github.com/yourusername/llm-cost-tracker
```

### Basic go.mod

```go
module github.com/yourusername/llm-cost-tracker

go 1.21

require (
    github.com/llmcostops/go-sdk v1.0.0
)
```

### Install Dependencies

```bash
go mod tidy
go mod download
```

### Environment Configuration

Create a `.env` file:

```bash
LLM_COST_OPS_API_KEY=your_api_key_here
LLM_COST_OPS_BASE_URL=https://api.llmcostops.com
LLM_COST_OPS_TIMEOUT=30s
```

### Load Environment Variables

```go
package main

import (
    "os"
    "github.com/joho/godotenv"
)

func init() {
    // Load .env file
    if err := godotenv.Load(); err != nil {
        log.Fatal("Error loading .env file")
    }
}

func main() {
    apiKey := os.Getenv("LLM_COST_OPS_API_KEY")
    baseURL := os.Getenv("LLM_COST_OPS_BASE_URL")
    // Use configuration
}
```

## Basic Usage

### Client Initialization

```go
package main

import (
    "context"
    "log"
    "os"
    "time"

    costops "github.com/llmcostops/go-sdk"
)

func main() {
    // Create configuration
    config := &costops.Config{
        APIKey:     os.Getenv("LLM_COST_OPS_API_KEY"),
        BaseURL:    os.Getenv("LLM_COST_OPS_BASE_URL"),
        Timeout:    30 * time.Second,
        MaxRetries: 3,
    }

    // Create client
    client, err := costops.NewClient(config)
    if err != nil {
        log.Fatalf("Failed to create client: %v", err)
    }
    defer client.Close()

    // Use client
    ctx := context.Background()
    costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    })
    if err != nil {
        log.Fatalf("Failed to get costs: %v", err)
    }

    log.Printf("Total cost: $%.2f", costs.TotalCost)
}
```

### Simple Cost Query

```go
package main

import (
    "context"
    "fmt"
    "log"
    "time"

    costops "github.com/llmcostops/go-sdk"
)

func getCosts(client *costops.Client) error {
    ctx := context.Background()

    // Calculate date range
    endDate := time.Now()
    startDate := endDate.AddDate(0, 0, -7)

    // Query costs
    costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: startDate.Format("2006-01-02"),
        EndDate:   endDate.Format("2006-01-02"),
    })
    if err != nil {
        return fmt.Errorf("failed to get costs: %w", err)
    }

    fmt.Printf("Total cost: $%.2f\n", costs.TotalCost)

    for _, item := range costs.Items {
        fmt.Printf("%s: $%.2f\n", item.Date, item.Amount)
    }

    return nil
}
```

### Track Usage

```go
func trackUsage(client *costops.Client) error {
    ctx := context.Background()

    usage, err := client.Usage.CreateUsage(ctx, &costops.UsageCreate{
        Model:            "gpt-4",
        TokensPrompt:     1000,
        TokensCompletion: 500,
        RequestCount:     1,
        Timestamp:        time.Now(),
        Metadata: map[string]interface{}{
            "user_id":    "user_123",
            "session_id": "session_456",
        },
    })
    if err != nil {
        return fmt.Errorf("failed to create usage: %w", err)
    }

    fmt.Printf("Usage ID: %s\n", usage.ID)
    fmt.Printf("Cost: $%.4f\n", usage.Cost)

    return nil
}
```

### Get Pricing

```go
func getPricing(client *costops.Client) error {
    ctx := context.Background()

    pricing, err := client.Pricing.GetModelPricing(ctx, &costops.PricingQuery{
        Model:    "gpt-4",
        Provider: "openai",
    })
    if err != nil {
        return fmt.Errorf("failed to get pricing: %w", err)
    }

    fmt.Printf("Model: %s\n", pricing.Model)
    fmt.Printf("Prompt: $%.4f per 1K tokens\n", pricing.PromptPricePer1K)
    fmt.Printf("Completion: $%.4f per 1K tokens\n", pricing.CompletionPricePer1K)

    return nil
}
```

## Context Usage Patterns

### Basic Context Usage

```go
import "context"

func fetchCostsWithContext(client *costops.Client) error {
    // Create base context
    ctx := context.Background()

    // Add timeout
    ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
    defer cancel()

    costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    })
    if err != nil {
        if ctx.Err() == context.DeadlineExceeded {
            return fmt.Errorf("request timed out: %w", err)
        }
        return err
    }

    fmt.Printf("Total cost: $%.2f\n", costs.TotalCost)
    return nil
}
```

### Context with Cancellation

```go
func fetchWithCancellation(client *costops.Client) error {
    ctx, cancel := context.WithCancel(context.Background())

    // Cancel after 5 seconds
    go func() {
        time.Sleep(5 * time.Second)
        cancel()
    }()

    costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    })

    if err != nil {
        if ctx.Err() == context.Canceled {
            return fmt.Errorf("request canceled: %w", err)
        }
        return err
    }

    fmt.Printf("Total cost: $%.2f\n", costs.TotalCost)
    return nil
}
```

### Context with Values

```go
type contextKey string

const (
    requestIDKey contextKey = "request_id"
    userIDKey    contextKey = "user_id"
)

func fetchWithContextValues(client *costops.Client) error {
    ctx := context.Background()

    // Add values to context
    ctx = context.WithValue(ctx, requestIDKey, "req_123")
    ctx = context.WithValue(ctx, userIDKey, "user_456")

    // Extract values in handler
    requestID := ctx.Value(requestIDKey).(string)
    userID := ctx.Value(userIDKey).(string)

    fmt.Printf("Request ID: %s, User ID: %s\n", requestID, userID)

    costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    })
    if err != nil {
        return err
    }

    fmt.Printf("Total cost: $%.2f\n", costs.TotalCost)
    return nil
}
```

### Deadline Context

```go
func fetchWithDeadline(client *costops.Client) error {
    // Set deadline to 1 minute from now
    deadline := time.Now().Add(1 * time.Minute)
    ctx, cancel := context.WithDeadline(context.Background(), deadline)
    defer cancel()

    costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    })

    if err != nil {
        if ctx.Err() == context.DeadlineExceeded {
            return fmt.Errorf("deadline exceeded: %w", err)
        }
        return err
    }

    fmt.Printf("Total cost: $%.2f\n", costs.TotalCost)
    return nil
}
```

## Service Implementations

### Usage Service

```go
package services

import (
    "context"
    "fmt"
    "time"

    costops "github.com/llmcostops/go-sdk"
)

type UsageService struct {
    client *costops.Client
}

func NewUsageService(client *costops.Client) *UsageService {
    return &UsageService{client: client}
}

func (s *UsageService) CreateUsage(ctx context.Context, req *costops.UsageCreate) (*costops.Usage, error) {
    return s.client.Usage.CreateUsage(ctx, req)
}

func (s *UsageService) GetUsageByID(ctx context.Context, id string) (*costops.Usage, error) {
    return s.client.Usage.GetUsageByID(ctx, id)
}

func (s *UsageService) ListUsage(ctx context.Context, query *costops.UsageQuery) (*costops.UsageList, error) {
    return s.client.Usage.ListUsage(ctx, query)
}

func (s *UsageService) UpdateUsage(ctx context.Context, id string, update *costops.UsageUpdate) (*costops.Usage, error) {
    return s.client.Usage.UpdateUsage(ctx, id, update)
}

func (s *UsageService) DeleteUsage(ctx context.Context, id string) error {
    return s.client.Usage.DeleteUsage(ctx, id)
}

func (s *UsageService) BatchCreateUsage(ctx context.Context, usages []*costops.UsageCreate) (*costops.BatchResult, error) {
    return s.client.Usage.BatchCreateUsage(ctx, usages)
}
```

### Costs Service

```go
type CostsService struct {
    client *costops.Client
}

func NewCostsService(client *costops.Client) *CostsService {
    return &CostsService{client: client}
}

func (s *CostsService) GetCosts(ctx context.Context, query *costops.CostsQuery) (*costops.Costs, error) {
    return s.client.Costs.GetCosts(ctx, query)
}

func (s *CostsService) GetCostBreakdown(ctx context.Context, query *costops.CostsQuery) (*costops.CostBreakdown, error) {
    return s.client.Costs.GetCostBreakdown(ctx, query)
}

func (s *CostsService) GetCostTrends(ctx context.Context, params *costops.TrendParams) (*costops.CostTrends, error) {
    return s.client.Costs.GetCostTrends(ctx, params)
}

func (s *CostsService) GetMonthlySummary(ctx context.Context, year int, month time.Month) (*costops.Costs, error) {
    startDate := time.Date(year, month, 1, 0, 0, 0, 0, time.UTC)
    endDate := startDate.AddDate(0, 1, -1)

    return s.client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: startDate.Format("2006-01-02"),
        EndDate:   endDate.Format("2006-01-02"),
    })
}
```

### Analytics Service

```go
type AnalyticsService struct {
    client *costops.Client
}

func NewAnalyticsService(client *costops.Client) *AnalyticsService {
    return &AnalyticsService{client: client}
}

func (s *AnalyticsService) GetUsageAnalytics(ctx context.Context, query *costops.AnalyticsQuery) (*costops.Analytics, error) {
    return s.client.Analytics.GetUsageAnalytics(ctx, query)
}

func (s *AnalyticsService) GetCostAnalytics(ctx context.Context, query *costops.AnalyticsQuery) (*costops.Analytics, error) {
    return s.client.Analytics.GetCostAnalytics(ctx, query)
}

func (s *AnalyticsService) GetPerformanceMetrics(ctx context.Context, query *costops.AnalyticsQuery) (*costops.PerformanceMetrics, error) {
    return s.client.Analytics.GetPerformanceMetrics(ctx, query)
}

func (s *AnalyticsService) GetTopConsumers(ctx context.Context, params *costops.TopConsumersParams) (*costops.TopConsumers, error) {
    return s.client.Analytics.GetTopConsumers(ctx, params)
}
```

### Budget Service

```go
type BudgetService struct {
    client *costops.Client
}

func NewBudgetService(client *costops.Client) *BudgetService {
    return &BudgetService{client: client}
}

func (s *BudgetService) CreateBudget(ctx context.Context, req *costops.BudgetCreate) (*costops.Budget, error) {
    return s.client.Budgets.CreateBudget(ctx, req)
}

func (s *BudgetService) ListBudgets(ctx context.Context, params *costops.BudgetListParams) (*costops.BudgetList, error) {
    return s.client.Budgets.ListBudgets(ctx, params)
}

func (s *BudgetService) GetBudget(ctx context.Context, id string) (*costops.Budget, error) {
    return s.client.Budgets.GetBudget(ctx, id)
}

func (s *BudgetService) UpdateBudget(ctx context.Context, id string, update *costops.BudgetUpdate) (*costops.Budget, error) {
    return s.client.Budgets.UpdateBudget(ctx, id, update)
}

func (s *BudgetService) DeleteBudget(ctx context.Context, id string) error {
    return s.client.Budgets.DeleteBudget(ctx, id)
}

func (s *BudgetService) GetBudgetAlerts(ctx context.Context, params *costops.BudgetAlertsParams) (*costops.BudgetAlerts, error) {
    return s.client.Budgets.GetBudgetAlerts(ctx, params)
}
```

## Error Handling

### Error Types

```go
package main

import (
    "errors"
    "fmt"
    "log"

    costops "github.com/llmcostops/go-sdk"
)

func handleErrors(client *costops.Client) error {
    ctx := context.Background()

    costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    })

    if err != nil {
        var authErr *costops.AuthenticationError
        var authzErr *costops.AuthorizationError
        var notFoundErr *costops.ResourceNotFoundError
        var validationErr *costops.ValidationError
        var rateLimitErr *costops.RateLimitError
        var serverErr *costops.ServerError
        var networkErr *costops.NetworkError
        var timeoutErr *costops.TimeoutError

        switch {
        case errors.As(err, &authErr):
            log.Printf("Authentication failed: %v", authErr)
            return fmt.Errorf("authentication error: %w", err)

        case errors.As(err, &authzErr):
            log.Printf("Authorization failed: %v", authzErr)
            return fmt.Errorf("authorization error: %w", err)

        case errors.As(err, &notFoundErr):
            log.Printf("Resource not found: %v", notFoundErr)
            return fmt.Errorf("not found: %w", err)

        case errors.As(err, &validationErr):
            log.Printf("Validation error: %v", validationErr)
            for field, msg := range validationErr.Errors {
                log.Printf("  %s: %s", field, msg)
            }
            return fmt.Errorf("validation error: %w", err)

        case errors.As(err, &rateLimitErr):
            log.Printf("Rate limit exceeded, retry after %d seconds", rateLimitErr.RetryAfter)
            time.Sleep(time.Duration(rateLimitErr.RetryAfter) * time.Second)
            // Retry logic here
            return fmt.Errorf("rate limit error: %w", err)

        case errors.As(err, &serverErr):
            log.Printf("Server error (status %d): %v", serverErr.StatusCode, serverErr)
            return fmt.Errorf("server error: %w", err)

        case errors.As(err, &networkErr):
            log.Printf("Network error: %v", networkErr)
            return fmt.Errorf("network error: %w", err)

        case errors.As(err, &timeoutErr):
            log.Printf("Request timeout: %v", timeoutErr)
            return fmt.Errorf("timeout error: %w", err)

        default:
            log.Printf("Unexpected error: %v", err)
            return fmt.Errorf("unexpected error: %w", err)
        }
    }

    log.Printf("Total cost: $%.2f", costs.TotalCost)
    return nil
}
```

### Custom Error Handler

```go
type ErrorHandler struct {
    logger *log.Logger
}

func NewErrorHandler(logger *log.Logger) *ErrorHandler {
    return &ErrorHandler{logger: logger}
}

func (h *ErrorHandler) Handle(err error) error {
    if err == nil {
        return nil
    }

    var rateLimitErr *costops.RateLimitError
    if errors.As(err, &rateLimitErr) {
        h.logger.Printf("Rate limit hit, waiting %d seconds", rateLimitErr.RetryAfter)
        time.Sleep(time.Duration(rateLimitErr.RetryAfter) * time.Second)
        return nil // Caller should retry
    }

    var networkErr *costops.NetworkError
    if errors.As(err, &networkErr) {
        h.logger.Printf("Network error, retrying in 1 second")
        time.Sleep(1 * time.Second)
        return nil // Caller should retry
    }

    // Log and return other errors
    h.logger.Printf("Error: %v", err)
    return err
}

// Usage
handler := NewErrorHandler(log.Default())

err := client.Costs.GetCosts(ctx, query)
if err != nil {
    if handlerErr := handler.Handle(err); handlerErr != nil {
        return handlerErr
    }
    // Retry the operation
    err = client.Costs.GetCosts(ctx, query)
}
```

### Error Wrapping

```go
func fetchCostsWithWrapping(client *costops.Client) error {
    ctx := context.Background()

    costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    })
    if err != nil {
        return fmt.Errorf("failed to fetch costs for January 2025: %w", err)
    }

    if costs.TotalCost > 10000 {
        return fmt.Errorf("costs exceeded budget: $%.2f > $10000", costs.TotalCost)
    }

    return nil
}

// Unwrapping errors
err := fetchCostsWithWrapping(client)
if err != nil {
    var validationErr *costops.ValidationError
    if errors.As(err, &validationErr) {
        log.Printf("Validation failed: %v", validationErr)
    }

    log.Printf("Error: %v", err)
}
```

## Functional Options Pattern

### Client Options

```go
package costops

import "time"

type Config struct {
    APIKey     string
    BaseURL    string
    Timeout    time.Duration
    MaxRetries int
    UserAgent  string
}

type ClientOption func(*Config)

func WithBaseURL(url string) ClientOption {
    return func(c *Config) {
        c.BaseURL = url
    }
}

func WithTimeout(timeout time.Duration) ClientOption {
    return func(c *Config) {
        c.Timeout = timeout
    }
}

func WithMaxRetries(retries int) ClientOption {
    return func(c *Config) {
        c.MaxRetries = retries
    }
}

func WithUserAgent(userAgent string) ClientOption {
    return func(c *Config) {
        c.UserAgent = userAgent
    }
}

func NewClient(apiKey string, opts ...ClientOption) (*Client, error) {
    config := &Config{
        APIKey:     apiKey,
        BaseURL:    "https://api.llmcostops.com",
        Timeout:    30 * time.Second,
        MaxRetries: 3,
        UserAgent:  "llm-cost-ops-go-sdk/1.0.0",
    }

    for _, opt := range opts {
        opt(config)
    }

    return &Client{
        config: config,
        // ... initialize fields
    }, nil
}

// Usage
client, err := costops.NewClient(
    apiKey,
    costops.WithTimeout(60*time.Second),
    costops.WithMaxRetries(5),
    costops.WithBaseURL("https://staging-api.llmcostops.com"),
)
```

### Request Options

```go
type RequestOptions struct {
    Timeout    time.Duration
    MaxRetries int
    Headers    map[string]string
}

type RequestOption func(*RequestOptions)

func WithRequestTimeout(timeout time.Duration) RequestOption {
    return func(o *RequestOptions) {
        o.Timeout = timeout
    }
}

func WithRequestRetries(retries int) RequestOption {
    return func(o *RequestOptions) {
        o.MaxRetries = retries
    }
}

func WithHeaders(headers map[string]string) RequestOption {
    return func(o *RequestOptions) {
        o.Headers = headers
    }
}

func (c *Client) GetCostsWithOptions(ctx context.Context, query *CostsQuery, opts ...RequestOption) (*Costs, error) {
    options := &RequestOptions{
        Timeout:    c.config.Timeout,
        MaxRetries: c.config.MaxRetries,
        Headers:    make(map[string]string),
    }

    for _, opt := range opts {
        opt(options)
    }

    // Use options in request
    // ...
}

// Usage
costs, err := client.GetCostsWithOptions(
    ctx,
    query,
    WithRequestTimeout(60*time.Second),
    WithRequestRetries(5),
    WithHeaders(map[string]string{
        "X-Request-ID": "req_123",
    }),
)
```

## Retry and Circuit Breaker

### Retry Logic

```go
package retry

import (
    "context"
    "errors"
    "math"
    "time"
)

type Config struct {
    MaxRetries     int
    InitialBackoff time.Duration
    MaxBackoff     time.Duration
    Multiplier     float64
    Jitter         bool
}

type Retryer struct {
    config Config
}

func NewRetryer(config Config) *Retryer {
    return &Retryer{config: config}
}

func (r *Retryer) Do(ctx context.Context, fn func() error) error {
    var lastErr error

    for attempt := 0; attempt <= r.config.MaxRetries; attempt++ {
        if attempt > 0 {
            backoff := r.calculateBackoff(attempt)
            select {
            case <-time.After(backoff):
            case <-ctx.Done():
                return ctx.Err()
            }
        }

        if err := fn(); err != nil {
            lastErr = err

            // Don't retry certain errors
            if !r.shouldRetry(err) {
                return err
            }

            continue
        }

        return nil
    }

    return fmt.Errorf("max retries exceeded: %w", lastErr)
}

func (r *Retryer) calculateBackoff(attempt int) time.Duration {
    backoff := float64(r.config.InitialBackoff) * math.Pow(r.config.Multiplier, float64(attempt-1))

    if r.config.Jitter {
        backoff = backoff * (0.5 + rand.Float64()*0.5)
    }

    if time.Duration(backoff) > r.config.MaxBackoff {
        return r.config.MaxBackoff
    }

    return time.Duration(backoff)
}

func (r *Retryer) shouldRetry(err error) bool {
    var networkErr *costops.NetworkError
    var timeoutErr *costops.TimeoutError
    var rateLimitErr *costops.RateLimitError
    var serverErr *costops.ServerError

    switch {
    case errors.As(err, &networkErr):
        return true
    case errors.As(err, &timeoutErr):
        return true
    case errors.As(err, &rateLimitErr):
        return true
    case errors.As(err, &serverErr):
        return serverErr.StatusCode >= 500
    default:
        return false
    }
}

// Usage
retryer := retry.NewRetryer(retry.Config{
    MaxRetries:     5,
    InitialBackoff: 1 * time.Second,
    MaxBackoff:     60 * time.Second,
    Multiplier:     2.0,
    Jitter:         true,
})

var costs *costops.Costs
err := retryer.Do(ctx, func() error {
    var err error
    costs, err = client.Costs.GetCosts(ctx, query)
    return err
})
```

### Circuit Breaker

```go
package circuitbreaker

import (
    "errors"
    "sync"
    "time"
)

type State int

const (
    StateClosed State = iota
    StateOpen
    StateHalfOpen
)

type CircuitBreaker struct {
    maxFailures  int
    timeout      time.Duration
    state        State
    failures     int
    lastFailTime time.Time
    mu           sync.RWMutex
}

func NewCircuitBreaker(maxFailures int, timeout time.Duration) *CircuitBreaker {
    return &CircuitBreaker{
        maxFailures: maxFailures,
        timeout:     timeout,
        state:       StateClosed,
    }
}

func (cb *CircuitBreaker) Call(fn func() error) error {
    cb.mu.RLock()
    state := cb.state
    cb.mu.RUnlock()

    if state == StateOpen {
        cb.mu.RLock()
        timeSinceLastFail := time.Since(cb.lastFailTime)
        cb.mu.RUnlock()

        if timeSinceLastFail < cb.timeout {
            return errors.New("circuit breaker is open")
        }

        cb.mu.Lock()
        cb.state = StateHalfOpen
        cb.mu.Unlock()
    }

    err := fn()

    cb.mu.Lock()
    defer cb.mu.Unlock()

    if err != nil {
        cb.failures++
        cb.lastFailTime = time.Now()

        if cb.failures >= cb.maxFailures {
            cb.state = StateOpen
        }

        return err
    }

    if cb.state == StateHalfOpen {
        cb.state = StateClosed
        cb.failures = 0
    }

    return nil
}

// Usage
cb := circuitbreaker.NewCircuitBreaker(5, 30*time.Second)

err := cb.Call(func() error {
    _, err := client.Costs.GetCosts(ctx, query)
    return err
})

if err != nil {
    log.Printf("Circuit breaker error: %v", err)
}
```

## Concurrent Requests

### Using Goroutines

```go
func fetchConcurrently(client *costops.Client) error {
    ctx := context.Background()

    type result struct {
        costs *costops.Costs
        err   error
    }

    results := make(chan result, 3)

    // Launch goroutines
    go func() {
        costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
            StartDate: "2025-01-01",
            EndDate:   "2025-01-31",
        })
        results <- result{costs, err}
    }()

    go func() {
        usage, err := client.Usage.ListUsage(ctx, &costops.UsageQuery{
            StartDate: "2025-01-01",
            EndDate:   "2025-01-31",
        })
        // Convert to costs for simplification
        results <- result{nil, err}
    }()

    go func() {
        analytics, err := client.Analytics.GetUsageAnalytics(ctx, &costops.AnalyticsQuery{
            StartDate: "2025-01-01",
            EndDate:   "2025-01-31",
        })
        results <- result{nil, err}
    }()

    // Collect results
    for i := 0; i < 3; i++ {
        res := <-results
        if res.err != nil {
            log.Printf("Error: %v", res.err)
            continue
        }

        if res.costs != nil {
            log.Printf("Total cost: $%.2f", res.costs.TotalCost)
        }
    }

    return nil
}
```

### Using WaitGroup

```go
func fetchWithWaitGroup(client *costops.Client) error {
    ctx := context.Background()
    var wg sync.WaitGroup

    // Costs
    wg.Add(1)
    go func() {
        defer wg.Done()

        costs, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
            StartDate: "2025-01-01",
            EndDate:   "2025-01-31",
        })
        if err != nil {
            log.Printf("Error fetching costs: %v", err)
            return
        }

        log.Printf("Total cost: $%.2f", costs.TotalCost)
    }()

    // Usage
    wg.Add(1)
    go func() {
        defer wg.Done()

        usage, err := client.Usage.ListUsage(ctx, &costops.UsageQuery{
            StartDate: "2025-01-01",
            EndDate:   "2025-01-31",
        })
        if err != nil {
            log.Printf("Error fetching usage: %v", err)
            return
        }

        log.Printf("Total requests: %d", usage.TotalCount)
    }()

    // Analytics
    wg.Add(1)
    go func() {
        defer wg.Done()

        analytics, err := client.Analytics.GetUsageAnalytics(ctx, &costops.AnalyticsQuery{
            StartDate: "2025-01-01",
            EndDate:   "2025-01-31",
        })
        if err != nil {
            log.Printf("Error fetching analytics: %v", err)
            return
        }

        log.Printf("Analytics groups: %d", len(analytics.Groups))
    }()

    wg.Wait()
    return nil
}
```

### Worker Pool Pattern

```go
type Job struct {
    ID    int
    Query *costops.CostsQuery
}

type Result struct {
    JobID int
    Costs *costops.Costs
    Error error
}

func workerPool(client *costops.Client, jobs []Job, numWorkers int) []Result {
    ctx := context.Background()
    jobsChan := make(chan Job, len(jobs))
    resultsChan := make(chan Result, len(jobs))

    // Start workers
    var wg sync.WaitGroup
    for i := 0; i < numWorkers; i++ {
        wg.Add(1)
        go func() {
            defer wg.Done()

            for job := range jobsChan {
                costs, err := client.Costs.GetCosts(ctx, job.Query)
                resultsChan <- Result{
                    JobID: job.ID,
                    Costs: costs,
                    Error: err,
                }
            }
        }()
    }

    // Send jobs
    for _, job := range jobs {
        jobsChan <- job
    }
    close(jobsChan)

    // Wait for completion
    go func() {
        wg.Wait()
        close(resultsChan)
    }()

    // Collect results
    results := make([]Result, 0, len(jobs))
    for result := range resultsChan {
        results = append(results, result)
    }

    return results
}

// Usage
jobs := []Job{
    {ID: 1, Query: &costops.CostsQuery{StartDate: "2025-01-01", EndDate: "2025-01-31"}},
    {ID: 2, Query: &costops.CostsQuery{StartDate: "2025-02-01", EndDate: "2025-02-28"}},
    {ID: 3, Query: &costops.CostsQuery{StartDate: "2025-03-01", EndDate: "2025-03-31"}},
}

results := workerPool(client, jobs, 3)

for _, result := range results {
    if result.Error != nil {
        log.Printf("Job %d failed: %v", result.JobID, result.Error)
        continue
    }

    log.Printf("Job %d: $%.2f", result.JobID, result.Costs.TotalCost)
}
```

### Semaphore Pattern

```go
type Semaphore struct {
    semaCh chan struct{}
}

func NewSemaphore(maxConcurrency int) *Semaphore {
    return &Semaphore{
        semaCh: make(chan struct{}, maxConcurrency),
    }
}

func (s *Semaphore) Acquire() {
    s.semaCh <- struct{}{}
}

func (s *Semaphore) Release() {
    <-s.semaCh
}

func fetchWithSemaphore(client *costops.Client, queries []*costops.CostsQuery) []*costops.Costs {
    ctx := context.Background()
    sem := NewSemaphore(5) // Max 5 concurrent requests

    results := make([]*costops.Costs, len(queries))
    var wg sync.WaitGroup

    for i, query := range queries {
        wg.Add(1)

        go func(index int, q *costops.CostsQuery) {
            defer wg.Done()

            sem.Acquire()
            defer sem.Release()

            costs, err := client.Costs.GetCosts(ctx, q)
            if err != nil {
                log.Printf("Error: %v", err)
                return
            }

            results[index] = costs
        }(i, query)
    }

    wg.Wait()
    return results
}
```

## Testing

### Unit Testing

```go
package main

import (
    "context"
    "testing"

    "github.com/stretchr/testify/assert"
    "github.com/stretchr/testify/mock"

    costops "github.com/llmcostops/go-sdk"
)

// Mock client
type MockCostsService struct {
    mock.Mock
}

func (m *MockCostsService) GetCosts(ctx context.Context, query *costops.CostsQuery) (*costops.Costs, error) {
    args := m.Called(ctx, query)
    if args.Get(0) == nil {
        return nil, args.Error(1)
    }
    return args.Get(0).(*costops.Costs), args.Error(1)
}

func TestGetCosts(t *testing.T) {
    mockService := new(MockCostsService)

    expectedCosts := &costops.Costs{
        TotalCost: 100.50,
        Items: []costops.CostItem{
            {Date: "2025-01-01", Amount: 50.25},
            {Date: "2025-01-02", Amount: 50.25},
        },
    }

    mockService.On("GetCosts", mock.Anything, mock.Anything).Return(expectedCosts, nil)

    ctx := context.Background()
    costs, err := mockService.GetCosts(ctx, &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    })

    assert.NoError(t, err)
    assert.NotNil(t, costs)
    assert.Equal(t, 100.50, costs.TotalCost)
    assert.Len(t, costs.Items, 2)

    mockService.AssertExpectations(t)
}
```

### Integration Testing

```go
func TestIntegrationCreateAndGetUsage(t *testing.T) {
    if testing.Short() {
        t.Skip("Skipping integration test")
    }

    apiKey := os.Getenv("LLM_COST_OPS_TEST_API_KEY")
    if apiKey == "" {
        t.Skip("No test API key provided")
    }

    client, err := costops.NewClient(&costops.Config{
        APIKey: apiKey,
    })
    assert.NoError(t, err)
    defer client.Close()

    ctx := context.Background()

    // Create usage
    created, err := client.Usage.CreateUsage(ctx, &costops.UsageCreate{
        Model:            "gpt-4",
        TokensPrompt:     1000,
        TokensCompletion: 500,
        RequestCount:     1,
    })
    assert.NoError(t, err)
    assert.NotEmpty(t, created.ID)

    // Get usage
    fetched, err := client.Usage.GetUsageByID(ctx, created.ID)
    assert.NoError(t, err)
    assert.Equal(t, created.ID, fetched.ID)
    assert.Equal(t, "gpt-4", fetched.Model)

    // Cleanup
    err = client.Usage.DeleteUsage(ctx, created.ID)
    assert.NoError(t, err)
}
```

### Table-Driven Tests

```go
func TestCostCalculation(t *testing.T) {
    tests := []struct {
        name              string
        tokensPrompt      int
        tokensCompletion  int
        expectedCost      float64
    }{
        {
            name:             "Small request",
            tokensPrompt:     100,
            tokensCompletion: 50,
            expectedCost:     0.0045,
        },
        {
            name:             "Medium request",
            tokensPrompt:     1000,
            tokensCompletion: 500,
            expectedCost:     0.045,
        },
        {
            name:             "Large request",
            tokensPrompt:     10000,
            tokensCompletion: 5000,
            expectedCost:     0.45,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            cost := calculateCost(tt.tokensPrompt, tt.tokensCompletion)
            assert.InDelta(t, tt.expectedCost, cost, 0.0001)
        })
    }
}
```

### Benchmark Tests

```go
func BenchmarkGetCosts(b *testing.B) {
    client, _ := costops.NewClient(&costops.Config{
        APIKey: "test_key",
    })

    ctx := context.Background()
    query := &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    }

    b.ResetTimer()

    for i := 0; i < b.N; i++ {
        _, _ = client.Costs.GetCosts(ctx, query)
    }
}

func BenchmarkConcurrentRequests(b *testing.B) {
    client, _ := costops.NewClient(&costops.Config{
        APIKey: "test_key",
    })

    ctx := context.Background()
    query := &costops.CostsQuery{
        StartDate: "2025-01-01",
        EndDate:   "2025-01-31",
    }

    b.ResetTimer()

    b.RunParallel(func(pb *testing.PB) {
        for pb.Next() {
            _, _ = client.Costs.GetCosts(ctx, query)
        }
    })
}
```

## Graceful Shutdown

### Basic Shutdown

```go
package main

import (
    "context"
    "log"
    "os"
    "os/signal"
    "syscall"
    "time"

    costops "github.com/llmcostops/go-sdk"
)

func main() {
    client, err := costops.NewClient(&costops.Config{
        APIKey: os.Getenv("LLM_COST_OPS_API_KEY"),
    })
    if err != nil {
        log.Fatalf("Failed to create client: %v", err)
    }

    // Create shutdown channel
    shutdown := make(chan os.Signal, 1)
    signal.Notify(shutdown, os.Interrupt, syscall.SIGTERM)

    // Create done channel
    done := make(chan bool)

    // Start background work
    go func() {
        ticker := time.NewTicker(5 * time.Second)
        defer ticker.Stop()

        for {
            select {
            case <-ticker.C:
                // Do periodic work
                ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
                _, err := client.Costs.GetCosts(ctx, &costops.CostsQuery{
                    StartDate: time.Now().AddDate(0, 0, -7).Format("2006-01-02"),
                    EndDate:   time.Now().Format("2006-01-02"),
                })
                cancel()

                if err != nil {
                    log.Printf("Error: %v", err)
                }

            case <-shutdown:
                log.Println("Shutting down...")
                done <- true
                return
            }
        }
    }()

    // Wait for shutdown
    <-done

    // Cleanup
    client.Close()
    log.Println("Shutdown complete")
}
```

### Graceful Shutdown with Timeout

```go
func runWithGracefulShutdown(client *costops.Client) {
    ctx, cancel := context.WithCancel(context.Background())
    defer cancel()

    // Shutdown channel
    shutdown := make(chan os.Signal, 1)
    signal.Notify(shutdown, os.Interrupt, syscall.SIGTERM)

    // Background worker
    go func() {
        for {
            select {
            case <-ctx.Done():
                return
            default:
                // Do work
                time.Sleep(1 * time.Second)
            }
        }
    }()

    // Wait for shutdown signal
    <-shutdown
    log.Println("Shutdown signal received")

    // Create shutdown context with timeout
    shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
    defer shutdownCancel()

    // Cancel main context
    cancel()

    // Wait for cleanup with timeout
    done := make(chan bool)
    go func() {
        client.Close()
        done <- true
    }()

    select {
    case <-done:
        log.Println("Graceful shutdown complete")
    case <-shutdownCtx.Done():
        log.Println("Shutdown timeout exceeded")
    }
}
```

## Performance Optimization

### Connection Pooling

```go
type ConnectionPool struct {
    pool chan *costops.Client
    mu   sync.Mutex
}

func NewConnectionPool(apiKey string, size int) (*ConnectionPool, error) {
    pool := &ConnectionPool{
        pool: make(chan *costops.Client, size),
    }

    for i := 0; i < size; i++ {
        client, err := costops.NewClient(&costops.Config{
            APIKey: apiKey,
        })
        if err != nil {
            return nil, err
        }

        pool.pool <- client
    }

    return pool, nil
}

func (p *ConnectionPool) Get() *costops.Client {
    return <-p.pool
}

func (p *ConnectionPool) Put(client *costops.Client) {
    p.pool <- client
}

func (p *ConnectionPool) Close() {
    close(p.pool)

    for client := range p.pool {
        client.Close()
    }
}

// Usage
pool, err := NewConnectionPool(apiKey, 10)
if err != nil {
    log.Fatal(err)
}
defer pool.Close()

client := pool.Get()
defer pool.Put(client)

costs, err := client.Costs.GetCosts(ctx, query)
```

### Caching

```go
type Cache struct {
    data sync.Map
    ttl  time.Duration
}

type cacheEntry struct {
    value      interface{}
    expiration time.Time
}

func NewCache(ttl time.Duration) *Cache {
    return &Cache{ttl: ttl}
}

func (c *Cache) Get(key string) (interface{}, bool) {
    val, ok := c.data.Load(key)
    if !ok {
        return nil, false
    }

    entry := val.(cacheEntry)
    if time.Now().After(entry.expiration) {
        c.data.Delete(key)
        return nil, false
    }

    return entry.value, true
}

func (c *Cache) Set(key string, value interface{}) {
    c.data.Store(key, cacheEntry{
        value:      value,
        expiration: time.Now().Add(c.ttl),
    })
}

// Cached client
type CachedClient struct {
    client *costops.Client
    cache  *Cache
}

func NewCachedClient(client *costops.Client, cacheTTL time.Duration) *CachedClient {
    return &CachedClient{
        client: client,
        cache:  NewCache(cacheTTL),
    }
}

func (c *CachedClient) GetCosts(ctx context.Context, query *costops.CostsQuery) (*costops.Costs, error) {
    cacheKey := fmt.Sprintf("costs:%s:%s", query.StartDate, query.EndDate)

    if cached, ok := c.cache.Get(cacheKey); ok {
        return cached.(*costops.Costs), nil
    }

    costs, err := c.client.Costs.GetCosts(ctx, query)
    if err != nil {
        return nil, err
    }

    c.cache.Set(cacheKey, costs)
    return costs, nil
}
```

### Batch Processing

```go
func batchProcess(client *costops.Client, records []*costops.UsageCreate, batchSize int) error {
    ctx := context.Background()

    for i := 0; i < len(records); i += batchSize {
        end := i + batchSize
        if end > len(records) {
            end = len(records)
        }

        batch := records[i:end]

        result, err := client.Usage.BatchCreateUsage(ctx, batch)
        if err != nil {
            return fmt.Errorf("batch %d failed: %w", i/batchSize, err)
        }

        log.Printf("Batch %d: created %d, failed %d", i/batchSize, result.CreatedCount, result.FailedCount)
    }

    return nil
}
```

## Advanced Patterns

### Repository Pattern

```go
type UsageRepository interface {
    Create(ctx context.Context, usage *costops.UsageCreate) (*costops.Usage, error)
    GetByID(ctx context.Context, id string) (*costops.Usage, error)
    List(ctx context.Context, query *costops.UsageQuery) (*costops.UsageList, error)
    Update(ctx context.Context, id string, update *costops.UsageUpdate) (*costops.Usage, error)
    Delete(ctx context.Context, id string) error
}

type usageRepository struct {
    client *costops.Client
}

func NewUsageRepository(client *costops.Client) UsageRepository {
    return &usageRepository{client: client}
}

func (r *usageRepository) Create(ctx context.Context, usage *costops.UsageCreate) (*costops.Usage, error) {
    return r.client.Usage.CreateUsage(ctx, usage)
}

func (r *usageRepository) GetByID(ctx context.Context, id string) (*costops.Usage, error) {
    return r.client.Usage.GetUsageByID(ctx, id)
}

func (r *usageRepository) List(ctx context.Context, query *costops.UsageQuery) (*costops.UsageList, error) {
    return r.client.Usage.ListUsage(ctx, query)
}

func (r *usageRepository) Update(ctx context.Context, id string, update *costops.UsageUpdate) (*costops.Usage, error) {
    return r.client.Usage.UpdateUsage(ctx, id, update)
}

func (r *usageRepository) Delete(ctx context.Context, id string) error {
    return r.client.Usage.DeleteUsage(ctx, id)
}
```

### Service Layer

```go
type CostAnalyticsService struct {
    client *costops.Client
    cache  *Cache
}

func NewCostAnalyticsService(client *costops.Client) *CostAnalyticsService {
    return &CostAnalyticsService{
        client: client,
        cache:  NewCache(5 * time.Minute),
    }
}

func (s *CostAnalyticsService) GetDailyCostTrend(ctx context.Context, days int) ([]costops.CostItem, error) {
    endDate := time.Now()
    startDate := endDate.AddDate(0, 0, -days)

    breakdown, err := s.client.Costs.GetCostBreakdown(ctx, &costops.CostsQuery{
        StartDate:   startDate.Format("2006-01-02"),
        EndDate:     endDate.Format("2006-01-02"),
        Granularity: "daily",
    })
    if err != nil {
        return nil, err
    }

    return breakdown.Items, nil
}

func (s *CostAnalyticsService) GetCostByModel(ctx context.Context, startDate, endDate string) (map[string]float64, error) {
    costs, err := s.client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: startDate,
        EndDate:   endDate,
        GroupBy:   []string{"model"},
    })
    if err != nil {
        return nil, err
    }

    costMap := make(map[string]float64)
    for _, item := range costs.Items {
        costMap[item.Model] = item.Amount
    }

    return costMap, nil
}

func (s *CostAnalyticsService) PredictNextMonthCost(ctx context.Context) (float64, error) {
    trends, err := s.client.Costs.GetCostTrends(ctx, &costops.TrendParams{
        Days:            30,
        IncludeForecast: true,
    })
    if err != nil {
        return 0, err
    }

    return trends.Forecast, nil
}
```

---

## Complete Example Application

```go
package main

import (
    "context"
    "fmt"
    "log"
    "os"
    "time"

    costops "github.com/llmcostops/go-sdk"
    "github.com/joho/godotenv"
)

type CostTracker struct {
    client *costops.Client
}

func NewCostTracker(apiKey string) (*CostTracker, error) {
    client, err := costops.NewClient(&costops.Config{
        APIKey:     apiKey,
        Timeout:    30 * time.Second,
        MaxRetries: 3,
    })
    if err != nil {
        return nil, err
    }

    return &CostTracker{client: client}, nil
}

func (t *CostTracker) TrackUsage(ctx context.Context, model string, tokensPrompt, tokensCompletion int) error {
    usage, err := t.client.Usage.CreateUsage(ctx, &costops.UsageCreate{
        Model:            model,
        TokensPrompt:     tokensPrompt,
        TokensCompletion: tokensCompletion,
        RequestCount:     1,
        Timestamp:        time.Now(),
    })
    if err != nil {
        return fmt.Errorf("failed to track usage: %w", err)
    }

    log.Printf("✓ Tracked usage: %s - $%.4f", usage.ID, usage.Cost)
    return nil
}

func (t *CostTracker) GetCostSummary(ctx context.Context, days int) error {
    endDate := time.Now()
    startDate := endDate.AddDate(0, 0, -days)

    costs, err := t.client.Costs.GetCosts(ctx, &costops.CostsQuery{
        StartDate: startDate.Format("2006-01-02"),
        EndDate:   endDate.Format("2006-01-02"),
    })
    if err != nil {
        return fmt.Errorf("failed to get costs: %w", err)
    }

    fmt.Printf("\n📊 Cost Summary (Last %d days)\n", days)
    fmt.Printf("Total Cost: $%.2f\n", costs.TotalCost)

    for i, item := range costs.Items {
        if i >= 10 {
            break
        }
        fmt.Printf("  %s: $%.2f\n", item.Date, item.Amount)
    }

    return nil
}

func (t *CostTracker) CheckBudgets(ctx context.Context) error {
    budgets, err := t.client.Budgets.ListBudgets(ctx, &costops.BudgetListParams{
        ActiveOnly: true,
    })
    if err != nil {
        return fmt.Errorf("failed to get budgets: %w", err)
    }

    fmt.Printf("\n💰 Active Budgets\n")

    for _, budget := range budgets.Items {
        percentage := budget.PercentageUsed
        var status string
        if percentage < 50 {
            status = "🟢"
        } else if percentage < 80 {
            status = "🟡"
        } else {
            status = "🔴"
        }

        fmt.Printf("%s %s: $%.2f / $%.2f (%.1f%%)\n",
            status, budget.Name, budget.Spent, budget.Amount, percentage)
    }

    return nil
}

func (t *CostTracker) RunAnalytics(ctx context.Context, days int) error {
    endDate := time.Now()
    startDate := endDate.AddDate(0, 0, -days)

    analytics, err := t.client.Analytics.GetUsageAnalytics(ctx, &costops.AnalyticsQuery{
        StartDate: startDate.Format("2006-01-02"),
        EndDate:   endDate.Format("2006-01-02"),
        GroupBy:   []string{"model"},
        Metrics:   []string{"total_tokens", "total_cost", "request_count"},
    })
    if err != nil {
        return fmt.Errorf("failed to get analytics: %w", err)
    }

    fmt.Printf("\n📈 Analytics (Last %d days)\n", days)

    for _, group := range analytics.Groups {
        fmt.Printf("\n%s:\n", group.Key)
        fmt.Printf("  Tokens: %d\n", group.TotalTokens)
        fmt.Printf("  Cost: $%.2f\n", group.TotalCost)
        fmt.Printf("  Requests: %d\n", group.RequestCount)
        fmt.Printf("  Avg Cost/Request: $%.4f\n", group.TotalCost/float64(group.RequestCount))
    }

    return nil
}

func (t *CostTracker) Close() error {
    return t.client.Close()
}

func main() {
    // Load environment
    if err := godotenv.Load(); err != nil {
        log.Println("No .env file found")
    }

    apiKey := os.Getenv("LLM_COST_OPS_API_KEY")
    if apiKey == "" {
        log.Fatal("LLM_COST_OPS_API_KEY not set")
    }

    // Create tracker
    tracker, err := NewCostTracker(apiKey)
    if err != nil {
        log.Fatalf("Failed to create tracker: %v", err)
    }
    defer tracker.Close()

    ctx := context.Background()

    // Track usage
    if err := tracker.TrackUsage(ctx, "gpt-4", 1000, 500); err != nil {
        log.Printf("Error tracking usage: %v", err)
    }

    // Get cost summary
    if err := tracker.GetCostSummary(ctx, 7); err != nil {
        log.Printf("Error getting costs: %v", err)
    }

    // Check budgets
    if err := tracker.CheckBudgets(ctx); err != nil {
        log.Printf("Error checking budgets: %v", err)
    }

    // Run analytics
    if err := tracker.RunAnalytics(ctx, 30); err != nil {
        log.Printf("Error running analytics: %v", err)
    }
}
```

---

## Additional Resources

- **API Reference**: https://pkg.go.dev/github.com/llmcostops/go-sdk
- **GitHub Repository**: https://github.com/llmcostops/go-sdk
- **Examples**: https://github.com/llmcostops/go-sdk/tree/main/examples
- **Support**: support@llmcostops.com

## Next Steps

1. Install the SDK and set up your Go module
2. Initialize the client with your API key
3. Try basic usage examples with context
4. Implement error handling
5. Add tests using testify
6. Optimize for production with goroutines and caching
7. Implement graceful shutdown for production services

For more advanced use cases, check out the [Advanced Integration Guide](./advanced-integration.md).
