# Performance Tuning Best Practices for LLM Cost Ops

## Table of Contents

1. [Introduction](#introduction)
2. [Application-Level Optimization](#application-level-optimization)
3. [Database Query Optimization](#database-query-optimization)
4. [Connection Pooling](#connection-pooling)
5. [Caching Strategies](#caching-strategies)
6. [Load Balancing](#load-balancing)
7. [Auto-Scaling Configuration](#auto-scaling-configuration)
8. [CDN Usage](#cdn-usage-for-static-assets)
9. [Async Processing Patterns](#async-processing-patterns)
10. [Batch Operations](#batch-operations)
11. [Monitoring and Profiling](#monitoring-and-profiling)
12. [Performance Benchmarking](#performance-benchmarking)
13. [Resource Sizing](#resource-sizing)
14. [Network Optimization](#network-optimization)
15. [Implementation Checklist](#implementation-checklist)
16. [Tools and Resources](#tools-and-resources)

---

## Introduction

### Performance Goals

Optimizing LLM Cost Ops platform performance impacts:

- **User Experience**: Sub-second response times
- **Cost Efficiency**: Lower infrastructure costs
- **Scalability**: Handle 10-100x traffic spikes
- **Reliability**: 99.9%+ uptime

### Key Performance Indicators (KPIs)

```yaml
performance_targets:
  api_latency:
    p50: "< 100ms"
    p95: "< 500ms"
    p99: "< 1000ms"

  throughput:
    requests_per_second: "> 1000"
    concurrent_users: "> 10000"

  resource_utilization:
    cpu: "< 70%"
    memory: "< 80%"
    disk_io: "< 60%"

  availability:
    uptime: "> 99.9%"
    error_rate: "< 0.1%"

  llm_api_performance:
    cache_hit_rate: "> 60%"
    avg_response_time: "< 2s"
    timeout_rate: "< 0.5%"
```

---

## Application-Level Optimization

### Code Profiling and Optimization

```python
import cProfile
import pstats
import io
from functools import wraps
import time
from typing import Callable

def profile_function(func: Callable):
    """Decorator to profile function performance"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        profiler = cProfile.Profile()
        profiler.enable()

        start_time = time.time()
        result = func(*args, **kwargs)
        end_time = time.time()

        profiler.disable()

        # Print stats
        s = io.StringIO()
        ps = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
        ps.print_stats(10)  # Top 10 functions

        print(f"\n{func.__name__} took {end_time - start_time:.4f} seconds")
        print(s.getvalue())

        return result

    return wrapper

@profile_function
def process_llm_request(prompt: str, model: str) -> dict:
    """Example function to profile"""
    # Simulate processing
    result = {
        'prompt': prompt,
        'model': model,
        'response': generate_response(prompt, model)
    }
    return result

def generate_response(prompt: str, model: str) -> str:
    # Actual LLM call
    return "response"

# Memory profiling
from memory_profiler import profile as memory_profile

@memory_profile
def memory_intensive_operation():
    """Profile memory usage"""
    large_list = [i for i in range(1000000)]
    processed = [x * 2 for x in large_list]
    return sum(processed)

# Performance monitoring decorator
class PerformanceMonitor:
    """Monitor function performance metrics"""

    def __init__(self):
        self.metrics = {}

    def track(self, func: Callable):
        @wraps(func)
        def wrapper(*args, **kwargs):
            start = time.time()
            try:
                result = func(*args, **kwargs)
                success = True
            except Exception as e:
                success = False
                raise e
            finally:
                duration = time.time() - start

                func_name = func.__name__
                if func_name not in self.metrics:
                    self.metrics[func_name] = {
                        'calls': 0,
                        'total_time': 0,
                        'successes': 0,
                        'failures': 0,
                        'min_time': float('inf'),
                        'max_time': 0
                    }

                self.metrics[func_name]['calls'] += 1
                self.metrics[func_name]['total_time'] += duration
                self.metrics[func_name]['min_time'] = min(
                    self.metrics[func_name]['min_time'],
                    duration
                )
                self.metrics[func_name]['max_time'] = max(
                    self.metrics[func_name]['max_time'],
                    duration
                )

                if success:
                    self.metrics[func_name]['successes'] += 1
                else:
                    self.metrics[func_name]['failures'] += 1

            return result
        return wrapper

    def get_stats(self):
        """Get performance statistics"""
        stats = {}
        for func_name, metrics in self.metrics.items():
            stats[func_name] = {
                'avg_time': metrics['total_time'] / metrics['calls'],
                'min_time': metrics['min_time'],
                'max_time': metrics['max_time'],
                'success_rate': metrics['successes'] / metrics['calls'] * 100,
                'total_calls': metrics['calls']
            }
        return stats

# Usage
monitor = PerformanceMonitor()

@monitor.track
def api_endpoint(request_data):
    # Process request
    return process_llm_request(request_data['prompt'], request_data['model'])

# After running for a while
print(monitor.get_stats())
```

### Lazy Loading and Pagination

```typescript
interface PaginationParams {
  page: number;
  pageSize: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

class PaginatedDataLoader<T> {
  private cache: Map<number, T[]> = new Map();

  async loadPage(
    params: PaginationParams,
    fetchFn: (params: PaginationParams) => Promise<T[]>
  ): Promise<{ data: T[]; total: number; hasMore: boolean }> {
    // Check cache first
    const cacheKey = this.getCacheKey(params);
    if (this.cache.has(cacheKey)) {
      return {
        data: this.cache.get(cacheKey)!,
        total: 0, // Would need separate total tracking
        hasMore: true
      };
    }

    // Fetch data
    const data = await fetchFn(params);

    // Cache result
    this.cache.set(cacheKey, data);

    // Auto-prefetch next page
    if (data.length === params.pageSize) {
      this.prefetchNextPage(params, fetchFn);
    }

    return {
      data,
      total: 0, // Would come from database count
      hasMore: data.length === params.pageSize
    };
  }

  private async prefetchNextPage(
    params: PaginationParams,
    fetchFn: (params: PaginationParams) => Promise<T[]>
  ): Promise<void> {
    const nextParams = { ...params, page: params.page + 1 };
    const nextCacheKey = this.getCacheKey(nextParams);

    if (!this.cache.has(nextCacheKey)) {
      // Prefetch in background
      fetchFn(nextParams).then(data => {
        this.cache.set(nextCacheKey, data);
      });
    }
  }

  private getCacheKey(params: PaginationParams): number {
    return params.page * 1000 + params.pageSize;
  }

  clearCache(): void {
    this.cache.clear();
  }
}

// Virtual scrolling for large lists
class VirtualScroller {
  private itemHeight: number;
  private containerHeight: number;
  private totalItems: number;

  constructor(itemHeight: number, containerHeight: number, totalItems: number) {
    this.itemHeight = itemHeight;
    this.containerHeight = containerHeight;
    this.totalItems = totalItems;
  }

  getVisibleRange(scrollTop: number): { start: number; end: number } {
    const start = Math.floor(scrollTop / this.itemHeight);
    const visibleCount = Math.ceil(this.containerHeight / this.itemHeight);
    const end = Math.min(start + visibleCount, this.totalItems);

    // Add buffer for smooth scrolling
    const buffer = 5;
    return {
      start: Math.max(0, start - buffer),
      end: Math.min(this.totalItems, end + buffer)
    };
  }

  getTotalHeight(): number {
    return this.totalItems * this.itemHeight;
  }

  getOffsetForIndex(index: number): number {
    return index * this.itemHeight;
  }
}

// Example usage
async function loadRequestHistory(page: number) {
  const loader = new PaginatedDataLoader<any>();

  const result = await loader.loadPage(
    { page, pageSize: 50, sortBy: 'timestamp', sortOrder: 'desc' },
    async (params) => {
      // Actual database query
      return fetchFromDatabase(params);
    }
  );

  return result;
}

async function fetchFromDatabase(params: PaginationParams): Promise<any[]> {
  // Mock database fetch
  return [];
}
```

### Request Deduplication

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub struct RequestDeduplicator<T> {
    pending: Arc<Mutex<HashMap<u64, broadcast::Sender<T>>>>,
}

impl<T: Clone> RequestDeduplicator<T> {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn execute<F, Fut>(&self, key: &str, fetch_fn: F) -> Result<T, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let hash = self.hash_key(key);

        // Check if request is already pending
        {
            let mut pending = self.pending.lock().unwrap();
            if let Some(sender) = pending.get(&hash) {
                // Subscribe to existing request
                let mut receiver = sender.subscribe();
                drop(pending); // Release lock

                // Wait for result
                return receiver.recv().await
                    .map_err(|_| "Failed to receive result".to_string());
            }

            // Create new broadcast channel for this request
            let (sender, _) = broadcast::channel(1);
            pending.insert(hash, sender);
        }

        // Execute the actual request
        let result = fetch_fn().await;

        // Broadcast result to all waiting subscribers
        {
            let mut pending = self.pending.lock().unwrap();
            if let Some(sender) = pending.remove(&hash) {
                let _ = sender.send(result.clone().unwrap());
            }
        }

        result
    }

    fn hash_key(&self, key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

// Example usage
#[tokio::main]
async fn main() {
    let deduplicator = Arc::new(RequestDeduplicator::new());

    // Simulate multiple identical requests arriving simultaneously
    let mut handles = vec![];

    for i in 0..10 {
        let dedup = Arc::clone(&deduplicator);
        let handle = tokio::spawn(async move {
            let result = dedup.execute("expensive-operation", || async {
                // This expensive operation will only run once
                println!("Executing expensive operation");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                Ok("Result".to_string())
            }).await;

            println!("Request {} got result: {:?}", i, result);
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let _ = handle.await;
    }

    // Output: "Executing expensive operation" printed only once
    // All 10 requests receive the same result
}
```

---

## Database Query Optimization

### Index Strategy

```sql
-- Create indexes for common query patterns

-- Index on user_id for user-specific queries
CREATE INDEX idx_llm_requests_user_id ON llm_requests(user_id);

-- Composite index for filtering and sorting
CREATE INDEX idx_llm_requests_user_timestamp
ON llm_requests(user_id, timestamp DESC);

-- Index for cost queries
CREATE INDEX idx_llm_requests_cost
ON llm_requests(user_id, timestamp, cost);

-- Partial index for active requests only
CREATE INDEX idx_llm_requests_active
ON llm_requests(user_id, timestamp)
WHERE status = 'active';

-- Index for full-text search on prompts
CREATE INDEX idx_llm_requests_prompt_fulltext
ON llm_requests USING gin(to_tsvector('english', prompt));

-- Analyze index usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC;

-- Find unused indexes
SELECT
    schemaname,
    tablename,
    indexname
FROM pg_stat_user_indexes
WHERE idx_scan = 0
    AND schemaname = 'public'
    AND indexname NOT LIKE 'pg_toast%';
```

### Query Optimization

```python
from sqlalchemy import create_engine, select, func
from sqlalchemy.orm import Session, joinedload, selectinload
from typing import List, Optional
import time

class OptimizedQueryManager:
    """Optimized database query patterns"""

    def __init__(self, db_session: Session):
        self.session = db_session

    def get_user_requests_efficient(self, user_id: str,
                                   limit: int = 100) -> List:
        """Optimized query with eager loading"""
        # BAD: N+1 query problem
        # requests = session.query(Request).filter_by(user_id=user_id).all()
        # for req in requests:
        #     req.user  # Additional query for each request

        # GOOD: Eager loading
        stmt = (
            select(Request)
            .options(joinedload(Request.user))  # JOIN to load user in same query
            .filter(Request.user_id == user_id)
            .order_by(Request.timestamp.desc())
            .limit(limit)
        )

        return self.session.execute(stmt).scalars().all()

    def get_cost_summary_optimized(self, user_id: str,
                                   start_date, end_date) -> dict:
        """Aggregate queries push computation to database"""
        # BAD: Load all rows and aggregate in Python
        # requests = session.query(Request).filter(...).all()
        # total_cost = sum(r.cost for r in requests)

        # GOOD: Database aggregation
        stmt = (
            select(
                func.sum(Request.cost).label('total_cost'),
                func.count(Request.id).label('total_requests'),
                func.avg(Request.cost).label('avg_cost')
            )
            .filter(
                Request.user_id == user_id,
                Request.timestamp.between(start_date, end_date)
            )
        )

        result = self.session.execute(stmt).first()

        return {
            'total_cost': float(result.total_cost or 0),
            'total_requests': result.total_requests,
            'avg_cost': float(result.avg_cost or 0)
        }

    def batch_insert_requests(self, requests: List[dict]):
        """Efficient bulk insert"""
        # BAD: Insert one by one
        # for req in requests:
        #     session.add(Request(**req))
        #     session.commit()

        # GOOD: Bulk insert
        self.session.bulk_insert_mappings(Request, requests)
        self.session.commit()

    def paginate_efficiently(self, user_id: str, page: int,
                           page_size: int) -> dict:
        """Efficient pagination using keyset pagination"""
        # BAD: OFFSET pagination (slow for large offsets)
        # SELECT * FROM requests OFFSET 10000 LIMIT 100;

        # GOOD: Keyset pagination
        last_id = self.get_last_id_from_previous_page(page, page_size)

        stmt = (
            select(Request)
            .filter(
                Request.user_id == user_id,
                Request.id > last_id if last_id else True
            )
            .order_by(Request.id)
            .limit(page_size)
        )

        requests = self.session.execute(stmt).scalars().all()

        return {
            'data': requests,
            'next_cursor': requests[-1].id if requests else None
        }

    def get_last_id_from_previous_page(self, page: int,
                                       page_size: int) -> Optional[int]:
        # Implement cursor tracking
        return None

# Query performance monitoring
class QueryPerformanceTracker:
    """Track and analyze query performance"""

    def __init__(self):
        self.query_stats = {}

    def track_query(self, query_name: str):
        def decorator(func):
            def wrapper(*args, **kwargs):
                start = time.time()
                result = func(*args, **kwargs)
                duration = time.time() - start

                if query_name not in self.query_stats:
                    self.query_stats[query_name] = {
                        'count': 0,
                        'total_time': 0,
                        'min_time': float('inf'),
                        'max_time': 0
                    }

                stats = self.query_stats[query_name]
                stats['count'] += 1
                stats['total_time'] += duration
                stats['min_time'] = min(stats['min_time'], duration)
                stats['max_time'] = max(stats['max_time'], duration)

                # Log slow queries
                if duration > 1.0:  # 1 second threshold
                    self.log_slow_query(query_name, duration)

                return result
            return wrapper
        return decorator

    def log_slow_query(self, query_name: str, duration: float):
        print(f"SLOW QUERY: {query_name} took {duration:.2f}s")

    def get_report(self):
        for query_name, stats in self.query_stats.items():
            avg_time = stats['total_time'] / stats['count']
            print(f"{query_name}:")
            print(f"  Count: {stats['count']}")
            print(f"  Avg: {avg_time:.4f}s")
            print(f"  Min: {stats['min_time']:.4f}s")
            print(f"  Max: {stats['max_time']:.4f}s")

# Usage
tracker = QueryPerformanceTracker()

@tracker.track_query('get_user_requests')
def get_user_requests(user_id: str):
    # Query implementation
    pass

# Materialized views for expensive queries
CREATE_MATERIALIZED_VIEW = """
CREATE MATERIALIZED VIEW user_cost_summary AS
SELECT
    user_id,
    DATE(timestamp) as date,
    SUM(cost) as total_cost,
    COUNT(*) as request_count,
    AVG(cost) as avg_cost
FROM llm_requests
GROUP BY user_id, DATE(timestamp);

CREATE INDEX idx_user_cost_summary_user_date
ON user_cost_summary(user_id, date);

-- Refresh periodically (e.g., every hour)
REFRESH MATERIALIZED VIEW CONCURRENTLY user_cost_summary;
"""
```

---

## Connection Pooling

### Database Connection Pool

```javascript
const { Pool } = require('pg');

class DatabaseConnectionPool {
  constructor(config) {
    this.pool = new Pool({
      host: config.host || 'localhost',
      port: config.port || 5432,
      database: config.database,
      user: config.user,
      password: config.password,

      // Connection pool settings
      max: config.maxConnections || 20,          // Maximum connections
      min: config.minConnections || 5,           // Minimum connections
      idleTimeoutMillis: 30000,                  // Close idle connections after 30s
      connectionTimeoutMillis: 2000,             // Wait 2s for connection
      maxUses: 7500,                             // Retire connections after 7500 uses

      // Health checks
      allowExitOnIdle: false,
    });

    this.setupEventHandlers();
    this.monitorPool();
  }

  setupEventHandlers() {
    this.pool.on('connect', (client) => {
      console.log('New client connected to pool');
    });

    this.pool.on('acquire', (client) => {
      console.log('Client acquired from pool');
    });

    this.pool.on('error', (err, client) => {
      console.error('Unexpected error on idle client', err);
    });

    this.pool.on('remove', (client) => {
      console.log('Client removed from pool');
    });
  }

  monitorPool() {
    setInterval(() => {
      const stats = {
        total: this.pool.totalCount,
        idle: this.pool.idleCount,
        waiting: this.pool.waitingCount
      };

      console.log('Pool stats:', stats);

      // Alert if pool is exhausted
      if (stats.waiting > 5) {
        console.warn('Connection pool under pressure! Waiting:', stats.waiting);
      }

      // Alert if too many idle connections
      if (stats.idle > stats.total * 0.8) {
        console.info('Many idle connections, consider reducing pool size');
      }
    }, 60000); // Every minute
  }

  async query(sql, params) {
    const start = Date.now();
    try {
      const result = await this.pool.query(sql, params);
      const duration = Date.now() - start;

      // Log slow queries
      if (duration > 1000) {
        console.warn(`Slow query (${duration}ms):`, sql);
      }

      return result;
    } catch (error) {
      console.error('Query error:', error);
      throw error;
    }
  }

  async transaction(callback) {
    const client = await this.pool.connect();
    try {
      await client.query('BEGIN');
      const result = await callback(client);
      await client.query('COMMIT');
      return result;
    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }

  async healthCheck() {
    try {
      await this.pool.query('SELECT 1');
      return { healthy: true, poolStats: this.getStats() };
    } catch (error) {
      return { healthy: false, error: error.message };
    }
  }

  getStats() {
    return {
      total: this.pool.totalCount,
      idle: this.pool.idleCount,
      waiting: this.pool.waitingCount
    };
  }

  async close() {
    await this.pool.end();
  }
}

// HTTP client connection pool
const http = require('http');
const https = require('https');

class HTTPConnectionPool {
  constructor() {
    this.httpAgent = new http.Agent({
      keepAlive: true,
      keepAliveMsecs: 1000,
      maxSockets: 50,
      maxFreeSockets: 10,
      timeout: 60000,
      scheduling: 'lifo'  // Last-in-first-out (better for keep-alive)
    });

    this.httpsAgent = new https.Agent({
      keepAlive: true,
      keepAliveMsecs: 1000,
      maxSockets: 50,
      maxFreeSockets: 10,
      timeout: 60000,
      scheduling: 'lifo'
    });
  }

  getAgent(url) {
    return url.startsWith('https') ? this.httpsAgent : this.httpAgent;
  }

  destroy() {
    this.httpAgent.destroy();
    this.httpsAgent.destroy();
  }
}

// Redis connection pool
const Redis = require('ioredis');

class RedisConnectionPool {
  constructor(config) {
    this.cluster = new Redis.Cluster(
      config.nodes || [
        { host: 'localhost', port: 6379 }
      ],
      {
        redisOptions: {
          password: config.password,
          db: config.db || 0
        },
        clusterRetryStrategy: (times) => {
          return Math.min(times * 50, 2000);
        },
        enableOfflineQueue: true,
        maxRetriesPerRequest: 3
      }
    );

    this.setupEvents();
  }

  setupEvents() {
    this.cluster.on('connect', () => {
      console.log('Redis cluster connected');
    });

    this.cluster.on('ready', () => {
      console.log('Redis cluster ready');
    });

    this.cluster.on('error', (err) => {
      console.error('Redis cluster error:', err);
    });

    this.cluster.on('close', () => {
      console.log('Redis cluster connection closed');
    });

    this.cluster.on('reconnecting', () => {
      console.log('Redis cluster reconnecting');
    });

    this.cluster.on('end', () => {
      console.log('Redis cluster connection ended');
    });
  }

  async get(key) {
    return await this.cluster.get(key);
  }

  async set(key, value, ttl) {
    if (ttl) {
      return await this.cluster.set(key, value, 'EX', ttl);
    }
    return await this.cluster.set(key, value);
  }

  async disconnect() {
    await this.cluster.quit();
  }
}

// Usage example
const dbPool = new DatabaseConnectionPool({
  host: 'localhost',
  database: 'llm_cost_ops',
  user: 'postgres',
  password: 'password',
  maxConnections: 20
});

// Execute query
const users = await dbPool.query('SELECT * FROM users WHERE id = $1', [userId]);

// Execute transaction
await dbPool.transaction(async (client) => {
  await client.query('UPDATE accounts SET balance = balance - 100 WHERE id = $1', [fromId]);
  await client.query('UPDATE accounts SET balance = balance + 100 WHERE id = $1', [toId]);
});
```

---

## Caching Strategies

### Multi-Layer Caching

```python
from typing import Optional, Any, Callable
import redis
import json
from functools import wraps
import hashlib
from datetime import timedelta

class MultiLayerCache:
    """Implement L1 (memory) + L2 (Redis) caching"""

    def __init__(self, redis_client: redis.Redis,
                 l1_size: int = 1000):
        self.redis = redis_client
        self.l1_cache = {}  # In-memory cache
        self.l1_size = l1_size
        self.l1_access_count = {}

    def get(self, key: str) -> Optional[Any]:
        """Get from cache (L1 first, then L2)"""
        # Try L1 cache
        if key in self.l1_cache:
            self.l1_access_count[key] = self.l1_access_count.get(key, 0) + 1
            return self.l1_cache[key]

        # Try L2 cache (Redis)
        value = self.redis.get(key)
        if value:
            # Promote to L1
            self._set_l1(key, json.loads(value))
            return json.loads(value)

        return None

    def set(self, key: str, value: Any, ttl: int = 3600):
        """Set in both cache layers"""
        # Set in L2 (Redis)
        self.redis.setex(key, ttl, json.dumps(value))

        # Set in L1 (memory)
        self._set_l1(key, value)

    def _set_l1(self, key: str, value: Any):
        """Set in L1 cache with LRU eviction"""
        if len(self.l1_cache) >= self.l1_size:
            # Evict least recently used
            lru_key = min(self.l1_access_count,
                         key=self.l1_access_count.get)
            del self.l1_cache[lru_key]
            del self.l1_access_count[lru_key]

        self.l1_cache[key] = value
        self.l1_access_count[key] = 1

    def delete(self, key: str):
        """Delete from both cache layers"""
        if key in self.l1_cache:
            del self.l1_cache[key]
            del self.l1_access_count[key]

        self.redis.delete(key)

    def clear_l1(self):
        """Clear L1 cache only"""
        self.l1_cache.clear()
        self.l1_access_count.clear()

    def get_stats(self) -> dict:
        """Get cache statistics"""
        return {
            'l1_size': len(self.l1_cache),
            'l1_max_size': self.l1_size,
            'l2_keys': self.redis.dbsize(),
            'memory_usage': self.redis.info('memory')['used_memory_human']
        }

def cached(cache: MultiLayerCache, ttl: int = 3600,
          key_prefix: str = ''):
    """Decorator for caching function results"""
    def decorator(func: Callable):
        @wraps(func)
        def wrapper(*args, **kwargs):
            # Generate cache key
            key_data = f"{key_prefix}{func.__name__}:{args}:{sorted(kwargs.items())}"
            cache_key = hashlib.md5(key_data.encode()).hexdigest()

            # Try cache
            result = cache.get(cache_key)
            if result is not None:
                return result

            # Execute function
            result = func(*args, **kwargs)

            # Store in cache
            cache.set(cache_key, result, ttl)

            return result
        return wrapper
    return decorator

# Cache warming
class CacheWarmer:
    """Proactively warm cache with frequently accessed data"""

    def __init__(self, cache: MultiLayerCache):
        self.cache = cache

    async def warm_user_data(self, user_ids: list):
        """Pre-load user data into cache"""
        for user_id in user_ids:
            user_data = await fetch_user_data(user_id)
            self.cache.set(f"user:{user_id}", user_data, ttl=3600)

    async def warm_popular_queries(self, queries: list):
        """Pre-load popular query results"""
        for query in queries:
            result = await execute_query(query)
            cache_key = hashlib.md5(query.encode()).hexdigest()
            self.cache.set(f"query:{cache_key}", result, ttl=1800)

# Example usage
redis_client = redis.Redis(host='localhost', port=6379, db=0)
cache = MultiLayerCache(redis_client, l1_size=1000)

@cached(cache, ttl=3600, key_prefix='llm:')
def get_llm_response(prompt: str, model: str) -> dict:
    # Expensive LLM API call
    return call_llm_api(prompt, model)

# First call: executes function
response1 = get_llm_response("What is Python?", "gpt-3.5-turbo")

# Second call: returns from cache (L1)
response2 = get_llm_response("What is Python?", "gpt-3.5-turbo")

# Cache statistics
print(cache.get_stats())

# Helper functions
async def fetch_user_data(user_id: str) -> dict:
    return {}

async def execute_query(query: str) -> dict:
    return {}

def call_llm_api(prompt: str, model: str) -> dict:
    return {}
```

### Cache Invalidation Strategies

```go
package main

import (
    "context"
    "encoding/json"
    "fmt"
    "time"
    "github.com/go-redis/redis/v8"
)

type CacheInvalidator struct {
    client *redis.Client
    ctx    context.Context
}

func NewCacheInvalidator(client *redis.Client) *CacheInvalidator {
    return &CacheInvalidator{
        client: client,
        ctx:    context.Background(),
    }
}

// Time-based invalidation
func (ci *CacheInvalidator) SetWithTTL(key string, value interface{}, ttl time.Duration) error {
    data, err := json.Marshal(value)
    if err != nil {
        return err
    }

    return ci.client.Set(ci.ctx, key, data, ttl).Err()
}

// Event-based invalidation
func (ci *CacheInvalidator) InvalidateOnEvent(event string, keys []string) {
    // Subscribe to event channel
    pubsub := ci.client.Subscribe(ci.ctx, event)
    defer pubsub.Close()

    ch := pubsub.Channel()

    go func() {
        for msg := range ch {
            fmt.Printf("Received event: %s, invalidating cache\n", msg.Payload)

            // Delete affected cache keys
            if len(keys) > 0 {
                ci.client.Del(ci.ctx, keys...)
            }
        }
    }()
}

// Tag-based invalidation
func (ci *CacheInvalidator) SetWithTags(key string, value interface{},
                                       tags []string, ttl time.Duration) error {
    // Store value
    data, err := json.Marshal(value)
    if err != nil {
        return err
    }

    err = ci.client.Set(ci.ctx, key, data, ttl).Err()
    if err != nil {
        return err
    }

    // Associate tags
    for _, tag := range tags {
        tagKey := fmt.Sprintf("tag:%s", tag)
        ci.client.SAdd(ci.ctx, tagKey, key)
        ci.client.Expire(ci.ctx, tagKey, ttl)
    }

    return nil
}

func (ci *CacheInvalidator) InvalidateByTag(tag string) error {
    tagKey := fmt.Sprintf("tag:%s", tag)

    // Get all keys with this tag
    keys, err := ci.client.SMembers(ci.ctx, tagKey).Result()
    if err != nil {
        return err
    }

    // Delete all keys
    if len(keys) > 0 {
        ci.client.Del(ci.ctx, keys...)
    }

    // Delete tag set
    ci.client.Del(ci.ctx, tagKey)

    return nil
}

// Versioned caching
func (ci *CacheInvalidator) SetVersioned(key string, value interface{},
                                        version int, ttl time.Duration) error {
    versionedKey := fmt.Sprintf("%s:v%d", key, version)

    data, err := json.Marshal(value)
    if err != nil {
        return err
    }

    return ci.client.Set(ci.ctx, versionedKey, data, ttl).Err()
}

func (ci *CacheInvalidator) GetVersioned(key string, version int) ([]byte, error) {
    versionedKey := fmt.Sprintf("%s:v%d", key, version)
    return ci.client.Get(ci.ctx, versionedKey).Bytes()
}

// Write-through cache
type WriteThroughCache struct {
    invalidator *CacheInvalidator
    db          Database
}

type Database interface {
    Save(key string, value interface{}) error
    Load(key string) (interface{}, error)
}

func (wtc *WriteThroughCache) Set(key string, value interface{}) error {
    // Write to database first
    err := wtc.db.Save(key, value)
    if err != nil {
        return err
    }

    // Then update cache
    return wtc.invalidator.SetWithTTL(key, value, 1*time.Hour)
}

func (wtc *WriteThroughCache) Get(key string) (interface{}, error) {
    // Try cache first
    data, err := wtc.invalidator.client.Get(
        wtc.invalidator.ctx,
        key,
    ).Bytes()

    if err == nil {
        var result interface{}
        json.Unmarshal(data, &result)
        return result, nil
    }

    // Cache miss - load from database
    result, err := wtc.db.Load(key)
    if err != nil {
        return nil, err
    }

    // Populate cache
    wtc.invalidator.SetWithTTL(key, result, 1*time.Hour)

    return result, nil
}

// Example usage
func main() {
    client := redis.NewClient(&redis.Options{
        Addr: "localhost:6379",
    })

    invalidator := NewCacheInvalidator(client)

    // Time-based invalidation
    invalidator.SetWithTTL("user:123", map[string]string{
        "name": "John Doe",
    }, 1*time.Hour)

    // Tag-based invalidation
    invalidator.SetWithTags("user:123", map[string]string{
        "name": "John Doe",
    }, []string{"users", "active"}, 1*time.Hour)

    // Invalidate all user caches
    invalidator.InvalidateByTag("users")

    // Versioned caching
    invalidator.SetVersioned("config", map[string]interface{}{
        "feature_flag": true,
    }, 1, 24*time.Hour)
}
```

---

*Continuing with remaining sections...*

## Load Balancing

### Application Load Balancer Configuration

```yaml
# AWS Application Load Balancer
load_balancer:
  name: llm-cost-ops-alb
  scheme: internet-facing
  type: application

  listeners:
    - port: 443
      protocol: HTTPS
      ssl_policy: ELBSecurityPolicy-TLS-1-2-2017-01
      certificates:
        - certificate_arn: arn:aws:acm:us-east-1:123456789:certificate/abc

      default_actions:
        - type: forward
          target_group_arn: arn:aws:elasticloadbalancing:...

  health_check:
    enabled: true
    interval: 30
    path: /health
    port: 8080
    protocol: HTTP
    timeout: 5
    healthy_threshold: 2
    unhealthy_threshold: 3
    matcher: "200-299"

  target_groups:
    - name: llm-api-servers
      port: 8080
      protocol: HTTP
      vpc_id: vpc-12345

      load_balancing_algorithm: least_outstanding_requests

      stickiness:
        enabled: true
        type: lb_cookie
        duration: 3600

      deregistration_delay: 30

  routing_rules:
    - priority: 1
      conditions:
        - field: path-pattern
          values: ["/api/v1/*"]
      actions:
        - type: forward
          target_group: llm-api-servers

    - priority: 2
      conditions:
        - field: host-header
          values: ["admin.example.com"]
      actions:
        - type: forward
          target_group: admin-servers

# NGINX Load Balancer Configuration
nginx_lb:
  upstream:
    - name: llm_backend
      servers:
        - address: 10.0.1.10:8080
          weight: 3
          max_fails: 3
          fail_timeout: 30s

        - address: 10.0.1.11:8080
          weight: 2
          max_fails: 3
          fail_timeout: 30s

        - address: 10.0.1.12:8080
          weight: 1
          backup: true

      load_balancing_method: least_conn
      keepalive: 32
      keepalive_requests: 100
      keepalive_timeout: 60s

  server:
    listen: 443 ssl http2
    server_name: api.example.com

    ssl_certificate: /path/to/cert.pem
    ssl_certificate_key: /path/to/key.pem
    ssl_protocols: TLSv1.2 TLSv1.3
    ssl_ciphers: HIGH:!aNULL:!MD5

    locations:
      - path: /
        proxy_pass: http://llm_backend
        proxy_http_version: "1.1"
        proxy_set_header:
          Connection: ""
          Host: $host
          X-Real-IP: $remote_addr
          X-Forwarded-For: $proxy_add_x_forwarded_for
          X-Forwarded-Proto: $scheme

        proxy_connect_timeout: 5s
        proxy_send_timeout: 60s
        proxy_read_timeout: 60s

        proxy_buffering: on
        proxy_buffer_size: 4k
        proxy_buffers: 8 4k

      - path: /api/stream
        proxy_pass: http://llm_backend
        proxy_buffering: off  # Disable for streaming
        proxy_cache: off
```

---

## Auto-Scaling Configuration

### Horizontal Pod Autoscaler (Kubernetes)

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: llm-cost-ops-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-cost-ops-api

  minReplicas: 3
  maxReplicas: 50

  metrics:
    # CPU-based scaling
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70

    # Memory-based scaling
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80

    # Custom metric: requests per second
    - type: Pods
      pods:
        metric:
          name: requests_per_second
        target:
          type: AverageValue
          averageValue: "100"

  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 50
          periodSeconds: 60
        - type: Pods
          value: 2
          periodSeconds: 60
      selectPolicy: Min

    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
        - type: Percent
          value: 100
          periodSeconds: 30
        - type: Pods
          value: 4
          periodSeconds: 30
      selectPolicy: Max

---
# Vertical Pod Autoscaler
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: llm-cost-ops-vpa
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-cost-ops-api

  updatePolicy:
    updateMode: "Auto"

  resourcePolicy:
    containerPolicies:
      - containerName: api
        minAllowed:
          cpu: 100m
          memory: 128Mi
        maxAllowed:
          cpu: 2000m
          memory: 2Gi
        controlledResources: ["cpu", "memory"]
```

### Predictive Autoscaling

```python
from datetime import datetime, timedelta
from typing import List, Tuple
import numpy as np
from sklearn.linear_model import LinearRegression

class PredictiveAutoscaler:
    """Predict future load and preemptively scale"""

    def __init__(self):
        self.history: List[Tuple[datetime, int]] = []
        self.model = LinearRegression()

    def record_load(self, timestamp: datetime, request_count: int):
        """Record historical load data"""
        self.history.append((timestamp, request_count))

        # Keep last 7 days
        cutoff = datetime.now() - timedelta(days=7)
        self.history = [
            (ts, count) for ts, count in self.history
            if ts > cutoff
        ]

    def predict_load(self, minutes_ahead: int = 30) -> int:
        """Predict load N minutes in the future"""
        if len(self.history) < 10:
            return 0

        # Prepare data
        X = np.array([
            (ts - self.history[0][0]).total_seconds() / 60
            for ts, _ in self.history
        ]).reshape(-1, 1)

        y = np.array([count for _, count in self.history])

        # Train model
        self.model.fit(X, y)

        # Predict
        future_time = (
            datetime.now() - self.history[0][0]
        ).total_seconds() / 60 + minutes_ahead

        prediction = self.model.predict([[future_time]])[0]

        return max(0, int(prediction))

    def calculate_required_replicas(self, predicted_load: int,
                                   requests_per_replica: int = 100) -> int:
        """Calculate required replicas for predicted load"""
        required = np.ceil(predicted_load / requests_per_replica)

        # Add 20% buffer
        buffered = int(required * 1.2)

        return max(3, buffered)  # Minimum 3 replicas

    def detect_patterns(self) -> dict:
        """Detect daily/weekly patterns"""
        if len(self.history) < 24 * 7:  # Need at least a week
            return {}

        # Group by hour of day
        hourly_avg = {}
        for ts, count in self.history:
            hour = ts.hour
            if hour not in hourly_avg:
                hourly_avg[hour] = []
            hourly_avg[hour].append(count)

        patterns = {}
        for hour, counts in hourly_avg.items():
            patterns[hour] = {
                'avg': np.mean(counts),
                'std': np.std(counts),
                'max': np.max(counts)
            }

        return patterns

    def get_scaling_recommendation(self) -> dict:
        """Get scaling recommendation"""
        current_load = self.history[-1][1] if self.history else 0
        predicted_load_15m = self.predict_load(15)
        predicted_load_30m = self.predict_load(30)

        current_replicas = self.calculate_required_replicas(current_load)
        required_replicas_15m = self.calculate_required_replicas(predicted_load_15m)
        required_replicas_30m = self.calculate_required_replicas(predicted_load_30m)

        patterns = self.detect_patterns()

        return {
            'current_load': current_load,
            'predicted_load_15m': predicted_load_15m,
            'predicted_load_30m': predicted_load_30m,
            'current_replicas': current_replicas,
            'recommended_replicas_15m': required_replicas_15m,
            'recommended_replicas_30m': required_replicas_30m,
            'action': self._determine_action(
                current_replicas,
                required_replicas_15m
            ),
            'hourly_patterns': patterns
        }

    def _determine_action(self, current: int, required: int) -> str:
        if required > current * 1.2:
            return 'SCALE_UP'
        elif required < current * 0.7:
            return 'SCALE_DOWN'
        else:
            return 'MAINTAIN'

# Example usage
autoscaler = PredictiveAutoscaler()

# Simulate recording load over time
for i in range(1000):
    timestamp = datetime.now() - timedelta(minutes=1000-i)
    # Simulate daily pattern
    hour = timestamp.hour
    base_load = 50
    if 9 <= hour <= 17:  # Business hours
        load = base_load + np.random.randint(50, 150)
    else:
        load = base_load + np.random.randint(0, 30)

    autoscaler.record_load(timestamp, load)

# Get recommendations
recommendation = autoscaler.get_scaling_recommendation()
print(f"Current load: {recommendation['current_load']}")
print(f"Predicted load (15m): {recommendation['predicted_load_15m']}")
print(f"Action: {recommendation['action']}")
print(f"Recommended replicas: {recommendation['recommended_replicas_15m']}")
```

---

## Implementation Checklist

### Phase 1: Quick Wins (Week 1)
- [ ] Enable database connection pooling
- [ ] Implement basic caching (Redis)
- [ ] Add database indexes for common queries
- [ ] Enable gzip compression
- [ ] Optimize slow queries (> 1s)

### Phase 2: Intermediate Optimizations (Weeks 2-4)
- [ ] Implement multi-layer caching
- [ ] Set up CDN for static assets
- [ ] Configure load balancer
- [ ] Optimize batch operations
- [ ] Implement request deduplication

### Phase 3: Advanced Performance (Weeks 5-8)
- [ ] Deploy auto-scaling
- [ ] Implement predictive scaling
- [ ] Set up performance monitoring
- [ ] Optimize async processing
- [ ] Database query optimization

### Phase 4: Fine-Tuning (Weeks 9-12)
- [ ] Load testing and benchmarking
- [ ] Resource sizing optimization
- [ ] Network optimization
- [ ] Performance regression testing

---

## Tools and Resources

### Monitoring Tools
- **Datadog** - Application performance monitoring
- **New Relic** - Full-stack observability
- **Prometheus + Grafana** - Metrics and dashboards

### Profiling Tools
- **py-spy** - Python profiling
- **pprof** - Go profiling
- **clinic.js** - Node.js profiling

### Load Testing
- **k6** - Modern load testing
- **Apache JMeter** - Load testing
- **Locust** - Python-based load testing

---

*Last Updated: 2025-11-16*
*Version: 1.0*
