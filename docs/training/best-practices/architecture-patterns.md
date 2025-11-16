# Architecture Patterns for LLM Cost Ops

## Table of Contents

1. [Introduction](#introduction)
2. [Microservices Architecture](#microservices-architecture)
3. [Event-Driven Architecture](#event-driven-architecture)
4. [CQRS and Event Sourcing](#cqrs-and-event-sourcing)
5. [API Gateway Pattern](#api-gateway-pattern)
6. [Service Mesh](#service-mesh)
7. [Circuit Breaker Pattern](#circuit-breaker-pattern)
8. [Saga Pattern](#saga-pattern-for-distributed-transactions)
9. [Repository Pattern](#repository-pattern)
10. [Factory Pattern](#factory-pattern)
11. [Strategy Pattern](#strategy-pattern-for-multi-provider-support)
12. [Observer Pattern](#observer-pattern-for-webhooks)
13. [Deployment Patterns](#deployment-patterns)
14. [Multi-Region Deployment](#multi-region-deployment)
15. [Disaster Recovery](#disaster-recovery-architectures)
16. [Implementation Checklist](#implementation-checklist)
17. [Tools and Resources](#tools-and-resources)

---

## Introduction

### Architecture Principles

Well-designed architecture for LLM Cost Ops should follow these principles:

1. **Scalability**: Handle increasing load gracefully
2. **Reliability**: Maintain high availability (99.9%+)
3. **Maintainability**: Easy to understand, modify, and extend
4. **Security**: Defense in depth, least privilege
5. **Cost-Efficiency**: Optimize resource utilization
6. **Observability**: Comprehensive monitoring and logging

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         Load Balancer                        │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
┌────────▼────────┐ ┌───▼────────┐ ┌───▼────────┐
│  API Gateway    │ │ API Gateway│ │ API Gateway│
│   (Region 1)    │ │ (Region 2) │ │ (Region 3) │
└────────┬────────┘ └────┬───────┘ └────┬───────┘
         │               │               │
    ┌────▼────┐     ┌───▼────┐     ┌───▼────┐
    │Services │     │Services│     │Services│
    └────┬────┘     └───┬────┘     └───┬────┘
         │              │               │
    ┌────▼────────────────▼──────────────▼────┐
    │         Message Queue / Event Bus        │
    └────────────────────┬─────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼────┐     ┌───▼────┐     ┌───▼────┐
    │Database │     │  Cache │     │Storage │
    │Cluster  │     │ (Redis)│     │  (S3)  │
    └─────────┘     └────────┘     └────────┘
```

---

## Microservices Architecture

### Service Decomposition

```python
from typing import Protocol
from dataclasses import dataclass
from abc import ABC, abstractmethod

# Bounded Contexts for LLM Cost Ops

@dataclass
class ServiceDefinition:
    name: str
    responsibilities: list[str]
    dependencies: list[str]
    api_endpoints: list[str]
    database: str
    cache_required: bool

class LLMCostOpsServices:
    """Define microservices architecture"""

    @staticmethod
    def get_services() -> dict[str, ServiceDefinition]:
        return {
            'authentication': ServiceDefinition(
                name='Authentication Service',
                responsibilities=[
                    'User authentication',
                    'Token management',
                    'API key validation',
                    'SSO integration'
                ],
                dependencies=[],
                api_endpoints=[
                    'POST /auth/login',
                    'POST /auth/logout',
                    'POST /auth/refresh',
                    'GET /auth/validate'
                ],
                database='auth_db',
                cache_required=True
            ),

            'llm_proxy': ServiceDefinition(
                name='LLM Proxy Service',
                responsibilities=[
                    'Route requests to LLM providers',
                    'Handle retries and failover',
                    'Response caching',
                    'Rate limiting per provider'
                ],
                dependencies=['authentication', 'cost_tracking'],
                api_endpoints=[
                    'POST /llm/chat',
                    'POST /llm/completion',
                    'POST /llm/embedding'
                ],
                database='proxy_cache_db',
                cache_required=True
            ),

            'cost_tracking': ServiceDefinition(
                name='Cost Tracking Service',
                responsibilities=[
                    'Track API usage',
                    'Calculate costs',
                    'Generate reports',
                    'Budget alerts'
                ],
                dependencies=['authentication'],
                api_endpoints=[
                    'GET /costs/summary',
                    'GET /costs/by-user',
                    'POST /costs/record',
                    'GET /costs/forecast'
                ],
                database='costs_db',
                cache_required=True
            ),

            'analytics': ServiceDefinition(
                name='Analytics Service',
                responsibilities=[
                    'Usage analytics',
                    'Performance metrics',
                    'Trend analysis',
                    'Anomaly detection'
                ],
                dependencies=['cost_tracking'],
                api_endpoints=[
                    'GET /analytics/usage',
                    'GET /analytics/trends',
                    'GET /analytics/anomalies'
                ],
                database='analytics_db',
                cache_required=True
            ),

            'webhook': ServiceDefinition(
                name='Webhook Service',
                responsibilities=[
                    'Manage webhook subscriptions',
                    'Deliver webhook events',
                    'Retry failed deliveries',
                    'Event filtering'
                ],
                dependencies=['authentication'],
                api_endpoints=[
                    'POST /webhooks',
                    'GET /webhooks',
                    'DELETE /webhooks/:id'
                ],
                database='webhooks_db',
                cache_required=False
            ),

            'billing': ServiceDefinition(
                name='Billing Service',
                responsibilities=[
                    'Invoice generation',
                    'Payment processing',
                    'Subscription management',
                    'Usage metering'
                ],
                dependencies=['authentication', 'cost_tracking'],
                api_endpoints=[
                    'GET /billing/invoices',
                    'POST /billing/payment',
                    'GET /billing/subscription'
                ],
                database='billing_db',
                cache_required=False
            )
        }

# Service Communication
class ServiceCommunication(Protocol):
    """Define inter-service communication interface"""

    async def call_service(self, service: str, endpoint: str,
                          data: dict) -> dict:
        """Make synchronous call to another service"""
        ...

    async def publish_event(self, event_type: str, data: dict):
        """Publish asynchronous event"""
        ...

class RestServiceCommunication(ServiceCommunication):
    """REST-based service communication"""

    def __init__(self, base_urls: dict[str, str]):
        self.base_urls = base_urls

    async def call_service(self, service: str, endpoint: str,
                          data: dict) -> dict:
        """Make HTTP call to service"""
        import aiohttp

        base_url = self.base_urls.get(service)
        if not base_url:
            raise ValueError(f"Unknown service: {service}")

        url = f"{base_url}{endpoint}"

        async with aiohttp.ClientSession() as session:
            async with session.post(url, json=data) as response:
                return await response.json()

    async def publish_event(self, event_type: str, data: dict):
        """Publish to message queue"""
        # Implementation using RabbitMQ, Kafka, etc.
        pass

# Service Registry and Discovery
class ServiceRegistry:
    """Service discovery and registration"""

    def __init__(self):
        self.services: dict[str, list[str]] = {}

    def register(self, service_name: str, instance_url: str):
        """Register service instance"""
        if service_name not in self.services:
            self.services[service_name] = []

        if instance_url not in self.services[service_name]:
            self.services[service_name].append(instance_url)

    def deregister(self, service_name: str, instance_url: str):
        """Deregister service instance"""
        if service_name in self.services:
            self.services[service_name] = [
                url for url in self.services[service_name]
                if url != instance_url
            ]

    def discover(self, service_name: str) -> str:
        """Discover service instance (round-robin)"""
        instances = self.services.get(service_name, [])
        if not instances:
            raise ValueError(f"No instances for service: {service_name}")

        # Simple round-robin
        import random
        return random.choice(instances)

    def get_all_instances(self, service_name: str) -> list[str]:
        """Get all instances of a service"""
        return self.services.get(service_name, [])

# Example microservice implementation
from fastapi import FastAPI, Depends, HTTPException
from pydantic import BaseModel

class CostTrackingService:
    """Cost Tracking Microservice"""

    def __init__(self):
        self.app = FastAPI(title="Cost Tracking Service")
        self.setup_routes()

    def setup_routes(self):
        @self.app.post("/costs/record")
        async def record_cost(request: CostRecordRequest):
            # Record cost in database
            cost_record = await self.save_cost(request)
            # Publish event for analytics
            await self.publish_cost_event(cost_record)
            return {"id": cost_record.id}

        @self.app.get("/costs/summary")
        async def get_summary(user_id: str, start_date: str, end_date: str):
            return await self.calculate_summary(user_id, start_date, end_date)

    async def save_cost(self, request: 'CostRecordRequest'):
        # Database operation
        pass

    async def publish_cost_event(self, cost_record):
        # Publish to event bus
        pass

    async def calculate_summary(self, user_id: str,
                               start_date: str, end_date: str):
        # Calculate cost summary
        pass

class CostRecordRequest(BaseModel):
    user_id: str
    model: str
    tokens: int
    cost: float
```

### API Contracts and Versioning

```typescript
// API Contract Definition using OpenAPI/Swagger

interface APIContract {
  version: string;
  service: string;
  endpoints: Endpoint[];
}

interface Endpoint {
  path: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  request: Schema;
  response: Schema;
  errors: ErrorResponse[];
}

interface Schema {
  type: string;
  properties: Record<string, any>;
  required: string[];
}

interface ErrorResponse {
  code: number;
  message: string;
}

class APIVersionManager {
  private contracts: Map<string, APIContract> = new Map();

  registerContract(contract: APIContract): void {
    const key = `${contract.service}:${contract.version}`;
    this.contracts.set(key, contract);
  }

  getContract(service: string, version: string): APIContract | undefined {
    const key = `${service}:${version}`;
    return this.contracts.get(key);
  }

  isCompatible(
    service: string,
    fromVersion: string,
    toVersion: string
  ): boolean {
    // Check backward compatibility
    const from = this.getContract(service, fromVersion);
    const to = this.getContract(service, toVersion);

    if (!from || !to) return false;

    // Simple check: all endpoints in 'from' exist in 'to'
    return from.endpoints.every(endpoint =>
      to.endpoints.some(e =>
        e.path === endpoint.path && e.method === endpoint.method
      )
    );
  }
}

// Example: Cost Tracking API v1
const costTrackingV1: APIContract = {
  version: 'v1',
  service: 'cost-tracking',
  endpoints: [
    {
      path: '/costs/record',
      method: 'POST',
      request: {
        type: 'object',
        properties: {
          user_id: { type: 'string' },
          model: { type: 'string' },
          tokens: { type: 'number' },
          cost: { type: 'number' }
        },
        required: ['user_id', 'model', 'tokens', 'cost']
      },
      response: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          timestamp: { type: 'string' }
        },
        required: ['id']
      },
      errors: [
        { code: 400, message: 'Invalid request' },
        { code: 401, message: 'Unauthorized' }
      ]
    },
    {
      path: '/costs/summary',
      method: 'GET',
      request: {
        type: 'object',
        properties: {
          user_id: { type: 'string' },
          start_date: { type: 'string' },
          end_date: { type: 'string' }
        },
        required: ['user_id']
      },
      response: {
        type: 'object',
        properties: {
          total_cost: { type: 'number' },
          total_requests: { type: 'number' },
          breakdown: { type: 'array' }
        },
        required: ['total_cost']
      },
      errors: [
        { code: 404, message: 'User not found' }
      ]
    }
  ]
};

// Version routing middleware
class VersionRouter {
  route(request: any): string {
    const version = this.extractVersion(request);
    return `/${version}${request.path}`;
  }

  private extractVersion(request: any): string {
    // From header
    if (request.headers['api-version']) {
      return request.headers['api-version'];
    }

    // From URL path
    const match = request.path.match(/^\/v(\d+)\//);
    if (match) {
      return `v${match[1]}`;
    }

    // Default to latest
    return 'v1';
  }
}
```

---

## Event-Driven Architecture

### Event Bus Implementation

```go
package main

import (
    "context"
    "encoding/json"
    "fmt"
    "sync"
    "time"
)

// Event represents a domain event
type Event struct {
    ID        string                 `json:"id"`
    Type      string                 `json:"type"`
    Timestamp time.Time              `json:"timestamp"`
    Source    string                 `json:"source"`
    Data      map[string]interface{} `json:"data"`
}

// EventHandler processes events
type EventHandler func(ctx context.Context, event Event) error

// EventBus manages event publishing and subscription
type EventBus struct {
    handlers map[string][]EventHandler
    mu       sync.RWMutex
}

func NewEventBus() *EventBus {
    return &EventBus{
        handlers: make(map[string][]EventHandler),
    }
}

// Subscribe to events of a specific type
func (eb *EventBus) Subscribe(eventType string, handler EventHandler) {
    eb.mu.Lock()
    defer eb.mu.Unlock()

    if _, exists := eb.handlers[eventType]; !exists {
        eb.handlers[eventType] = []EventHandler{}
    }

    eb.handlers[eventType] = append(eb.handlers[eventType], handler)
}

// Publish an event to all subscribers
func (eb *EventBus) Publish(ctx context.Context, event Event) error {
    eb.mu.RLock()
    handlers, exists := eb.handlers[event.Type]
    eb.mu.RUnlock()

    if !exists {
        return nil // No subscribers
    }

    // Process handlers concurrently
    var wg sync.WaitGroup
    errChan := make(chan error, len(handlers))

    for _, handler := range handlers {
        wg.Add(1)
        go func(h EventHandler) {
            defer wg.Done()
            if err := h(ctx, event); err != nil {
                errChan <- err
            }
        }(handler)
    }

    wg.Wait()
    close(errChan)

    // Collect errors
    for err := range errChan {
        if err != nil {
            return err
        }
    }

    return nil
}

// Event types for LLM Cost Ops
const (
    EventTypeLLMRequestCompleted = "llm.request.completed"
    EventTypeCostThresholdExceeded = "cost.threshold.exceeded"
    EventTypeUserRegistered = "user.registered"
    EventTypeBillingInvoiceGenerated = "billing.invoice.generated"
)

// Example event handlers
func handleLLMRequestCompleted(ctx context.Context, event Event) error {
    fmt.Printf("Processing LLM request completion: %v\n", event.Data)

    // Extract data
    userID := event.Data["user_id"].(string)
    cost := event.Data["cost"].(float64)

    // Update cost tracking
    // Send to analytics
    // Update user usage stats

    return nil
}

func handleCostThresholdExceeded(ctx context.Context, event Event) error {
    fmt.Printf("Cost threshold exceeded: %v\n", event.Data)

    userID := event.Data["user_id"].(string)
    threshold := event.Data["threshold"].(float64)
    current := event.Data["current_cost"].(float64)

    // Send notification to user
    // Send alert to admin
    // Optionally throttle user requests

    return nil
}

// Distributed event bus using message queue
type DistributedEventBus struct {
    publisher  MessagePublisher
    subscriber MessageSubscriber
    localBus   *EventBus
}

type MessagePublisher interface {
    Publish(topic string, message []byte) error
}

type MessageSubscriber interface {
    Subscribe(topic string, handler func([]byte) error) error
}

func NewDistributedEventBus(pub MessagePublisher, sub MessageSubscriber) *DistributedEventBus {
    return &DistributedEventBus{
        publisher:  pub,
        subscriber: sub,
        localBus:   NewEventBus(),
    }
}

func (deb *DistributedEventBus) Publish(ctx context.Context, event Event) error {
    // Serialize event
    data, err := json.Marshal(event)
    if err != nil {
        return err
    }

    // Publish to message queue
    return deb.publisher.Publish(event.Type, data)
}

func (deb *DistributedEventBus) Subscribe(eventType string, handler EventHandler) error {
    return deb.subscriber.Subscribe(eventType, func(data []byte) error {
        var event Event
        if err := json.Unmarshal(data, &event); err != nil {
            return err
        }

        return handler(context.Background(), event)
    })
}

// Example usage
func main() {
    bus := NewEventBus()

    // Subscribe to events
    bus.Subscribe(EventTypeLLMRequestCompleted, handleLLMRequestCompleted)
    bus.Subscribe(EventTypeCostThresholdExceeded, handleCostThresholdExceeded)

    // Publish event
    event := Event{
        ID:        "evt_123",
        Type:      EventTypeLLMRequestCompleted,
        Timestamp: time.Now(),
        Source:    "llm-proxy-service",
        Data: map[string]interface{}{
            "user_id":      "user-123",
            "request_id":   "req-456",
            "model":        "gpt-4",
            "tokens":       1500,
            "cost":         0.015,
            "duration_ms":  2500,
        },
    }

    bus.Publish(context.Background(), event)
}
```

### Event Sourcing

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    UserRegistered,
    APIKeyCreated,
    LLMRequestStarted,
    LLMRequestCompleted,
    LLMRequestFailed,
    CostRecorded,
    BudgetAlertTriggered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: String,
    pub event_type: EventType,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub version: i64,
}

pub trait Aggregate {
    fn apply_event(&mut self, event: &DomainEvent);
    fn get_version(&self) -> i64;
}

// Example: User Aggregate
#[derive(Debug, Clone)]
pub struct UserAggregate {
    pub id: String,
    pub email: String,
    pub api_keys: Vec<String>,
    pub total_cost: f64,
    pub total_requests: i64,
    pub version: i64,
}

impl UserAggregate {
    pub fn new(id: String) -> Self {
        Self {
            id,
            email: String::new(),
            api_keys: Vec::new(),
            total_cost: 0.0,
            total_requests: 0,
            version: 0,
        }
    }

    pub fn from_events(id: String, events: Vec<DomainEvent>) -> Self {
        let mut aggregate = Self::new(id);

        for event in events {
            aggregate.apply_event(&event);
        }

        aggregate
    }
}

impl Aggregate for UserAggregate {
    fn apply_event(&mut self, event: &DomainEvent) {
        match event.event_type {
            EventType::UserRegistered => {
                self.email = event.data.get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
            }

            EventType::APIKeyCreated => {
                if let Some(key) = event.data.get("api_key").and_then(|v| v.as_str()) {
                    self.api_keys.push(key.to_string());
                }
            }

            EventType::CostRecorded => {
                if let Some(cost) = event.data.get("cost").and_then(|v| v.as_f64()) {
                    self.total_cost += cost;
                    self.total_requests += 1;
                }
            }

            _ => {}
        }

        self.version = event.version;
    }

    fn get_version(&self) -> i64 {
        self.version
    }
}

// Event Store
pub trait EventStore {
    fn append_event(&mut self, event: DomainEvent) -> Result<(), String>;
    fn get_events(&self, aggregate_id: &str) -> Result<Vec<DomainEvent>, String>;
    fn get_events_from_version(&self, aggregate_id: &str, from_version: i64)
        -> Result<Vec<DomainEvent>, String>;
}

pub struct InMemoryEventStore {
    events: Vec<DomainEvent>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
}

impl EventStore for InMemoryEventStore {
    fn append_event(&mut self, event: DomainEvent) -> Result<(), String> {
        self.events.push(event);
        Ok(())
    }

    fn get_events(&self, aggregate_id: &str) -> Result<Vec<DomainEvent>, String> {
        Ok(self.events
            .iter()
            .filter(|e| e.aggregate_id == aggregate_id)
            .cloned()
            .collect())
    }

    fn get_events_from_version(&self, aggregate_id: &str, from_version: i64)
        -> Result<Vec<DomainEvent>, String> {
        Ok(self.events
            .iter()
            .filter(|e| e.aggregate_id == aggregate_id && e.version > from_version)
            .cloned()
            .collect())
    }
}

// Example usage
pub fn example_event_sourcing() {
    let mut event_store = InMemoryEventStore::new();

    // Create events
    let user_id = "user-123".to_string();

    let event1 = DomainEvent {
        id: "evt-1".to_string(),
        event_type: EventType::UserRegistered,
        aggregate_id: user_id.clone(),
        aggregate_type: "User".to_string(),
        data: {
            let mut map = HashMap::new();
            map.insert("email".to_string(),
                      serde_json::Value::String("user@example.com".to_string()));
            map
        },
        timestamp: Utc::now(),
        version: 1,
    };

    let event2 = DomainEvent {
        id: "evt-2".to_string(),
        event_type: EventType::APIKeyCreated,
        aggregate_id: user_id.clone(),
        aggregate_type: "User".to_string(),
        data: {
            let mut map = HashMap::new();
            map.insert("api_key".to_string(),
                      serde_json::Value::String("sk-123".to_string()));
            map
        },
        timestamp: Utc::now(),
        version: 2,
    };

    let event3 = DomainEvent {
        id: "evt-3".to_string(),
        event_type: EventType::CostRecorded,
        aggregate_id: user_id.clone(),
        aggregate_type: "User".to_string(),
        data: {
            let mut map = HashMap::new();
            map.insert("cost".to_string(), serde_json::Value::from(10.50));
            map
        },
        timestamp: Utc::now(),
        version: 3,
    };

    // Append events
    event_store.append_event(event1).unwrap();
    event_store.append_event(event2).unwrap();
    event_store.append_event(event3).unwrap();

    // Rebuild aggregate from events
    let events = event_store.get_events(&user_id).unwrap();
    let user = UserAggregate::from_events(user_id, events);

    println!("User email: {}", user.email);
    println!("API keys: {:?}", user.api_keys);
    println!("Total cost: ${}", user.total_cost);
    println!("Total requests: {}", user.total_requests);
    println!("Version: {}", user.version);
}
```

---

## CQRS and Event Sourcing

### Command Query Responsibility Segregation

```python
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Optional
from datetime import datetime

# Commands (Write Model)
@dataclass
class Command(ABC):
    """Base command class"""
    command_id: str
    timestamp: datetime
    user_id: str

@dataclass
class CreateAPIKeyCommand(Command):
    description: str
    permissions: list[str]

@dataclass
class RecordLLMRequestCommand(Command):
    model: str
    prompt: str
    response: str
    tokens: int
    cost: float

# Command Handlers
class CommandHandler(ABC):
    @abstractmethod
    async def handle(self, command: Command) -> Any:
        pass

class CreateAPIKeyHandler(CommandHandler):
    def __init__(self, event_store, event_bus):
        self.event_store = event_store
        self.event_bus = event_bus

    async def handle(self, command: CreateAPIKeyCommand) -> str:
        # Validate command
        if not command.description:
            raise ValueError("Description required")

        # Generate API key
        api_key = self.generate_api_key()

        # Create domain event
        event = {
            'event_type': 'APIKeyCreated',
            'aggregate_id': command.user_id,
            'data': {
                'api_key': api_key,
                'description': command.description,
                'permissions': command.permissions
            },
            'timestamp': datetime.now()
        }

        # Store event
        await self.event_store.append(event)

        # Publish event
        await self.event_bus.publish(event)

        return api_key

    def generate_api_key(self) -> str:
        import secrets
        return f"sk-{secrets.token_urlsafe(32)}"

# Queries (Read Model)
@dataclass
class Query(ABC):
    """Base query class"""
    query_id: str

@dataclass
class GetUserCostsQuery(Query):
    user_id: str
    start_date: datetime
    end_date: datetime

@dataclass
class GetAPIKeysQuery(Query):
    user_id: str

# Query Handlers
class QueryHandler(ABC):
    @abstractmethod
    async def handle(self, query: Query) -> Any:
        pass

class GetUserCostsHandler(QueryHandler):
    def __init__(self, read_model_db):
        self.db = read_model_db

    async def handle(self, query: GetUserCostsQuery) -> dict:
        # Query optimized read model
        return await self.db.query("""
            SELECT
                SUM(cost) as total_cost,
                COUNT(*) as total_requests,
                AVG(cost) as avg_cost
            FROM llm_requests_summary
            WHERE user_id = $1
              AND date >= $2
              AND date <= $3
        """, query.user_id, query.start_date, query.end_date)

# Read Model Projector
class ReadModelProjector:
    """Project events to read model"""

    def __init__(self, event_bus, read_model_db):
        self.event_bus = event_bus
        self.db = read_model_db

        # Subscribe to events
        self.event_bus.subscribe('LLMRequestCompleted',
                                self.on_llm_request_completed)
        self.event_bus.subscribe('APIKeyCreated',
                                self.on_api_key_created)

    async def on_llm_request_completed(self, event: dict):
        """Update read model when LLM request completes"""
        await self.db.execute("""
            INSERT INTO llm_requests_summary
            (user_id, date, model, tokens, cost)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, date, model)
            DO UPDATE SET
                tokens = llm_requests_summary.tokens + $4,
                cost = llm_requests_summary.cost + $5,
                request_count = llm_requests_summary.request_count + 1
        """,
            event['data']['user_id'],
            event['timestamp'].date(),
            event['data']['model'],
            event['data']['tokens'],
            event['data']['cost']
        )

    async def on_api_key_created(self, event: dict):
        """Update API keys read model"""
        await self.db.execute("""
            INSERT INTO api_keys_view
            (user_id, api_key_hash, description, permissions, created_at)
            VALUES ($1, $2, $3, $4, $5)
        """,
            event['aggregate_id'],
            self.hash_key(event['data']['api_key']),
            event['data']['description'],
            event['data']['permissions'],
            event['timestamp']
        )

    def hash_key(self, key: str) -> str:
        import hashlib
        return hashlib.sha256(key.encode()).hexdigest()

# CQRS Bus
class CQRSBus:
    """Central bus for commands and queries"""

    def __init__(self):
        self.command_handlers: dict[type, CommandHandler] = {}
        self.query_handlers: dict[type, QueryHandler] = {}

    def register_command_handler(self, command_type: type,
                                 handler: CommandHandler):
        self.command_handlers[command_type] = handler

    def register_query_handler(self, query_type: type,
                               handler: QueryHandler):
        self.query_handlers[query_type] = handler

    async def execute_command(self, command: Command) -> Any:
        handler = self.command_handlers.get(type(command))
        if not handler:
            raise ValueError(f"No handler for command: {type(command)}")

        return await handler.handle(command)

    async def execute_query(self, query: Query) -> Any:
        handler = self.query_handlers.get(type(query))
        if not handler:
            raise ValueError(f"No handler for query: {type(query)}")

        return await handler.handle(query)

# Example usage
async def example_cqrs():
    bus = CQRSBus()

    # Register handlers
    bus.register_command_handler(
        CreateAPIKeyCommand,
        CreateAPIKeyHandler(event_store, event_bus)
    )

    bus.register_query_handler(
        GetUserCostsQuery,
        GetUserCostsHandler(read_model_db)
    )

    # Execute command (write)
    command = CreateAPIKeyCommand(
        command_id='cmd-123',
        timestamp=datetime.now(),
        user_id='user-123',
        description='Production API key',
        permissions=['read', 'write']
    )

    api_key = await bus.execute_command(command)
    print(f"Created API key: {api_key}")

    # Execute query (read)
    query = GetUserCostsQuery(
        query_id='qry-456',
        user_id='user-123',
        start_date=datetime(2025, 1, 1),
        end_date=datetime(2025, 1, 31)
    )

    costs = await bus.execute_query(query)
    print(f"User costs: {costs}")
```

---

## API Gateway Pattern

### Gateway Implementation

```javascript
const express = require('express');
const { createProxyMiddleware } = require('http-proxy-middleware');
const rateLimit = require('express-rate-limit');
const jwt = require('jsonwebtoken');

class APIGateway {
  constructor(config) {
    this.app = express();
    this.config = config;
    this.setupMiddleware();
    this.setupRoutes();
  }

  setupMiddleware() {
    // CORS
    this.app.use((req, res, next) => {
      res.header('Access-Control-Allow-Origin', '*');
      res.header('Access-Control-Allow-Methods', 'GET,PUT,POST,DELETE');
      res.header('Access-Control-Allow-Headers', 'Content-Type, Authorization');
      next();
    });

    // Request logging
    this.app.use((req, res, next) => {
      console.log(`${new Date().toISOString()} ${req.method} ${req.path}`);
      next();
    });

    // Rate limiting
    const limiter = rateLimit({
      windowMs: 15 * 60 * 1000, // 15 minutes
      max: 100, // Limit each IP to 100 requests per windowMs
      message: 'Too many requests from this IP'
    });

    this.app.use('/api/', limiter);

    // Authentication
    this.app.use('/api/', this.authenticateRequest.bind(this));
  }

  authenticateRequest(req, res, next) {
    const token = req.headers['authorization']?.replace('Bearer ', '');

    if (!token) {
      return res.status(401).json({ error: 'No token provided' });
    }

    try {
      const decoded = jwt.verify(token, process.env.JWT_SECRET);
      req.user = decoded;
      next();
    } catch (error) {
      return res.status(401).json({ error: 'Invalid token' });
    }
  }

  setupRoutes() {
    // Route to Authentication Service
    this.app.use('/api/auth', createProxyMiddleware({
      target: this.config.services.auth,
      changeOrigin: true,
      pathRewrite: { '^/api/auth': '' },
      onProxyReq: this.addRequestId,
      onProxyRes: this.logResponse
    }));

    // Route to LLM Proxy Service
    this.app.use('/api/llm', createProxyMiddleware({
      target: this.config.services.llmProxy,
      changeOrigin: true,
      pathRewrite: { '^/api/llm': '' },
      onProxyReq: (proxyReq, req) => {
        this.addRequestId(proxyReq, req);
        this.addUserContext(proxyReq, req);
      }
    }));

    // Route to Cost Tracking Service
    this.app.use('/api/costs', createProxyMiddleware({
      target: this.config.services.costTracking,
      changeOrigin: true,
      pathRewrite: { '^/api/costs': '' },
      onProxyReq: (proxyReq, req) => {
        this.addRequestId(proxyReq, req);
        this.addUserContext(proxyReq, req);
      }
    }));

    // Route to Analytics Service
    this.app.use('/api/analytics', createProxyMiddleware({
      target: this.config.services.analytics,
      changeOrigin: true,
      pathRewrite: { '^/api/analytics': '' }
    }));

    // Health check
    this.app.get('/health', (req, res) => {
      res.json({ status: 'healthy', timestamp: new Date().toISOString() });
    });

    // Aggregation endpoint (combines multiple services)
    this.app.get('/api/dashboard', async (req, res) => {
      try {
        const [costs, usage, analytics] = await Promise.all([
          this.fetchCosts(req.user.id),
          this.fetchUsage(req.user.id),
          this.fetchAnalytics(req.user.id)
        ]);

        res.json({
          costs,
          usage,
          analytics
        });
      } catch (error) {
        res.status(500).json({ error: 'Failed to fetch dashboard data' });
      }
    });
  }

  addRequestId(proxyReq, req) {
    const requestId = `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    proxyReq.setHeader('X-Request-ID', requestId);
    req.requestId = requestId;
  }

  addUserContext(proxyReq, req) {
    if (req.user) {
      proxyReq.setHeader('X-User-ID', req.user.id);
      proxyReq.setHeader('X-User-Tier', req.user.tier || 'free');
    }
  }

  logResponse(proxyRes, req, res) {
    console.log(`Response: ${proxyRes.statusCode} for ${req.requestId}`);
  }

  async fetchCosts(userId) {
    const axios = require('axios');
    const response = await axios.get(
      `${this.config.services.costTracking}/summary`,
      { params: { user_id: userId } }
    );
    return response.data;
  }

  async fetchUsage(userId) {
    const axios = require('axios');
    const response = await axios.get(
      `${this.config.services.llmProxy}/usage`,
      { params: { user_id: userId } }
    );
    return response.data;
  }

  async fetchAnalytics(userId) {
    const axios = require('axios');
    const response = await axios.get(
      `${this.config.services.analytics}/summary`,
      { params: { user_id: userId } }
    );
    return response.data;
  }

  start(port) {
    this.app.listen(port, () => {
      console.log(`API Gateway listening on port ${port}`);
    });
  }
}

// Usage
const gateway = new APIGateway({
  services: {
    auth: 'http://auth-service:8080',
    llmProxy: 'http://llm-proxy:8081',
    costTracking: 'http://cost-tracking:8082',
    analytics: 'http://analytics:8083'
  }
});

gateway.start(3000);
```

---

*Due to length, I'll summarize the remaining sections with key code examples and concepts...*

## Circuit Breaker Pattern

```python
from enum import Enum
from datetime import datetime, timedelta
from typing import Callable, Any

class CircuitState(Enum):
    CLOSED = "closed"      # Normal operation
    OPEN = "open"          # Failing, reject requests
    HALF_OPEN = "half_open"  # Testing if service recovered

class CircuitBreaker:
    """Prevent cascading failures"""

    def __init__(self, failure_threshold: int = 5,
                 timeout: int = 60, recovery_timeout: int = 30):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.recovery_timeout = recovery_timeout

        self.failure_count = 0
        self.last_failure_time = None
        self.state = CircuitState.CLOSED

    def call(self, func: Callable, *args, **kwargs) -> Any:
        if self.state == CircuitState.OPEN:
            if self._should_attempt_reset():
                self.state = CircuitState.HALF_OPEN
            else:
                raise Exception("Circuit breaker is OPEN")

        try:
            result = func(*args, **kwargs)
            self._on_success()
            return result
        except Exception as e:
            self._on_failure()
            raise e

    def _on_success(self):
        self.failure_count = 0
        self.state = CircuitState.CLOSED

    def _on_failure(self):
        self.failure_count += 1
        self.last_failure_time = datetime.now()

        if self.failure_count >= self.failure_threshold:
            self.state = CircuitState.OPEN

    def _should_attempt_reset(self) -> bool:
        return (datetime.now() - self.last_failure_time).seconds >= self.recovery_timeout

# Usage with LLM API calls
breaker = CircuitBreaker(failure_threshold=3, recovery_timeout=30)

def call_llm_with_circuit_breaker(prompt):
    return breaker.call(call_llm_api, prompt)
```

---

## Implementation Checklist

### Phase 1: Foundation (Weeks 1-2)
- [ ] Design microservices boundaries
- [ ] Set up API gateway
- [ ] Implement service discovery
- [ ] Configure load balancer

### Phase 2: Resilience (Weeks 3-4)
- [ ] Implement circuit breakers
- [ ] Add retry logic
- [ ] Set up health checks
- [ ] Configure timeouts

### Phase 3: Events & CQRS (Weeks 5-8)
- [ ] Set up event bus
- [ ] Implement event sourcing
- [ ] Build read models
- [ ] Add projections

### Phase 4: Advanced Patterns (Weeks 9-12)
- [ ] Implement saga pattern
- [ ] Set up service mesh
- [ ] Multi-region deployment
- [ ] Disaster recovery

---

## Tools and Resources

### Architecture Tools
- **Draw.io** - Architecture diagrams
- **PlantUML** - Diagram as code
- **C4 Model** - Software architecture documentation

### Microservices
- **Kubernetes** - Container orchestration
- **Istio** - Service mesh
- **Consul** - Service discovery

### Event Processing
- **Apache Kafka** - Event streaming
- **RabbitMQ** - Message broker
- **AWS EventBridge** - Serverless events

---

*Last Updated: 2025-11-16*
*Version: 1.0*
