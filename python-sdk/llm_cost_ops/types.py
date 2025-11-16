"""
Type definitions and Pydantic models for LLM-CostOps SDK.

This module defines all request and response models used by the SDK,
with full validation and serialization support via Pydantic.
"""

from datetime import datetime
from decimal import Decimal
from enum import Enum
from typing import Any, Dict, List, Optional

from pydantic import BaseModel, ConfigDict, Field, field_validator

# ===== Enums =====


class Provider(str, Enum):
    """LLM provider enumeration."""

    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    GOOGLE = "google"
    AZURE_OPENAI = "azure_openai"
    AWS_BEDROCK = "aws_bedrock"
    COHERE = "cohere"
    MISTRAL = "mistral"


class Currency(str, Enum):
    """Currency enumeration."""

    USD = "USD"
    EUR = "EUR"
    GBP = "GBP"
    JPY = "JPY"


class CostGroupBy(str, Enum):
    """Cost grouping options."""

    NONE = "none"
    PROVIDER = "provider"
    MODEL = "model"
    DAY = "day"
    WEEK = "week"
    MONTH = "month"


class AnalyticsMetric(str, Enum):
    """Analytics metric types."""

    TOTAL_COST = "total_cost"
    TOTAL_TOKENS = "total_tokens"
    TOTAL_REQUESTS = "total_requests"
    AVERAGE_COST_PER_REQUEST = "average_cost_per_request"
    AVERAGE_TOKENS_PER_REQUEST = "average_tokens_per_request"


class AnalyticsDimension(str, Enum):
    """Analytics dimension types."""

    PROVIDER = "provider"
    MODEL = "model"
    ORGANIZATION = "organization"


class AnalyticsInterval(str, Enum):
    """Analytics interval types."""

    HOUR = "hour"
    DAY = "day"
    WEEK = "week"
    MONTH = "month"


class HealthStatus(str, Enum):
    """Health status enumeration."""

    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"


# ===== Base Models =====


class CostOpsModel(BaseModel):
    """Base model for all SDK models."""

    model_config = ConfigDict(
        use_enum_values=True,
        populate_by_name=True,
    )

    @classmethod
    def model_dump_json(cls, *args, **kwargs):  # type: ignore
        """Serialize model to JSON with custom encoders."""
        # Custom serialization is now handled by Pydantic V2
        return super().model_dump_json(*args, **kwargs)


# ===== Usage Models =====


class SubmitUsageRequest(CostOpsModel):
    """Request model for submitting usage data."""

    organization_id: str = Field(..., min_length=1, max_length=100, description="Organization ID")
    provider: Provider = Field(..., description="LLM provider")
    model_id: str = Field(..., min_length=1, max_length=255, description="Model identifier")
    input_tokens: int = Field(..., ge=0, description="Number of input tokens")
    output_tokens: int = Field(..., ge=0, description="Number of output tokens")
    total_tokens: int = Field(..., ge=0, description="Total number of tokens")
    timestamp: Optional[datetime] = Field(default=None, description="Usage timestamp")
    metadata: Optional[Dict[str, Any]] = Field(default=None, description="Additional metadata")

    @field_validator("total_tokens")
    @classmethod
    def validate_total_tokens(cls, v: int, info: Any) -> int:
        """Validate that total_tokens equals input_tokens + output_tokens if all are provided."""
        data = info.data
        if "input_tokens" in data and "output_tokens" in data:
            expected = data["input_tokens"] + data["output_tokens"]
            if v != expected:
                raise ValueError(
                    f"total_tokens ({v}) must equal input_tokens + output_tokens ({expected})"
                )
        return v


class SubmitUsageResponse(CostOpsModel):
    """Response model for submit usage request."""

    usage_id: str = Field(..., description="Unique usage record ID")
    organization_id: str = Field(..., description="Organization ID")
    estimated_cost: Decimal = Field(..., description="Estimated cost")
    currency: Currency = Field(..., description="Currency code")
    processed_at: datetime = Field(..., description="Processing timestamp")


# ===== Cost Models =====


class CostBreakdown(CostOpsModel):
    """Cost breakdown by dimension."""

    dimension: str = Field(..., description="Breakdown dimension")
    value: str = Field(..., description="Dimension value")
    cost: Decimal = Field(..., description="Cost amount")
    tokens: int = Field(..., ge=0, description="Token count")
    requests: int = Field(..., ge=0, description="Request count")


class CostSummary(CostOpsModel):
    """Cost summary response."""

    total_cost: Decimal = Field(..., description="Total cost")
    currency: Currency = Field(..., description="Currency code")
    total_tokens: int = Field(..., ge=0, description="Total tokens")
    total_requests: int = Field(..., ge=0, description="Total requests")
    period_start: datetime = Field(..., description="Period start timestamp")
    period_end: datetime = Field(..., description="Period end timestamp")
    breakdown: Optional[List[CostBreakdown]] = Field(default=None, description="Cost breakdown")


# ===== Pricing Models =====


class CreatePricingRequest(CostOpsModel):
    """Request model for creating pricing entry."""

    provider: Provider = Field(..., description="LLM provider")
    model_id: str = Field(..., min_length=1, max_length=255, description="Model identifier")
    input_price_per_1k: Decimal = Field(..., description="Input price per 1K tokens")
    output_price_per_1k: Decimal = Field(..., description="Output price per 1K tokens")
    currency: Currency = Field(default=Currency.USD, description="Currency code")
    effective_date: Optional[datetime] = Field(default=None, description="Effective date")


class PricingResponse(CostOpsModel):
    """Response model for pricing entry."""

    id: str = Field(..., description="Unique pricing ID")
    provider: Provider = Field(..., description="LLM provider")
    model_id: str = Field(..., description="Model identifier")
    input_price_per_1k: Decimal = Field(..., description="Input price per 1K tokens")
    output_price_per_1k: Decimal = Field(..., description="Output price per 1K tokens")
    currency: Currency = Field(..., description="Currency code")
    effective_date: datetime = Field(..., description="Effective date")
    created_at: datetime = Field(..., description="Creation timestamp")
    updated_at: datetime = Field(..., description="Last update timestamp")


# ===== Analytics Models =====


class TimeSeriesPoint(CostOpsModel):
    """Time series data point."""

    timestamp: datetime = Field(..., description="Data point timestamp")
    metrics: Dict[str, Any] = Field(..., description="Metric values")


class AnalyticsSummary(CostOpsModel):
    """Analytics summary statistics."""

    total_cost: Decimal = Field(..., description="Total cost")
    total_tokens: int = Field(..., ge=0, description="Total tokens")
    total_requests: int = Field(..., ge=0, description="Total requests")
    average_cost_per_request: Decimal = Field(..., description="Average cost per request")
    average_tokens_per_request: float = Field(..., ge=0, description="Average tokens per request")


class AnalyticsResponse(CostOpsModel):
    """Analytics response."""

    time_series: List[TimeSeriesPoint] = Field(default_factory=list, description="Time series data")
    summary: AnalyticsSummary = Field(..., description="Summary statistics")


# ===== Health Check Models =====


class ComponentHealth(CostOpsModel):
    """Component health status."""

    name: str = Field(..., description="Component name")
    status: HealthStatus = Field(..., description="Health status")
    message: Optional[str] = Field(default=None, description="Status message")


class HealthResponse(CostOpsModel):
    """Health check response."""

    status: HealthStatus = Field(..., description="Overall health status")
    version: str = Field(..., description="API version")
    uptime_seconds: int = Field(..., ge=0, description="Uptime in seconds")
    components: List[ComponentHealth] = Field(default_factory=list, description="Component statuses")


# ===== Pagination Models =====


class PaginationMeta(CostOpsModel):
    """Pagination metadata."""

    page: int = Field(..., ge=1, description="Current page number")
    per_page: int = Field(..., ge=1, le=100, description="Items per page")
    total_items: int = Field(..., ge=0, description="Total item count")
    total_pages: int = Field(..., ge=0, description="Total page count")


class PaginatedResponse(CostOpsModel):
    """Paginated response wrapper."""

    data: List[Any] = Field(default_factory=list, description="Response data")
    pagination: PaginationMeta = Field(..., description="Pagination metadata")


# ===== Response Wrapper =====


class ResponseMetadata(CostOpsModel):
    """API response metadata."""

    request_id: Optional[str] = Field(default=None, description="Request ID")
    timestamp: datetime = Field(default_factory=datetime.utcnow, description="Response timestamp")
    version: str = Field(default="v1", description="API version")


class APIResponse(CostOpsModel):
    """Standard API response wrapper."""

    data: Any = Field(..., description="Response data")
    meta: Optional[ResponseMetadata] = Field(default=None, description="Response metadata")
