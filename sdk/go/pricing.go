package llmcostops

import (
	"context"
	"fmt"
	"net/http"
	"net/url"
)

// PricingService handles pricing-related API operations
type PricingService struct {
	client *Client
}

// PricingAddParams represents parameters for adding pricing information
type PricingAddParams struct {
	Provider              Provider `json:"provider"`
	Model                 string   `json:"model"`
	InputPricePerMillion  float64  `json:"input_price_per_million"`
	OutputPricePerMillion float64  `json:"output_price_per_million"`
	CachedInputDiscount   *float64 `json:"cached_input_discount,omitempty"`
	Currency              Currency `json:"currency,omitempty"`
	EffectiveDate         string   `json:"effective_date,omitempty"`
}

// PricingListParams represents parameters for listing pricing information
type PricingListParams struct {
	Provider Provider `json:"provider,omitempty"`
	Model    string   `json:"model,omitempty"`
	Active   *bool    `json:"active,omitempty"`
	PaginationParams
}

// Add adds a new pricing table entry
func (s *PricingService) Add(ctx context.Context, params *PricingAddParams) (*PricingTable, error) {
	if params == nil {
		return nil, fmt.Errorf("%w: params cannot be nil", ErrBadRequest)
	}

	if params.Provider == "" {
		return nil, fmt.Errorf("%w: provider is required", ErrBadRequest)
	}

	if params.Model == "" {
		return nil, fmt.Errorf("%w: model is required", ErrBadRequest)
	}

	req, err := s.client.newRequest(http.MethodPost, "/api/v1/pricing", params)
	if err != nil {
		return nil, err
	}

	var result PricingTable
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// Get retrieves a specific pricing table by ID
func (s *PricingService) Get(ctx context.Context, id string) (*PricingTable, error) {
	if id == "" {
		return nil, fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/pricing/%s", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodGet, path, nil)
	if err != nil {
		return nil, err
	}

	var result PricingTable
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// List retrieves a list of pricing tables with optional filters
func (s *PricingService) List(ctx context.Context, params *PricingListParams) ([]PricingTable, error) {
	req, err := s.client.newRequest(http.MethodGet, "/api/v1/pricing", nil)
	if err != nil {
		return nil, err
	}

	// Add query parameters
	if params != nil {
		q := req.URL.Query()
		if params.Provider != "" {
			q.Set("provider", string(params.Provider))
		}
		if params.Model != "" {
			q.Set("model", params.Model)
		}
		if params.Active != nil {
			q.Set("active", fmt.Sprintf("%t", *params.Active))
		}
		if params.Page > 0 {
			q.Set("page", fmt.Sprintf("%d", params.Page))
		}
		if params.PageSize > 0 {
			q.Set("page_size", fmt.Sprintf("%d", params.PageSize))
		}
		req.URL.RawQuery = q.Encode()
	}

	var result []PricingTable
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return result, nil
}

// Delete removes a pricing table by ID
func (s *PricingService) Delete(ctx context.Context, id string) error {
	if id == "" {
		return fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/pricing/%s", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodDelete, path, nil)
	if err != nil {
		return err
	}

	return s.client.do(ctx, req, nil)
}

// GetActive retrieves the active pricing for a specific provider and model
func (s *PricingService) GetActive(ctx context.Context, provider Provider, model string) (*PricingTable, error) {
	if provider == "" {
		return nil, fmt.Errorf("%w: provider is required", ErrBadRequest)
	}
	if model == "" {
		return nil, fmt.Errorf("%w: model is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/pricing/active/%s/%s", url.PathEscape(string(provider)), url.PathEscape(model))
	req, err := s.client.newRequest(http.MethodGet, path, nil)
	if err != nil {
		return nil, err
	}

	var result PricingTable
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}
