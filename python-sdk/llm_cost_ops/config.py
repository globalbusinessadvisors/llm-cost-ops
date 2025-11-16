"""
Configuration management for LLM-CostOps SDK.

This module provides configuration classes for customizing SDK behavior,
including authentication, timeouts, retries, and connection pooling.
"""

import logging
import os
from dataclasses import dataclass, field
from typing import Optional

from .exceptions import ConfigurationError


@dataclass
class RateLimitConfig:
    """Rate limiting configuration."""

    max_requests: int = 100
    """Maximum number of requests allowed."""

    time_window: float = 60.0
    """Time window in seconds."""

    def __post_init__(self) -> None:
        """Validate configuration after initialization."""
        if self.max_requests <= 0:
            raise ConfigurationError("max_requests must be positive", parameter="max_requests")
        if self.time_window <= 0:
            raise ConfigurationError("time_window must be positive", parameter="time_window")


@dataclass
class RetryConfig:
    """Retry configuration."""

    max_retries: int = 3
    """Maximum number of retry attempts."""

    backoff_factor: float = 2.0
    """Exponential backoff multiplier."""

    max_backoff: float = 60.0
    """Maximum backoff time in seconds."""

    retry_on_status: tuple = (408, 429, 500, 502, 503, 504)
    """HTTP status codes that trigger retries."""

    def __post_init__(self) -> None:
        """Validate configuration after initialization."""
        if self.max_retries < 0:
            raise ConfigurationError("max_retries cannot be negative", parameter="max_retries")
        if self.backoff_factor <= 0:
            raise ConfigurationError("backoff_factor must be positive", parameter="backoff_factor")
        if self.max_backoff <= 0:
            raise ConfigurationError("max_backoff must be positive", parameter="max_backoff")


@dataclass
class ConnectionPoolConfig:
    """HTTP connection pool configuration."""

    max_connections: int = 100
    """Maximum number of connections in the pool."""

    max_keepalive_connections: int = 20
    """Maximum number of keepalive connections."""

    keepalive_expiry: float = 5.0
    """Keepalive expiry time in seconds."""

    def __post_init__(self) -> None:
        """Validate configuration after initialization."""
        if self.max_connections <= 0:
            raise ConfigurationError(
                "max_connections must be positive", parameter="max_connections"
            )
        if self.max_keepalive_connections < 0:
            raise ConfigurationError(
                "max_keepalive_connections cannot be negative",
                parameter="max_keepalive_connections",
            )
        if self.max_keepalive_connections > self.max_connections:
            raise ConfigurationError(
                "max_keepalive_connections cannot exceed max_connections",
                parameter="max_keepalive_connections",
            )
        if self.keepalive_expiry < 0:
            raise ConfigurationError(
                "keepalive_expiry cannot be negative", parameter="keepalive_expiry"
            )


@dataclass
class LoggingConfig:
    """Logging configuration."""

    enabled: bool = True
    """Enable structured logging."""

    level: str = "INFO"
    """Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)."""

    logger_name: str = "llm_cost_ops"
    """Logger name."""

    format: str = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    """Log format string."""

    def __post_init__(self) -> None:
        """Validate configuration after initialization."""
        valid_levels = {"DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"}
        if self.level.upper() not in valid_levels:
            raise ConfigurationError(
                f"Invalid log level: {self.level}. Must be one of {valid_levels}",
                parameter="level",
            )
        self.level = self.level.upper()

    def get_logger(self) -> logging.Logger:
        """
        Get configured logger instance.

        Returns:
            Configured logger
        """
        logger = logging.getLogger(self.logger_name)
        if self.enabled and not logger.handlers:
            handler = logging.StreamHandler()
            handler.setFormatter(logging.Formatter(self.format))
            logger.addHandler(handler)
            logger.setLevel(getattr(logging, self.level))
        return logger


@dataclass
class MetricsConfig:
    """Metrics and observability configuration."""

    enabled: bool = False
    """Enable metrics collection."""

    prometheus_enabled: bool = False
    """Enable Prometheus metrics."""

    opentelemetry_enabled: bool = False
    """Enable OpenTelemetry tracing."""

    service_name: str = "llm-cost-ops-sdk"
    """Service name for tracing."""


@dataclass
class ClientConfig:
    """
    Main SDK client configuration.

    This class provides comprehensive configuration options for the SDK client,
    including authentication, networking, retries, and observability.
    """

    api_key: Optional[str] = None
    """API key for authentication. Can also be set via LLM_COST_OPS_API_KEY env var."""

    base_url: str = "https://api.llm-cost-ops.dev"
    """Base URL for the API. Can also be set via LLM_COST_OPS_BASE_URL env var."""

    timeout: float = 30.0
    """Request timeout in seconds."""

    retry: RetryConfig = field(default_factory=RetryConfig)
    """Retry configuration."""

    connection_pool: ConnectionPoolConfig = field(default_factory=ConnectionPoolConfig)
    """Connection pool configuration."""

    rate_limit: Optional[RateLimitConfig] = None
    """Rate limiting configuration."""

    logging: LoggingConfig = field(default_factory=LoggingConfig)
    """Logging configuration."""

    metrics: MetricsConfig = field(default_factory=MetricsConfig)
    """Metrics configuration."""

    user_agent: str = "llm-cost-ops-python/0.1.0"
    """User-Agent header value."""

    verify_ssl: bool = True
    """Verify SSL certificates."""

    def __post_init__(self) -> None:
        """Validate and finalize configuration."""
        # Load from environment variables if not set
        if self.api_key is None:
            self.api_key = os.environ.get("LLM_COST_OPS_API_KEY")

        base_url_env = os.environ.get("LLM_COST_OPS_BASE_URL")
        if base_url_env:
            self.base_url = base_url_env

        # Validate required settings
        if not self.api_key:
            raise ConfigurationError(
                "API key is required. Set api_key parameter or LLM_COST_OPS_API_KEY environment variable.",
                parameter="api_key",
            )

        if not self.base_url:
            raise ConfigurationError("base_url cannot be empty", parameter="base_url")

        # Remove trailing slash from base_url
        self.base_url = self.base_url.rstrip("/")

        # Validate timeout
        if self.timeout <= 0:
            raise ConfigurationError("timeout must be positive", parameter="timeout")

        # Validate nested configs
        if isinstance(self.retry, dict):
            self.retry = RetryConfig(**self.retry)

        if isinstance(self.connection_pool, dict):
            self.connection_pool = ConnectionPoolConfig(**self.connection_pool)

        if self.rate_limit is not None and isinstance(self.rate_limit, dict):
            self.rate_limit = RateLimitConfig(**self.rate_limit)

        if isinstance(self.logging, dict):
            self.logging = LoggingConfig(**self.logging)

        if isinstance(self.metrics, dict):
            self.metrics = MetricsConfig(**self.metrics)

    @property
    def headers(self) -> dict:
        """
        Get default HTTP headers.

        Returns:
            Dictionary of default headers
        """
        return {
            "User-Agent": self.user_agent,
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

    @classmethod
    def from_env(cls) -> "ClientConfig":
        """
        Create configuration from environment variables.

        Returns:
            ClientConfig instance

        Raises:
            ConfigurationError: If required environment variables are missing
        """
        return cls(
            api_key=os.environ.get("LLM_COST_OPS_API_KEY"),
            base_url=os.environ.get(
                "LLM_COST_OPS_BASE_URL", "https://api.llm-cost-ops.dev"
            ),
            timeout=float(os.environ.get("LLM_COST_OPS_TIMEOUT", "30.0")),
        )
