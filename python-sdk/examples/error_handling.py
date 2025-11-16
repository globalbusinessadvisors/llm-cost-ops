"""
Error handling example for LLM-CostOps SDK.

This example demonstrates proper error handling for various scenarios.
"""

from llm_cost_ops import CostOpsClient
from llm_cost_ops.exceptions import (
    CostOpsError,
    APIError,
    AuthenticationError,
    AuthorizationError,
    ValidationError,
    NotFoundError,
    RateLimitError,
    ServerError,
    NetworkError,
    TimeoutError,
    ConfigurationError,
)
import time


def handle_specific_errors() -> None:
    """Demonstrate handling specific error types."""
    print("=== Handling Specific Errors ===")

    try:
        # This will fail with invalid API key
        client = CostOpsClient(api_key="invalid-key")
        client.usage.submit(
            organization_id="org-123",
            provider="openai",
            model_id="gpt-4",
            input_tokens=1000,
            output_tokens=500,
            total_tokens=1500,
        )
    except AuthenticationError as e:
        print(f"Authentication failed: {e.message}")
        print(f"Status code: {e.status_code}")
        print(f"Request ID: {e.request_id}")
    except ValidationError as e:
        print(f"Validation error: {e.message}")
        print(f"Errors: {e.errors}")
    except RateLimitError as e:
        print(f"Rate limit exceeded: {e.message}")
        print(f"Retry after: {e.retry_after} seconds")
        print(f"Limit: {e.limit} requests")
        # Wait and retry
        if e.retry_after:
            print(f"Waiting {e.retry_after} seconds before retry...")
            time.sleep(e.retry_after)
    except NotFoundError as e:
        print(f"Resource not found: {e.message}")
        print(f"Resource type: {e.resource_type}")
    except ServerError as e:
        print(f"Server error: {e.message}")
        print(f"Status code: {e.status_code}")
        # Implement exponential backoff for server errors
    except NetworkError as e:
        print(f"Network error: {e.message}")
        print(f"Cause: {e.cause}")
        # Check network connection and retry
    except TimeoutError as e:
        print(f"Request timed out: {e.message}")
        print(f"Timeout: {e.timeout} seconds")
        # Increase timeout or retry
    except APIError as e:
        # Catch-all for other API errors
        print(f"API error: {e.message}")
        print(f"Status code: {e.status_code}")
    except ConfigurationError as e:
        print(f"Configuration error: {e.message}")
        print(f"Parameter: {e.parameter}")
    except CostOpsError as e:
        # Base exception - catch-all
        print(f"General SDK error: {e.message}")
    finally:
        print()


def handle_with_retry() -> None:
    """Demonstrate error handling with retry logic."""
    print("=== Error Handling with Retry ===")

    max_retries = 3
    retry_delay = 1.0

    for attempt in range(max_retries):
        try:
            client = CostOpsClient(api_key="your-api-key-here")
            usage = client.usage.submit(
                organization_id="org-123",
                provider="openai",
                model_id="gpt-4",
                input_tokens=1000,
                output_tokens=500,
                total_tokens=1500,
            )
            print(f"Success! Usage ID: {usage.usage_id}")
            client.close()
            break

        except (ServerError, NetworkError, TimeoutError) as e:
            print(f"Attempt {attempt + 1} failed: {e.message}")

            if attempt < max_retries - 1:
                wait_time = retry_delay * (2**attempt)  # Exponential backoff
                print(f"Retrying in {wait_time} seconds...")
                time.sleep(wait_time)
            else:
                print("Max retries reached. Giving up.")
                raise

        except RateLimitError as e:
            print(f"Rate limited: {e.message}")
            if e.retry_after and attempt < max_retries - 1:
                print(f"Waiting {e.retry_after} seconds as instructed...")
                time.sleep(e.retry_after)
            else:
                print("Cannot retry rate limit.")
                raise

        except (AuthenticationError, ValidationError) as e:
            # Don't retry authentication or validation errors
            print(f"Non-retryable error: {e.message}")
            raise

    print()


def main() -> None:
    """Run error handling examples."""
    handle_specific_errors()

    # Uncomment to test retry logic
    # handle_with_retry()


if __name__ == "__main__":
    main()
