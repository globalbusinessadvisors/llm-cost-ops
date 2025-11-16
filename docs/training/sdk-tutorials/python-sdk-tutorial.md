# Python SDK Tutorial

## Table of Contents
- [Prerequisites](#prerequisites)
- [Installation and Setup](#installation-and-setup)
- [Basic Usage](#basic-usage)
- [Async/Await Patterns](#asyncawait-patterns)
- [Resource Management](#resource-management)
- [Error Handling](#error-handling)
- [Retry and Timeout Configuration](#retry-and-timeout-configuration)
- [Pagination](#pagination)
- [Filtering and Sorting](#filtering-and-sorting)
- [Batch Operations](#batch-operations)
- [Webhook Handling](#webhook-handling)
- [Testing](#testing)
- [Type Safety with Pydantic](#type-safety-with-pydantic)
- [Advanced Patterns](#advanced-patterns)
- [Performance Optimization](#performance-optimization)

## Prerequisites

Before getting started with the Python SDK, ensure you have:

- Python 3.8 or higher
- pip or poetry for package management
- API key from LLM Cost Ops platform
- Basic understanding of async/await in Python
- Familiarity with type hints

## Installation and Setup

### Using pip

```bash
pip install llm-cost-ops
```

### Using poetry

```bash
poetry add llm-cost-ops
```

### Using pipenv

```bash
pipenv install llm-cost-ops
```

### Verify Installation

```python
import llm_cost_ops
print(llm_cost_ops.__version__)
```

### Environment Setup

Create a `.env` file in your project root:

```bash
LLM_COST_OPS_API_KEY=your_api_key_here
LLM_COST_OPS_BASE_URL=https://api.llmcostops.com
LLM_COST_OPS_TIMEOUT=30
```

### Configuration

```python
import os
from dotenv import load_dotenv
from llm_cost_ops import CostOpsClient, Config

# Load environment variables
load_dotenv()

# Basic configuration
config = Config(
    api_key=os.getenv("LLM_COST_OPS_API_KEY"),
    base_url=os.getenv("LLM_COST_OPS_BASE_URL"),
    timeout=30,
    max_retries=3,
    verify_ssl=True
)

# Create client
client = CostOpsClient(config)
```

## Basic Usage

### Simple Cost Query

```python
from llm_cost_ops import CostOpsClient
from datetime import datetime, timedelta

# Initialize client
client = CostOpsClient(api_key="your_api_key")

# Get costs for last 7 days
end_date = datetime.now()
start_date = end_date - timedelta(days=7)

costs = client.costs.get_costs(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat()
)

print(f"Total cost: ${costs.total_cost:.2f}")
for item in costs.items:
    print(f"{item.date}: ${item.amount:.2f}")
```

### Usage Tracking

```python
# Track model usage
usage = client.usage.create_usage(
    model="gpt-4",
    tokens_prompt=1000,
    tokens_completion=500,
    request_count=1,
    timestamp=datetime.now().isoformat(),
    metadata={
        "user_id": "user_123",
        "session_id": "session_456"
    }
)

print(f"Usage ID: {usage.id}")
print(f"Cost: ${usage.cost:.4f}")
```

### Get Pricing Information

```python
# Get pricing for specific model
pricing = client.pricing.get_model_pricing(
    model="gpt-4",
    provider="openai"
)

print(f"Model: {pricing.model}")
print(f"Prompt: ${pricing.prompt_price_per_1k} per 1K tokens")
print(f"Completion: ${pricing.completion_price_per_1k} per 1K tokens")
```

### Analytics Queries

```python
# Get usage analytics
analytics = client.analytics.get_usage_analytics(
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat(),
    group_by=["model", "user_id"]
)

for group in analytics.groups:
    print(f"{group.key}: {group.total_tokens} tokens, ${group.total_cost:.2f}")
```

## Async/Await Patterns

### AsyncCostOpsClient

```python
import asyncio
from llm_cost_ops import AsyncCostOpsClient

async def main():
    # Create async client
    async with AsyncCostOpsClient(api_key="your_api_key") as client:
        # Fetch costs asynchronously
        costs = await client.costs.get_costs(
            start_date="2025-01-01",
            end_date="2025-01-31"
        )
        print(f"Total cost: ${costs.total_cost:.2f}")

# Run async function
asyncio.run(main())
```

### Concurrent Requests

```python
import asyncio
from llm_cost_ops import AsyncCostOpsClient

async def fetch_costs(client, start_date, end_date):
    """Fetch costs for a date range"""
    return await client.costs.get_costs(
        start_date=start_date,
        end_date=end_date
    )

async def fetch_usage(client, start_date, end_date):
    """Fetch usage for a date range"""
    return await client.usage.get_usage(
        start_date=start_date,
        end_date=end_date
    )

async def fetch_analytics(client, start_date, end_date):
    """Fetch analytics for a date range"""
    return await client.analytics.get_usage_analytics(
        start_date=start_date,
        end_date=end_date
    )

async def main():
    async with AsyncCostOpsClient(api_key="your_api_key") as client:
        # Run multiple requests concurrently
        results = await asyncio.gather(
            fetch_costs(client, "2025-01-01", "2025-01-31"),
            fetch_usage(client, "2025-01-01", "2025-01-31"),
            fetch_analytics(client, "2025-01-01", "2025-01-31"),
            return_exceptions=True
        )

        costs, usage, analytics = results

        if not isinstance(costs, Exception):
            print(f"Total cost: ${costs.total_cost:.2f}")

        if not isinstance(usage, Exception):
            print(f"Total requests: {usage.total_requests}")

        if not isinstance(analytics, Exception):
            print(f"Analytics groups: {len(analytics.groups)}")

asyncio.run(main())
```

### Async Context Manager

```python
from llm_cost_ops import AsyncCostOpsClient

class CostTracker:
    def __init__(self, api_key: str):
        self.client = AsyncCostOpsClient(api_key=api_key)

    async def __aenter__(self):
        await self.client.__aenter__()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.__aexit__(exc_type, exc_val, exc_tb)

    async def track_usage(self, model: str, tokens: int):
        return await self.client.usage.create_usage(
            model=model,
            tokens_prompt=tokens,
            tokens_completion=0,
            request_count=1
        )

async def main():
    async with CostTracker(api_key="your_api_key") as tracker:
        usage = await tracker.track_usage("gpt-4", 1000)
        print(f"Cost: ${usage.cost:.4f}")

asyncio.run(main())
```

### Async Generator for Streaming

```python
from typing import AsyncGenerator
from llm_cost_ops import AsyncCostOpsClient

async def stream_usage_records(
    client: AsyncCostOpsClient,
    start_date: str,
    end_date: str
) -> AsyncGenerator:
    """Stream usage records page by page"""
    page = 1
    page_size = 100

    while True:
        usage = await client.usage.get_usage(
            start_date=start_date,
            end_date=end_date,
            page=page,
            page_size=page_size
        )

        for record in usage.items:
            yield record

        if not usage.has_next:
            break

        page += 1

async def main():
    async with AsyncCostOpsClient(api_key="your_api_key") as client:
        total_cost = 0.0

        async for record in stream_usage_records(
            client,
            "2025-01-01",
            "2025-01-31"
        ):
            total_cost += record.cost
            print(f"{record.model}: ${record.cost:.4f}")

        print(f"Total: ${total_cost:.2f}")

asyncio.run(main())
```

## Resource Management

### Usage Resource

```python
from llm_cost_ops import CostOpsClient
from datetime import datetime

client = CostOpsClient(api_key="your_api_key")

# Create usage record
usage = client.usage.create_usage(
    model="gpt-4",
    tokens_prompt=1500,
    tokens_completion=500,
    request_count=1,
    timestamp=datetime.now().isoformat(),
    metadata={
        "user_id": "user_123",
        "application": "chatbot",
        "environment": "production"
    }
)

# Get usage by ID
usage = client.usage.get_usage_by_id(usage.id)

# List usage records
usage_list = client.usage.get_usage(
    start_date="2025-01-01",
    end_date="2025-01-31",
    filters={
        "model": "gpt-4",
        "user_id": "user_123"
    }
)

# Update usage metadata
updated = client.usage.update_usage(
    usage_id=usage.id,
    metadata={
        "user_id": "user_123",
        "application": "chatbot",
        "environment": "production",
        "updated": True
    }
)

# Delete usage record
client.usage.delete_usage(usage.id)
```

### Cost Resource

```python
# Get costs summary
costs = client.costs.get_costs(
    start_date="2025-01-01",
    end_date="2025-01-31",
    group_by=["model", "provider"]
)

print(f"Total cost: ${costs.total_cost:.2f}")
for item in costs.items:
    print(f"{item.model} ({item.provider}): ${item.amount:.2f}")

# Get cost breakdown
breakdown = client.costs.get_cost_breakdown(
    start_date="2025-01-01",
    end_date="2025-01-31",
    granularity="daily"  # hourly, daily, weekly, monthly
)

for day in breakdown.items:
    print(f"{day.date}: ${day.amount:.2f}")

# Get cost trends
trends = client.costs.get_cost_trends(
    days=30,
    include_forecast=True
)

print(f"30-day trend: {trends.trend_percentage:+.2f}%")
print(f"Forecasted next month: ${trends.forecast:.2f}")
```

### Pricing Resource

```python
# List all pricing
pricing_list = client.pricing.list_pricing(
    provider="openai",
    model_type="chat"
)

for pricing in pricing_list.items:
    print(f"{pricing.model}: ${pricing.prompt_price_per_1k}/1K prompt tokens")

# Get specific model pricing
pricing = client.pricing.get_model_pricing(
    model="gpt-4-turbo",
    provider="openai"
)

# Calculate cost estimate
estimate = client.pricing.estimate_cost(
    model="gpt-4",
    tokens_prompt=1000,
    tokens_completion=500
)

print(f"Estimated cost: ${estimate.total_cost:.4f}")
print(f"Prompt cost: ${estimate.prompt_cost:.4f}")
print(f"Completion cost: ${estimate.completion_cost:.4f}")

# Update custom pricing
client.pricing.update_pricing(
    model="custom-model",
    prompt_price_per_1k=0.01,
    completion_price_per_1k=0.02,
    metadata={"custom": True}
)
```

### Analytics Resource

```python
# Get usage analytics
analytics = client.analytics.get_usage_analytics(
    start_date="2025-01-01",
    end_date="2025-01-31",
    group_by=["model", "user_id"],
    metrics=["total_tokens", "total_cost", "request_count"]
)

for group in analytics.groups:
    print(f"{group.key}:")
    print(f"  Tokens: {group.total_tokens:,}")
    print(f"  Cost: ${group.total_cost:.2f}")
    print(f"  Requests: {group.request_count:,}")

# Get cost analytics
cost_analytics = client.analytics.get_cost_analytics(
    start_date="2025-01-01",
    end_date="2025-01-31",
    group_by=["provider"],
    include_comparison=True
)

# Get performance metrics
performance = client.analytics.get_performance_metrics(
    start_date="2025-01-01",
    end_date="2025-01-31"
)

print(f"Average latency: {performance.avg_latency_ms:.2f}ms")
print(f"Success rate: {performance.success_rate:.2%}")
print(f"P95 latency: {performance.p95_latency_ms:.2f}ms")

# Get top consumers
top_users = client.analytics.get_top_consumers(
    start_date="2025-01-01",
    end_date="2025-01-31",
    limit=10,
    metric="total_cost"
)

for user in top_users.items:
    print(f"{user.user_id}: ${user.total_cost:.2f}")
```

### Budget Resource

```python
# Create budget
budget = client.budgets.create_budget(
    name="Monthly Production Budget",
    amount=1000.00,
    period="monthly",
    start_date="2025-01-01",
    alerts=[
        {"threshold": 0.5, "type": "email"},
        {"threshold": 0.8, "type": "slack"},
        {"threshold": 1.0, "type": "pagerduty"}
    ],
    filters={
        "environment": "production"
    }
)

# List budgets
budgets = client.budgets.list_budgets(active_only=True)

for b in budgets.items:
    print(f"{b.name}: ${b.spent:.2f} / ${b.amount:.2f} ({b.percentage_used:.1f}%)")

# Get budget by ID
budget = client.budgets.get_budget(budget.id)

# Update budget
client.budgets.update_budget(
    budget_id=budget.id,
    amount=1500.00,
    alerts=[
        {"threshold": 0.7, "type": "email"},
        {"threshold": 0.9, "type": "slack"}
    ]
)

# Delete budget
client.budgets.delete_budget(budget.id)

# Get budget alerts
alerts = client.budgets.get_budget_alerts(
    budget_id=budget.id,
    start_date="2025-01-01",
    end_date="2025-01-31"
)

for alert in alerts.items:
    print(f"{alert.timestamp}: {alert.type} - {alert.message}")
```

### Export Resource

```python
# Create export job
export = client.export.create_export(
    format="csv",  # csv, json, parquet
    start_date="2025-01-01",
    end_date="2025-01-31",
    include_usage=True,
    include_costs=True,
    include_analytics=True,
    filters={
        "model": "gpt-4"
    }
)

print(f"Export job ID: {export.id}")
print(f"Status: {export.status}")

# Check export status
status = client.export.get_export_status(export.id)

print(f"Status: {status.status}")
print(f"Progress: {status.progress}%")

# Wait for completion
import time
while status.status == "processing":
    time.sleep(5)
    status = client.export.get_export_status(export.id)
    print(f"Progress: {status.progress}%")

# Download export
if status.status == "completed":
    download_url = client.export.get_export_download_url(export.id)
    print(f"Download URL: {download_url}")

    # Download to file
    client.export.download_export(export.id, "export_data.csv")

# List exports
exports = client.export.list_exports(
    status="completed",
    limit=10
)

# Delete export
client.export.delete_export(export.id)
```

### Health Resource

```python
# Check API health
health = client.health.check_health()

print(f"Status: {health.status}")
print(f"Version: {health.version}")
print(f"Uptime: {health.uptime_seconds}s")

# Get detailed health metrics
metrics = client.health.get_health_metrics()

print(f"Request rate: {metrics.requests_per_second:.2f} req/s")
print(f"Error rate: {metrics.error_rate:.2%}")
print(f"Average latency: {metrics.avg_latency_ms:.2f}ms")

# Check specific service health
database_health = client.health.check_service("database")
cache_health = client.health.check_service("cache")
queue_health = client.health.check_service("queue")

services = [
    ("Database", database_health),
    ("Cache", cache_health),
    ("Queue", queue_health)
]

for name, health in services:
    status = "✓" if health.healthy else "✗"
    print(f"{status} {name}: {health.status}")
```

## Error Handling

### Built-in Exception Types

```python
from llm_cost_ops.exceptions import (
    CostOpsError,
    AuthenticationError,
    AuthorizationError,
    ResourceNotFoundError,
    ValidationError,
    RateLimitError,
    ServerError,
    NetworkError,
    TimeoutError
)

# Basic error handling
try:
    costs = client.costs.get_costs(
        start_date="2025-01-01",
        end_date="2025-01-31"
    )
except AuthenticationError as e:
    print(f"Authentication failed: {e.message}")
    print("Please check your API key")
except AuthorizationError as e:
    print(f"Authorization failed: {e.message}")
    print("You don't have permission for this resource")
except ResourceNotFoundError as e:
    print(f"Resource not found: {e.message}")
except ValidationError as e:
    print(f"Validation error: {e.message}")
    print(f"Errors: {e.errors}")
except RateLimitError as e:
    print(f"Rate limit exceeded: {e.message}")
    print(f"Retry after: {e.retry_after} seconds")
except TimeoutError as e:
    print(f"Request timeout: {e.message}")
except ServerError as e:
    print(f"Server error: {e.message}")
    print(f"Status code: {e.status_code}")
except NetworkError as e:
    print(f"Network error: {e.message}")
except CostOpsError as e:
    print(f"General error: {e.message}")
```

### Custom Exception Handler

```python
from typing import Callable, TypeVar, Any
from functools import wraps

T = TypeVar('T')

def handle_api_errors(
    fallback_value: Any = None,
    log_errors: bool = True
) -> Callable:
    """Decorator to handle API errors"""
    def decorator(func: Callable[..., T]) -> Callable[..., T]:
        @wraps(func)
        def wrapper(*args, **kwargs):
            try:
                return func(*args, **kwargs)
            except RateLimitError as e:
                if log_errors:
                    print(f"Rate limit hit, waiting {e.retry_after}s")
                time.sleep(e.retry_after)
                return func(*args, **kwargs)  # Retry once
            except (NetworkError, TimeoutError) as e:
                if log_errors:
                    print(f"Network error: {e}, retrying...")
                time.sleep(1)
                return func(*args, **kwargs)  # Retry once
            except CostOpsError as e:
                if log_errors:
                    print(f"API error: {e}")
                return fallback_value
        return wrapper
    return decorator

# Usage
@handle_api_errors(fallback_value={"total_cost": 0.0}, log_errors=True)
def get_monthly_costs(client, month):
    return client.costs.get_costs(
        start_date=f"{month}-01",
        end_date=f"{month}-31"
    )

result = get_monthly_costs(client, "2025-01")
```

### Async Error Handling

```python
import asyncio
from llm_cost_ops import AsyncCostOpsClient
from llm_cost_ops.exceptions import CostOpsError, RateLimitError

async def safe_fetch_costs(client, start_date, end_date, max_retries=3):
    """Fetch costs with retry logic"""
    retries = 0

    while retries < max_retries:
        try:
            return await client.costs.get_costs(
                start_date=start_date,
                end_date=end_date
            )
        except RateLimitError as e:
            print(f"Rate limited, waiting {e.retry_after}s")
            await asyncio.sleep(e.retry_after)
            retries += 1
        except NetworkError as e:
            print(f"Network error: {e}, retrying in 2s")
            await asyncio.sleep(2)
            retries += 1
        except CostOpsError as e:
            print(f"API error: {e}")
            raise

    raise Exception(f"Failed after {max_retries} retries")

async def main():
    async with AsyncCostOpsClient(api_key="your_api_key") as client:
        try:
            costs = await safe_fetch_costs(
                client,
                "2025-01-01",
                "2025-01-31"
            )
            print(f"Total cost: ${costs.total_cost:.2f}")
        except Exception as e:
            print(f"Failed to fetch costs: {e}")

asyncio.run(main())
```

## Retry and Timeout Configuration

### Basic Retry Configuration

```python
from llm_cost_ops import CostOpsClient, Config, RetryConfig

# Configure retry behavior
retry_config = RetryConfig(
    max_retries=5,
    backoff_factor=2.0,  # Exponential backoff
    retry_on_status=[429, 500, 502, 503, 504],
    retry_on_exceptions=[NetworkError, TimeoutError]
)

config = Config(
    api_key="your_api_key",
    retry_config=retry_config,
    timeout=30
)

client = CostOpsClient(config)
```

### Advanced Retry Strategy

```python
from llm_cost_ops.retry import (
    ExponentialBackoff,
    JitteredBackoff,
    RetryStrategy
)

# Exponential backoff with jitter
strategy = RetryStrategy(
    max_retries=5,
    backoff=JitteredBackoff(
        base=1.0,
        multiplier=2.0,
        max_delay=60.0,
        jitter=0.1
    )
)

config = Config(
    api_key="your_api_key",
    retry_strategy=strategy
)

client = CostOpsClient(config)
```

### Timeout Configuration

```python
from llm_cost_ops import CostOpsClient, TimeoutConfig

# Configure different timeouts
timeout_config = TimeoutConfig(
    connect_timeout=10,  # Connection timeout
    read_timeout=30,     # Read timeout
    total_timeout=60     # Total request timeout
)

config = Config(
    api_key="your_api_key",
    timeout_config=timeout_config
)

client = CostOpsClient(config)
```

### Per-Request Configuration

```python
# Override timeout for specific request
costs = client.costs.get_costs(
    start_date="2025-01-01",
    end_date="2025-01-31",
    request_options={
        "timeout": 60,
        "max_retries": 3
    }
)
```

## Pagination

### Manual Pagination

```python
# Paginate through usage records
page = 1
page_size = 100
all_records = []

while True:
    usage = client.usage.get_usage(
        start_date="2025-01-01",
        end_date="2025-01-31",
        page=page,
        page_size=page_size
    )

    all_records.extend(usage.items)

    print(f"Page {page}: {len(usage.items)} records")

    if not usage.has_next:
        break

    page += 1

print(f"Total records: {len(all_records)}")
```

### Automatic Pagination Iterator

```python
from llm_cost_ops.pagination import paginate

# Iterate through all pages automatically
for usage_record in paginate(
    client.usage.get_usage,
    start_date="2025-01-01",
    end_date="2025-01-31",
    page_size=100
):
    print(f"{usage_record.model}: ${usage_record.cost:.4f}")
```

### Async Pagination

```python
import asyncio
from llm_cost_ops import AsyncCostOpsClient

async def fetch_all_usage(client, start_date, end_date):
    """Fetch all usage records with async pagination"""
    all_records = []
    page = 1

    while True:
        usage = await client.usage.get_usage(
            start_date=start_date,
            end_date=end_date,
            page=page,
            page_size=100
        )

        all_records.extend(usage.items)

        if not usage.has_next:
            break

        page += 1

    return all_records

async def main():
    async with AsyncCostOpsClient(api_key="your_api_key") as client:
        records = await fetch_all_usage(
            client,
            "2025-01-01",
            "2025-01-31"
        )
        print(f"Total records: {len(records)}")

asyncio.run(main())
```

### Cursor-Based Pagination

```python
# Use cursor-based pagination for large datasets
cursor = None
all_records = []

while True:
    usage = client.usage.get_usage(
        start_date="2025-01-01",
        end_date="2025-01-31",
        cursor=cursor,
        page_size=1000
    )

    all_records.extend(usage.items)

    if not usage.next_cursor:
        break

    cursor = usage.next_cursor

print(f"Total records: {len(all_records)}")
```

## Filtering and Sorting

### Basic Filtering

```python
# Filter usage records
usage = client.usage.get_usage(
    start_date="2025-01-01",
    end_date="2025-01-31",
    filters={
        "model": "gpt-4",
        "user_id": "user_123",
        "environment": "production"
    }
)
```

### Advanced Filtering

```python
from llm_cost_ops.filters import Filter, Q

# Complex filter using query builder
filter_query = Q(
    Q(model="gpt-4") | Q(model="gpt-4-turbo"),
    Q(cost__gte=1.0),
    Q(tokens__gte=1000)
)

usage = client.usage.get_usage(
    start_date="2025-01-01",
    end_date="2025-01-31",
    filters=filter_query
)
```

### Sorting

```python
# Sort by cost (descending)
usage = client.usage.get_usage(
    start_date="2025-01-01",
    end_date="2025-01-31",
    sort_by="cost",
    sort_order="desc"
)

# Multiple sort fields
usage = client.usage.get_usage(
    start_date="2025-01-01",
    end_date="2025-01-31",
    sort_by=["cost", "timestamp"],
    sort_order=["desc", "asc"]
)
```

### Combined Filtering and Sorting

```python
# Get top 10 most expensive requests for GPT-4
usage = client.usage.get_usage(
    start_date="2025-01-01",
    end_date="2025-01-31",
    filters={
        "model": "gpt-4"
    },
    sort_by="cost",
    sort_order="desc",
    page_size=10
)

for record in usage.items:
    print(f"{record.timestamp}: ${record.cost:.4f} ({record.total_tokens} tokens)")
```

## Batch Operations

### Batch Create Usage

```python
from datetime import datetime, timedelta

# Create multiple usage records
records = []
base_time = datetime.now()

for i in range(100):
    records.append({
        "model": "gpt-4",
        "tokens_prompt": 1000 + i * 100,
        "tokens_completion": 500 + i * 50,
        "request_count": 1,
        "timestamp": (base_time - timedelta(hours=i)).isoformat(),
        "metadata": {
            "user_id": f"user_{i % 10}",
            "request_id": f"req_{i}"
        }
    })

# Batch create
result = client.usage.batch_create_usage(records)

print(f"Created: {result.created_count}")
print(f"Failed: {result.failed_count}")

for error in result.errors:
    print(f"Error at index {error.index}: {error.message}")
```

### Batch Update

```python
# Batch update usage records
updates = [
    {
        "id": "usage_1",
        "metadata": {"updated": True, "batch": 1}
    },
    {
        "id": "usage_2",
        "metadata": {"updated": True, "batch": 1}
    },
    # ... more updates
]

result = client.usage.batch_update_usage(updates)

print(f"Updated: {result.updated_count}")
print(f"Failed: {result.failed_count}")
```

### Batch Delete

```python
# Batch delete usage records
usage_ids = ["usage_1", "usage_2", "usage_3"]

result = client.usage.batch_delete_usage(usage_ids)

print(f"Deleted: {result.deleted_count}")
print(f"Failed: {result.failed_count}")
```

### Async Batch Operations

```python
import asyncio
from llm_cost_ops import AsyncCostOpsClient

async def batch_create_usage_async(client, records, batch_size=100):
    """Create usage records in batches asynchronously"""
    tasks = []

    for i in range(0, len(records), batch_size):
        batch = records[i:i + batch_size]
        task = client.usage.batch_create_usage(batch)
        tasks.append(task)

    results = await asyncio.gather(*tasks)

    total_created = sum(r.created_count for r in results)
    total_failed = sum(r.failed_count for r in results)

    return {
        "created": total_created,
        "failed": total_failed
    }

async def main():
    # Create 1000 records
    records = [
        {
            "model": "gpt-4",
            "tokens_prompt": 1000,
            "tokens_completion": 500,
            "request_count": 1,
            "timestamp": datetime.now().isoformat()
        }
        for _ in range(1000)
    ]

    async with AsyncCostOpsClient(api_key="your_api_key") as client:
        result = await batch_create_usage_async(client, records)
        print(f"Created: {result['created']}, Failed: {result['failed']}")

asyncio.run(main())
```

## Webhook Handling

### Register Webhook

```python
# Register webhook endpoint
webhook = client.webhooks.create_webhook(
    url="https://your-app.com/webhooks/cost-ops",
    events=[
        "usage.created",
        "budget.alert",
        "export.completed"
    ],
    secret="your_webhook_secret",
    active=True
)

print(f"Webhook ID: {webhook.id}")
```

### Verify Webhook Signature

```python
import hmac
import hashlib
from flask import Flask, request, jsonify

app = Flask(__name__)

WEBHOOK_SECRET = "your_webhook_secret"

def verify_webhook_signature(payload: bytes, signature: str) -> bool:
    """Verify webhook signature"""
    expected_signature = hmac.new(
        WEBHOOK_SECRET.encode(),
        payload,
        hashlib.sha256
    ).hexdigest()

    return hmac.compare_digest(signature, expected_signature)

@app.route('/webhooks/cost-ops', methods=['POST'])
def handle_webhook():
    # Get signature from header
    signature = request.headers.get('X-CostOps-Signature')

    # Verify signature
    if not verify_webhook_signature(request.data, signature):
        return jsonify({"error": "Invalid signature"}), 401

    # Process webhook
    event = request.json
    event_type = event['type']
    data = event['data']

    if event_type == 'usage.created':
        handle_usage_created(data)
    elif event_type == 'budget.alert':
        handle_budget_alert(data)
    elif event_type == 'export.completed':
        handle_export_completed(data)

    return jsonify({"status": "processed"}), 200

def handle_usage_created(data):
    print(f"New usage: {data['id']}, cost: ${data['cost']}")

def handle_budget_alert(data):
    print(f"Budget alert: {data['budget_name']} at {data['percentage']}%")

def handle_export_completed(data):
    print(f"Export completed: {data['export_id']}")

if __name__ == '__main__':
    app.run(port=8000)
```

### Webhook Event Handler Class

```python
from typing import Dict, Callable
from llm_cost_ops.webhooks import WebhookEvent, verify_signature

class WebhookHandler:
    def __init__(self, secret: str):
        self.secret = secret
        self.handlers: Dict[str, Callable] = {}

    def register(self, event_type: str):
        """Decorator to register event handlers"""
        def decorator(func: Callable):
            self.handlers[event_type] = func
            return func
        return decorator

    def handle(self, payload: bytes, signature: str) -> bool:
        """Handle incoming webhook"""
        # Verify signature
        if not verify_signature(payload, signature, self.secret):
            raise ValueError("Invalid signature")

        # Parse event
        event = WebhookEvent.from_json(payload)

        # Get handler
        handler = self.handlers.get(event.type)
        if not handler:
            print(f"No handler for event type: {event.type}")
            return False

        # Execute handler
        try:
            handler(event.data)
            return True
        except Exception as e:
            print(f"Error handling event: {e}")
            return False

# Usage
webhook_handler = WebhookHandler(secret="your_webhook_secret")

@webhook_handler.register("usage.created")
def on_usage_created(data):
    print(f"Usage created: {data['model']}, cost: ${data['cost']}")

@webhook_handler.register("budget.alert")
def on_budget_alert(data):
    print(f"Budget alert: {data['budget_name']}")
    # Send notification
    send_slack_notification(
        f"Budget {data['budget_name']} is at {data['percentage']}%"
    )

@webhook_handler.register("export.completed")
def on_export_completed(data):
    print(f"Export ready: {data['download_url']}")
    # Download export
    download_file(data['download_url'], f"export_{data['export_id']}.csv")
```

## Testing

### Basic Unit Testing

```python
import pytest
from unittest.mock import Mock, patch
from llm_cost_ops import CostOpsClient
from llm_cost_ops.models import CostResponse

def test_get_costs():
    """Test getting costs"""
    # Mock the client
    client = CostOpsClient(api_key="test_key")

    # Create mock response
    mock_response = CostResponse(
        total_cost=100.50,
        items=[
            {"date": "2025-01-01", "amount": 50.25},
            {"date": "2025-01-02", "amount": 50.25}
        ]
    )

    # Patch the API call
    with patch.object(client.costs, 'get_costs', return_value=mock_response):
        costs = client.costs.get_costs(
            start_date="2025-01-01",
            end_date="2025-01-31"
        )

        assert costs.total_cost == 100.50
        assert len(costs.items) == 2
```

### Pytest Fixtures

```python
import pytest
from llm_cost_ops import CostOpsClient

@pytest.fixture
def client():
    """Create test client"""
    return CostOpsClient(api_key="test_key")

@pytest.fixture
def mock_costs_response():
    """Create mock costs response"""
    return {
        "total_cost": 100.50,
        "items": [
            {"date": "2025-01-01", "amount": 50.25},
            {"date": "2025-01-02", "amount": 50.25}
        ]
    }

def test_get_costs_with_fixtures(client, mock_costs_response):
    """Test with fixtures"""
    with patch.object(
        client.costs,
        'get_costs',
        return_value=mock_costs_response
    ):
        costs = client.costs.get_costs(
            start_date="2025-01-01",
            end_date="2025-01-31"
        )
        assert costs.total_cost == 100.50
```

### Integration Testing

```python
import pytest
import os
from llm_cost_ops import CostOpsClient

@pytest.fixture
def integration_client():
    """Create client for integration tests"""
    api_key = os.getenv("LLM_COST_OPS_TEST_API_KEY")
    if not api_key:
        pytest.skip("No test API key provided")

    return CostOpsClient(api_key=api_key)

@pytest.mark.integration
def test_create_and_get_usage(integration_client):
    """Integration test for usage creation"""
    # Create usage
    usage = integration_client.usage.create_usage(
        model="gpt-4",
        tokens_prompt=1000,
        tokens_completion=500,
        request_count=1
    )

    assert usage.id is not None
    assert usage.model == "gpt-4"

    # Get usage by ID
    fetched_usage = integration_client.usage.get_usage_by_id(usage.id)

    assert fetched_usage.id == usage.id
    assert fetched_usage.model == usage.model

    # Cleanup
    integration_client.usage.delete_usage(usage.id)
```

### Async Testing

```python
import pytest
import asyncio
from llm_cost_ops import AsyncCostOpsClient

@pytest.fixture
async def async_client():
    """Create async test client"""
    client = AsyncCostOpsClient(api_key="test_key")
    yield client
    await client.close()

@pytest.mark.asyncio
async def test_async_get_costs(async_client):
    """Test async cost fetching"""
    with patch.object(
        async_client.costs,
        'get_costs',
        return_value={"total_cost": 100.50}
    ):
        costs = await async_client.costs.get_costs(
            start_date="2025-01-01",
            end_date="2025-01-31"
        )
        assert costs.total_cost == 100.50
```

### Mock Server for Testing

```python
import pytest
from unittest.mock import Mock
import responses
from llm_cost_ops import CostOpsClient

@responses.activate
def test_get_costs_with_mock_server():
    """Test with mock HTTP server"""
    # Mock the API endpoint
    responses.add(
        responses.GET,
        "https://api.llmcostops.com/v1/costs",
        json={
            "total_cost": 100.50,
            "items": [
                {"date": "2025-01-01", "amount": 50.25}
            ]
        },
        status=200
    )

    client = CostOpsClient(api_key="test_key")
    costs = client.costs.get_costs(
        start_date="2025-01-01",
        end_date="2025-01-31"
    )

    assert costs.total_cost == 100.50
    assert len(responses.calls) == 1
```

## Type Safety with Pydantic

### Defining Models

```python
from pydantic import BaseModel, Field, validator
from typing import Optional, Dict, List
from datetime import datetime

class UsageCreate(BaseModel):
    """Model for creating usage records"""
    model: str = Field(..., min_length=1, max_length=100)
    tokens_prompt: int = Field(..., ge=0)
    tokens_completion: int = Field(..., ge=0)
    request_count: int = Field(default=1, ge=1)
    timestamp: datetime = Field(default_factory=datetime.now)
    metadata: Optional[Dict[str, Any]] = None

    @validator('model')
    def validate_model(cls, v):
        allowed_models = ['gpt-4', 'gpt-4-turbo', 'gpt-3.5-turbo']
        if v not in allowed_models:
            raise ValueError(f"Model must be one of {allowed_models}")
        return v

    @validator('metadata')
    def validate_metadata(cls, v):
        if v and len(v) > 50:
            raise ValueError("Metadata cannot have more than 50 keys")
        return v

# Usage
usage_data = UsageCreate(
    model="gpt-4",
    tokens_prompt=1000,
    tokens_completion=500,
    metadata={"user_id": "user_123"}
)

# Automatically validated
usage = client.usage.create_usage(**usage_data.dict())
```

### Response Models

```python
from pydantic import BaseModel
from typing import List
from datetime import datetime

class UsageItem(BaseModel):
    id: str
    model: str
    tokens_prompt: int
    tokens_completion: int
    total_tokens: int
    cost: float
    timestamp: datetime
    metadata: Optional[Dict[str, Any]] = None

class UsageResponse(BaseModel):
    items: List[UsageItem]
    total_count: int
    page: int
    page_size: int
    has_next: bool

    @property
    def total_cost(self) -> float:
        return sum(item.cost for item in self.items)

    @property
    def total_tokens(self) -> int:
        return sum(item.total_tokens for item in self.items)

# Usage
response = client.usage.get_usage(
    start_date="2025-01-01",
    end_date="2025-01-31"
)

# Type-safe access
usage_response = UsageResponse(**response)
print(f"Total cost: ${usage_response.total_cost:.2f}")
print(f"Total tokens: {usage_response.total_tokens:,}")
```

### Validation Helpers

```python
from pydantic import BaseModel, validator, root_validator
from datetime import datetime

class DateRangeQuery(BaseModel):
    start_date: datetime
    end_date: datetime

    @validator('end_date')
    def end_date_must_be_after_start(cls, v, values):
        if 'start_date' in values and v < values['start_date']:
            raise ValueError('end_date must be after start_date')
        return v

    @root_validator
    def check_date_range(cls, values):
        start = values.get('start_date')
        end = values.get('end_date')

        if start and end:
            delta = end - start
            if delta.days > 365:
                raise ValueError('Date range cannot exceed 365 days')

        return values

# Usage
query = DateRangeQuery(
    start_date=datetime(2025, 1, 1),
    end_date=datetime(2025, 1, 31)
)

costs = client.costs.get_costs(
    start_date=query.start_date.isoformat(),
    end_date=query.end_date.isoformat()
)
```

## Advanced Patterns

### Context Manager for Resource Tracking

```python
from contextlib import contextmanager
from datetime import datetime

@contextmanager
def track_cost(client, operation_name: str, metadata: dict = None):
    """Context manager to track operation costs"""
    start_time = datetime.now()
    start_tokens = 0  # Get from token counter

    try:
        yield
    finally:
        end_time = datetime.now()
        end_tokens = 0  # Get from token counter

        # Calculate usage
        tokens_used = end_tokens - start_tokens

        # Track usage
        if tokens_used > 0:
            client.usage.create_usage(
                model="custom",
                tokens_prompt=tokens_used,
                tokens_completion=0,
                request_count=1,
                metadata={
                    "operation": operation_name,
                    "duration_ms": (end_time - start_time).total_seconds() * 1000,
                    **(metadata or {})
                }
            )

# Usage
with track_cost(client, "data_processing", {"batch_size": 100}):
    # Your operation here
    process_data()
```

### Decorator for Cost Tracking

```python
from functools import wraps
import time

def track_llm_cost(model: str, client: CostOpsClient):
    """Decorator to track LLM costs"""
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            start_time = time.time()

            try:
                result = func(*args, **kwargs)

                # Extract token usage from result
                if isinstance(result, dict) and 'usage' in result:
                    usage = result['usage']

                    client.usage.create_usage(
                        model=model,
                        tokens_prompt=usage.get('prompt_tokens', 0),
                        tokens_completion=usage.get('completion_tokens', 0),
                        request_count=1,
                        metadata={
                            "function": func.__name__,
                            "duration_ms": (time.time() - start_time) * 1000
                        }
                    )

                return result
            except Exception as e:
                # Track failed requests
                client.usage.create_usage(
                    model=model,
                    tokens_prompt=0,
                    tokens_completion=0,
                    request_count=1,
                    metadata={
                        "function": func.__name__,
                        "error": str(e),
                        "status": "failed"
                    }
                )
                raise

        return wrapper
    return decorator

# Usage
@track_llm_cost("gpt-4", client)
def call_openai_api(prompt: str):
    # Your OpenAI API call
    response = openai.ChatCompletion.create(
        model="gpt-4",
        messages=[{"role": "user", "content": prompt}]
    )
    return response
```

### Rate Limiter

```python
import time
from collections import deque
from threading import Lock

class RateLimiter:
    """Rate limiter for API calls"""
    def __init__(self, max_requests: int, time_window: int):
        self.max_requests = max_requests
        self.time_window = time_window
        self.requests = deque()
        self.lock = Lock()

    def acquire(self):
        """Acquire permission to make a request"""
        with self.lock:
            now = time.time()

            # Remove old requests
            while self.requests and self.requests[0] < now - self.time_window:
                self.requests.popleft()

            # Check if we can make a request
            if len(self.requests) >= self.max_requests:
                sleep_time = self.requests[0] + self.time_window - now
                time.sleep(sleep_time)
                return self.acquire()

            # Add new request
            self.requests.append(now)

# Usage
rate_limiter = RateLimiter(max_requests=100, time_window=60)

def get_costs_with_rate_limit(client, start_date, end_date):
    rate_limiter.acquire()
    return client.costs.get_costs(
        start_date=start_date,
        end_date=end_date
    )
```

### Connection Pooling

```python
from llm_cost_ops import CostOpsClient
from queue import Queue
from threading import Lock

class ClientPool:
    """Pool of CostOpsClient instances"""
    def __init__(self, api_key: str, pool_size: int = 5):
        self.pool = Queue(maxsize=pool_size)
        self.lock = Lock()

        for _ in range(pool_size):
            client = CostOpsClient(api_key=api_key)
            self.pool.put(client)

    def get_client(self) -> CostOpsClient:
        """Get a client from the pool"""
        return self.pool.get()

    def return_client(self, client: CostOpsClient):
        """Return a client to the pool"""
        self.pool.put(client)

    def __enter__(self):
        self.client = self.get_client()
        return self.client

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.return_client(self.client)

# Usage
pool = ClientPool(api_key="your_api_key", pool_size=10)

with pool as client:
    costs = client.costs.get_costs(
        start_date="2025-01-01",
        end_date="2025-01-31"
    )
```

## Performance Optimization

### Caching

```python
from functools import lru_cache
from datetime import datetime, timedelta

class CachedCostOpsClient:
    """Client with caching support"""
    def __init__(self, api_key: str):
        self.client = CostOpsClient(api_key=api_key)

    @lru_cache(maxsize=100)
    def get_pricing_cached(self, model: str, provider: str):
        """Get pricing with caching"""
        return self.client.pricing.get_model_pricing(
            model=model,
            provider=provider
        )

    def estimate_cost_fast(self, model: str, tokens_prompt: int, tokens_completion: int):
        """Fast cost estimation using cached pricing"""
        pricing = self.get_pricing_cached(model, "openai")

        prompt_cost = (tokens_prompt / 1000) * pricing.prompt_price_per_1k
        completion_cost = (tokens_completion / 1000) * pricing.completion_price_per_1k

        return prompt_cost + completion_cost

# Usage
cached_client = CachedCostOpsClient(api_key="your_api_key")

# First call hits API
cost1 = cached_client.estimate_cost_fast("gpt-4", 1000, 500)

# Second call uses cache
cost2 = cached_client.estimate_cost_fast("gpt-4", 2000, 1000)
```

### Batch Processing

```python
from concurrent.futures import ThreadPoolExecutor, as_completed

def process_usage_batch(client, records, batch_size=100, max_workers=5):
    """Process usage records in parallel batches"""
    results = []

    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        futures = []

        for i in range(0, len(records), batch_size):
            batch = records[i:i + batch_size]
            future = executor.submit(
                client.usage.batch_create_usage,
                batch
            )
            futures.append(future)

        for future in as_completed(futures):
            try:
                result = future.result()
                results.append(result)
            except Exception as e:
                print(f"Batch failed: {e}")

    return results

# Usage
records = [...]  # Your usage records
results = process_usage_batch(client, records)

total_created = sum(r.created_count for r in results)
print(f"Created {total_created} records")
```

### Async Batch Processing

```python
import asyncio
from llm_cost_ops import AsyncCostOpsClient

async def process_large_dataset(records, api_key, batch_size=100, concurrency=10):
    """Process large dataset with controlled concurrency"""
    semaphore = asyncio.Semaphore(concurrency)

    async def process_batch(client, batch):
        async with semaphore:
            return await client.usage.batch_create_usage(batch)

    async with AsyncCostOpsClient(api_key=api_key) as client:
        tasks = []

        for i in range(0, len(records), batch_size):
            batch = records[i:i + batch_size]
            task = process_batch(client, batch)
            tasks.append(task)

        results = await asyncio.gather(*tasks, return_exceptions=True)

        successful = [r for r in results if not isinstance(r, Exception)]
        failed = [r for r in results if isinstance(r, Exception)]

        return {
            "successful": len(successful),
            "failed": len(failed),
            "total_created": sum(r.created_count for r in successful)
        }

# Usage
records = [...]  # 10,000+ records
result = asyncio.run(process_large_dataset(records, "your_api_key"))
print(f"Processed: {result['total_created']} records")
```

---

## Complete Example Application

Here's a complete example application that demonstrates many of the concepts covered:

```python
#!/usr/bin/env python3
"""
Complete LLM Cost Ops SDK Example Application
"""

import asyncio
import os
from datetime import datetime, timedelta
from typing import List, Dict
from dotenv import load_dotenv
from llm_cost_ops import AsyncCostOpsClient, Config
from llm_cost_ops.exceptions import CostOpsError, RateLimitError

# Load environment
load_dotenv()

class CostTracker:
    """Complete cost tracking application"""

    def __init__(self, api_key: str):
        config = Config(
            api_key=api_key,
            timeout=30,
            max_retries=3
        )
        self.client = AsyncCostOpsClient(config)

    async def __aenter__(self):
        await self.client.__aenter__()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.client.__aexit__(exc_type, exc_val, exc_tb)

    async def track_usage(self, model: str, tokens_prompt: int, tokens_completion: int, metadata: dict = None):
        """Track LLM usage"""
        try:
            usage = await self.client.usage.create_usage(
                model=model,
                tokens_prompt=tokens_prompt,
                tokens_completion=tokens_completion,
                request_count=1,
                timestamp=datetime.now().isoformat(),
                metadata=metadata or {}
            )
            print(f"✓ Tracked usage: {usage.id} - ${usage.cost:.4f}")
            return usage
        except RateLimitError as e:
            print(f"Rate limited, waiting {e.retry_after}s")
            await asyncio.sleep(e.retry_after)
            return await self.track_usage(model, tokens_prompt, tokens_completion, metadata)
        except CostOpsError as e:
            print(f"✗ Error tracking usage: {e}")
            return None

    async def get_cost_summary(self, days: int = 7):
        """Get cost summary for last N days"""
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)

        try:
            costs = await self.client.costs.get_costs(
                start_date=start_date.isoformat(),
                end_date=end_date.isoformat()
            )

            print(f"\n📊 Cost Summary (Last {days} days)")
            print(f"Total Cost: ${costs.total_cost:.2f}")

            for item in costs.items[:10]:
                print(f"  {item.date}: ${item.amount:.2f}")

            return costs
        except CostOpsError as e:
            print(f"✗ Error fetching costs: {e}")
            return None

    async def check_budgets(self):
        """Check budget status"""
        try:
            budgets = await self.client.budgets.list_budgets(active_only=True)

            print(f"\n💰 Active Budgets")

            for budget in budgets.items:
                percentage = budget.percentage_used
                status = "🟢" if percentage < 50 else "🟡" if percentage < 80 else "🔴"

                print(f"{status} {budget.name}: ${budget.spent:.2f} / ${budget.amount:.2f} ({percentage:.1f}%)")

            return budgets
        except CostOpsError as e:
            print(f"✗ Error fetching budgets: {e}")
            return None

    async def run_analytics(self, days: int = 30):
        """Run comprehensive analytics"""
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)

        try:
            # Get analytics
            analytics = await self.client.analytics.get_usage_analytics(
                start_date=start_date.isoformat(),
                end_date=end_date.isoformat(),
                group_by=["model"],
                metrics=["total_tokens", "total_cost", "request_count"]
            )

            print(f"\n📈 Analytics (Last {days} days)")

            for group in analytics.groups:
                print(f"\n{group.key}:")
                print(f"  Tokens: {group.total_tokens:,}")
                print(f"  Cost: ${group.total_cost:.2f}")
                print(f"  Requests: {group.request_count:,}")
                print(f"  Avg Cost/Request: ${group.total_cost / group.request_count:.4f}")

            return analytics
        except CostOpsError as e:
            print(f"✗ Error running analytics: {e}")
            return None

async def main():
    """Main application"""
    api_key = os.getenv("LLM_COST_OPS_API_KEY")

    if not api_key:
        print("Error: LLM_COST_OPS_API_KEY not set")
        return

    async with CostTracker(api_key) as tracker:
        # Track some usage
        await tracker.track_usage(
            model="gpt-4",
            tokens_prompt=1000,
            tokens_completion=500,
            metadata={"user_id": "user_123", "app": "demo"}
        )

        # Get cost summary
        await tracker.get_cost_summary(days=7)

        # Check budgets
        await tracker.check_budgets()

        # Run analytics
        await tracker.run_analytics(days=30)

if __name__ == "__main__":
    asyncio.run(main())
```

---

## Additional Resources

- **API Reference**: https://docs.llmcostops.com/api
- **GitHub Repository**: https://github.com/llmcostops/python-sdk
- **Examples**: https://github.com/llmcostops/python-sdk/tree/main/examples
- **Support**: support@llmcostops.com

## Next Steps

1. Set up your development environment
2. Install the SDK and configure your API key
3. Try the basic usage examples
4. Explore async patterns for better performance
5. Implement error handling and retries
6. Add testing to your application
7. Optimize for production with caching and batching

For more advanced use cases, check out the [Advanced Integration Guide](./advanced-integration.md).
