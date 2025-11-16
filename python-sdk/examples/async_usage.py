"""
Async usage example for LLM-CostOps SDK.

This example demonstrates asynchronous operations for high-performance applications.
"""

import asyncio
from datetime import datetime, timedelta
from llm_cost_ops import AsyncCostOpsClient


async def submit_multiple_usages(client: AsyncCostOpsClient) -> None:
    """Submit multiple usage records concurrently."""
    print("=== Submitting Multiple Usage Records ===")

    # Prepare multiple usage submissions
    tasks = [
        client.usage.submit(
            organization_id="org-123",
            provider="openai",
            model_id="gpt-4",
            input_tokens=1000 + i * 100,
            output_tokens=500 + i * 50,
            total_tokens=1500 + i * 150,
            metadata={"batch_id": "batch-1", "index": i},
        )
        for i in range(5)
    ]

    # Execute all submissions concurrently
    results = await asyncio.gather(*tasks)

    print(f"Submitted {len(results)} usage records")
    for i, usage in enumerate(results):
        print(f"  {i + 1}. ID: {usage.usage_id}, Cost: ${usage.estimated_cost}")
    print()


async def get_costs_and_analytics(client: AsyncCostOpsClient) -> None:
    """Get costs and analytics concurrently."""
    print("=== Getting Costs and Analytics Concurrently ===")

    end_date = datetime.utcnow()
    start_date = end_date - timedelta(days=30)

    # Execute multiple API calls concurrently
    costs_task = client.costs.get(
        organization_id="org-123",
        start_date=start_date,
        end_date=end_date,
    )

    analytics_task = client.analytics.get(
        organization_id="org-123",
        start_date=start_date,
        end_date=end_date,
        interval="day",
    )

    # Wait for both to complete
    costs, analytics = await asyncio.gather(costs_task, analytics_task)

    print(f"Total Cost: ${costs.total_cost}")
    print(f"Total Requests: {costs.total_requests}")
    print(f"Average Cost per Request: ${analytics.summary.average_cost_per_request}")
    print()


async def main() -> None:
    """Run async examples."""
    # Use async context manager for automatic cleanup
    async with AsyncCostOpsClient(api_key="your-api-key-here") as client:
        # Submit multiple usages concurrently
        await submit_multiple_usages(client)

        # Get costs and analytics concurrently
        await get_costs_and_analytics(client)

        print("=== All Operations Completed ===")


if __name__ == "__main__":
    asyncio.run(main())
