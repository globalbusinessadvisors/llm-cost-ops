# Video 03: Python SDK Deep Dive

## Metadata

- **Duration**: 22-25 minutes
- **Level**: Intermediate
- **Prerequisites**: Videos 01, 02
- **Target Audience**: Python developers building LLM applications
- **Video ID**: LLMCO-V03-PYTHON
- **Version**: 1.0.0

## Learning Objectives

By the end of this video, viewers will be able to:
- Use advanced Python SDK features for cost tracking
- Implement batch tracking for multiple requests
- Handle errors and retries gracefully
- Use custom pricing models for private LLM deployments
- Implement caching strategies to reduce costs
- Track async operations with asyncio
- Integrate with popular frameworks (LangChain, LlamaIndex)
- Optimize SDK performance for production

## Equipment/Software Needed

### Recording
- Screen recording software (1920x1080, 30fps)
- Professional microphone
- VS Code with Python extensions

### Demonstration Environment
- Python 3.9+ with virtual environment
- LLM Cost Ops instance running (from Video 02)
- OpenAI and Anthropic API keys
- Code examples repository
- Jupyter Notebook (optional for demonstrations)

## Scene Breakdown

### Scene 1: Opening & Overview
**Duration**: 0:00-1:00

**Visual**:
- Animated intro
- Code editor showing Python SDK logo
- Overview of topics to cover

**Narration**:
"Welcome back! In this video, we're doing a deep dive into the LLM Cost Ops Python SDK. If you're building AI applications in Python, this is your complete guide to advanced cost tracking.

We'll cover batch operations, async support, error handling, custom pricing, caching strategies, and integration with popular frameworks like LangChain. By the end, you'll know everything you need to track costs effectively in production Python applications.

Let's get started!"

**On-Screen Text**:
- "Python SDK Deep Dive"
- "Topics:"
  - "Batch Tracking"
  - "Async Operations"
  - "Error Handling"
  - "Custom Pricing"
  - "Caching Strategies"
  - "Framework Integration"

**Transition**: Fade to code editor

---

### Scene 2: Advanced Configuration
**Duration**: 1:00-3:30

**Visual**:
- VS Code with Python file
- Configuration object being built
- Documentation sidebar reference

**Narration**:
"Let's start with advanced configuration. In the Getting Started video, we used a simple initialization. But the SDK supports much more sophisticated setup.

Here's a production-ready configuration. We're setting up automatic retries with exponential backoff—if the tracker endpoint is temporarily unavailable, it'll retry with increasing delays.

Buffering allows the SDK to batch requests for better performance. Instead of sending each track event immediately, it buffers them and sends in batches of 100 or every 5 seconds, whichever comes first.

The timeout ensures we never block your application—if tracking takes too long, it'll fail gracefully without impacting your LLM call.

Debug mode is useful during development. It logs all tracking attempts so you can troubleshoot issues."

**On-Screen Text**:
- "Production Configuration"
- Key features highlighted:
  - "Automatic retries"
  - "Request buffering"
  - "Timeout protection"
  - "Debug logging"

**Code/Demo**:
```python
from llm_cost_ops import CostTracker, TrackerConfig
import logging

# Configure logging for debug mode
logging.basicConfig(level=logging.DEBUG)

# Advanced configuration
config = TrackerConfig(
    api_key=os.getenv("LCOPS_API_KEY"),
    endpoint=os.getenv("LCOPS_ENDPOINT"),

    # Retry configuration
    max_retries=3,
    retry_delay=1.0,  # seconds
    retry_backoff=2.0,  # exponential backoff multiplier

    # Buffering for performance
    buffer_size=100,  # batch size
    buffer_timeout=5.0,  # flush every 5 seconds

    # Timeout protection
    timeout=10.0,  # seconds

    # Debug mode
    debug=True,

    # Default tags applied to all requests
    default_tags={
        "application": "my-app",
        "environment": os.getenv("ENV", "development"),
        "version": "1.0.0"
    }
)

tracker = CostTracker(config)
```

**Highlight Callouts**:
- "Retries prevent data loss"
- "Buffering improves performance"
- "Never blocks your app"

**Transition**: Scroll to next code section

---

### Scene 3: Batch Tracking
**Duration**: 3:30-6:00

**Visual**:
- Code editor showing batch operations
- Terminal output showing execution
- Dashboard showing batch results

**Narration**:
"When you need to track multiple LLM requests at once, use batch tracking. This is common when processing lists of items or running bulk operations.

Here's an example: we're summarizing a list of articles. Instead of tracking each one individually, we collect all the requests and track them as a batch.

The track_batch method accepts a list of request metadata. Notice we're using a context manager—this ensures the batch is flushed even if an error occurs.

Watch the terminal as we run this. All five articles are summarized, and then the tracking data is sent in one efficient batch request.

Now look at the dashboard—there's our batch, all requests grouped together with batch metadata. This is much more efficient than individual tracking for bulk operations."

**On-Screen Text**:
- "Batch Tracking Benefits:"
  - "Reduced network overhead"
  - "Better performance"
  - "Grouped analytics"
  - "Atomic operations"

**Code/Demo**:
```python
from llm_cost_ops import CostTracker
from openai import OpenAI

tracker = CostTracker(api_key=os.getenv("LCOPS_API_KEY"))
client = OpenAI()

articles = [
    "Article 1 content...",
    "Article 2 content...",
    "Article 3 content...",
    # ... more articles
]

# Batch tracking with context manager
with tracker.batch_context(tags={"operation": "article_summarization"}) as batch:
    for idx, article in enumerate(articles):
        response = client.chat.completions.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": f"Summarize: {article}"}
            ]
        )

        # Add to batch
        batch.track(
            response,
            tags={
                "article_id": f"article_{idx}",
                "batch_position": idx
            }
        )

        print(f"Summarized article {idx + 1}")

print(f"Batch complete! Tracked {len(articles)} requests")
```

**Alternative approach:**
```python
# Or use the batch decorator
@tracker.track_batch(tags={"operation": "bulk_classification"})
def classify_texts(texts: list[str]) -> list[str]:
    results = []
    for text in texts:
        response = client.chat.completions.create(
            model="gpt-3.5-turbo",
            messages=[{"role": "user", "content": f"Classify: {text}"}]
        )
        results.append(response.choices[0].message.content)
    return results

# Automatically tracks all requests in the function
classifications = classify_texts(my_text_list)
```

**Highlight Callouts**:
- "Context manager ensures cleanup"
- "Decorator for automatic tracking"
- "All requests grouped in dashboard"

**Transition**: New file opens

---

### Scene 4: Async Operations
**Duration**: 6:00-9:00

**Visual**:
- New Python file for async code
- Terminal showing concurrent execution
- Performance comparison graphs

**Narration**:
"Modern Python applications use asyncio for concurrent operations. The SDK fully supports async tracking with zero overhead.

Here's an async example using the OpenAI async client. Notice we're using 'async def' and 'await'. The tracker's async methods mirror the sync API—track_async for single requests, track_batch_async for batches.

Let's run a performance comparison. First, we'll make 10 requests synchronously, then 10 requests concurrently with asyncio.

[Run sync version]

The sync version took about 15 seconds—each request waits for the previous one.

[Run async version]

The async version? Just 2 seconds. All requests ran concurrently, and all were tracked correctly. Check the dashboard—all 20 requests are there, properly categorized.

This is crucial for production applications where performance matters. Async tracking adds virtually no overhead to your concurrent operations."

**On-Screen Text**:
- "Async Support:"
  - "Full asyncio integration"
  - "Concurrent request tracking"
  - "Zero overhead"
  - "Batch async operations"

**Code/Demo**:
```python
import asyncio
from llm_cost_ops import AsyncCostTracker
from openai import AsyncOpenAI

# Async tracker
tracker = AsyncCostTracker(api_key=os.getenv("LCOPS_API_KEY"))
client = AsyncOpenAI()

async def process_request(prompt: str, request_id: int):
    """Process single LLM request with tracking."""
    response = await tracker.track_async(
        client.chat.completions.create(
            model="gpt-3.5-turbo",
            messages=[{"role": "user", "content": prompt}]
        ),
        tags={
            "request_id": request_id,
            "operation": "async_processing"
        }
    )
    return response.choices[0].message.content

async def main():
    # Concurrent processing
    prompts = [f"Explain concept {i}" for i in range(10)]

    # Process all concurrently
    tasks = [
        process_request(prompt, idx)
        for idx, prompt in enumerate(prompts)
    ]

    results = await asyncio.gather(*tasks)
    print(f"Processed {len(results)} requests concurrently")

# Run
asyncio.run(main())
```

**Performance comparison:**
```python
import time

# Sync version
start = time.time()
for i in range(10):
    response = tracker.track(
        client.chat.completions.create(...),
        tags={"mode": "sync"}
    )
sync_time = time.time() - start
print(f"Sync: {sync_time:.2f}s")

# Async version
start = time.time()
await asyncio.gather(*[process_request(...) for i in range(10)])
async_time = time.time() - start
print(f"Async: {async_time:.2f}s")
print(f"Speedup: {sync_time/async_time:.1f}x")
```

**Highlight Callouts**:
- "7x faster with async"
- "All requests tracked accurately"
- "Production-ready concurrency"

**Transition**: New code file

---

### Scene 5: Error Handling & Resilience
**Duration**: 9:00-11:30

**Visual**:
- Code showing error scenarios
- Try/except blocks
- Logs showing graceful degradation

**Narration**:
"In production, things go wrong. Networks fail, APIs have errors, services go down. Your cost tracking should never break your application.

The SDK is designed to fail gracefully. If tracking fails, your LLM call still succeeds. Watch this example where we simulate a network error.

[Run code with tracker endpoint unavailable]

See? The LLM request completed successfully, and we got our response. The tracker logged a warning about the failed tracking, but it didn't crash our application.

You can configure how to handle failures. The 'on_error' callback lets you log to your monitoring system, queue for retry, or take custom action.

For critical tracking where you need guaranteed delivery, enable 'strict mode'. This will raise exceptions if tracking fails, allowing you to implement your own retry logic or fallback behavior."

**On-Screen Text**:
- "Error Handling:"
  - "Graceful degradation"
  - "Never blocks app"
  - "Custom error handlers"
  - "Strict mode option"

**Code/Demo**:
```python
from llm_cost_ops import CostTracker, TrackingError
import logging

logger = logging.getLogger(__name__)

# Custom error handler
def handle_tracking_error(error: Exception, request_data: dict):
    """Custom handling for tracking failures."""
    logger.error(f"Tracking failed: {error}")
    # Could send to error monitoring service
    # Could queue for retry
    # Could store locally for batch upload later

tracker = CostTracker(
    api_key=os.getenv("LCOPS_API_KEY"),
    on_error=handle_tracking_error,
    strict=False  # Don't raise exceptions, just log
)

# Your code continues to work even if tracking fails
try:
    response = tracker.track(
        client.chat.completions.create(
            model="gpt-4",
            messages=[{"role": "user", "content": "Hello"}]
        ),
        tags={"critical": "true"}
    )
    print("Success:", response.choices[0].message.content)
except TrackingError as e:
    # Only raised in strict mode
    logger.error(f"Tracking failed critically: {e}")
    # Implement retry or fallback
```

**Strict mode example:**
```python
# Strict mode for critical tracking
strict_tracker = CostTracker(
    api_key=os.getenv("LCOPS_API_KEY"),
    strict=True  # Raise exceptions on tracking failure
)

try:
    response = strict_tracker.track(llm_call, tags={"critical": "true"})
except TrackingError:
    # Retry logic
    time.sleep(1)
    response = strict_tracker.track(llm_call, tags={"critical": "true"})
```

**Highlight Callouts**:
- "App always continues working"
- "Custom error handlers"
- "Strict mode for guarantees"

**Transition**: Split screen: code + dashboard

---

### Scene 6: Custom Pricing Models
**Duration**: 11:30-14:00

**Visual**:
- Code editor showing pricing configuration
- Dashboard showing custom model costs
- Pricing comparison tables

**Narration**:
"What if you're using a private LLM deployment or a provider not supported out of the box? You can define custom pricing models.

Here's an example: you've deployed Llama 2 on your own infrastructure. You know your hourly costs and average throughput. Let's configure custom pricing.

We define a pricing model with input and output token costs. For self-hosted models, you might calculate this based on GPU costs divided by expected tokens per hour.

The SDK will use this pricing for cost calculations. You can even define different pricing for different tiers—maybe you offer a free tier and a paid tier with different underlying models.

Look at the dashboard now—our custom model appears in the provider list with accurate cost tracking based on our pricing configuration."

**On-Screen Text**:
- "Custom Pricing:"
  - "Self-hosted models"
  - "Private deployments"
  - "Unsupported providers"
  - "Tiered pricing"

**Code/Demo**:
```python
from llm_cost_ops import CostTracker, PricingModel

# Define custom pricing for self-hosted Llama 2
llama2_pricing = PricingModel(
    provider="self-hosted",
    model="llama-2-70b",
    input_token_cost=0.00001,  # $0.01 per 1K tokens
    output_token_cost=0.00002,  # $0.02 per 1K tokens
    currency="USD"
)

tracker = CostTracker(
    api_key=os.getenv("LCOPS_API_KEY"),
    custom_pricing=[llama2_pricing]
)

# Track request with custom pricing
response = tracker.track(
    your_llm_call,
    provider="self-hosted",
    model="llama-2-70b",
    tags={"deployment": "on-prem"}
)
```

**Complex example with tiered pricing:**
```python
# Different pricing for different customer tiers
free_tier_pricing = PricingModel(
    provider="custom",
    model="gpt-3.5-turbo-free",
    input_token_cost=0.0,  # Free for users
    output_token_cost=0.0,
    internal_cost=0.0015,  # But we pay OpenAI
    metadata={"tier": "free"}
)

paid_tier_pricing = PricingModel(
    provider="custom",
    model="gpt-4-paid",
    input_token_cost=0.03,  # Charge users
    output_token_cost=0.06,
    internal_cost=0.01,  # We pay OpenAI
    markup=3.0,  # 3x markup
    metadata={"tier": "paid"}
)

tracker = CostTracker(
    custom_pricing=[free_tier_pricing, paid_tier_pricing]
)

# Track with user tier
def handle_request(user: User, prompt: str):
    model = "gpt-4-paid" if user.is_premium else "gpt-3.5-turbo-free"

    response = tracker.track(
        llm_call,
        model=model,
        tags={"user_tier": user.tier}
    )
    return response
```

**Highlight Callouts**:
- "Full pricing flexibility"
- "Track internal vs. customer costs"
- "Support any model"

**Transition**: New file with caching code

---

### Scene 7: Caching Strategies
**Duration**: 14:00-17:00

**Visual**:
- Code showing cache implementation
- Cache hit/miss metrics
- Cost savings calculations

**Narration**:
"One of the best ways to reduce LLM costs is caching. If you're seeing repeated queries, why pay for the same completion twice?

The SDK supports automatic cache integration. Here's an example using Redis as a cache backend. Before making an LLM call, we check the cache. On a hit, we return the cached result and mark it as a cached request for tracking purposes.

Watch this example run. The first request takes 2 seconds and costs $0.03. The second identical request? Instant, and essentially free—just the cache lookup cost.

The dashboard shows cache hits separately, so you can measure your cache effectiveness. Look at this cost savings report—by caching 60% of requests, we've reduced our monthly bill from $10,000 to $4,000.

The SDK also supports semantic caching, where similar queries return cached results. This is perfect for customer support bots where questions are phrased differently but mean the same thing."

**On-Screen Text**:
- "Caching Strategies:"
  - "Exact match caching"
  - "Semantic similarity caching"
  - "Cache hit tracking"
  - "Cost savings metrics"

**Code/Demo**:
```python
from llm_cost_ops import CostTracker
import redis
import hashlib
import json

tracker = CostTracker(api_key=os.getenv("LCOPS_API_KEY"))
cache = redis.Redis(host='localhost', port=6379, decode_responses=True)

def cached_llm_call(prompt: str, model: str = "gpt-3.5-turbo"):
    """LLM call with caching."""
    # Create cache key
    cache_key = hashlib.sha256(
        f"{model}:{prompt}".encode()
    ).hexdigest()

    # Check cache
    cached = cache.get(cache_key)
    if cached:
        print("Cache hit!")
        result = json.loads(cached)

        # Track as cached request (no cost)
        tracker.track_cached(
            provider="openai",
            model=model,
            cached_response=result,
            tags={"cache_hit": "true"}
        )

        return result

    # Cache miss - make actual LLM call
    print("Cache miss - calling LLM")
    response = tracker.track(
        client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": prompt}]
        ),
        tags={"cache_hit": "false"}
    )

    # Store in cache (24 hour TTL)
    result = response.choices[0].message.content
    cache.setex(cache_key, 86400, json.dumps(result))

    return result

# Usage
result1 = cached_llm_call("What is Python?")  # Cache miss
result2 = cached_llm_call("What is Python?")  # Cache hit!
```

**Semantic caching example:**
```python
from llm_cost_ops.caching import SemanticCache
from sentence_transformers import SentenceTransformer

# Semantic cache using embeddings
semantic_cache = SemanticCache(
    embedding_model=SentenceTransformer('all-MiniLM-L6-v2'),
    similarity_threshold=0.85,  # 85% similar = cache hit
    redis_client=cache
)

def semantic_cached_call(prompt: str):
    # Check for semantically similar prompts
    cached_result = semantic_cache.get(prompt)
    if cached_result:
        tracker.track_cached(
            cached_response=cached_result,
            tags={"cache_type": "semantic"}
        )
        return cached_result

    # Make LLM call and cache
    response = tracker.track(llm_call(prompt))
    semantic_cache.set(prompt, response)
    return response

# These will hit the cache even though worded differently
semantic_cached_call("How do I learn Python?")
semantic_cached_call("What's the best way to study Python?")  # Hit!
```

**Highlight Callouts**:
- "60% cache hit rate = 60% cost savings"
- "Semantic caching for variations"
- "Full cache analytics in dashboard"

**Transition**: Framework logos appear

---

### Scene 8: LangChain Integration
**Duration**: 17:00-19:30

**Visual**:
- LangChain code examples
- Chain execution visualization
- Tracking of multi-step chains

**Narration**:
"If you're using LangChain, integration is seamless. LangChain is one of the most popular frameworks for building LLM applications, and LLM Cost Ops has first-class support.

The SDK provides a LangChain callback handler that automatically tracks all LLM calls in your chains. Install the LangChain extras package, and you get automatic tracking with zero code changes to your chains.

Watch this example. We've built a multi-step chain: first it does research, then writes a summary, then critiques the summary. Three separate LLM calls in one chain.

With the callback handler, all three calls are automatically tracked, grouped together as part of the same chain execution. Look at the dashboard—we can see the cost breakdown for each step and the total chain cost.

This works with agents too. Complex agent executions with multiple tools and LLM calls—all tracked automatically."

**On-Screen Text**:
- "LangChain Integration:"
  - "Automatic tracking"
  - "Chain execution grouping"
  - "Agent support"
  - "Zero code changes"

**Code/Demo**:
```python
from llm_cost_ops.integrations import LangChainCallback
from langchain.chains import LLMChain, SequentialChain
from langchain.llms import OpenAI
from langchain.prompts import PromptTemplate

# Initialize callback
callback = LangChainCallback(
    tracker=tracker,
    tags={
        "framework": "langchain",
        "chain_type": "sequential"
    }
)

# Build chain (normal LangChain code)
research_template = PromptTemplate(
    input_variables=["topic"],
    template="Research this topic: {topic}"
)
research_chain = LLMChain(
    llm=OpenAI(model="gpt-3.5-turbo"),
    prompt=research_template,
    output_key="research"
)

summary_template = PromptTemplate(
    input_variables=["research"],
    template="Summarize: {research}"
)
summary_chain = LLMChain(
    llm=OpenAI(model="gpt-4"),
    prompt=summary_template,
    output_key="summary"
)

# Combine into sequential chain
overall_chain = SequentialChain(
    chains=[research_chain, summary_chain],
    input_variables=["topic"],
    output_variables=["research", "summary"]
)

# Run with tracking - just add callbacks parameter
result = overall_chain(
    {"topic": "quantum computing"},
    callbacks=[callback]
)

print("Chain complete! Check dashboard for detailed cost breakdown")
```

**Agent example:**
```python
from langchain.agents import initialize_agent, Tool
from langchain.agents import AgentType

# Define tools
tools = [
    Tool(name="Search", func=search_tool, description="..."),
    Tool(name="Calculator", func=calc_tool, description="..."),
]

# Initialize agent with callback
agent = initialize_agent(
    tools=tools,
    llm=OpenAI(model="gpt-4"),
    agent=AgentType.ZERO_SHOT_REACT_DESCRIPTION,
    callbacks=[callback],  # Automatic tracking!
    verbose=True
)

# Run - all LLM calls tracked automatically
result = agent.run("What is 15% of the population of France?")
```

**Highlight Callouts**:
- "Just add callbacks parameter"
- "All chain steps tracked"
- "Works with agents and tools"

**Transition**: LlamaIndex logo

---

### Scene 9: LlamaIndex Integration
**Duration**: 19:30-21:00

**Visual**:
- LlamaIndex code examples
- Query engine execution
- RAG pipeline tracking

**Narration**:
"LlamaIndex is another popular framework, especially for RAG applications. The integration is equally simple.

LlamaIndex uses a similar callback pattern. We provide a callback handler that tracks all LLM calls in your query engines, including the retrieval and generation steps.

This RAG example shows a typical LlamaIndex workflow: index creation, query engine setup, and querying. With our callback, every LLM call is tracked—the embedding generation, the query planning, and the final answer generation.

The dashboard breaks down costs by RAG stage. You can see how much you're spending on retrieval versus generation, which helps optimize your RAG pipeline."

**On-Screen Text**:
- "LlamaIndex Integration:"
  - "RAG pipeline tracking"
  - "Embedding cost tracking"
  - "Query engine support"
  - "Stage-level breakdown"

**Code/Demo**:
```python
from llm_cost_ops.integrations import LlamaIndexCallback
from llama_index import VectorStoreIndex, SimpleDirectoryReader
from llama_index.llms import OpenAI

# Initialize callback
callback = LlamaIndexCallback(
    tracker=tracker,
    tags={"framework": "llamaindex", "pipeline": "rag"}
)

# Load documents (normal LlamaIndex code)
documents = SimpleDirectoryReader('data').load_data()

# Create index with callback
index = VectorStoreIndex.from_documents(
    documents,
    callback_manager=callback.callback_manager
)

# Create query engine
query_engine = index.as_query_engine(
    llm=OpenAI(model="gpt-4"),
    callback_manager=callback.callback_manager
)

# Query - all costs tracked automatically
response = query_engine.query(
    "What are the key findings in the research?"
)

print(f"Answer: {response}")
print("Check dashboard for embedding + generation costs")
```

**Cost breakdown visualization:**
```python
# The dashboard shows:
# - Embedding costs: $0.05
# - Retrieval costs: $0.02
# - Generation costs: $0.30
# - Total RAG pipeline cost: $0.37
#
# You can optimize by:
# - Reducing chunk size (less embeddings)
# - Using cheaper models for initial retrieval
# - Caching frequent queries
```

**Highlight Callouts**:
- "Track entire RAG pipeline"
- "Optimize by stage"
- "Embedding costs visible"

**Transition**: Performance graphs

---

### Scene 10: Performance Optimization
**Duration**: 21:00-22:30

**Visual**:
- Performance benchmarks
- Overhead measurements
- Optimization recommendations

**Narration**:
"Let's talk performance. Cost tracking should be invisible to your users.

We've benchmarked the SDK overhead extensively. For synchronous tracking, the average overhead is under 1 millisecond. For async tracking, it's under 0.1 milliseconds.

The buffering system ensures that even in high-throughput applications, tracking doesn't impact your request rate. Buffers are flushed in background threads, keeping your main application thread free.

For extreme performance scenarios, use sampling. Track 10% of requests to get statistical accuracy with minimal overhead. The SDK uses consistent hashing, so related requests are sampled together for accurate cost attribution.

Here's a benchmark running 10,000 requests. Without tracking: 45 seconds. With tracking: 45.3 seconds. That's 0.6% overhead—barely measurable."

**On-Screen Text**:
- "Performance:"
  - "< 1ms sync overhead"
  - "< 0.1ms async overhead"
  - "Background buffering"
  - "Sampling for scale"

**Code/Demo**:
```python
import time
from llm_cost_ops import CostTracker

# Performance benchmark
def benchmark(use_tracking=False):
    tracker = CostTracker(...) if use_tracking else None

    start = time.time()
    for i in range(1000):
        response = client.chat.completions.create(...)
        if use_tracking:
            tracker.track(response)

    return time.time() - start

without_tracking = benchmark(False)
with_tracking = benchmark(True)

overhead = ((with_tracking - without_tracking) / without_tracking) * 100
print(f"Overhead: {overhead:.2f}%")  # Output: ~0.5%
```

**Sampling configuration:**
```python
# High-throughput configuration with sampling
high_perf_tracker = CostTracker(
    sampling_rate=0.1,  # Track 10% of requests
    buffer_size=1000,  # Large buffer
    buffer_timeout=30.0,  # Flush every 30 seconds
    async_mode=True  # Background thread
)
```

**Transition**: Recap screen

---

### Scene 11: Recap & Next Steps
**Duration**: 22:30-23:30

**Visual**:
- Summary of all features covered
- Code snippets highlight reel
- Links to resources

**Narration**:
"That's a wrap on the Python SDK deep dive! We've covered a lot: advanced configuration, batch tracking, async operations, error handling, custom pricing, caching strategies, and framework integration.

You now have all the tools to implement production-ready cost tracking in your Python LLM applications. All the code examples from this video are available in the GitHub repository—link in the description.

Next up, if you're a TypeScript developer, check out video 04 for the TypeScript SDK guide. Or skip ahead to video 05 to learn about building custom dashboards and analytics.

Thanks for watching, and happy cost tracking!"

**On-Screen Text**:
- "Covered Today:"
  - "✅ Advanced configuration"
  - "✅ Batch & async tracking"
  - "✅ Error handling"
  - "✅ Custom pricing"
  - "✅ Caching strategies"
  - "✅ Framework integration"
- "Resources:"
  - "Code examples on GitHub"
  - "SDK documentation"
  - "Next: TypeScript SDK (Video 04)"

**Transition**: Closing screen

---

## Post-Production Notes

### Chapter Markers
- 0:00 - Introduction
- 1:00 - Advanced Configuration
- 3:30 - Batch Tracking
- 6:00 - Async Operations
- 9:00 - Error Handling
- 11:30 - Custom Pricing
- 14:00 - Caching Strategies
- 17:00 - LangChain Integration
- 19:30 - LlamaIndex Integration
- 21:00 - Performance Optimization
- 22:30 - Recap

### Code Repository
Create companion repository with:
- All example code from video
- requirements.txt
- README with setup instructions
- Jupyter notebooks for interactive learning

### Graphics Needed
- Performance benchmark charts
- Cost savings visualization
- Framework integration diagrams
- Cache hit rate graphs

---

**Script Version**: 1.0.0
**Last Updated**: 2025-11-16
**Code Examples**: Available at github.com/llm-cost-ops/examples
**Estimated Production Time**: 4-5 days
