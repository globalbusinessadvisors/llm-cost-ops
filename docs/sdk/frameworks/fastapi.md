# FastAPI Integration Guide

Integrate LLM-CostOps with FastAPI applications to automatically track LLM costs for all your API endpoints.

## Installation

```bash
pip install llm-cost-ops fastapi openai anthropic
```

## Basic Integration

### 1. Create Cost Tracking Middleware

```python
# app/middleware/cost_tracking.py
from llm_cost_ops import CostOpsClient
from fastapi import Request, Response
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.types import ASGIApp
import os
import time

class CostTrackingMiddleware(BaseHTTPMiddleware):
    def __init__(self, app: ASGIApp, cost_ops_client: CostOpsClient):
        super().__init__(app)
        self.cost_ops = cost_ops_client
        self.organization_id = os.getenv("ORGANIZATION_ID", "org-default")

    async def dispatch(self, request: Request, call_next):
        # Store request start time
        start_time = time.time()

        # Process request
        response = await call_next(request)

        # Track LLM usage if present in request state
        if hasattr(request.state, "llm_usage"):
            usage = request.state.llm_usage
            await self.cost_ops.usage.submit_async(
                organization_id=self.organization_id,
                provider=usage["provider"],
                model_id=usage["model_id"],
                input_tokens=usage["input_tokens"],
                output_tokens=usage["output_tokens"],
                total_tokens=usage["total_tokens"],
                metadata={
                    "endpoint": request.url.path,
                    "method": request.method,
                    "latency_ms": int((time.time() - start_time) * 1000),
                    "user_id": getattr(request.state, "user_id", None),
                }
            )

        return response
```

### 2. Set Up FastAPI Application

```python
# app/main.py
from fastapi import FastAPI, Depends, HTTPException, Request
from llm_cost_ops import CostOpsClient
from .middleware.cost_tracking import CostTrackingMiddleware
import os
import openai

# Initialize clients
app = FastAPI(title="LLM Application with Cost Tracking")
cost_ops_client = CostOpsClient(api_key=os.getenv("LLM_COST_OPS_API_KEY"))
openai.api_key = os.getenv("OPENAI_API_KEY")

# Add cost tracking middleware
app.add_middleware(CostTrackingMiddleware, cost_ops_client=cost_ops_client)

@app.post("/api/chat")
async def chat(request: Request, message: str):
    """Chat endpoint with automatic cost tracking."""

    # Call OpenAI
    response = openai.ChatCompletion.create(
        model="gpt-4",
        messages=[{"role": "user", "content": message}]
    )

    # Extract usage information
    usage = response.usage

    # Store usage in request state for middleware
    request.state.llm_usage = {
        "provider": "openai",
        "model_id": "gpt-4",
        "input_tokens": usage.prompt_tokens,
        "output_tokens": usage.completion_tokens,
        "total_tokens": usage.total_tokens
    }

    return {
        "response": response.choices[0].message.content,
        "cost_tracked": True
    }
```

### 3. Run the Application

```bash
uvicorn app.main:app --reload --port 8000
```

## Advanced Integration

### Dependency Injection Pattern

```python
# app/dependencies.py
from llm_cost_ops import CostOpsClient
from fastapi import Depends, Request
import os

def get_cost_ops_client() -> CostOpsClient:
    """Dependency to get CostOps client."""
    return CostOpsClient(api_key=os.getenv("LLM_COST_OPS_API_KEY"))

def get_organization_id(request: Request) -> str:
    """Get organization ID from request context."""
    # Get from JWT token, header, or default
    return getattr(request.state, "organization_id", "org-default")
```

### Enhanced Chat Endpoint

```python
# app/routers/chat.py
from fastapi import APIRouter, Depends, HTTPException, Request
from llm_cost_ops import CostOpsClient
from pydantic import BaseModel
import openai
from ..dependencies import get_cost_ops_client, get_organization_id

router = APIRouter(prefix="/api", tags=["chat"])

class ChatRequest(BaseModel):
    message: str
    model: str = "gpt-4"
    max_tokens: int = 1000

class ChatResponse(BaseModel):
    response: str
    usage: dict
    estimated_cost: float

@router.post("/chat", response_model=ChatResponse)
async def chat(
    request: Request,
    chat_request: ChatRequest,
    cost_ops: CostOpsClient = Depends(get_cost_ops_client),
    org_id: str = Depends(get_organization_id)
):
    """
    Chat with LLM and automatically track costs.

    This endpoint:
    - Calls OpenAI/Anthropic based on model
    - Tracks token usage
    - Submits usage to LLM-CostOps
    - Returns estimated cost
    """

    try:
        # Call OpenAI
        response = openai.ChatCompletion.create(
            model=chat_request.model,
            messages=[{"role": "user", "content": chat_request.message}],
            max_tokens=chat_request.max_tokens
        )

        # Extract usage
        usage = {
            "input_tokens": response.usage.prompt_tokens,
            "output_tokens": response.usage.completion_tokens,
            "total_tokens": response.usage.total_tokens
        }

        # Track usage in LLM-CostOps
        cost_result = await cost_ops.usage.submit_async(
            organization_id=org_id,
            provider="openai",
            model_id=chat_request.model,
            input_tokens=usage["input_tokens"],
            output_tokens=usage["output_tokens"],
            total_tokens=usage["total_tokens"],
            metadata={
                "endpoint": "/api/chat",
                "user_id": getattr(request.state, "user_id", None),
                "max_tokens": chat_request.max_tokens
            }
        )

        return ChatResponse(
            response=response.choices[0].message.content,
            usage=usage,
            estimated_cost=float(cost_result.estimated_cost)
        )

    except openai.error.OpenAIError as e:
        raise HTTPException(status_code=500, detail=f"OpenAI error: {str(e)}")
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Error: {str(e)}")
```

### Cost Analytics Endpoint

```python
# app/routers/analytics.py
from fastapi import APIRouter, Depends, Query
from llm_cost_ops import CostOpsClient
from datetime import datetime, timedelta
from ..dependencies import get_cost_ops_client, get_organization_id

router = APIRouter(prefix="/api/analytics", tags=["analytics"])

@router.get("/costs")
async def get_costs(
    days: int = Query(7, ge=1, le=90, description="Number of days to analyze"),
    group_by: str = Query("day", regex="^(day|week|month|provider|model)$"),
    cost_ops: CostOpsClient = Depends(get_cost_ops_client),
    org_id: str = Depends(get_organization_id)
):
    """Get cost analytics for the organization."""

    end_date = datetime.utcnow()
    start_date = end_date - timedelta(days=days)

    costs = await cost_ops.costs.get_async(
        organization_id=org_id,
        start_date=start_date,
        end_date=end_date,
        group_by=group_by
    )

    return {
        "period": {
            "start": start_date.isoformat(),
            "end": end_date.isoformat(),
            "days": days
        },
        "total_cost": float(costs.total_cost),
        "total_tokens": costs.total_tokens,
        "total_requests": costs.total_requests,
        "average_cost_per_request": float(costs.total_cost / max(costs.total_requests, 1)),
        "breakdown": [
            {
                "dimension": item.dimension,
                "value": item.value,
                "cost": float(item.cost),
                "tokens": item.tokens,
                "requests": item.requests
            }
            for item in costs.breakdown
        ]
    }

@router.get("/budget-status")
async def get_budget_status(
    cost_ops: CostOpsClient = Depends(get_cost_ops_client),
    org_id: str = Depends(get_organization_id)
):
    """Get current budget utilization."""

    # Get current month costs
    start_of_month = datetime.utcnow().replace(day=1, hour=0, minute=0, second=0)
    current_costs = await cost_ops.costs.get_async(
        organization_id=org_id,
        start_date=start_of_month,
        end_date=datetime.utcnow()
    )

    # Get budget (example: $500/month)
    monthly_budget = 500.00
    utilization = (float(current_costs.total_cost) / monthly_budget) * 100

    # Calculate daily average and projected month-end
    days_in_month = (datetime.utcnow() - start_of_month).days + 1
    daily_average = float(current_costs.total_cost) / days_in_month
    days_remaining = 30 - days_in_month
    projected_total = float(current_costs.total_cost) + (daily_average * days_remaining)

    return {
        "current_spend": float(current_costs.total_cost),
        "monthly_budget": monthly_budget,
        "utilization_percentage": round(utilization, 2),
        "remaining_budget": monthly_budget - float(current_costs.total_cost),
        "daily_average": round(daily_average, 2),
        "projected_month_end": round(projected_total, 2),
        "on_track": projected_total <= monthly_budget,
        "status": "healthy" if utilization < 80 else "warning" if utilization < 100 else "critical"
    }
```

## Multi-Provider Support

### Universal LLM Client

```python
# app/services/llm_client.py
from llm_cost_ops import CostOpsClient
from typing import Dict, Any
import openai
import anthropic

class UniversalLLMClient:
    """Universal LLM client with automatic cost tracking."""

    def __init__(self, cost_ops: CostOpsClient, organization_id: str):
        self.cost_ops = cost_ops
        self.organization_id = organization_id
        self.openai_client = openai.ChatCompletion
        self.anthropic_client = anthropic.Anthropic()

    async def chat(
        self,
        provider: str,
        model: str,
        messages: list,
        max_tokens: int = 1000,
        metadata: Dict[str, Any] = None
    ) -> Dict[str, Any]:
        """
        Universal chat interface with automatic cost tracking.

        Supports: OpenAI, Anthropic, and more.
        """

        if provider == "openai":
            return await self._chat_openai(model, messages, max_tokens, metadata)
        elif provider == "anthropic":
            return await self._chat_anthropic(model, messages, max_tokens, metadata)
        else:
            raise ValueError(f"Unsupported provider: {provider}")

    async def _chat_openai(
        self,
        model: str,
        messages: list,
        max_tokens: int,
        metadata: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Chat with OpenAI and track costs."""

        response = self.openai_client.create(
            model=model,
            messages=messages,
            max_tokens=max_tokens
        )

        # Track usage
        await self.cost_ops.usage.submit_async(
            organization_id=self.organization_id,
            provider="openai",
            model_id=model,
            input_tokens=response.usage.prompt_tokens,
            output_tokens=response.usage.completion_tokens,
            total_tokens=response.usage.total_tokens,
            metadata=metadata or {}
        )

        return {
            "content": response.choices[0].message.content,
            "usage": {
                "input_tokens": response.usage.prompt_tokens,
                "output_tokens": response.usage.completion_tokens,
                "total_tokens": response.usage.total_tokens
            },
            "provider": "openai",
            "model": model
        }

    async def _chat_anthropic(
        self,
        model: str,
        messages: list,
        max_tokens: int,
        metadata: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Chat with Anthropic and track costs."""

        # Convert messages format
        system = next((m["content"] for m in messages if m["role"] == "system"), None)
        user_messages = [m for m in messages if m["role"] != "system"]

        response = self.anthropic_client.messages.create(
            model=model,
            max_tokens=max_tokens,
            system=system,
            messages=user_messages
        )

        # Track usage
        await self.cost_ops.usage.submit_async(
            organization_id=self.organization_id,
            provider="anthropic",
            model_id=model,
            input_tokens=response.usage.input_tokens,
            output_tokens=response.usage.output_tokens,
            total_tokens=response.usage.input_tokens + response.usage.output_tokens,
            metadata=metadata or {}
        )

        return {
            "content": response.content[0].text,
            "usage": {
                "input_tokens": response.usage.input_tokens,
                "output_tokens": response.usage.output_tokens,
                "total_tokens": response.usage.input_tokens + response.usage.output_tokens
            },
            "provider": "anthropic",
            "model": model
        }
```

### Using the Universal Client

```python
# app/routers/chat_universal.py
from fastapi import APIRouter, Depends
from pydantic import BaseModel
from ..services.llm_client import UniversalLLMClient
from ..dependencies import get_cost_ops_client, get_organization_id

router = APIRouter(prefix="/api", tags=["chat"])

class UniversalChatRequest(BaseModel):
    provider: str  # "openai" or "anthropic"
    model: str
    message: str
    max_tokens: int = 1000

@router.post("/chat/universal")
async def universal_chat(
    request: UniversalChatRequest,
    cost_ops = Depends(get_cost_ops_client),
    org_id: str = Depends(get_organization_id)
):
    """
    Universal chat endpoint supporting multiple providers.

    Automatically tracks costs for any supported provider.
    """

    llm_client = UniversalLLMClient(cost_ops, org_id)

    result = await llm_client.chat(
        provider=request.provider,
        model=request.model,
        messages=[{"role": "user", "content": request.message}],
        max_tokens=request.max_tokens,
        metadata={"endpoint": "/api/chat/universal"}
    )

    return result
```

## Background Task for Cost Tracking

```python
# app/tasks/cost_tracking.py
from fastapi import BackgroundTasks
from llm_cost_ops import CostOpsClient

def track_usage_background(
    cost_ops: CostOpsClient,
    organization_id: str,
    provider: str,
    model_id: str,
    input_tokens: int,
    output_tokens: int,
    metadata: dict = None
):
    """Background task to track usage without blocking the response."""

    cost_ops.usage.submit(
        organization_id=organization_id,
        provider=provider,
        model_id=model_id,
        input_tokens=input_tokens,
        output_tokens=output_tokens,
        total_tokens=input_tokens + output_tokens,
        metadata=metadata or {}
    )

# Usage in endpoint
@app.post("/api/chat-fast")
async def chat_fast(
    message: str,
    background_tasks: BackgroundTasks,
    cost_ops = Depends(get_cost_ops_client),
    org_id: str = Depends(get_organization_id)
):
    """Chat endpoint with background cost tracking for faster responses."""

    response = openai.ChatCompletion.create(
        model="gpt-4",
        messages=[{"role": "user", "content": message}]
    )

    # Track usage in background
    background_tasks.add_task(
        track_usage_background,
        cost_ops=cost_ops,
        organization_id=org_id,
        provider="openai",
        model_id="gpt-4",
        input_tokens=response.usage.prompt_tokens,
        output_tokens=response.usage.completion_tokens,
        metadata={"endpoint": "/api/chat-fast"}
    )

    return {"response": response.choices[0].message.content}
```

## Complete Example Application

```python
# app/main.py - Complete FastAPI application with cost tracking

from fastapi import FastAPI, Depends, HTTPException, Request, BackgroundTasks
from llm_cost_ops import CostOpsClient
from pydantic import BaseModel
from typing import Optional, Dict, Any
import os
import openai

# Initialize FastAPI
app = FastAPI(
    title="LLM Application",
    description="FastAPI application with automatic LLM cost tracking",
    version="1.0.0"
)

# Initialize clients
cost_ops_client = CostOpsClient(api_key=os.getenv("LLM_COST_OPS_API_KEY"))
openai.api_key = os.getenv("OPENAI_API_KEY")

# Request/Response models
class ChatRequest(BaseModel):
    message: str
    model: str = "gpt-4"
    max_tokens: int = 1000
    user_id: Optional[str] = None

class ChatResponse(BaseModel):
    response: str
    usage: Dict[str, int]
    estimated_cost: float
    provider: str
    model: str

# Endpoints
@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {"status": "healthy", "service": "llm-api"}

@app.post("/api/v1/chat", response_model=ChatResponse)
async def chat(
    request: ChatRequest,
    background_tasks: BackgroundTasks
):
    """
    Chat endpoint with automatic cost tracking.

    - Calls OpenAI GPT models
    - Tracks token usage
    - Submits usage to LLM-CostOps in background
    - Returns response with estimated cost
    """

    try:
        # Call OpenAI
        response = openai.ChatCompletion.create(
            model=request.model,
            messages=[{"role": "user", "content": request.message}],
            max_tokens=request.max_tokens
        )

        usage = {
            "input_tokens": response.usage.prompt_tokens,
            "output_tokens": response.usage.completion_tokens,
            "total_tokens": response.usage.total_tokens
        }

        # Track usage in background
        background_tasks.add_task(
            cost_ops_client.usage.submit,
            organization_id=os.getenv("ORGANIZATION_ID", "org-default"),
            provider="openai",
            model_id=request.model,
            input_tokens=usage["input_tokens"],
            output_tokens=usage["output_tokens"],
            total_tokens=usage["total_tokens"],
            metadata={
                "endpoint": "/api/v1/chat",
                "user_id": request.user_id,
                "max_tokens": request.max_tokens
            }
        )

        # Estimate cost (rough calculation)
        # GPT-4: $0.01 input, $0.03 output per 1K tokens
        estimated_cost = (
            (usage["input_tokens"] / 1000 * 0.01) +
            (usage["output_tokens"] / 1000 * 0.03)
        )

        return ChatResponse(
            response=response.choices[0].message.content,
            usage=usage,
            estimated_cost=round(estimated_cost, 6),
            provider="openai",
            model=request.model
        )

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/v1/costs")
async def get_costs(days: int = 7):
    """Get cost analytics for the last N days."""

    from datetime import datetime, timedelta

    end_date = datetime.utcnow()
    start_date = end_date - timedelta(days=days)

    costs = cost_ops_client.costs.get(
        organization_id=os.getenv("ORGANIZATION_ID", "org-default"),
        start_date=start_date,
        end_date=end_date,
        group_by="day"
    )

    return {
        "total_cost": float(costs.total_cost),
        "total_tokens": costs.total_tokens,
        "total_requests": costs.total_requests,
        "breakdown": [
            {
                "date": item.value,
                "cost": float(item.cost),
                "tokens": item.tokens,
                "requests": item.requests
            }
            for item in costs.breakdown
        ]
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
```

## Environment Variables

```bash
# .env
LLM_COST_OPS_API_KEY=your_api_key_here
ORGANIZATION_ID=org-123
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key
```

## Testing

```python
# tests/test_chat.py
from fastapi.testclient import TestClient
from app.main import app

client = TestClient(app)

def test_chat_endpoint():
    response = client.post(
        "/api/v1/chat",
        json={
            "message": "Hello, world!",
            "model": "gpt-4",
            "max_tokens": 100
        }
    )

    assert response.status_code == 200
    data = response.json()
    assert "response" in data
    assert "usage" in data
    assert "estimated_cost" in data
    assert data["estimated_cost"] > 0

def test_costs_endpoint():
    response = client.get("/api/v1/costs?days=7")

    assert response.status_code == 200
    data = response.json()
    assert "total_cost" in data
    assert "total_tokens" in data
```

## Next Steps

- [Django Integration](django.md)
- [Flask Integration](flask.md)
- [React Integration](react.md)
- [Cost Optimization Guide](../guides/cost-optimization.md)
