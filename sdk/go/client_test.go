package llmcostops

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"go.uber.org/zap"
)

func TestNewClient(t *testing.T) {
	tests := []struct {
		name    string
		opts    []Option
		wantErr bool
		errType error
	}{
		{
			name: "valid configuration",
			opts: []Option{
				WithAPIKey("test-key"),
				WithBaseURL("https://api.example.com"),
			},
			wantErr: false,
		},
		{
			name:    "missing API key",
			opts:    []Option{},
			wantErr: true,
			errType: ErrInvalidConfig,
		},
		{
			name: "empty API key",
			opts: []Option{
				WithAPIKey(""),
			},
			wantErr: true,
			errType: ErrInvalidConfig,
		},
		{
			name: "invalid base URL",
			opts: []Option{
				WithAPIKey("test-key"),
				WithBaseURL("ht!tp://invalid"),
			},
			wantErr: true,
			errType: ErrInvalidConfig,
		},
		{
			name: "custom timeout",
			opts: []Option{
				WithAPIKey("test-key"),
				WithTimeout(10 * time.Second),
			},
			wantErr: false,
		},
		{
			name: "negative timeout",
			opts: []Option{
				WithAPIKey("test-key"),
				WithTimeout(-1 * time.Second),
			},
			wantErr: true,
			errType: ErrInvalidConfig,
		},
		{
			name: "custom rate limit",
			opts: []Option{
				WithAPIKey("test-key"),
				WithRateLimit(50),
			},
			wantErr: false,
		},
		{
			name: "invalid rate limit",
			opts: []Option{
				WithAPIKey("test-key"),
				WithRateLimit(-10),
			},
			wantErr: true,
			errType: ErrInvalidConfig,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			client, err := NewClient(tt.opts...)
			if tt.wantErr {
				if err == nil {
					t.Errorf("NewClient() expected error, got nil")
				}
				if tt.errType != nil && !isError(err, tt.errType) {
					t.Errorf("NewClient() error = %v, want error type %v", err, tt.errType)
				}
				return
			}
			if err != nil {
				t.Errorf("NewClient() unexpected error = %v", err)
				return
			}
			if client == nil {
				t.Error("NewClient() returned nil client")
			}
			defer client.Close()
		})
	}
}

func TestClient_Close(t *testing.T) {
	client, err := NewClient(WithAPIKey("test-key"))
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}

	if err := client.Close(); err != nil {
		t.Errorf("Close() error = %v", err)
	}

	// Closing again should not error
	if err := client.Close(); err != nil {
		t.Errorf("Close() second call error = %v", err)
	}

	// Operations on closed client should error
	ctx := context.Background()
	_, err = client.Health.Check(ctx)
	if err == nil {
		t.Error("Expected error when using closed client")
	}
}

func TestClient_ContextCancellation(t *testing.T) {
	// Create test server with delay
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(100 * time.Millisecond)
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
	}))
	defer server.Close()

	client, err := NewClient(
		WithAPIKey("test-key"),
		WithBaseURL(server.URL),
	)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	defer client.Close()

	// Create context with short timeout
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Millisecond)
	defer cancel()

	_, err = client.Health.Check(ctx)
	if err == nil {
		t.Error("Expected error due to context cancellation")
	}
	if !isError(err, ErrContextCanceled) {
		t.Errorf("Expected ErrContextCanceled, got %v", err)
	}
}

func TestClient_RetryLogic(t *testing.T) {
	attempts := 0
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		attempts++
		if attempts < 3 {
			w.WriteHeader(http.StatusInternalServerError)
			return
		}
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(HealthStatus{Status: "ok"})
	}))
	defer server.Close()

	logger, _ := zap.NewDevelopment()
	client, err := NewClient(
		WithAPIKey("test-key"),
		WithBaseURL(server.URL),
		WithMaxRetries(3),
		WithRetryDelay(10*time.Millisecond),
		WithLogger(logger),
	)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	defer client.Close()

	ctx := context.Background()
	_, err = client.Health.Check(ctx)
	if err != nil {
		t.Errorf("Expected success after retries, got error: %v", err)
	}

	if attempts != 3 {
		t.Errorf("Expected 3 attempts, got %d", attempts)
	}
}

func TestClient_ErrorHandling(t *testing.T) {
	tests := []struct {
		name           string
		statusCode     int
		responseBody   interface{}
		expectedErr    error
		checkErrString bool
	}{
		{
			name:        "bad request",
			statusCode:  http.StatusBadRequest,
			expectedErr: ErrBadRequest,
		},
		{
			name:        "unauthorized",
			statusCode:  http.StatusUnauthorized,
			expectedErr: ErrUnauthorized,
		},
		{
			name:        "not found",
			statusCode:  http.StatusNotFound,
			expectedErr: ErrNotFound,
		},
		{
			name:        "rate limited",
			statusCode:  http.StatusTooManyRequests,
			expectedErr: ErrRateLimited,
		},
		{
			name:        "server error",
			statusCode:  http.StatusInternalServerError,
			expectedErr: ErrServerError,
		},
		{
			name:       "API error with details",
			statusCode: http.StatusBadRequest,
			responseBody: APIError{
				Code:    "INVALID_PARAM",
				Message: "Invalid parameter",
				Details: "Field 'model' is required",
			},
			checkErrString: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(tt.statusCode)
				if tt.responseBody != nil {
					json.NewEncoder(w).Encode(tt.responseBody)
				}
			}))
			defer server.Close()

			client, err := NewClient(
				WithAPIKey("test-key"),
				WithBaseURL(server.URL),
				WithMaxRetries(0), // Disable retries for error tests
			)
			if err != nil {
				t.Fatalf("Failed to create client: %v", err)
			}
			defer client.Close()

			ctx := context.Background()
			_, err = client.Health.Check(ctx)
			if err == nil {
				t.Error("Expected error, got nil")
				return
			}

			if tt.checkErrString {
				// For API errors, just check that we got an error
				if err == nil {
					t.Error("Expected API error")
				}
			} else if !isError(err, tt.expectedErr) {
				t.Errorf("Expected error type %v, got %v", tt.expectedErr, err)
			}
		})
	}
}

func TestClient_ConcurrentRequests(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(HealthStatus{Status: "ok"})
	}))
	defer server.Close()

	client, err := NewClient(
		WithAPIKey("test-key"),
		WithBaseURL(server.URL),
		WithRateLimit(1000), // High rate limit for concurrency test
	)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	defer client.Close()

	// Run multiple concurrent requests
	const numRequests = 10
	errChan := make(chan error, numRequests)

	for i := 0; i < numRequests; i++ {
		go func() {
			ctx := context.Background()
			_, err := client.Health.Check(ctx)
			errChan <- err
		}()
	}

	// Check results
	for i := 0; i < numRequests; i++ {
		if err := <-errChan; err != nil {
			t.Errorf("Concurrent request %d failed: %v", i, err)
		}
	}
}

// Helper function to check if error is of specific type
func isError(err, target error) bool {
	if err == nil || target == nil {
		return err == target
	}
	// Check if err wraps target
	for {
		if err == target {
			return true
		}
		wrapper, ok := err.(interface{ Unwrap() error })
		if !ok {
			return false
		}
		err = wrapper.Unwrap()
		if err == nil {
			return false
		}
	}
}
