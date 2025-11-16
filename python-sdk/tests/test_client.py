"""Tests for client classes."""

import pytest
import respx
import httpx
from datetime import datetime
from decimal import Decimal

from llm_cost_ops import CostOpsClient, AsyncCostOpsClient, ClientConfig
from llm_cost_ops.exceptions import (
    APIError,
    AuthenticationError,
    RateLimitError,
    ValidationError,
)


class TestCostOpsClient:
    """Tests for synchronous client."""

    def test_client_initialization(self, api_key: str) -> None:
        """Test client initialization."""
        client = CostOpsClient(api_key=api_key)
        assert client._config.api_key == api_key
        client.close()

    def test_client_with_config(self, client_config: ClientConfig) -> None:
        """Test client initialization with config object."""
        client = CostOpsClient(config=client_config)
        assert client._config.api_key == client_config.api_key
        assert client._config.base_url == client_config.base_url
        client.close()

    def test_client_context_manager(self, api_key: str) -> None:
        """Test client as context manager."""
        with CostOpsClient(api_key=api_key) as client:
            assert client._http_client is not None

    @respx.mock
    def test_submit_usage(
        self, api_key: str, base_url: str, mock_usage_response: dict
    ) -> None:
        """Test submitting usage data."""
        route = respx.post(f"{base_url}/api/v1/usage").mock(
            return_value=httpx.Response(200, json=mock_usage_response)
        )

        with CostOpsClient(api_key=api_key, base_url=base_url) as client:
            usage = client.usage.submit(
                organization_id="org-123",
                provider="openai",
                model_id="gpt-4",
                input_tokens=1000,
                output_tokens=500,
                total_tokens=1500,
            )

            assert usage.usage_id == "usage-123"
            assert usage.estimated_cost == Decimal("0.015")
            assert route.called

    @respx.mock
    def test_get_costs(
        self, api_key: str, base_url: str, mock_cost_summary_response: dict
    ) -> None:
        """Test getting cost summary."""
        route = respx.get(f"{base_url}/api/v1/costs").mock(
            return_value=httpx.Response(200, json=mock_cost_summary_response)
        )

        with CostOpsClient(api_key=api_key, base_url=base_url) as client:
            costs = client.costs.get(
                organization_id="org-123",
                start_date=datetime(2025, 1, 1),
                end_date=datetime(2025, 1, 31),
            )

            assert costs.total_cost == Decimal("45.50")
            assert costs.total_requests == 100
            assert len(costs.breakdown or []) == 2
            assert route.called

    @respx.mock
    def test_authentication_error(self, base_url: str) -> None:
        """Test authentication error handling."""
        respx.post(f"{base_url}/api/v1/usage").mock(
            return_value=httpx.Response(
                401, json={"message": "Invalid API key"}
            )
        )

        with pytest.raises(AuthenticationError) as exc_info:
            with CostOpsClient(api_key="invalid-key", base_url=base_url) as client:
                client.usage.submit(
                    organization_id="org-123",
                    provider="openai",
                    model_id="gpt-4",
                    input_tokens=1000,
                    output_tokens=500,
                    total_tokens=1500,
                )

        assert exc_info.value.status_code == 401

    def test_validation_error(self, api_key: str, base_url: str) -> None:
        """Test validation error handling."""
        # Test client-side validation (Pydantic)
        from pydantic import ValidationError as PydanticValidationError

        with pytest.raises(PydanticValidationError):
            with CostOpsClient(api_key=api_key, base_url=base_url) as client:
                client.usage.submit(
                    organization_id="",  # Invalid - too short
                    provider="openai",
                    model_id="gpt-4",
                    input_tokens=1000,
                    output_tokens=500,
                    total_tokens=1500,
                )

    @respx.mock
    def test_server_validation_error(self, api_key: str, base_url: str) -> None:
        """Test server-side validation error handling."""
        respx.post(f"{base_url}/api/v1/usage").mock(
            return_value=httpx.Response(
                400,
                json={
                    "message": "Validation failed",
                    "errors": {"organization_id": "Required field"},
                },
            )
        )

        with pytest.raises(ValidationError) as exc_info:
            with CostOpsClient(api_key=api_key, base_url=base_url) as client:
                # This would pass client validation but fail server validation
                # We need to actually make a request to test server validation
                client.usage.submit(
                    organization_id="x",  # Valid length but might fail server checks
                    provider="openai",
                    model_id="gpt-4",
                    input_tokens=1000,
                    output_tokens=500,
                    total_tokens=1500,
                )

        assert exc_info.value.status_code == 400

    @respx.mock
    def test_rate_limit_error(self, api_key: str, base_url: str) -> None:
        """Test rate limit error handling."""
        respx.post(f"{base_url}/api/v1/usage").mock(
            return_value=httpx.Response(
                429,
                json={
                    "message": "Rate limit exceeded",
                    "retry_after": 60.0,
                    "limit": 100,
                },
            )
        )

        with pytest.raises(RateLimitError) as exc_info:
            with CostOpsClient(api_key=api_key, base_url=base_url) as client:
                client.usage.submit(
                    organization_id="org-123",
                    provider="openai",
                    model_id="gpt-4",
                    input_tokens=1000,
                    output_tokens=500,
                    total_tokens=1500,
                )

        assert exc_info.value.status_code == 429
        assert exc_info.value.retry_after == 60.0


class TestAsyncCostOpsClient:
    """Tests for asynchronous client."""

    @pytest.mark.asyncio
    async def test_async_client_initialization(self, api_key: str) -> None:
        """Test async client initialization."""
        client = AsyncCostOpsClient(api_key=api_key)
        assert client._config.api_key == api_key
        await client.close()

    @pytest.mark.asyncio
    async def test_async_client_context_manager(self, api_key: str) -> None:
        """Test async client as context manager."""
        async with AsyncCostOpsClient(api_key=api_key) as client:
            assert client._http_client is not None

    @pytest.mark.asyncio
    @respx.mock
    async def test_async_submit_usage(
        self, api_key: str, base_url: str, mock_usage_response: dict
    ) -> None:
        """Test async usage submission."""
        route = respx.post(f"{base_url}/api/v1/usage").mock(
            return_value=httpx.Response(200, json=mock_usage_response)
        )

        async with AsyncCostOpsClient(api_key=api_key, base_url=base_url) as client:
            usage = await client.usage.submit(
                organization_id="org-123",
                provider="openai",
                model_id="gpt-4",
                input_tokens=1000,
                output_tokens=500,
                total_tokens=1500,
            )

            assert usage.usage_id == "usage-123"
            assert usage.estimated_cost == Decimal("0.015")
            assert route.called

    @pytest.mark.asyncio
    @respx.mock
    async def test_async_get_costs(
        self, api_key: str, base_url: str, mock_cost_summary_response: dict
    ) -> None:
        """Test async cost retrieval."""
        route = respx.get(f"{base_url}/api/v1/costs").mock(
            return_value=httpx.Response(200, json=mock_cost_summary_response)
        )

        async with AsyncCostOpsClient(api_key=api_key, base_url=base_url) as client:
            costs = await client.costs.get(
                organization_id="org-123",
                start_date=datetime(2025, 1, 1),
                end_date=datetime(2025, 1, 31),
            )

            assert costs.total_cost == Decimal("45.50")
            assert costs.total_requests == 100
            assert route.called
