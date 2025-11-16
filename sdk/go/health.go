package llmcostops

import (
	"context"
	"net/http"
)

// HealthService handles health check operations
type HealthService struct {
	client *Client
}

// Check performs a health check on the API
func (s *HealthService) Check(ctx context.Context) (*HealthStatus, error) {
	req, err := s.client.newRequest(http.MethodGet, "/health", nil)
	if err != nil {
		return nil, err
	}

	var result HealthStatus
	if err := s.client.do(ctx, req, &result); err != nil {
		return nil, err
	}

	return &result, nil
}

// Live checks if the service is alive (liveness probe)
func (s *HealthService) Live(ctx context.Context) error {
	req, err := s.client.newRequest(http.MethodGet, "/health/live", nil)
	if err != nil {
		return err
	}

	return s.client.do(ctx, req, nil)
}

// Ready checks if the service is ready to accept traffic (readiness probe)
func (s *HealthService) Ready(ctx context.Context) error {
	req, err := s.client.newRequest(http.MethodGet, "/health/ready", nil)
	if err != nil {
		return err
	}

	return s.client.do(ctx, req, nil)
}
