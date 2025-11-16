package llmcostops

import (
	"context"
	"fmt"
	"net/http"
	"net/url"
	"time"
)

// UsageService handles usage-related API operations
type UsageService struct {
	client *Client
}

// UsageIngestParams represents parameters for ingesting usage data
type UsageIngestParams struct {
	Records []UsageRecord `json:"records"`
}

// UsageListParams represents parameters for listing usage records
type UsageListParams struct {
	Range          TimeRange  `json:"range,omitempty"`
	StartTime      *time.Time `json:"start_time,omitempty"`
	EndTime        *time.Time `json:"end_time,omitempty"`
	OrganizationID string     `json:"organization_id,omitempty"`
	ProjectID      string     `json:"project_id,omitempty"`
	Provider       Provider   `json:"provider,omitempty"`
	Model          string     `json:"model,omitempty"`
	UserID         string     `json:"user_id,omitempty"`
	Tags           []string   `json:"tags,omitempty"`
	SortBy         string     `json:"sort_by,omitempty"`
	SortOrder      SortOrder  `json:"sort_order,omitempty"`
	PaginationParams
}

// UsageStatsParams represents parameters for usage statistics
type UsageStatsParams struct {
	Range          TimeRange  `json:"range,omitempty"`
	StartTime      *time.Time `json:"start_time,omitempty"`
	EndTime        *time.Time `json:"end_time,omitempty"`
	OrganizationID string     `json:"organization_id,omitempty"`
	ProjectID      string     `json:"project_id,omitempty"`
	GroupBy        []string   `json:"group_by,omitempty"` // e.g., "provider", "model", "project"
}

// UsageStats represents aggregated usage statistics
type UsageStats struct {
	Period            Period                      `json:"period"`
	TotalRequests     int64                       `json:"total_requests"`
	TotalPromptTokens int64                       `json:"total_prompt_tokens"`
	TotalCompTokens   int64                       `json:"total_completion_tokens"`
	TotalTokens       int64                       `json:"total_tokens"`
	AvgLatencyMs      *int64                      `json:"avg_latency_ms,omitempty"`
	ByProvider        map[Provider]*ProviderStats `json:"by_provider,omitempty"`
	ByModel           map[string]*ModelStats      `json:"by_model,omitempty"`
	ByProject         map[string]*ProjectStats    `json:"by_project,omitempty"`
}

// ProviderStats represents provider-level statistics
type ProviderStats struct {
	Requests     int64 `json:"requests"`
	PromptTokens int64 `json:"prompt_tokens"`
	CompTokens   int64 `json:"completion_tokens"`
	TotalTokens  int64 `json:"total_tokens"`
}

// ModelStats represents model-level statistics
type ModelStats struct {
	Requests     int64 `json:"requests"`
	PromptTokens int64 `json:"prompt_tokens"`
	CompTokens   int64 `json:"completion_tokens"`
	TotalTokens  int64 `json:"total_tokens"`
}

// ProjectStats represents project-level statistics
type ProjectStats struct {
	Requests     int64 `json:"requests"`
	PromptTokens int64 `json:"prompt_tokens"`
	CompTokens   int64 `json:"completion_tokens"`
	TotalTokens  int64 `json:"total_tokens"`
}

// Ingest ingests one or more usage records
func (s *UsageService) Ingest(ctx context.Context, params *UsageIngestParams) error {
	if params == nil || len(params.Records) == 0 {
		return fmt.Errorf("%w: at least one record is required", ErrBadRequest)
	}

	req, err := s.client.newRequest(http.MethodPost, "/api/v1/usage/ingest", params)
	if err != nil {
		return err
	}

	return s.client.do(ctx, req, nil)
}

// Get retrieves a specific usage record by ID
func (s *UsageService) Get(ctx context.Context, id string) (*UsageRecord, error) {
	if id == "" {
		return nil, fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/usage/%s", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodGet, path, nil)
	if err != nil {
		return nil, err
	}

	var result UsageRecord
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// List retrieves a list of usage records with optional filters
func (s *UsageService) List(ctx context.Context, params *UsageListParams) ([]UsageRecord, error) {
	req, err := s.client.newRequest(http.MethodGet, "/api/v1/usage", nil)
	if err != nil {
		return nil, err
	}

	// Add query parameters
	if params != nil {
		q := req.URL.Query()
		if params.Range != "" {
			q.Set("range", string(params.Range))
		}
		if params.StartTime != nil {
			q.Set("start_time", params.StartTime.Format(time.RFC3339))
		}
		if params.EndTime != nil {
			q.Set("end_time", params.EndTime.Format(time.RFC3339))
		}
		if params.OrganizationID != "" {
			q.Set("organization_id", params.OrganizationID)
		}
		if params.ProjectID != "" {
			q.Set("project_id", params.ProjectID)
		}
		if params.Provider != "" {
			q.Set("provider", string(params.Provider))
		}
		if params.Model != "" {
			q.Set("model", params.Model)
		}
		if params.UserID != "" {
			q.Set("user_id", params.UserID)
		}
		if params.SortBy != "" {
			q.Set("sort_by", params.SortBy)
		}
		if params.SortOrder != "" {
			q.Set("sort_order", string(params.SortOrder))
		}
		if params.Page > 0 {
			q.Set("page", fmt.Sprintf("%d", params.Page))
		}
		if params.PageSize > 0 {
			q.Set("page_size", fmt.Sprintf("%d", params.PageSize))
		}
		req.URL.RawQuery = q.Encode()
	}

	var result []UsageRecord
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return result, nil
}

// Stats retrieves aggregated usage statistics
func (s *UsageService) Stats(ctx context.Context, params *UsageStatsParams) (*UsageStats, error) {
	req, err := s.client.newRequest(http.MethodGet, "/api/v1/usage/stats", nil)
	if err != nil {
		return nil, err
	}

	// Add query parameters
	if params != nil {
		q := req.URL.Query()
		if params.Range != "" {
			q.Set("range", string(params.Range))
		}
		if params.StartTime != nil {
			q.Set("start_time", params.StartTime.Format(time.RFC3339))
		}
		if params.EndTime != nil {
			q.Set("end_time", params.EndTime.Format(time.RFC3339))
		}
		if params.OrganizationID != "" {
			q.Set("organization_id", params.OrganizationID)
		}
		if params.ProjectID != "" {
			q.Set("project_id", params.ProjectID)
		}
		for _, groupBy := range params.GroupBy {
			q.Add("group_by", groupBy)
		}
		req.URL.RawQuery = q.Encode()
	}

	var result UsageStats
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// Delete deletes a usage record by ID
func (s *UsageService) Delete(ctx context.Context, id string) error {
	if id == "" {
		return fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/usage/%s", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodDelete, path, nil)
	if err != nil {
		return err
	}

	return s.client.do(ctx, req, nil)
}
