package llmcostops

import (
	"context"
	"fmt"
	"net/http"
	"net/url"
	"time"
)

// CostService handles cost-related API operations
type CostService struct {
	client *Client
}

// CostListParams represents parameters for listing cost records
type CostListParams struct {
	Range          TimeRange  `json:"range,omitempty"`
	StartTime      *time.Time `json:"start_time,omitempty"`
	EndTime        *time.Time `json:"end_time,omitempty"`
	OrganizationID string     `json:"organization_id,omitempty"`
	ProjectID      string     `json:"project_id,omitempty"`
	Provider       Provider   `json:"provider,omitempty"`
	Model          string     `json:"model,omitempty"`
	UserID         string     `json:"user_id,omitempty"`
	MinCost        *float64   `json:"min_cost,omitempty"`
	MaxCost        *float64   `json:"max_cost,omitempty"`
	SortBy         string     `json:"sort_by,omitempty"`
	SortOrder      SortOrder  `json:"sort_order,omitempty"`
	PaginationParams
}

// CostSummaryParams represents parameters for cost summary
type CostSummaryParams struct {
	Range          TimeRange  `json:"range,omitempty"`
	StartTime      *time.Time `json:"start_time,omitempty"`
	EndTime        *time.Time `json:"end_time,omitempty"`
	OrganizationID string     `json:"organization_id,omitempty"`
	ProjectID      string     `json:"project_id,omitempty"`
	GroupBy        []string   `json:"group_by,omitempty"` // e.g., "provider", "model", "project"
}

// CostAnalyticsParams represents parameters for cost analytics
type CostAnalyticsParams struct {
	Range          TimeRange  `json:"range,omitempty"`
	StartTime      *time.Time `json:"start_time,omitempty"`
	EndTime        *time.Time `json:"end_time,omitempty"`
	OrganizationID string     `json:"organization_id,omitempty"`
	ProjectID      string     `json:"project_id,omitempty"`
	Granularity    string     `json:"granularity,omitempty"` // "hour", "day", "week", "month"
}

// CostAnalytics represents time-series cost analytics
type CostAnalytics struct {
	Period      Period          `json:"period"`
	Granularity string          `json:"granularity"`
	DataPoints  []CostDataPoint `json:"data_points"`
	Trend       *TrendAnalysis  `json:"trend,omitempty"`
}

// CostDataPoint represents a single data point in cost analytics
type CostDataPoint struct {
	Timestamp time.Time `json:"timestamp"`
	Cost      string    `json:"cost"`
	Requests  int64     `json:"requests"`
}

// TrendAnalysis represents trend analysis results
type TrendAnalysis struct {
	Direction  string  `json:"direction"` // "increasing", "decreasing", "stable"
	ChangeRate float64 `json:"change_rate"`
	Confidence float64 `json:"confidence"`
}

// Get retrieves a specific cost record by ID
func (s *CostService) Get(ctx context.Context, id string) (*CostRecord, error) {
	if id == "" {
		return nil, fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/costs/%s", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodGet, path, nil)
	if err != nil {
		return nil, err
	}

	var result CostRecord
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// List retrieves a list of cost records with optional filters
func (s *CostService) List(ctx context.Context, params *CostListParams) ([]CostRecord, error) {
	req, err := s.client.newRequest(http.MethodGet, "/api/v1/costs", nil)
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
		if params.MinCost != nil {
			q.Set("min_cost", fmt.Sprintf("%.10f", *params.MinCost))
		}
		if params.MaxCost != nil {
			q.Set("max_cost", fmt.Sprintf("%.10f", *params.MaxCost))
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

	var result []CostRecord
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return result, nil
}

// Summary retrieves cost summary with aggregations
func (s *CostService) Summary(ctx context.Context, params *CostSummaryParams) (*CostSummary, error) {
	req, err := s.client.newRequest(http.MethodGet, "/api/v1/costs/summary", nil)
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

	var result CostSummary
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// Analytics retrieves time-series cost analytics
func (s *CostService) Analytics(ctx context.Context, params *CostAnalyticsParams) (*CostAnalytics, error) {
	req, err := s.client.newRequest(http.MethodGet, "/api/v1/costs/analytics", nil)
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
		if params.Granularity != "" {
			q.Set("granularity", params.Granularity)
		}
		req.URL.RawQuery = q.Encode()
	}

	var result CostAnalytics
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// ByProvider retrieves costs grouped by provider
func (s *CostService) ByProvider(ctx context.Context, params *CostSummaryParams) (map[Provider]string, error) {
	summary, err := s.Summary(ctx, params)
	if err != nil {
		return nil, err
	}
	return summary.ByProvider, nil
}

// ByModel retrieves costs grouped by model
func (s *CostService) ByModel(ctx context.Context, params *CostSummaryParams) (map[string]string, error) {
	summary, err := s.Summary(ctx, params)
	if err != nil {
		return nil, err
	}
	return summary.ByModel, nil
}
