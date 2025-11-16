package llmcostops

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestUsageService_Ingest(t *testing.T) {
	tests := []struct {
		name    string
		params  *UsageIngestParams
		wantErr bool
		errType error
	}{
		{
			name: "single record",
			params: &UsageIngestParams{
				Records: []UsageRecord{
					{
						ID:               "usage-1",
						Provider:         ProviderOpenAI,
						Model:            Model{Name: "gpt-4"},
						OrganizationID:   "org-123",
						PromptTokens:     100,
						CompletionTokens: 50,
						TotalTokens:      150,
					},
				},
			},
			wantErr: false,
		},
		{
			name: "multiple records",
			params: &UsageIngestParams{
				Records: []UsageRecord{
					{
						ID:               "usage-1",
						Provider:         ProviderOpenAI,
						Model:            Model{Name: "gpt-4"},
						OrganizationID:   "org-123",
						PromptTokens:     100,
						CompletionTokens: 50,
						TotalTokens:      150,
					},
					{
						ID:               "usage-2",
						Provider:         ProviderAnthropic,
						Model:            Model{Name: "claude-3-opus"},
						OrganizationID:   "org-123",
						PromptTokens:     200,
						CompletionTokens: 100,
						TotalTokens:      300,
					},
				},
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
			name: "empty records",
			params: &UsageIngestParams{
				Records: []UsageRecord{},
			},
			wantErr: true,
			errType: ErrBadRequest,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if r.Method != http.MethodPost {
					t.Errorf("Expected POST, got %s", r.Method)
				}
				if r.URL.Path != "/api/v1/usage/ingest" {
					t.Errorf("Expected /api/v1/usage/ingest, got %s", r.URL.Path)
				}

				w.WriteHeader(http.StatusOK)
			}))
			defer server.Close()

			client := setupTestClient(t, server.URL)
			defer client.Close()

			ctx := context.Background()
			err := client.Usage.Ingest(ctx, tt.params)

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
			}
		})
	}
}

func TestUsageService_Get(t *testing.T) {
	tests := []struct {
		name    string
		id      string
		wantErr bool
		errType error
	}{
		{
			name:    "valid ID",
			id:      "usage-123",
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
				w.WriteHeader(http.StatusOK)
				json.NewEncoder(w).Encode(UsageRecord{
					ID:             tt.id,
					Provider:       ProviderOpenAI,
					Model:          Model{Name: "gpt-4"},
					OrganizationID: "org-123",
				})
			}))
			defer server.Close()

			client := setupTestClient(t, server.URL)
			defer client.Close()

			ctx := context.Background()
			result, err := client.Usage.Get(ctx, tt.id)

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

func TestUsageService_List(t *testing.T) {
	now := time.Now()
	tests := []struct {
		name   string
		params *UsageListParams
	}{
		{
			name:   "no filters",
			params: nil,
		},
		{
			name: "with time range",
			params: &UsageListParams{
				Range: RangeLast24Hours,
			},
		},
		{
			name: "with custom time",
			params: &UsageListParams{
				StartTime: &now,
				EndTime:   &now,
			},
		},
		{
			name: "with organization filter",
			params: &UsageListParams{
				OrganizationID: "org-123",
			},
		},
		{
			name: "with provider filter",
			params: &UsageListParams{
				Provider: ProviderOpenAI,
			},
		},
		{
			name: "with sorting",
			params: &UsageListParams{
				SortBy:    "timestamp",
				SortOrder: SortDesc,
			},
		},
		{
			name: "with pagination",
			params: &UsageListParams{
				PaginationParams: PaginationParams{
					Page:     2,
					PageSize: 50,
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusOK)
				json.NewEncoder(w).Encode([]UsageRecord{
					{
						ID:             "usage-1",
						Provider:       ProviderOpenAI,
						Model:          Model{Name: "gpt-4"},
						OrganizationID: "org-123",
					},
				})
			}))
			defer server.Close()

			client := setupTestClient(t, server.URL)
			defer client.Close()

			ctx := context.Background()
			result, err := client.Usage.List(ctx, tt.params)

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

func TestUsageService_Stats(t *testing.T) {
	now := time.Now()
	tests := []struct {
		name   string
		params *UsageStatsParams
	}{
		{
			name:   "no params",
			params: nil,
		},
		{
			name: "with range",
			params: &UsageStatsParams{
				Range: RangeLast7Days,
			},
		},
		{
			name: "with custom time",
			params: &UsageStatsParams{
				StartTime: &now,
				EndTime:   &now,
			},
		},
		{
			name: "with organization",
			params: &UsageStatsParams{
				OrganizationID: "org-123",
			},
		},
		{
			name: "with grouping",
			params: &UsageStatsParams{
				GroupBy: []string{"provider", "model"},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusOK)
				json.NewEncoder(w).Encode(UsageStats{
					TotalRequests:     100,
					TotalPromptTokens: 10000,
					TotalCompTokens:   5000,
					TotalTokens:       15000,
				})
			}))
			defer server.Close()

			client := setupTestClient(t, server.URL)
			defer client.Close()

			ctx := context.Background()
			result, err := client.Usage.Stats(ctx, tt.params)

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
