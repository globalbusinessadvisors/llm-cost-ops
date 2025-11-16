"""
Pricing resource for managing pricing tables.

This module provides methods for creating and querying pricing information.
"""

from datetime import datetime
from decimal import Decimal
from typing import Any, Dict, List, Optional

from .._internal.http_client import AsyncHTTPClient, SyncHTTPClient
from ..types import CreatePricingRequest, Currency, PricingResponse, Provider


class PricingResource:
    """Synchronous pricing resource."""

    def __init__(self, client: SyncHTTPClient) -> None:
        """
        Initialize pricing resource.

        Args:
            client: HTTP client instance
        """
        self._client = client

    def create(
        self,
        provider: str,
        model_id: str,
        input_price_per_1k: Decimal,
        output_price_per_1k: Decimal,
        currency: str = "USD",
        effective_date: Optional[datetime] = None,
    ) -> PricingResponse:
        """
        Create pricing entry.

        Args:
            provider: LLM provider
            model_id: Model identifier
            input_price_per_1k: Input price per 1000 tokens
            output_price_per_1k: Output price per 1000 tokens
            currency: Currency code (default: USD)
            effective_date: When pricing becomes effective

        Returns:
            Created pricing entry

        Raises:
            ValidationError: If request data is invalid
            APIError: If the API returns an error

        Example:
            >>> pricing = client.pricing.create(
            ...     provider="openai",
            ...     model_id="gpt-4",
            ...     input_price_per_1k=Decimal("0.01"),
            ...     output_price_per_1k=Decimal("0.03")
            ... )
        """
        request = CreatePricingRequest(
            provider=Provider(provider),
            model_id=model_id,
            input_price_per_1k=input_price_per_1k,
            output_price_per_1k=output_price_per_1k,
            currency=Currency(currency),
            effective_date=effective_date,
        )

        response = self._client.request(
            "POST",
            "/api/v1/pricing",
            json=request.model_dump(mode="json", exclude_none=True),
        )

        data = response.get("data", response)
        return PricingResponse.model_validate(data)

    def list(
        self,
        provider: Optional[str] = None,
        model_id: Optional[str] = None,
        page: int = 1,
        per_page: int = 20,
    ) -> List[PricingResponse]:
        """
        List pricing entries.

        Args:
            provider: Filter by provider
            model_id: Filter by model
            page: Page number (1-indexed)
            per_page: Items per page

        Returns:
            List of pricing entries

        Raises:
            APIError: If the API returns an error

        Example:
            >>> pricing_list = client.pricing.list(provider="openai")
        """
        params: Dict[str, Any] = {"page": page, "per_page": per_page}

        if provider:
            params["provider"] = provider
        if model_id:
            params["model_id"] = model_id

        response = self._client.request("GET", "/api/v1/pricing", params=params)

        data = response.get("data", [])
        return [PricingResponse.model_validate(item) for item in data]

    def get(self, pricing_id: str) -> PricingResponse:
        """
        Get pricing entry by ID.

        Args:
            pricing_id: Pricing entry ID

        Returns:
            Pricing entry

        Raises:
            NotFoundError: If pricing entry not found
            APIError: If the API returns an error

        Example:
            >>> pricing = client.pricing.get("price_123")
        """
        response = self._client.request("GET", f"/api/v1/pricing/{pricing_id}")

        data = response.get("data", response)
        return PricingResponse.model_validate(data)


class AsyncPricingResource:
    """Asynchronous pricing resource."""

    def __init__(self, client: AsyncHTTPClient) -> None:
        """
        Initialize async pricing resource.

        Args:
            client: Async HTTP client instance
        """
        self._client = client

    async def create(
        self,
        provider: str,
        model_id: str,
        input_price_per_1k: Decimal,
        output_price_per_1k: Decimal,
        currency: str = "USD",
        effective_date: Optional[datetime] = None,
    ) -> PricingResponse:
        """
        Create pricing entry asynchronously.

        Args:
            provider: LLM provider
            model_id: Model identifier
            input_price_per_1k: Input price per 1000 tokens
            output_price_per_1k: Output price per 1000 tokens
            currency: Currency code (default: USD)
            effective_date: When pricing becomes effective

        Returns:
            Created pricing entry

        Raises:
            ValidationError: If request data is invalid
            APIError: If the API returns an error
        """
        request = CreatePricingRequest(
            provider=Provider(provider),
            model_id=model_id,
            input_price_per_1k=input_price_per_1k,
            output_price_per_1k=output_price_per_1k,
            currency=Currency(currency),
            effective_date=effective_date,
        )

        response = await self._client.request(
            "POST",
            "/api/v1/pricing",
            json=request.model_dump(mode="json", exclude_none=True),
        )

        data = response.get("data", response)
        return PricingResponse.model_validate(data)

    async def list(
        self,
        provider: Optional[str] = None,
        model_id: Optional[str] = None,
        page: int = 1,
        per_page: int = 20,
    ) -> List[PricingResponse]:
        """
        List pricing entries asynchronously.

        Args:
            provider: Filter by provider
            model_id: Filter by model
            page: Page number (1-indexed)
            per_page: Items per page

        Returns:
            List of pricing entries

        Raises:
            APIError: If the API returns an error
        """
        params: Dict[str, Any] = {"page": page, "per_page": per_page}

        if provider:
            params["provider"] = provider
        if model_id:
            params["model_id"] = model_id

        response = await self._client.request("GET", "/api/v1/pricing", params=params)

        data = response.get("data", [])
        return [PricingResponse.model_validate(item) for item in data]

    async def get(self, pricing_id: str) -> PricingResponse:
        """
        Get pricing entry by ID asynchronously.

        Args:
            pricing_id: Pricing entry ID

        Returns:
            Pricing entry

        Raises:
            NotFoundError: If pricing entry not found
            APIError: If the API returns an error
        """
        response = await self._client.request("GET", f"/api/v1/pricing/{pricing_id}")

        data = response.get("data", response)
        return PricingResponse.model_validate(data)
