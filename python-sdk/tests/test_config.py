"""Tests for configuration module."""

import pytest
import os

from llm_cost_ops.config import (
    ClientConfig,
    RetryConfig,
    RateLimitConfig,
    ConnectionPoolConfig,
    LoggingConfig,
)
from llm_cost_ops.exceptions import ConfigurationError


class TestRetryConfig:
    """Tests for RetryConfig."""

    def test_default_config(self) -> None:
        """Test default retry configuration."""
        config = RetryConfig()
        assert config.max_retries == 3
        assert config.backoff_factor == 2.0
        assert config.max_backoff == 60.0

    def test_custom_config(self) -> None:
        """Test custom retry configuration."""
        config = RetryConfig(max_retries=5, backoff_factor=1.5)
        assert config.max_retries == 5
        assert config.backoff_factor == 1.5

    def test_invalid_max_retries(self) -> None:
        """Test validation of max_retries."""
        with pytest.raises(ConfigurationError):
            RetryConfig(max_retries=-1)

    def test_invalid_backoff_factor(self) -> None:
        """Test validation of backoff_factor."""
        with pytest.raises(ConfigurationError):
            RetryConfig(backoff_factor=0)


class TestRateLimitConfig:
    """Tests for RateLimitConfig."""

    def test_default_config(self) -> None:
        """Test default rate limit configuration."""
        config = RateLimitConfig()
        assert config.max_requests == 100
        assert config.time_window == 60.0

    def test_custom_config(self) -> None:
        """Test custom rate limit configuration."""
        config = RateLimitConfig(max_requests=50, time_window=30.0)
        assert config.max_requests == 50
        assert config.time_window == 30.0

    def test_invalid_max_requests(self) -> None:
        """Test validation of max_requests."""
        with pytest.raises(ConfigurationError):
            RateLimitConfig(max_requests=0)

    def test_invalid_time_window(self) -> None:
        """Test validation of time_window."""
        with pytest.raises(ConfigurationError):
            RateLimitConfig(time_window=-1)


class TestConnectionPoolConfig:
    """Tests for ConnectionPoolConfig."""

    def test_default_config(self) -> None:
        """Test default connection pool configuration."""
        config = ConnectionPoolConfig()
        assert config.max_connections == 100
        assert config.max_keepalive_connections == 20

    def test_custom_config(self) -> None:
        """Test custom connection pool configuration."""
        config = ConnectionPoolConfig(
            max_connections=50, max_keepalive_connections=10
        )
        assert config.max_connections == 50
        assert config.max_keepalive_connections == 10

    def test_invalid_max_connections(self) -> None:
        """Test validation of max_connections."""
        with pytest.raises(ConfigurationError):
            ConnectionPoolConfig(max_connections=0)

    def test_invalid_keepalive_exceeds_max(self) -> None:
        """Test validation when keepalive exceeds max."""
        with pytest.raises(ConfigurationError):
            ConnectionPoolConfig(
                max_connections=10, max_keepalive_connections=20
            )


class TestLoggingConfig:
    """Tests for LoggingConfig."""

    def test_default_config(self) -> None:
        """Test default logging configuration."""
        config = LoggingConfig()
        assert config.enabled is True
        assert config.level == "INFO"

    def test_custom_config(self) -> None:
        """Test custom logging configuration."""
        config = LoggingConfig(enabled=False, level="DEBUG")
        assert config.enabled is False
        assert config.level == "DEBUG"

    def test_invalid_log_level(self) -> None:
        """Test validation of log level."""
        with pytest.raises(ConfigurationError):
            LoggingConfig(level="INVALID")

    def test_get_logger(self) -> None:
        """Test logger retrieval."""
        config = LoggingConfig(level="DEBUG")
        logger = config.get_logger()
        assert logger.name == "llm_cost_ops"


class TestClientConfig:
    """Tests for ClientConfig."""

    def test_initialization_with_api_key(self) -> None:
        """Test client config initialization with API key."""
        config = ClientConfig(api_key="test-key")
        assert config.api_key == "test-key"
        assert config.base_url == "https://api.llm-cost-ops.dev"

    def test_initialization_with_all_params(self) -> None:
        """Test client config with all parameters."""
        config = ClientConfig(
            api_key="test-key",
            base_url="https://custom.api.com",
            timeout=60.0,
        )
        assert config.api_key == "test-key"
        assert config.base_url == "https://custom.api.com"
        assert config.timeout == 60.0

    def test_missing_api_key(self) -> None:
        """Test error when API key is missing."""
        with pytest.raises(ConfigurationError):
            ClientConfig()

    def test_headers_property(self) -> None:
        """Test headers property."""
        config = ClientConfig(api_key="test-key")
        headers = config.headers
        assert "Authorization" in headers
        assert headers["Authorization"] == "Bearer test-key"
        assert "Content-Type" in headers

    def test_from_env(self, monkeypatch: pytest.MonkeyPatch) -> None:
        """Test configuration from environment variables."""
        monkeypatch.setenv("LLM_COST_OPS_API_KEY", "env-test-key")
        monkeypatch.setenv("LLM_COST_OPS_BASE_URL", "https://env.api.com")
        monkeypatch.setenv("LLM_COST_OPS_TIMEOUT", "45.0")

        config = ClientConfig.from_env()
        assert config.api_key == "env-test-key"
        assert config.base_url == "https://env.api.com"
        assert config.timeout == 45.0

    def test_base_url_trailing_slash_removed(self) -> None:
        """Test that trailing slash is removed from base_url."""
        config = ClientConfig(
            api_key="test-key", base_url="https://api.example.com/"
        )
        assert config.base_url == "https://api.example.com"

    def test_invalid_timeout(self) -> None:
        """Test validation of timeout."""
        with pytest.raises(ConfigurationError):
            ClientConfig(api_key="test-key", timeout=0)
