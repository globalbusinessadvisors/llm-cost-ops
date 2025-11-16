"""
Basic usage example for LLM-CostOps SDK.

This example demonstrates the most common operations:
- Submitting usage data
- Querying costs
- Getting analytics
"""

from datetime import datetime, timedelta
from llm_cost_ops import CostOpsClient


def main() -> None:
    """Run basic usage examples."""
    # Initialize client
    # Set your API key via environment variable LLM_COST_OPS_API_KEY
    # or pass it directly
    client = CostOpsClient(api_key="your-api-key-here")

    try:
        # Example 1: Submit usage data
        print("=== Submitting Usage Data ===")
        usage = client.usage.submit(
            organization_id="org-123",
            provider="openai",
            model_id="gpt-4",
            input_tokens=1000,
            output_tokens=500,
            total_tokens=1500,
            metadata={
                "request_id": "req-abc-123",
                "user_id": "user-456",
                "environment": "production",
            },
        )

        print(f"Usage ID: {usage.usage_id}")
        print(f"Estimated Cost: ${usage.estimated_cost}")
        print(f"Processed At: {usage.processed_at}")
        print()

        # Example 2: Get cost summary
        print("=== Getting Cost Summary ===")
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=30)

        costs = client.costs.get(
            organization_id="org-123",
            start_date=start_date,
            end_date=end_date,
            group_by="provider",  # Group by provider
        )

        print(f"Total Cost: ${costs.total_cost}")
        print(f"Total Tokens: {costs.total_tokens:,}")
        print(f"Total Requests: {costs.total_requests:,}")
        print(f"Period: {costs.period_start} to {costs.period_end}")

        if costs.breakdown:
            print("\nBreakdown by Provider:")
            for item in costs.breakdown:
                print(f"  {item.value}: ${item.cost}")
        print()

        # Example 3: Get analytics
        print("=== Getting Analytics ===")
        analytics = client.analytics.get(
            organization_id="org-123",
            start_date=start_date,
            end_date=end_date,
            interval="day",
            metrics=["total_cost", "total_tokens"],
        )

        print(f"Total Cost: ${analytics.summary.total_cost}")
        print(f"Avg Cost per Request: ${analytics.summary.average_cost_per_request}")
        print(f"Avg Tokens per Request: {analytics.summary.average_tokens_per_request:.0f}")

        print("\nDaily Trend:")
        for point in analytics.time_series[:5]:  # Show first 5 days
            print(f"  {point.timestamp.date()}: ${point.metrics.get('total_cost', 0)}")
        print()

        # Example 4: Get usage history
        print("=== Getting Usage History ===")
        history = client.usage.get_history(
            organization_id="org-123",
            provider="openai",
            start_date=start_date,
            end_date=end_date,
            page=1,
            per_page=10,
        )

        print(f"Retrieved {len(history)} usage records")
        print()

    finally:
        # Always close the client when done
        client.close()


if __name__ == "__main__":
    main()
