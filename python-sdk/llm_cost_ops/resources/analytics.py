"""
Analytics resource for cost analytics and insights.

This module provides methods for querying analytics data and time-series metrics.
"""

from datetime import datetime
from typing import List, Optional

from .._internal.http_client import AsyncHTTPClient, SyncHTTPClient
from ..types import AnalyticsResponse


class AnalyticsResource:
    """Synchronous analytics resource."""

    def __init__(self, client: SyncHTTPClient) -> None:
        """
        Initialize analytics resource.

        Args:
            client: HTTP client instance
        """
        self._client = client

    def get(
        self,
        start_date: datetime,
        end_date: datetime,
        organization_id: Optional[str] = None,
        provider: Optional[str] = None,
        model_id: Optional[str] = None,
        metrics: Optional[List[str]] = None,
        group_by: Optional[List[str]] = None,
        interval: str = "day",
    ) -> AnalyticsResponse:
        """
        Get analytics data.

        Args:
            start_date: Analysis start date
            end_date: Analysis end date
            organization_id: Filter by organization
            provider: Filter by provider
            model_id: Filter by model
            metrics: Metrics to include (total_cost, total_tokens, etc.)
            group_by: Dimensions to group by (provider, model, organization)
            interval: Time interval (hour, day, week, month)

        Returns:
            Analytics response with time-series and summary

        Raises:
            ValidationError: If request parameters are invalid
            APIError: If the API returns an error

        Example:
            >>> analytics = client.analytics.get(
            ...     start_date=datetime(2025, 1, 1),
            ...     end_date=datetime(2025, 1, 31),
            ...     organization_id="org-123",
            ...     interval="day",
            ...     metrics=["total_cost", "total_tokens"]
            ... )
            >>> print(f"Total cost: ${analytics.summary.total_cost}")
            >>> for point in analytics.time_series:
            ...     print(f"{point.timestamp}: {point.metrics}")
        """
        params = {
            "start_date": start_date.isoformat(),
            "end_date": end_date.isoformat(),
            "interval": interval,
        }

        if organization_id:
            params["organization_id"] = organization_id
        if provider:
            params["provider"] = provider
        if model_id:
            params["model_id"] = model_id
        if metrics:
            params["metrics"] = ",".join(metrics)
        if group_by:
            params["group_by"] = ",".join(group_by)

        response = self._client.request("GET", "/api/v1/analytics", params=params)

        data = response.get("data", response)
        return AnalyticsResponse.model_validate(data)


class AsyncAnalyticsResource:
    """Asynchronous analytics resource."""

    def __init__(self, client: AsyncHTTPClient) -> None:
        """
        Initialize async analytics resource.

        Args:
            client: Async HTTP client instance
        """
        self._client = client

    async def get(
        self,
        start_date: datetime,
        end_date: datetime,
        organization_id: Optional[str] = None,
        provider: Optional[str] = None,
        model_id: Optional[str] = None,
        metrics: Optional[List[str]] = None,
        group_by: Optional[List[str]] = None,
        interval: str = "day",
    ) -> AnalyticsResponse:
        """
        Get analytics data asynchronously.

        Args:
            start_date: Analysis start date
            end_date: Analysis end date
            organization_id: Filter by organization
            provider: Filter by provider
            model_id: Filter by model
            metrics: Metrics to include (total_cost, total_tokens, etc.)
            group_by: Dimensions to group by (provider, model, organization)
            interval: Time interval (hour, day, week, month)

        Returns:
            Analytics response with time-series and summary

        Raises:
            ValidationError: If request parameters are invalid
            APIError: If the API returns an error
        """
        params = {
            "start_date": start_date.isoformat(),
            "end_date": end_date.isoformat(),
            "interval": interval,
        }

        if organization_id:
            params["organization_id"] = organization_id
        if provider:
            params["provider"] = provider
        if model_id:
            params["model_id"] = model_id
        if metrics:
            params["metrics"] = ",".join(metrics)
        if group_by:
            params["group_by"] = ",".join(group_by)

        response = await self._client.request("GET", "/api/v1/analytics", params=params)

        data = response.get("data", response)
        return AnalyticsResponse.model_validate(data)
