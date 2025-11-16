"""
Usage resource for tracking LLM usage data.

This module provides methods for submitting and querying usage records.
"""

from datetime import datetime
from typing import Any, Dict, List, Optional

from .._internal.http_client import AsyncHTTPClient, SyncHTTPClient
from ..types import Provider, SubmitUsageRequest, SubmitUsageResponse


class UsageResource:
    """Synchronous usage resource."""

    def __init__(self, client: SyncHTTPClient) -> None:
        """
        Initialize usage resource.

        Args:
            client: HTTP client instance
        """
        self._client = client

    def submit(
        self,
        organization_id: str,
        provider: str,
        model_id: str,
        input_tokens: int,
        output_tokens: int,
        total_tokens: int,
        timestamp: Optional[datetime] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> SubmitUsageResponse:
        """
        Submit usage data.

        Args:
            organization_id: Organization identifier
            provider: LLM provider (openai, anthropic, etc.)
            model_id: Model identifier (e.g., gpt-4, claude-3-sonnet)
            input_tokens: Number of input/prompt tokens
            output_tokens: Number of output/completion tokens
            total_tokens: Total number of tokens
            timestamp: Usage timestamp (defaults to current time)
            metadata: Additional metadata

        Returns:
            Submit usage response with usage ID and estimated cost

        Raises:
            ValidationError: If request data is invalid
            APIError: If the API returns an error

        Example:
            >>> client = CostOpsClient(api_key="...")
            >>> usage = client.usage.submit(
            ...     organization_id="org-123",
            ...     provider="openai",
            ...     model_id="gpt-4",
            ...     input_tokens=1000,
            ...     output_tokens=500,
            ...     total_tokens=1500
            ... )
            >>> print(f"Usage ID: {usage.usage_id}")
            >>> print(f"Cost: ${usage.estimated_cost}")
        """
        request = SubmitUsageRequest(
            organization_id=organization_id,
            provider=Provider(provider),
            model_id=model_id,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            total_tokens=total_tokens,
            timestamp=timestamp,
            metadata=metadata,
        )

        response = self._client.request(
            "POST",
            "/api/v1/usage",
            json=request.model_dump(mode="json", exclude_none=True),
        )

        # Extract data from response wrapper
        data = response.get("data", response)
        return SubmitUsageResponse.model_validate(data)

    def get_history(
        self,
        organization_id: Optional[str] = None,
        provider: Optional[str] = None,
        model_id: Optional[str] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        page: int = 1,
        per_page: int = 20,
    ) -> List[Dict[str, Any]]:
        """
        Get usage history.

        Args:
            organization_id: Filter by organization
            provider: Filter by provider
            model_id: Filter by model
            start_date: Start date for filtering
            end_date: End date for filtering
            page: Page number (1-indexed)
            per_page: Items per page

        Returns:
            List of usage records

        Raises:
            APIError: If the API returns an error

        Example:
            >>> history = client.usage.get_history(
            ...     organization_id="org-123",
            ...     start_date=datetime(2025, 1, 1),
            ...     end_date=datetime(2025, 1, 31)
            ... )
        """
        params: Dict[str, Any] = {
            "page": page,
            "per_page": per_page,
        }

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

        response = self._client.request("GET", "/api/v1/usage/history", params=params)

        # Extract data from paginated response
        return response.get("data", [])


class AsyncUsageResource:
    """Asynchronous usage resource."""

    def __init__(self, client: AsyncHTTPClient) -> None:
        """
        Initialize async usage resource.

        Args:
            client: Async HTTP client instance
        """
        self._client = client

    async def submit(
        self,
        organization_id: str,
        provider: str,
        model_id: str,
        input_tokens: int,
        output_tokens: int,
        total_tokens: int,
        timestamp: Optional[datetime] = None,
        metadata: Optional[Dict[str, Any]] = None,
    ) -> SubmitUsageResponse:
        """
        Submit usage data asynchronously.

        Args:
            organization_id: Organization identifier
            provider: LLM provider (openai, anthropic, etc.)
            model_id: Model identifier (e.g., gpt-4, claude-3-sonnet)
            input_tokens: Number of input/prompt tokens
            output_tokens: Number of output/completion tokens
            total_tokens: Total number of tokens
            timestamp: Usage timestamp (defaults to current time)
            metadata: Additional metadata

        Returns:
            Submit usage response with usage ID and estimated cost

        Raises:
            ValidationError: If request data is invalid
            APIError: If the API returns an error

        Example:
            >>> async with AsyncCostOpsClient(api_key="...") as client:
            ...     usage = await client.usage.submit(
            ...         organization_id="org-123",
            ...         provider="openai",
            ...         model_id="gpt-4",
            ...         input_tokens=1000,
            ...         output_tokens=500,
            ...         total_tokens=1500
            ...     )
        """
        request = SubmitUsageRequest(
            organization_id=organization_id,
            provider=Provider(provider),
            model_id=model_id,
            input_tokens=input_tokens,
            output_tokens=output_tokens,
            total_tokens=total_tokens,
            timestamp=timestamp,
            metadata=metadata,
        )

        response = await self._client.request(
            "POST",
            "/api/v1/usage",
            json=request.model_dump(mode="json", exclude_none=True),
        )

        # Extract data from response wrapper
        data = response.get("data", response)
        return SubmitUsageResponse.model_validate(data)

    async def get_history(
        self,
        organization_id: Optional[str] = None,
        provider: Optional[str] = None,
        model_id: Optional[str] = None,
        start_date: Optional[datetime] = None,
        end_date: Optional[datetime] = None,
        page: int = 1,
        per_page: int = 20,
    ) -> List[Dict[str, Any]]:
        """
        Get usage history asynchronously.

        Args:
            organization_id: Filter by organization
            provider: Filter by provider
            model_id: Filter by model
            start_date: Start date for filtering
            end_date: End date for filtering
            page: Page number (1-indexed)
            per_page: Items per page

        Returns:
            List of usage records

        Raises:
            APIError: If the API returns an error
        """
        params: Dict[str, Any] = {
            "page": page,
            "per_page": per_page,
        }

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

        response = await self._client.request("GET", "/api/v1/usage/history", params=params)

        # Extract data from paginated response
        return response.get("data", [])
