"""
HTTP client implementation with retry logic and connection pooling.

This module provides both synchronous and asynchronous HTTP clients
with enterprise-grade features like automatic retries, rate limiting,
and comprehensive error handling.
"""

import time
from typing import Any, Dict, Optional
from urllib.parse import urljoin

import httpx
from tenacity import (
    retry,
    retry_if_exception_type,
    stop_after_attempt,
    wait_exponential,
)

from ..config import ClientConfig, RateLimitConfig
from ..exceptions import (
    APIError,
    NetworkError,
    create_error_from_response,
)
from ..exceptions import (
    TimeoutError as SDKTimeoutError,
)


class RateLimiter:
    """Simple rate limiter using token bucket algorithm."""

    def __init__(self, config: RateLimitConfig) -> None:
        """
        Initialize rate limiter.

        Args:
            config: Rate limit configuration
        """
        self.max_requests = config.max_requests
        self.time_window = config.time_window
        self.tokens = float(config.max_requests)
        self.last_update = time.time()
        self._lock = False

    def acquire(self) -> None:
        """
        Acquire a token, blocking if necessary.

        This method will wait if no tokens are available.
        """
        while True:
            self._refill()
            if self.tokens >= 1.0:
                self.tokens -= 1.0
                return
            time.sleep(0.1)

    def _refill(self) -> None:
        """Refill tokens based on elapsed time."""
        now = time.time()
        elapsed = now - self.last_update
        self.tokens = min(
            self.max_requests,
            self.tokens + (elapsed * self.max_requests / self.time_window),
        )
        self.last_update = now


class BaseHTTPClient:
    """Base HTTP client with common functionality."""

    def __init__(self, config: ClientConfig) -> None:
        """
        Initialize HTTP client.

        Args:
            config: Client configuration
        """
        self.config = config
        self.logger = config.logging.get_logger()
        self.rate_limiter: Optional[RateLimiter] = None

        if config.rate_limit:
            self.rate_limiter = RateLimiter(config.rate_limit)

    def _get_url(self, path: str) -> str:
        """
        Construct full URL from path.

        Args:
            path: API path

        Returns:
            Full URL
        """
        if not path.startswith("/"):
            path = "/" + path
        return urljoin(self.config.base_url, path)

    def _log_request(
        self, method: str, url: str, request_id: Optional[str] = None
    ) -> None:
        """
        Log HTTP request.

        Args:
            method: HTTP method
            url: Request URL
            request_id: Request ID for tracking
        """
        extra = {"method": method, "url": url}
        if request_id:
            extra["request_id"] = request_id
        self.logger.debug(f"HTTP {method} {url}", extra=extra)

    def _log_response(
        self, status_code: int, url: str, duration: float, request_id: Optional[str] = None
    ) -> None:
        """
        Log HTTP response.

        Args:
            status_code: HTTP status code
            url: Request URL
            duration: Request duration in seconds
            request_id: Request ID for tracking
        """
        extra = {
            "status_code": status_code,
            "url": url,
            "duration_ms": round(duration * 1000, 2),
        }
        if request_id:
            extra["request_id"] = request_id

        if 200 <= status_code < 300:
            self.logger.debug(f"HTTP {status_code} {url}", extra=extra)
        elif 400 <= status_code < 500:
            self.logger.warning(f"HTTP {status_code} {url}", extra=extra)
        else:
            self.logger.error(f"HTTP {status_code} {url}", extra=extra)

    def _should_retry(self, exception: Exception) -> bool:
        """
        Determine if request should be retried.

        Args:
            exception: Exception that occurred

        Returns:
            True if request should be retried
        """
        if isinstance(exception, APIError):
            return (
                exception.status_code in self.config.retry.retry_on_status
            )
        if isinstance(exception, (NetworkError, httpx.NetworkError, httpx.TimeoutException)):
            return True
        return False


class SyncHTTPClient(BaseHTTPClient):
    """Synchronous HTTP client with retry logic."""

    def __init__(self, config: ClientConfig) -> None:
        """
        Initialize synchronous HTTP client.

        Args:
            config: Client configuration
        """
        super().__init__(config)

        limits = httpx.Limits(
            max_connections=config.connection_pool.max_connections,
            max_keepalive_connections=config.connection_pool.max_keepalive_connections,
            keepalive_expiry=config.connection_pool.keepalive_expiry,
        )

        self.client = httpx.Client(
            headers=config.headers,
            timeout=httpx.Timeout(config.timeout),
            limits=limits,
            verify=config.verify_ssl,
        )

    def request(
        self,
        method: str,
        path: str,
        *,
        json: Optional[Dict[str, Any]] = None,
        params: Optional[Dict[str, Any]] = None,
        headers: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        """
        Make HTTP request with retry logic.

        Args:
            method: HTTP method
            path: API path
            json: JSON request body
            params: Query parameters
            headers: Additional headers

        Returns:
            Response data

        Raises:
            APIError: For API errors
            NetworkError: For network errors
            TimeoutError: For timeout errors
        """
        if self.rate_limiter:
            self.rate_limiter.acquire()

        url = self._get_url(path)
        request_id = None

        @retry(
            stop=stop_after_attempt(self.config.retry.max_retries + 1),
            wait=wait_exponential(
                multiplier=self.config.retry.backoff_factor,
                max=self.config.retry.max_backoff,
            ),
            retry=retry_if_exception_type(
                (NetworkError, httpx.NetworkError, httpx.TimeoutException)
            ),
            reraise=True,
        )
        def _make_request() -> httpx.Response:
            self._log_request(method, url, request_id)
            start_time = time.time()

            try:
                response = self.client.request(
                    method=method,
                    url=url,
                    json=json,
                    params=params,
                    headers=headers,
                )
                duration = time.time() - start_time
                self._log_response(response.status_code, url, duration, request_id)
                return response

            except httpx.TimeoutException as e:
                raise SDKTimeoutError(
                    f"Request to {url} timed out after {self.config.timeout}s",
                    timeout=self.config.timeout,
                ) from e
            except httpx.NetworkError as e:
                raise NetworkError(f"Network error while requesting {url}: {e}", cause=e) from e

        try:
            response = _make_request()

            # Extract request ID from response headers
            request_id = response.headers.get("X-Request-Id")

            # Check for error status codes
            if response.status_code >= 400:
                response_body = None
                try:
                    response_body = response.json()
                except Exception:
                    pass

                error = create_error_from_response(
                    response.status_code, response_body, request_id
                )

                # Retry if configured to do so
                if self._should_retry(error):
                    raise error

                raise error

            # Parse successful response
            try:
                return response.json()
            except Exception as e:
                raise APIError(
                    f"Failed to parse response from {url}: {e}",
                    status_code=response.status_code,
                    request_id=request_id,
                ) from e

        except (NetworkError, SDKTimeoutError):
            raise
        except APIError:
            raise
        except Exception as e:
            raise NetworkError(f"Unexpected error during request to {url}: {e}", cause=e) from e

    def close(self) -> None:
        """Close the HTTP client and release resources."""
        self.client.close()

    def __enter__(self) -> "SyncHTTPClient":
        """Context manager entry."""
        return self

    def __exit__(self, *args: Any) -> None:
        """Context manager exit."""
        self.close()


class AsyncHTTPClient(BaseHTTPClient):
    """Asynchronous HTTP client with retry logic."""

    def __init__(self, config: ClientConfig) -> None:
        """
        Initialize asynchronous HTTP client.

        Args:
            config: Client configuration
        """
        super().__init__(config)

        limits = httpx.Limits(
            max_connections=config.connection_pool.max_connections,
            max_keepalive_connections=config.connection_pool.max_keepalive_connections,
            keepalive_expiry=config.connection_pool.keepalive_expiry,
        )

        self.client = httpx.AsyncClient(
            headers=config.headers,
            timeout=httpx.Timeout(config.timeout),
            limits=limits,
            verify=config.verify_ssl,
        )

    async def request(
        self,
        method: str,
        path: str,
        *,
        json: Optional[Dict[str, Any]] = None,
        params: Optional[Dict[str, Any]] = None,
        headers: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        """
        Make async HTTP request with retry logic.

        Args:
            method: HTTP method
            path: API path
            json: JSON request body
            params: Query parameters
            headers: Additional headers

        Returns:
            Response data

        Raises:
            APIError: For API errors
            NetworkError: For network errors
            TimeoutError: For timeout errors
        """
        if self.rate_limiter:
            self.rate_limiter.acquire()

        url = self._get_url(path)
        request_id = None

        @retry(
            stop=stop_after_attempt(self.config.retry.max_retries + 1),
            wait=wait_exponential(
                multiplier=self.config.retry.backoff_factor,
                max=self.config.retry.max_backoff,
            ),
            retry=retry_if_exception_type(
                (NetworkError, httpx.NetworkError, httpx.TimeoutException)
            ),
            reraise=True,
        )
        async def _make_request() -> httpx.Response:
            self._log_request(method, url, request_id)
            start_time = time.time()

            try:
                response = await self.client.request(
                    method=method,
                    url=url,
                    json=json,
                    params=params,
                    headers=headers,
                )
                duration = time.time() - start_time
                self._log_response(response.status_code, url, duration, request_id)
                return response

            except httpx.TimeoutException as e:
                raise SDKTimeoutError(
                    f"Request to {url} timed out after {self.config.timeout}s",
                    timeout=self.config.timeout,
                ) from e
            except httpx.NetworkError as e:
                raise NetworkError(f"Network error while requesting {url}: {e}", cause=e) from e

        try:
            response = await _make_request()

            # Extract request ID from response headers
            request_id = response.headers.get("X-Request-Id")

            # Check for error status codes
            if response.status_code >= 400:
                response_body = None
                try:
                    response_body = response.json()
                except Exception:
                    pass

                error = create_error_from_response(
                    response.status_code, response_body, request_id
                )

                # Retry if configured to do so
                if self._should_retry(error):
                    raise error

                raise error

            # Parse successful response
            try:
                return response.json()
            except Exception as e:
                raise APIError(
                    f"Failed to parse response from {url}: {e}",
                    status_code=response.status_code,
                    request_id=request_id,
                ) from e

        except (NetworkError, SDKTimeoutError):
            raise
        except APIError:
            raise
        except Exception as e:
            raise NetworkError(f"Unexpected error during request to {url}: {e}", cause=e) from e

    async def close(self) -> None:
        """Close the HTTP client and release resources."""
        await self.client.aclose()

    async def __aenter__(self) -> "AsyncHTTPClient":
        """Async context manager entry."""
        return self

    async def __aexit__(self, *args: Any) -> None:
        """Async context manager exit."""
        await self.close()
