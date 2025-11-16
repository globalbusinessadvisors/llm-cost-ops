"""
Costs resource for querying cost data.

This module provides methods for retrieving cost summaries and breakdowns.
"""

from datetime import datetime
from typing import Optional

from .._internal.http_client import AsyncHTTPClient, SyncHTTPClient
from ..types import CostSummary


class CostsResource:
    """Synchronous costs resource."""

    def __init__(self, client: SyncHTTPClient) -> None:
        """
        Initialize costs resource.

        Args:
            client: HTTP client instance
        """
        self._client = client

    def get(
        self,
        organization_id: Optional[str] = None,
        provider: Optional[str] = None,
        model_id: Optional[str] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        group_by: Optional[str] = None,
    ) -> CostSummary:
        """
        Get cost summary.

        Args:
            organization_id: Filter by organization
            provider: Filter by provider
            model_id: Filter by model
            start_date: Start date for cost period
            end_date: End date for cost period
            group_by: Group costs by dimension (none, provider, model, day, week, month)

        Returns:
            Cost summary with totals and optional breakdown

        Raises:
            ValidationError: If request parameters are invalid
            APIError: If the API returns an error

        Example:
            >>> costs = client.costs.get(
            ...     organization_id="org-123",
            ...     start_date=datetime(2025, 1, 1),
            ...     end_date=datetime(2025, 1, 31),
            ...     group_by="provider"
            ... )
            >>> print(f"Total: ${costs.total_cost}")
            >>> for item in costs.breakdown:
            ...     print(f"{item.value}: ${item.cost}")
        """
        params = {}

        if organization_id:
            params["organization_id"] = organization_id
        if provider:
            params["provider"] = provider
        if model_id:
            params["model_id"] = model_id
        if start_date:
            params["start_date"] = start_date.isoformat()
        if end_date:
            params["end_date"] = end_date.isoformat()
        if group_by:
            params["group_by"] = group_by

        response = self._client.request("GET", "/api/v1/costs", params=params)

        # Extract data from response wrapper
        data = response.get("data", response)
        return CostSummary.model_validate(data)


class AsyncCostsResource:
    """Asynchronous costs resource."""

    def __init__(self, client: AsyncHTTPClient) -> None:
        """
        Initialize async costs resource.

        Args:
            client: Async HTTP client instance
        """
        self._client = client

    async def get(
        self,
        organization_id: Optional[str] = None,
        provider: Optional[str] = None,
        model_id: Optional[str] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        group_by: Optional[str] = None,
    ) -> CostSummary:
        """
        Get cost summary asynchronously.

        Args:
            organization_id: Filter by organization
            provider: Filter by provider
            model_id: Filter by model
            start_date: Start date for cost period
            end_date: End date for cost period
            group_by: Group costs by dimension (none, provider, model, day, week, month)

        Returns:
            Cost summary with totals and optional breakdown

        Raises:
            ValidationError: If request parameters are invalid
            APIError: If the API returns an error
        """
        params = {}

        if organization_id:
            params["organization_id"] = organization_id
        if provider:
            params["provider"] = provider
        if model_id:
            params["model_id"] = model_id
        if start_date:
            params["start_date"] = start_date.isoformat()
        if end_date:
            params["end_date"] = end_date.isoformat()
        if group_by:
            params["group_by"] = group_by

        response = await self._client.request("GET", "/api/v1/costs", params=params)

        # Extract data from response wrapper
        data = response.get("data", response)
        return CostSummary.model_validate(data)
