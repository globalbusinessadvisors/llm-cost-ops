"""
Main client classes for LLM-CostOps SDK.

This module provides the primary entry points for interacting with the API,
including both synchronous and asynchronous clients.
"""

from typing import Any, Optional

from ._internal.http_client import AsyncHTTPClient, SyncHTTPClient
from .config import ClientConfig
from .resources import (
    AnalyticsResource,
    AsyncAnalyticsResource,
    AsyncCostsResource,
    AsyncPricingResource,
    AsyncUsageResource,
    CostsResource,
    PricingResource,
    UsageResource,
)


class CostOpsClient:
    """
    Synchronous client for LLM-CostOps API.

    This is the main entry point for making synchronous API calls.
    It provides access to all API resources through convenient properties.

    Example:
        >>> from llm_cost_ops import CostOpsClient
        >>>
        >>> client = CostOpsClient(api_key="your-api-key")
        >>>
        >>> # Submit usage
        >>> usage = client.usage.submit(
        ...     organization_id="org-123",
        ...     provider="openai",
        ...     model_id="gpt-4",
        ...     input_tokens=1000,
        ...     output_tokens=500,
        ...     total_tokens=1500
        ... )
        >>>
        >>> # Get costs
        >>> costs = client.costs.get(
        ...     organization_id="org-123",
        ...     start_date=datetime(2025, 1, 1),
        ...     end_date=datetime(2025, 1, 31)
        ... )
        >>>
        >>> client.close()

    Example (with context manager):
        >>> with CostOpsClient(api_key="your-api-key") as client:
        ...     usage = client.usage.submit(...)
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        config: Optional[ClientConfig] = None,
        **kwargs: Any,
    ) -> None:
        """
        Initialize the synchronous client.

        Args:
            api_key: API key for authentication (or use LLM_COST_OPS_API_KEY env var)
            base_url: Base URL for the API (or use LLM_COST_OPS_BASE_URL env var)
            config: Complete client configuration object
            **kwargs: Additional configuration options

        Raises:
            ConfigurationError: If configuration is invalid

        Example:
            >>> # Simple initialization
            >>> client = CostOpsClient(api_key="your-api-key")
            >>>
            >>> # With custom configuration
            >>> from llm_cost_ops import ClientConfig, RateLimitConfig
            >>> config = ClientConfig(
            ...     api_key="your-api-key",
            ...     timeout=60.0,
            ...     rate_limit=RateLimitConfig(max_requests=50, time_window=60.0)
            ... )
            >>> client = CostOpsClient(config=config)
        """
        # Build configuration
        if config is None:
            config_kwargs: dict = {}
            if api_key:
                config_kwargs["api_key"] = api_key
            if base_url:
                config_kwargs["base_url"] = base_url
            config_kwargs.update(kwargs)
            config = ClientConfig(**config_kwargs)  # type: ignore[arg-type]

        self._config = config
        self._http_client = SyncHTTPClient(config)

        # Initialize resource properties
        self._usage: Optional[UsageResource] = None
        self._costs: Optional[CostsResource] = None
        self._pricing: Optional[PricingResource] = None
        self._analytics: Optional[AnalyticsResource] = None

    @property
    def usage(self) -> UsageResource:
        """
        Access usage resource.

        Returns:
            UsageResource instance for usage operations
        """
        if self._usage is None:
            self._usage = UsageResource(self._http_client)
        return self._usage

    @property
    def costs(self) -> CostsResource:
        """
        Access costs resource.

        Returns:
            CostsResource instance for cost operations
        """
        if self._costs is None:
            self._costs = CostsResource(self._http_client)
        return self._costs

    @property
    def pricing(self) -> PricingResource:
        """
        Access pricing resource.

        Returns:
            PricingResource instance for pricing operations
        """
        if self._pricing is None:
            self._pricing = PricingResource(self._http_client)
        return self._pricing

    @property
    def analytics(self) -> AnalyticsResource:
        """
        Access analytics resource.

        Returns:
            AnalyticsResource instance for analytics operations
        """
        if self._analytics is None:
            self._analytics = AnalyticsResource(self._http_client)
        return self._analytics

    def close(self) -> None:
        """
        Close the client and release resources.

        This should be called when you're done with the client to properly
        clean up HTTP connections.
        """
        self._http_client.close()

    def __enter__(self) -> "CostOpsClient":
        """Context manager entry."""
        return self

    def __exit__(self, *args: Any) -> None:
        """Context manager exit."""
        self.close()


class AsyncCostOpsClient:
    """
    Asynchronous client for LLM-CostOps API.

    This is the main entry point for making asynchronous API calls.
    It provides access to all API resources through convenient properties.

    Example:
        >>> import asyncio
        >>> from llm_cost_ops import AsyncCostOpsClient
        >>>
        >>> async def main():
        ...     async with AsyncCostOpsClient(api_key="your-api-key") as client:
        ...         # Submit usage
        ...         usage = await client.usage.submit(
        ...             organization_id="org-123",
        ...             provider="openai",
        ...             model_id="gpt-4",
        ...             input_tokens=1000,
        ...             output_tokens=500,
        ...             total_tokens=1500
        ...         )
        ...
        ...         # Get costs
        ...         costs = await client.costs.get(
        ...             organization_id="org-123",
        ...             start_date=datetime(2025, 1, 1),
        ...             end_date=datetime(2025, 1, 31)
        ...         )
        >>>
        >>> asyncio.run(main())
    """

    def __init__(
        self,
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        config: Optional[ClientConfig] = None,
        **kwargs: Any,
    ) -> None:
        """
        Initialize the asynchronous client.

        Args:
            api_key: API key for authentication (or use LLM_COST_OPS_API_KEY env var)
            base_url: Base URL for the API (or use LLM_COST_OPS_BASE_URL env var)
            config: Complete client configuration object
            **kwargs: Additional configuration options

        Raises:
            ConfigurationError: If configuration is invalid

        Example:
            >>> # Simple initialization
            >>> client = AsyncCostOpsClient(api_key="your-api-key")
            >>>
            >>> # With custom configuration
            >>> config = ClientConfig(
            ...     api_key="your-api-key",
            ...     timeout=60.0
            ... )
            >>> client = AsyncCostOpsClient(config=config)
        """
        # Build configuration
        if config is None:
            config_kwargs: dict = {}
            if api_key:
                config_kwargs["api_key"] = api_key
            if base_url:
                config_kwargs["base_url"] = base_url
            config_kwargs.update(kwargs)
            config = ClientConfig(**config_kwargs)  # type: ignore[arg-type]

        self._config = config
        self._http_client = AsyncHTTPClient(config)

        # Initialize resource properties
        self._usage: Optional[AsyncUsageResource] = None
        self._costs: Optional[AsyncCostsResource] = None
        self._pricing: Optional[AsyncPricingResource] = None
        self._analytics: Optional[AsyncAnalyticsResource] = None

    @property
    def usage(self) -> AsyncUsageResource:
        """
        Access usage resource.

        Returns:
            AsyncUsageResource instance for usage operations
        """
        if self._usage is None:
            self._usage = AsyncUsageResource(self._http_client)
        return self._usage

    @property
    def costs(self) -> AsyncCostsResource:
        """
        Access costs resource.

        Returns:
            AsyncCostsResource instance for cost operations
        """
        if self._costs is None:
            self._costs = AsyncCostsResource(self._http_client)
        return self._costs

    @property
    def pricing(self) -> AsyncPricingResource:
        """
        Access pricing resource.

        Returns:
            AsyncPricingResource instance for pricing operations
        """
        if self._pricing is None:
            self._pricing = AsyncPricingResource(self._http_client)
        return self._pricing

    @property
    def analytics(self) -> AsyncAnalyticsResource:
        """
        Access analytics resource.

        Returns:
            AsyncAnalyticsResource instance for analytics operations
        """
        if self._analytics is None:
            self._analytics = AsyncAnalyticsResource(self._http_client)
        return self._analytics

    async def close(self) -> None:
        """
        Close the client and release resources.

        This should be called when you're done with the client to properly
        clean up HTTP connections.
        """
        await self._http_client.close()

    async def __aenter__(self) -> "AsyncCostOpsClient":
        """Async context manager entry."""
        return self

    async def __aexit__(self, *args: Any) -> None:
        """Async context manager exit."""
        await self.close()
