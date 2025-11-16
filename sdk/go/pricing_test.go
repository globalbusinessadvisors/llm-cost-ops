package llmcostops

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestPricingService_Add(t *testing.T) {
	tests := []struct {
		name    string
		params  *PricingAddParams
		wantErr bool
		errType error
	}{
		{
			name: "valid pricing",
			params: &PricingAddParams{
				Provider:              ProviderOpenAI,
				Model:                 "gpt-4",
				InputPricePerMillion:  10.0,
				OutputPricePerMillion: 30.0,
				Currency:              CurrencyUSD,
			},
			wantErr: false,
		},
		{
			name:    "nil params",
			params:  nil,
			wantErr: true,
			errType: ErrBadRequest,
		},
		{
			name: "missing provider",
			params: &PricingAddParams{
				Model:                 "gpt-4",
				InputPricePerMillion:  10.0,
				OutputPricePerMillion: 30.0,
			},
			wantErr: true,
			errType: ErrBadRequest,
		},
		{
			name: "missing model",
			params: &PricingAddParams{
				Provider:              ProviderOpenAI,
				InputPricePerMillion:  10.0,
				OutputPricePerMillion: 30.0,
			},
			wantErr: true,
			errType: ErrBadRequest,
		},
		{
			name: "with cached discount",
			params: &PricingAddParams{
				Provider:              ProviderAnthropic,
				Model:                 "claude-3-opus",
				InputPricePerMillion:  15.0,
				OutputPricePerMillion: 75.0,
				CachedInputDiscount:   floatPtr(0.5),
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if r.Method != http.MethodPost {
					t.Errorf("Expected POST, got %s", r.Method)
				}
				if r.URL.Path != "/api/v1/pricing" {
					t.Errorf("Expected /api/v1/pricing, got %s", r.URL.Path)
				}

				w.WriteHeader(http.StatusOK)
				json.NewEncoder(w).Encode(PricingTable{
					ID:       "price-123",
					Provider: tt.params.Provider,
					Model:    tt.params.Model,
					Currency: CurrencyUSD,
				})
			}))
			defer server.Close()

			client := setupTestClient(t, server.URL)
			defer client.Close()

			ctx := context.Background()
			result, err := client.Pricing.Add(ctx, tt.params)

			if tt.wantErr {
				if err == nil {
					t.Error("Expected error, got nil")
				}
				if tt.errType != nil && !isError(err, tt.errType) {
					t.Errorf("Expected error type %v, got %v", tt.errType, err)
				}
				return
			}

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			if result == nil {
				t.Error("Expected result, got nil")
			}
		})
	}
}

func TestPricingService_Get(t *testing.T) {
	tests := []struct {
		name    string
		id      string
		wantErr bool
		errType error
	}{
		{
			name:    "valid ID",
			id:      "price-123",
			wantErr: false,
		},
		{
			name:    "empty ID",
			id:      "",
			wantErr: true,
			errType: ErrBadRequest,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if r.Method != http.MethodGet {
					t.Errorf("Expected GET, got %s", r.Method)
				}

				w.WriteHeader(http.StatusOK)
				json.NewEncoder(w).Encode(PricingTable{
					ID:       tt.id,
					Provider: ProviderOpenAI,
					Model:    "gpt-4",
					Currency: CurrencyUSD,
				})
			}))
			defer server.Close()

			client := setupTestClient(t, server.URL)
			defer client.Close()

			ctx := context.Background()
			result, err := client.Pricing.Get(ctx, tt.id)

			if tt.wantErr {
				if err == nil {
					t.Error("Expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			if result == nil {
				t.Error("Expected result, got nil")
			}
		})
	}
}

func TestPricingService_List(t *testing.T) {
	tests := []struct {
		name   string
		params *PricingListParams
	}{
		{
			name:   "no filters",
			params: nil,
		},
		{
			name: "with provider filter",
			params: &PricingListParams{
				Provider: ProviderOpenAI,
			},
		},
		{
			name: "with model filter",
			params: &PricingListParams{
				Model: "gpt-4",
			},
		},
		{
			name: "with active filter",
			params: &PricingListParams{
				Active: boolPtr(true),
			},
		},
		{
			name: "with pagination",
			params: &PricingListParams{
				PaginationParams: PaginationParams{
					Page:     1,
					PageSize: 10,
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if r.Method != http.MethodGet {
					t.Errorf("Expected GET, got %s", r.Method)
				}

				// Verify query parameters
				if tt.params != nil {
					q := r.URL.Query()
					if tt.params.Provider != "" && q.Get("provider") != string(tt.params.Provider) {
						t.Errorf("Expected provider=%s, got %s", tt.params.Provider, q.Get("provider"))
					}
					if tt.params.Model != "" && q.Get("model") != tt.params.Model {
						t.Errorf("Expected model=%s, got %s", tt.params.Model, q.Get("model"))
					}
				}

				w.WriteHeader(http.StatusOK)
				json.NewEncoder(w).Encode([]PricingTable{
					{
						ID:       "price-1",
						Provider: ProviderOpenAI,
						Model:    "gpt-4",
					},
				})
			}))
			defer server.Close()

			client := setupTestClient(t, server.URL)
			defer client.Close()

			ctx := context.Background()
			result, err := client.Pricing.List(ctx, tt.params)

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			if result == nil {
				t.Error("Expected result, got nil")
			}
		})
	}
}

func TestPricingService_GetActive(t *testing.T) {
	tests := []struct {
		name     string
		provider Provider
		model    string
		wantErr  bool
		errType  error
	}{
		{
			name:     "valid request",
			provider: ProviderOpenAI,
			model:    "gpt-4",
			wantErr:  false,
		},
		{
			name:     "empty provider",
			provider: "",
			model:    "gpt-4",
			wantErr:  true,
			errType:  ErrBadRequest,
		},
		{
			name:     "empty model",
			provider: ProviderOpenAI,
			model:    "",
			wantErr:  true,
			errType:  ErrBadRequest,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusOK)
				json.NewEncoder(w).Encode(PricingTable{
					ID:       "price-123",
					Provider: tt.provider,
					Model:    tt.model,
					PricingStructure: PricingStructure{
						Type:                  "per_token",
						InputPricePerMillion:  floatPtr(10.0),
						OutputPricePerMillion: floatPtr(30.0),
					},
					EffectiveDate: time.Now(),
				})
			}))
			defer server.Close()

			client := setupTestClient(t, server.URL)
			defer client.Close()

			ctx := context.Background()
			result, err := client.Pricing.GetActive(ctx, tt.provider, tt.model)

			if tt.wantErr {
				if err == nil {
					t.Error("Expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			if result == nil {
				t.Error("Expected result, got nil")
			}
		})
	}
}

// Helper functions
func setupTestClient(t *testing.T, baseURL string) *Client {
	t.Helper()
	client, err := NewClient(
		WithAPIKey("test-key"),
		WithBaseURL(baseURL),
		WithMaxRetries(0),
	)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	return client
}

func floatPtr(f float64) *float64 {
	return &f
}

func boolPtr(b bool) *bool {
	return &b
}
