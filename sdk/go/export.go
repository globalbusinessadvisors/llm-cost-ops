package llmcostops

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/url"
)

// ExportService handles export-related API operations
type ExportService struct {
	client *Client
}

// ExportParams represents parameters for exporting data
type ExportParams struct {
	Format         ExportFormat `json:"format"`
	Range          TimeRange    `json:"range,omitempty"`
	OrganizationID string       `json:"organization_id,omitempty"`
	ProjectID      string       `json:"project_id,omitempty"`
	Provider       Provider     `json:"provider,omitempty"`
	Model          string       `json:"model,omitempty"`
	IncludeHeaders bool         `json:"include_headers,omitempty"`
}

// ReportScheduleParams represents parameters for scheduling a report
type ReportScheduleParams struct {
	Name           string                 `json:"name"`
	Schedule       string                 `json:"schedule"` // Cron expression
	Format         ExportFormat           `json:"format"`
	ReportType     string                 `json:"report_type"` // "cost", "usage", "forecast", "audit"
	OrganizationID string                 `json:"organization_id,omitempty"`
	ProjectID      string                 `json:"project_id,omitempty"`
	DeliveryMethod string                 `json:"delivery_method"` // "email", "storage", "webhook"
	DeliveryConfig map[string]interface{} `json:"delivery_config,omitempty"`
	Enabled        bool                   `json:"enabled"`
}

// ScheduledReport represents a scheduled report
type ScheduledReport struct {
	ID             string                 `json:"id"`
	Name           string                 `json:"name"`
	Schedule       string                 `json:"schedule"`
	Format         ExportFormat           `json:"format"`
	ReportType     string                 `json:"report_type"`
	OrganizationID string                 `json:"organization_id,omitempty"`
	ProjectID      string                 `json:"project_id,omitempty"`
	DeliveryMethod string                 `json:"delivery_method"`
	DeliveryConfig map[string]interface{} `json:"delivery_config,omitempty"`
	Enabled        bool                   `json:"enabled"`
	LastRun        *string                `json:"last_run,omitempty"`
	NextRun        *string                `json:"next_run,omitempty"`
	CreatedAt      string                 `json:"created_at"`
	UpdatedAt      string                 `json:"updated_at"`
}

// Export exports data in the specified format and returns the raw data
func (s *ExportService) Export(ctx context.Context, params *ExportParams) ([]byte, error) {
	if params == nil {
		return nil, fmt.Errorf("%w: params cannot be nil", ErrBadRequest)
	}

	if params.Format == "" {
		return nil, fmt.Errorf("%w: format is required", ErrBadRequest)
	}

	req, err := s.client.newRequest(http.MethodPost, "/api/v1/export", params)
	if err != nil {
		return nil, err
	}

	// Execute request
	c := s.client
	c.mu.RLock()
	if c.closed {
		c.mu.RUnlock()
		return nil, fmt.Errorf("client is closed")
	}
	c.mu.RUnlock()

	// Wait for rate limiter
	if err := c.rateLimiter.Wait(ctx); err != nil {
		return nil, fmt.Errorf("%w: %v", ErrContextCanceled, err)
	}

	// Set headers
	req.Header.Set("Authorization", "Bearer "+c.apiKey)
	req.Header.Set("User-Agent", UserAgent)
	req.Header.Set("Accept", "*/*")

	resp, err := c.httpClient.Do(req.WithContext(ctx))
	if err != nil {
		return nil, fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()

	// Read response body
	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response: %w", err)
	}

	// Check status code
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return nil, fmt.Errorf("export failed with status %d: %s", resp.StatusCode, string(data))
	}

	return data, nil
}

// ExportToWriter exports data and writes it to the provided writer
func (s *ExportService) ExportToWriter(ctx context.Context, params *ExportParams, w io.Writer) error {
	data, err := s.Export(ctx, params)
	if err != nil {
		return err
	}

	_, err = w.Write(data)
	return err
}

// ScheduleReport creates a scheduled report
func (s *ExportService) ScheduleReport(ctx context.Context, params *ReportScheduleParams) (*ScheduledReport, error) {
	if params == nil {
		return nil, fmt.Errorf("%w: params cannot be nil", ErrBadRequest)
	}

	if params.Name == "" {
		return nil, fmt.Errorf("%w: name is required", ErrBadRequest)
	}

	if params.Schedule == "" {
		return nil, fmt.Errorf("%w: schedule is required", ErrBadRequest)
	}

	req, err := s.client.newRequest(http.MethodPost, "/api/v1/export/schedule", params)
	if err != nil {
		return nil, err
	}

	var result ScheduledReport
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// GetScheduledReport retrieves a scheduled report by ID
func (s *ExportService) GetScheduledReport(ctx context.Context, id string) (*ScheduledReport, error) {
	if id == "" {
		return nil, fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/export/schedule/%s", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodGet, path, nil)
	if err != nil {
		return nil, err
	}

	var result ScheduledReport
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// ListScheduledReports lists all scheduled reports
func (s *ExportService) ListScheduledReports(ctx context.Context) ([]ScheduledReport, error) {
	req, err := s.client.newRequest(http.MethodGet, "/api/v1/export/schedule", nil)
	if err != nil {
		return nil, err
	}

	var result []ScheduledReport
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return result, nil
}

// UpdateScheduledReport updates a scheduled report
func (s *ExportService) UpdateScheduledReport(ctx context.Context, id string, params *ReportScheduleParams) (*ScheduledReport, error) {
	if id == "" {
		return nil, fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	if params == nil {
		return nil, fmt.Errorf("%w: params cannot be nil", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/export/schedule/%s", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodPut, path, params)
	if err != nil {
		return nil, err
	}

	var result ScheduledReport
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// DeleteScheduledReport deletes a scheduled report
func (s *ExportService) DeleteScheduledReport(ctx context.Context, id string) error {
	if id == "" {
		return fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/export/schedule/%s", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodDelete, path, nil)
	if err != nil {
		return err
	}

	return s.client.do(ctx, req, nil)
}

// TriggerScheduledReport manually triggers a scheduled report
func (s *ExportService) TriggerScheduledReport(ctx context.Context, id string) error {
	if id == "" {
		return fmt.Errorf("%w: id is required", ErrBadRequest)
	}

	path := fmt.Sprintf("/api/v1/export/schedule/%s/trigger", url.PathEscape(id))
	req, err := s.client.newRequest(http.MethodPost, path, nil)
	if err != nil {
		return err
	}

	return s.client.do(ctx, req, nil)
}
