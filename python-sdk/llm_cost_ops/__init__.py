"""
LLM-CostOps Python SDK

Enterprise-grade Python SDK for the LLM Cost Operations Platform.
Track, analyze, and optimize costs across multiple LLM providers.

Example:
    >>> from llm_cost_ops import CostOpsClient
    >>>
    >>> client = CostOpsClient(api_key="your-api-key")
    >>> usage = client.usage.submit(
    ...     organization_id="org-123",
    ...     provider="openai",
    ...     model_id="gpt-4",
    ...     input_tokens=1000,
    ...     output_tokens=500,
    ...     total_tokens=1500
    ... )
    >>> print(f"Estimated cost: ${usage.estimated_cost}")
"""

__version__ = "0.1.0"

from .client import AsyncCostOpsClient, CostOpsClient
from .config import (
    ClientConfig,
    ConnectionPoolConfig,
    LoggingConfig,
    MetricsConfig,
    RateLimitConfig,
    RetryConfig,
)
from .exceptions import (
    APIError,
    AuthenticationError,
    AuthorizationError,
    ConfigurationError,
    CostOpsError,
    NetworkError,
    NotFoundError,
    RateLimitError,
    ServerError,
    TimeoutError,
    ValidationError,
)
from .types import (
    AnalyticsDimension,
    AnalyticsInterval,
    AnalyticsMetric,
    AnalyticsResponse,
    AnalyticsSummary,
    CostBreakdown,
    CostGroupBy,
    CostSummary,
    CreatePricingRequest,
    Currency,
    PricingResponse,
    Provider,
    SubmitUsageRequest,
    SubmitUsageResponse,
    TimeSeriesPoint,
)

__all__ = [
    # Version
    "__version__",
    # Clients
    "CostOpsClient",
    "AsyncCostOpsClient",
    # Configuration
    "ClientConfig",
    "RetryConfig",
    "ConnectionPoolConfig",
    "RateLimitConfig",
    "LoggingConfig",
    "MetricsConfig",
    # Exceptions
    "CostOpsError",
    "APIError",
    "AuthenticationError",
    "AuthorizationError",
    "ValidationError",
    "NotFoundError",
    "RateLimitError",
    "ServerError",
    "NetworkError",
    "TimeoutError",
    "ConfigurationError",
    # Types
    "Provider",
    "Currency",
    "CostGroupBy",
    "AnalyticsMetric",
    "AnalyticsDimension",
    "AnalyticsInterval",
    "SubmitUsageRequest",
    "SubmitUsageResponse",
    "CostSummary",
    "CostBreakdown",
    "CreatePricingRequest",
    "PricingResponse",
    "AnalyticsResponse",
    "AnalyticsSummary",
    "TimeSeriesPoint",
]
