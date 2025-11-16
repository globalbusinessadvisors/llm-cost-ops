// Package llmcostops provides an enterprise-grade Go SDK for the LLM Cost Operations platform.
//
// The SDK offers idiomatic Go interfaces for tracking, analyzing, and optimizing costs
// across multiple LLM providers with production-ready accuracy.
//
// Basic usage:
//
//	client, err := llmcostops.NewClient(
//		llmcostops.WithAPIKey("your-api-key"),
//		llmcostops.WithBaseURL("https://api.costops.example.com"),
//	)
//	if err != nil {
//		log.Fatal(err)
//	}
//	defer client.Close()
//
//	ctx := context.Background()
//	usage, err := client.Usage.List(ctx, &llmcostops.UsageListParams{
//		Range: llmcostops.RangeLast24Hours,
//	})
package llmcostops

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"sync"
	"time"

	"go.uber.org/zap"
	"golang.org/x/time/rate"
)

const (
	// DefaultBaseURL is the default base URL for the LLM Cost Ops API
	DefaultBaseURL = "https://api.costops.example.com"

	// DefaultTimeout is the default request timeout
	DefaultTimeout = 30 * time.Second

	// DefaultMaxRetries is the default number of retry attempts
	DefaultMaxRetries = 3

	// DefaultRateLimit is the default rate limit (requests per second)
	DefaultRateLimit = 100

	// Version is the SDK version
	Version = "1.0.0"

	// UserAgent is the default user agent string
	UserAgent = "llm-cost-ops-go-sdk/" + Version
)

// Sentinel errors for better error handling
var (
	// ErrInvalidConfig indicates invalid client configuration
	ErrInvalidConfig = errors.New("invalid client configuration")

	// ErrUnauthorized indicates authentication failure
	ErrUnauthorized = errors.New("unauthorized: invalid API key")

	// ErrNotFound indicates the requested resource was not found
	ErrNotFound = errors.New("resource not found")

	// ErrRateLimited indicates rate limit exceeded
	ErrRateLimited = errors.New("rate limit exceeded")

	// ErrServerError indicates a server-side error
	ErrServerError = errors.New("internal server error")

	// ErrBadRequest indicates invalid request parameters
	ErrBadRequest = errors.New("bad request")

	// ErrContextCanceled indicates the context was canceled
	ErrContextCanceled = errors.New("context canceled")
)

// Client is the main SDK client for interacting with the LLM Cost Ops API.
// It is safe for concurrent use by multiple goroutines.
type Client struct {
	// Configuration
	baseURL    *url.URL
	apiKey     string
	httpClient *http.Client
	logger     *zap.Logger

	// Rate limiting
	rateLimiter *rate.Limiter

	// Retry configuration
	maxRetries int
	retryDelay time.Duration

	// Service clients
	Pricing *PricingService
	Usage   *UsageService
	Costs   *CostService
	Export  *ExportService
	Health  *HealthService

	// Metrics hooks (optional)
	metrics MetricsCollector

	// Connection pooling
	mu     sync.RWMutex
	closed bool
}

// Config holds client configuration options
type Config struct {
	BaseURL    string
	APIKey     string
	HTTPClient *http.Client
	Logger     *zap.Logger
	MaxRetries int
	RetryDelay time.Duration
	RateLimit  rate.Limit
	Timeout    time.Duration
	Metrics    MetricsCollector
}

// Option is a functional option for configuring the Client
type Option func(*Config) error

// WithAPIKey sets the API key for authentication
func WithAPIKey(apiKey string) Option {
	return func(c *Config) error {
		if apiKey == "" {
			return fmt.Errorf("%w: API key cannot be empty", ErrInvalidConfig)
		}
		c.APIKey = apiKey
		return nil
	}
}

// WithBaseURL sets the base URL for the API
func WithBaseURL(baseURL string) Option {
	return func(c *Config) error {
		if baseURL == "" {
			return fmt.Errorf("%w: base URL cannot be empty", ErrInvalidConfig)
		}
		c.BaseURL = baseURL
		return nil
	}
}

// WithHTTPClient sets a custom HTTP client
func WithHTTPClient(client *http.Client) Option {
	return func(c *Config) error {
		if client == nil {
			return fmt.Errorf("%w: HTTP client cannot be nil", ErrInvalidConfig)
		}
		c.HTTPClient = client
		return nil
	}
}

// WithLogger sets a custom logger
func WithLogger(logger *zap.Logger) Option {
	return func(c *Config) error {
		if logger == nil {
			return fmt.Errorf("%w: logger cannot be nil", ErrInvalidConfig)
		}
		c.Logger = logger
		return nil
	}
}

// WithMaxRetries sets the maximum number of retry attempts
func WithMaxRetries(maxRetries int) Option {
	return func(c *Config) error {
		if maxRetries < 0 {
			return fmt.Errorf("%w: max retries cannot be negative", ErrInvalidConfig)
		}
		c.MaxRetries = maxRetries
		return nil
	}
}

// WithRetryDelay sets the delay between retry attempts
func WithRetryDelay(delay time.Duration) Option {
	return func(c *Config) error {
		if delay < 0 {
			return fmt.Errorf("%w: retry delay cannot be negative", ErrInvalidConfig)
		}
		c.RetryDelay = delay
		return nil
	}
}

// WithRateLimit sets the rate limit (requests per second)
func WithRateLimit(rps float64) Option {
	return func(c *Config) error {
		if rps <= 0 {
			return fmt.Errorf("%w: rate limit must be positive", ErrInvalidConfig)
		}
		c.RateLimit = rate.Limit(rps)
		return nil
	}
}

// WithTimeout sets the request timeout
func WithTimeout(timeout time.Duration) Option {
	return func(c *Config) error {
		if timeout <= 0 {
			return fmt.Errorf("%w: timeout must be positive", ErrInvalidConfig)
		}
		c.Timeout = timeout
		return nil
	}
}

// WithMetrics sets a custom metrics collector
func WithMetrics(metrics MetricsCollector) Option {
	return func(c *Config) error {
		c.Metrics = metrics
		return nil
	}
}

// NewClient creates a new LLM Cost Ops API client with the provided options.
// It returns an error if the configuration is invalid.
func NewClient(opts ...Option) (*Client, error) {
	// Default configuration
	config := &Config{
		BaseURL:    DefaultBaseURL,
		MaxRetries: DefaultMaxRetries,
		RetryDelay: time.Second,
		RateLimit:  DefaultRateLimit,
		Timeout:    DefaultTimeout,
	}

	// Apply options
	for _, opt := range opts {
		if err := opt(config); err != nil {
			return nil, err
		}
	}

	// Validate required fields
	if config.APIKey == "" {
		return nil, fmt.Errorf("%w: API key is required", ErrInvalidConfig)
	}

	// Parse base URL
	baseURL, err := url.Parse(config.BaseURL)
	if err != nil {
		return nil, fmt.Errorf("%w: invalid base URL: %v", ErrInvalidConfig, err)
	}

	// Create HTTP client if not provided
	if config.HTTPClient == nil {
		config.HTTPClient = &http.Client{
			Timeout: config.Timeout,
			Transport: &http.Transport{
				MaxIdleConns:        100,
				MaxIdleConnsPerHost: 10,
				IdleConnTimeout:     90 * time.Second,
			},
		}
	}

	// Create logger if not provided
	if config.Logger == nil {
		config.Logger, _ = zap.NewProduction()
	}

	// Create client
	client := &Client{
		baseURL:     baseURL,
		apiKey:      config.APIKey,
		httpClient:  config.HTTPClient,
		logger:      config.Logger,
		rateLimiter: rate.NewLimiter(config.RateLimit, int(config.RateLimit)),
		maxRetries:  config.MaxRetries,
		retryDelay:  config.RetryDelay,
		metrics:     config.Metrics,
	}

	// Initialize service clients
	client.Pricing = &PricingService{client: client}
	client.Usage = &UsageService{client: client}
	client.Costs = &CostService{client: client}
	client.Export = &ExportService{client: client}
	client.Health = &HealthService{client: client}

	return client, nil
}

// Close releases any resources held by the client.
// It should be called when the client is no longer needed.
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.closed {
		return nil
	}

	c.closed = true

	// Sync logger
	if c.logger != nil {
		_ = c.logger.Sync()
	}

	return nil
}

// do executes an HTTP request with retry logic and rate limiting.
// It is goroutine-safe and respects context cancellation.
func (c *Client) do(ctx context.Context, req *http.Request, v interface{}) error {
	c.mu.RLock()
	if c.closed {
		c.mu.RUnlock()
		return errors.New("client is closed")
	}
	c.mu.RUnlock()

	// Wait for rate limiter
	if err := c.rateLimiter.Wait(ctx); err != nil {
		return fmt.Errorf("%w: %v", ErrContextCanceled, err)
	}

	var lastErr error
	startTime := time.Now()

	for attempt := 0; attempt <= c.maxRetries; attempt++ {
		// Check context before retry
		select {
		case <-ctx.Done():
			return fmt.Errorf("%w: %v", ErrContextCanceled, ctx.Err())
		default:
		}

		// Clone request for retry
		reqClone := req.Clone(ctx)

		// Execute request
		resp, err := c.executeRequest(ctx, reqClone)
		if err != nil {
			lastErr = err
			c.logRetry(attempt, err)

			// Don't retry on context cancellation
			if errors.Is(err, context.Canceled) || errors.Is(err, context.DeadlineExceeded) {
				return fmt.Errorf("%w: %v", ErrContextCanceled, err)
			}

			// Exponential backoff
			if attempt < c.maxRetries {
				backoff := c.retryDelay * time.Duration(1<<uint(attempt))
				select {
				case <-time.After(backoff):
					// Continue to retry
				case <-ctx.Done():
					return fmt.Errorf("%w: %v", ErrContextCanceled, ctx.Err())
				}
			}
			continue
		}

		// Record metrics
		if c.metrics != nil {
			c.metrics.RecordRequest(req.Method, resp.StatusCode, time.Since(startTime))
		}

		// Handle response
		err = c.handleResponse(resp, v)
		if err != nil {
			// Check if this is a server error that should be retried
			if isRetryableError(err) && attempt < c.maxRetries {
				lastErr = err
				c.logRetry(attempt, err)
				backoff := c.retryDelay * time.Duration(1<<uint(attempt))
				select {
				case <-time.After(backoff):
					continue
				case <-ctx.Done():
					return fmt.Errorf("%w: %v", ErrContextCanceled, ctx.Err())
				}
			}
			return err
		}

		return nil
	}

	return fmt.Errorf("request failed after %d attempts: %w", c.maxRetries+1, lastErr)
}

// isRetryableError checks if an error should trigger a retry
func isRetryableError(err error) bool {
	if err == nil {
		return false
	}
	// Retry on server errors and rate limit errors
	return errors.Is(err, ErrServerError) || errors.Is(err, ErrRateLimited)
}

// executeRequest executes a single HTTP request
func (c *Client) executeRequest(ctx context.Context, req *http.Request) (*http.Response, error) {
	// Set headers
	req.Header.Set("Authorization", "Bearer "+c.apiKey)
	req.Header.Set("User-Agent", UserAgent)
	req.Header.Set("Accept", "application/json")
	if req.Method == http.MethodPost || req.Method == http.MethodPut || req.Method == http.MethodPatch {
		req.Header.Set("Content-Type", "application/json")
	}

	// Log request
	c.logger.Debug("executing request",
		zap.String("method", req.Method),
		zap.String("url", req.URL.String()),
	)

	// Execute request
	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}

	return resp, nil
}

// handleResponse processes the HTTP response and decodes the body
func (c *Client) handleResponse(resp *http.Response, v interface{}) error {
	defer resp.Body.Close()

	// Read body
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read response body: %w", err)
	}

	// Check status code
	if err := c.checkResponseStatus(resp.StatusCode, body); err != nil {
		return err
	}

	// Decode response if needed
	if v != nil && len(body) > 0 {
		if err := json.Unmarshal(body, v); err != nil {
			return fmt.Errorf("failed to decode response: %w", err)
		}
	}

	return nil
}

// checkResponseStatus checks the HTTP status code and returns appropriate errors
func (c *Client) checkResponseStatus(statusCode int, body []byte) error {
	if statusCode >= 200 && statusCode < 300 {
		return nil
	}

	// Try to parse error response
	var apiErr APIError
	if err := json.Unmarshal(body, &apiErr); err == nil && apiErr.Message != "" {
		apiErr.StatusCode = statusCode
		return &apiErr
	}

	// Return sentinel error based on status code
	switch statusCode {
	case http.StatusBadRequest:
		return fmt.Errorf("%w: %s", ErrBadRequest, string(body))
	case http.StatusUnauthorized:
		return ErrUnauthorized
	case http.StatusNotFound:
		return ErrNotFound
	case http.StatusTooManyRequests:
		return ErrRateLimited
	case http.StatusInternalServerError, http.StatusBadGateway, http.StatusServiceUnavailable:
		return fmt.Errorf("%w: %s", ErrServerError, string(body))
	default:
		return fmt.Errorf("unexpected status code %d: %s", statusCode, string(body))
	}
}

// logRetry logs retry attempts
func (c *Client) logRetry(attempt int, err error) {
	if attempt > 0 {
		c.logger.Warn("retrying request",
			zap.Int("attempt", attempt),
			zap.Error(err),
		)
	}
}

// newRequest creates a new HTTP request
func (c *Client) newRequest(method, path string, body interface{}) (*http.Request, error) {
	// Build URL
	u := *c.baseURL
	u.Path = path

	// Encode body
	var bodyReader io.Reader
	if body != nil {
		bodyBytes, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("failed to encode request body: %w", err)
		}
		bodyReader = bytes.NewReader(bodyBytes)
	}

	// Create request
	req, err := http.NewRequest(method, u.String(), bodyReader)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	return req, nil
}

// APIError represents an error response from the API
type APIError struct {
	StatusCode int    `json:"-"`
	Code       string `json:"code"`
	Message    string `json:"message"`
	Details    string `json:"details,omitempty"`
}

// Error implements the error interface
func (e *APIError) Error() string {
	if e.Details != "" {
		return fmt.Sprintf("API error [%d]: %s - %s (code: %s)", e.StatusCode, e.Message, e.Details, e.Code)
	}
	return fmt.Sprintf("API error [%d]: %s (code: %s)", e.StatusCode, e.Message, e.Code)
}

// MetricsCollector is an interface for collecting SDK metrics.
// Implement this interface to integrate with your monitoring system.
type MetricsCollector interface {
	// RecordRequest records an API request
	RecordRequest(method string, statusCode int, duration time.Duration)

	// RecordError records an error
	RecordError(operation string, err error)
}
