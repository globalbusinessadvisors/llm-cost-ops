"""Pytest configuration and fixtures."""

import pytest
from decimal import Decimal
from datetime import datetime

from llm_cost_ops import ClientConfig
from llm_cost_ops.types import (
    SubmitUsageResponse,
    CostSummary,
    PricingResponse,
    AnalyticsResponse,
    AnalyticsSummary,
    Currency,
    Provider,
)


@pytest.fixture
def api_key() -> str:
    """Return test API key."""
    return "test-api-key-12345"


@pytest.fixture
def base_url() -> str:
    """Return test base URL."""
    return "https://api.test.llm-cost-ops.dev"


@pytest.fixture
def client_config(api_key: str, base_url: str) -> ClientConfig:
    """Return test client configuration."""
    return ClientConfig(
        api_key=api_key,
        base_url=base_url,
        timeout=10.0,
    )


@pytest.fixture
def mock_usage_response() -> dict:
    """Return mock usage response."""
    return {
        "data": {
            "usage_id": "usage-123",
            "organization_id": "org-123",
            "estimated_cost": "0.015",
            "currency": "USD",
            "processed_at": "2025-01-15T10:00:00Z",
        }
    }


@pytest.fixture
def mock_cost_summary_response() -> dict:
    """Return mock cost summary response."""
    return {
        "data": {
            "total_cost": "45.50",
            "currency": "USD",
            "total_tokens": 150000,
            "total_requests": 100,
            "period_start": "2025-01-01T00:00:00Z",
            "period_end": "2025-01-31T23:59:59Z",
            "breakdown": [
                {
                    "dimension": "provider",
                    "value": "openai",
                    "cost": "30.00",
                    "tokens": 100000,
                    "requests": 60,
                },
                {
                    "dimension": "provider",
                    "value": "anthropic",
                    "cost": "15.50",
                    "tokens": 50000,
                    "requests": 40,
                },
            ],
        }
    }


@pytest.fixture
def mock_pricing_response() -> dict:
    """Return mock pricing response."""
    return {
        "data": {
            "id": "price-123",
            "provider": "openai",
            "model_id": "gpt-4",
            "input_price_per_1k": "0.01",
            "output_price_per_1k": "0.03",
            "currency": "USD",
            "effective_date": "2025-01-01T00:00:00Z",
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T00:00:00Z",
        }
    }


@pytest.fixture
def mock_analytics_response() -> dict:
    """Return mock analytics response."""
    return {
        "data": {
            "time_series": [
                {
                    "timestamp": "2025-01-01T00:00:00Z",
                    "metrics": {"total_cost": "10.00", "total_tokens": 10000},
                },
                {
                    "timestamp": "2025-01-02T00:00:00Z",
                    "metrics": {"total_cost": "15.50", "total_tokens": 15000},
                },
            ],
            "summary": {
                "total_cost": "25.50",
                "total_tokens": 25000,
                "total_requests": 50,
                "average_cost_per_request": "0.51",
                "average_tokens_per_request": 500.0,
            },
        }
    }
