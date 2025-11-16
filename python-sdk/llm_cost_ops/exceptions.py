"""
Custom exceptions for LLM-CostOps SDK.

This module defines the exception hierarchy for the SDK, providing
specific error types for different failure scenarios.
"""

from typing import Any, Dict, Optional


class CostOpsError(Exception):
    """Base exception for all LLM-CostOps SDK errors."""

    def __init__(self, message: str, **kwargs: Any) -> None:
        """
        Initialize the exception.

        Args:
            message: Error message
            **kwargs: Additional error context
        """
        super().__init__(message)
        self.message = message
        self.context = kwargs

    def __str__(self) -> str:
        """Return string representation of the error."""
        if self.context:
            context_str = ", ".join(f"{k}={v}" for k, v in self.context.items())
            return f"{self.message} ({context_str})"
        return self.message


class APIError(CostOpsError):
    """Exception raised for API-level errors."""

    def __init__(
        self,
        message: str,
        status_code: Optional[int] = None,
        response: Optional[Dict[str, Any]] = None,
        request_id: Optional[str] = None,
    ) -> None:
        """
        Initialize API error.

        Args:
            message: Error message
            status_code: HTTP status code
            response: Response body
            request_id: Request ID for tracking
        """
        super().__init__(
            message,
            status_code=status_code,
            response=response,
            request_id=request_id,
        )
        self.status_code = status_code
        self.response = response
        self.request_id = request_id


class AuthenticationError(APIError):
    """Exception raised for authentication failures."""

    def __init__(self, message: str = "Authentication failed", **kwargs: Any) -> None:
        """
        Initialize authentication error.

        Args:
            message: Error message
            **kwargs: Additional error context
        """
        super().__init__(message, status_code=401, **kwargs)


class AuthorizationError(APIError):
    """Exception raised for authorization failures."""

    def __init__(self, message: str = "Authorization failed", **kwargs: Any) -> None:
        """
        Initialize authorization error.

        Args:
            message: Error message
            **kwargs: Additional error context
        """
        super().__init__(message, status_code=403, **kwargs)


class ValidationError(APIError):
    """Exception raised for request validation errors."""

    def __init__(
        self, message: str, errors: Optional[Dict[str, Any]] = None, **kwargs: Any
    ) -> None:
        """
        Initialize validation error.

        Args:
            message: Error message
            errors: Validation error details
            **kwargs: Additional error context
        """
        super().__init__(message, status_code=400, **kwargs)
        self.errors = errors or {}


class NotFoundError(APIError):
    """Exception raised when a resource is not found."""

    def __init__(
        self, message: str = "Resource not found", resource_type: Optional[str] = None, **kwargs: Any
    ) -> None:
        """
        Initialize not found error.

        Args:
            message: Error message
            resource_type: Type of resource that was not found
            **kwargs: Additional error context
        """
        super().__init__(message, status_code=404, **kwargs)
        self.resource_type = resource_type


class RateLimitError(APIError):
    """Exception raised when rate limit is exceeded."""

    def __init__(
        self,
        message: str = "Rate limit exceeded",
        retry_after: Optional[float] = None,
        limit: Optional[int] = None,
        **kwargs: Any,
    ) -> None:
        """
        Initialize rate limit error.

        Args:
            message: Error message
            retry_after: Seconds to wait before retrying
            limit: Rate limit threshold
            **kwargs: Additional error context
        """
        super().__init__(message, status_code=429, **kwargs)
        self.retry_after = retry_after
        self.limit = limit


class ServerError(APIError):
    """Exception raised for server-side errors (5xx)."""

    def __init__(self, message: str = "Server error occurred", **kwargs: Any) -> None:
        """
        Initialize server error.

        Args:
            message: Error message
            **kwargs: Additional error context
        """
        super().__init__(message, status_code=kwargs.get("status_code", 500), **kwargs)


class NetworkError(CostOpsError):
    """Exception raised for network-related errors."""

    def __init__(self, message: str, cause: Optional[Exception] = None) -> None:
        """
        Initialize network error.

        Args:
            message: Error message
            cause: Original exception that caused the error
        """
        super().__init__(message)
        self.cause = cause
        if cause:
            self.__cause__ = cause


class TimeoutError(NetworkError):
    """Exception raised when a request times out."""

    def __init__(self, message: str = "Request timed out", timeout: Optional[float] = None) -> None:
        """
        Initialize timeout error.

        Args:
            message: Error message
            timeout: Timeout value in seconds
        """
        super().__init__(message)
        self.timeout = timeout


class ConfigurationError(CostOpsError):
    """Exception raised for configuration errors."""

    def __init__(self, message: str, parameter: Optional[str] = None) -> None:
        """
        Initialize configuration error.

        Args:
            message: Error message
            parameter: Name of the misconfigured parameter
        """
        super().__init__(message, parameter=parameter)
        self.parameter = parameter


def create_error_from_response(
    status_code: int, response_body: Optional[Dict[str, Any]] = None, request_id: Optional[str] = None
) -> APIError:
    """
    Create appropriate error from HTTP response.

    Args:
        status_code: HTTP status code
        response_body: Response body
        request_id: Request ID

    Returns:
        Appropriate APIError subclass
    """
    message = "An error occurred"
    if response_body:
        message = response_body.get("message", response_body.get("error", message))

    error_map = {
        400: ValidationError,
        401: AuthenticationError,
        403: AuthorizationError,
        404: NotFoundError,
        429: RateLimitError,
    }

    if status_code >= 500:
        return ServerError(message, status_code=status_code, response=response_body, request_id=request_id)

    error_class = error_map.get(status_code, APIError)

    if error_class == RateLimitError and response_body:
        retry_after = response_body.get("retry_after")
        limit = response_body.get("limit")
        return RateLimitError(
            message, retry_after=retry_after, limit=limit, response=response_body, request_id=request_id
        )

    if error_class == ValidationError and response_body:
        errors = response_body.get("errors")
        return ValidationError(message, errors=errors, response=response_body, request_id=request_id)

    return error_class(message, response=response_body, request_id=request_id)
