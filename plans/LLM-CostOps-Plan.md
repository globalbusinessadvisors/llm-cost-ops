# LLM-CostOps: Technical Research and Build Plan

**Version:** 1.0.0
**Date:** 2025-11-15
**Status:** Complete SPARC Specification
**Author:** LLM DevOps Platform Team

---

## Executive Summary

LLM-CostOps is a comprehensive cost operations platform designed as a critical component of the LLM DevOps ecosystem. This document presents a complete technical research and build plan following Reuven Cohen's SPARC methodology (Specification, Pseudocode, Architecture, Refinement, and Completion). The system provides token accounting, cost forecasting, ROI correlation, and multi-provider cost analysis for Large Language Model deployments.

**Core Purpose:** Enable organizations to understand, predict, and optimize LLM operational expenditures across diverse providers through real-time tracking, intelligent forecasting, and automated cost governance.

**Technology Stack:** Rust-based microservices, TimescaleDB for time-series data, statistical and ML-based forecasting, with deployment modes spanning standalone CLI, API microservice, and embedded library.

**Timeline:** 24-week phased roadmap (MVP → Beta → v1.0) with clear milestones, dependencies, and validation metrics.

---

## Table of Contents

1. [SPARC Phase 1: Specification](#sparc-phase-1-specification)
2. [SPARC Phase 2: Pseudocode](#sparc-phase-2-pseudocode)
3. [SPARC Phase 3: Architecture](#sparc-phase-3-architecture)
4. [SPARC Phase 4: Refinement](#sparc-phase-4-refinement)
5. [SPARC Phase 5: Completion](#sparc-phase-5-completion)
6. [References](#references)

---

# SPARC Phase 1: Specification

## 1. Purpose & Scope

### 1.1 Module Overview

**LLM-CostOps** is a critical financial intelligence module within the LLM DevOps ecosystem, responsible for comprehensive token accounting, cost forecasting, ROI correlation, and multi-provider cost analysis. It serves as the financial backbone that enables organizations to understand, predict, and optimize their LLM operational expenditures across diverse providers and deployment scenarios.

### 1.2 Ecosystem Position

**Core Alignment:** Governance Core
**Module Category:** Financial Intelligence & Accountability

LLM-CostOps sits at the intersection of operational monitoring and financial governance, bridging the gap between:
- Technical metrics (LLM-Observatory, LLM-Test-Bench)
- Business intelligence (LLM-Governance-Core)
- Operational optimization (LLM-Auto-Optimizer, LLM-Edge-Agent)

```
┌─────────────────────────────────────────────────────────┐
│                   LLM DevOps Ecosystem                   │
├─────────────────────────────────────────────────────────┤
│  Intelligence Core                                       │
│    └─> LLM-Observatory ──────┐                         │
│                               ↓                          │
│  Automation Core             [LLM-CostOps]              │
│    ├─> LLM-Auto-Optimizer ←──┤                         │
│    └─> LLM-Edge-Agent ───────┘                         │
│                               ↑                          │
│  Governance Core             │                          │
│    ├─> LLM-Governance-Core ←─┤                         │
│    └─> LLM-Registry ──────────┘                         │
│                                                          │
│  Research Core                                           │
│    └─> LLM-Test-Bench ────> Cost Correlation            │
└─────────────────────────────────────────────────────────┘
```

### 1.3 Core Value Proposition

LLM-CostOps delivers four primary value streams:

1. **Financial Visibility**: Real-time and historical cost tracking across all LLM operations
2. **Predictive Intelligence**: Forecasting future costs based on usage patterns and trends
3. **ROI Attribution**: Correlating costs with performance outcomes and business value
4. **Cost Optimization**: Enabling intelligent routing and provider selection based on cost efficiency

### 1.4 Key Stakeholders

#### Primary Stakeholders

| Stakeholder Group | Needs | Interaction Mode |
|------------------|-------|------------------|
| **Platform Engineers** | Real-time cost monitoring, debugging cost spikes, provider comparison | API, CLI, Dashboards |
| **DevOps Teams** | Budget alerts, cost attribution by service/team, deployment cost analysis | Integration APIs, Webhooks |
| **Financial Controllers** | Monthly/quarterly reports, budget forecasting, variance analysis | Reports, Exports, APIs |
| **ML/AI Engineers** | Cost-per-experiment tracking, model efficiency comparison | Programmatic API |
| **C-Level Executives** | Total cost of ownership (TCO), ROI metrics, strategic cost insights | Executive Dashboards, Reports |

#### Secondary Stakeholders

- **Security Teams**: Cost anomaly detection as a security signal
- **Procurement Teams**: Provider rate negotiation data, volume discounts
- **Product Managers**: Feature cost analysis, user segment profitability

### 1.5 Scope Boundaries

#### In Scope

- Token counting and normalization across providers
- Cost calculation using current and historical provider rates
- Budget tracking and enforcement
- Cost forecasting using statistical and ML models
- ROI correlation with performance metrics
- Multi-provider cost comparison and optimization
- Cost attribution (by team, service, model, endpoint)
- Rate card management and updates
- Cost alerting and anomaly detection

#### Out of Scope

- Direct billing/payment processing (handled by providers)
- Invoice reconciliation (handled by finance systems)
- General cloud infrastructure costs (compute, storage outside LLM operations)
- Human resource costs and operational overhead
- Marketing and customer acquisition costs
- License management for non-LLM software

#### Future Considerations

- Predictive autoscaling based on cost thresholds
- Automated provider switching for cost optimization
- Carbon cost and environmental impact tracking
- Cost optimization recommendations using AI agents

---

## 2. Functional Requirements

### 2.1 Usage Metric Collection

**FR-001: Real-time Token Counting**
- **Priority:** P0 (Critical)
- **Description:** Capture token usage from all LLM interactions with sub-second latency
- **Acceptance Criteria:**
  - Capture input tokens, output tokens, and total tokens for each request
  - Support streaming responses with incremental token counting
  - Handle batch processing with aggregated metrics
  - Record timestamp, model ID, endpoint, and request metadata
  - Support provider-specific token counting algorithms (GPT-4, Claude, Llama, etc.)

**FR-002: Integration with LLM-Observatory**
- **Priority:** P0 (Critical)
- **Description:** Consume usage metrics from LLM-Observatory's monitoring pipeline
- **Acceptance Criteria:**
  - Subscribe to real-time metric streams via pub/sub or event bus
  - Parse and normalize metrics from multiple data formats
  - Handle metric buffering during network interruptions
  - Support batch ingestion for historical data backfill
  - Validate metric integrity and detect missing data points

**FR-003: Integration with LLM-Edge-Agent**
- **Priority:** P1 (High)
- **Description:** Collect usage data from distributed edge deployments
- **Acceptance Criteria:**
  - Receive aggregated metrics from edge agents at configurable intervals
  - Handle offline edge scenarios with eventual consistency
  - Support metric compression for bandwidth-constrained environments
  - Correlate edge usage with centralized cost tracking
  - Track edge-specific cost factors (e.g., on-device vs API costs)

**FR-004: Custom Metric Extensions**
- **Priority:** P2 (Medium)
- **Description:** Allow teams to define custom cost-related metrics
- **Acceptance Criteria:**
  - Support plugin architecture for custom metric collectors
  - Define schema for custom metrics (name, type, aggregation method)
  - Enable custom dimensions for cost attribution
  - Validate custom metrics against predefined constraints

### 2.2 Token-to-Cost Mapping

**FR-005: Multi-Provider Rate Cards**
- **Priority:** P0 (Critical)
- **Description:** Maintain accurate pricing information for all supported LLM providers
- **Acceptance Criteria:**
  - Store rate cards for 10+ major providers (OpenAI, Anthropic, Google, AWS, Azure, etc.)
  - Support tiered pricing (pay-as-you-go, committed use, enterprise contracts)
  - Handle regional pricing variations
  - Track promotional pricing and temporary discounts
  - Version rate cards with effective date ranges
  - Support both input/output token pricing and request-based pricing

**FR-006: Dynamic Rate Updates**
- **Priority:** P1 (High)
- **Description:** Automatically update pricing information from provider sources
- **Acceptance Criteria:**
  - Integrate with LLM-Registry for provider metadata
  - Poll provider pricing APIs or scrape public pricing pages
  - Detect rate changes and trigger alerts
  - Support manual rate overrides for negotiated contracts
  - Maintain audit trail of all rate changes

**FR-007: Cost Calculation Engine**
- **Priority:** P0 (Critical)
- **Description:** Convert token usage to monetary costs with high precision
- **Acceptance Criteria:**
  - Calculate costs using appropriate rate card version
  - Support multiple currencies (USD, EUR, GBP, etc.)
  - Handle fractional token costs (micro-transactions)
  - Apply discounts and credits automatically
  - Support cost recalculation for historical data when rates change
  - Precision: 6 decimal places minimum for currency calculations

**FR-008: Cost Allocation and Tagging**
- **Priority:** P1 (High)
- **Description:** Attribute costs to organizational units, projects, and services
- **Acceptance Criteria:**
  - Support hierarchical cost centers (org > team > project > service)
  - Enable tagging at request time (environment, feature, user segment)
  - Implement cost allocation rules (shared services, overhead)
  - Support retroactive tag assignment
  - Generate cost reports by any tag dimension

### 2.3 Performance Correlation

**FR-009: Integration with LLM-Test-Bench**
- **Priority:** P1 (High)
- **Description:** Correlate costs with performance metrics from testing infrastructure
- **Acceptance Criteria:**
  - Import performance benchmarks (latency, throughput, quality scores)
  - Calculate cost-per-quality-point metrics
  - Generate efficiency reports comparing models/providers
  - Track cost vs performance tradeoffs over time
  - Support A/B test cost analysis

**FR-010: ROI Calculation Framework**
- **Priority:** P2 (Medium)
- **Description:** Measure return on investment for LLM deployments
- **Acceptance Criteria:**
  - Define ROI calculation templates (revenue/cost, efficiency gains, etc.)
  - Integrate with external business metrics systems
  - Track cost savings from optimization initiatives
  - Generate ROI reports for specific time periods
  - Support what-if analysis for proposed changes

**FR-011: Cost-Efficiency Scoring**
- **Priority:** P2 (Medium)
- **Description:** Assign efficiency scores to models, providers, and deployments
- **Acceptance Criteria:**
  - Normalize costs across different quality/performance levels
  - Calculate cost-efficiency index (0-100 scale)
  - Identify cost outliers and inefficiencies
  - Rank providers by cost-efficiency for similar workloads
  - Generate recommendations for cost-optimal configurations

### 2.4 Budget Enforcement

**FR-012: Budget Definition and Tracking**
- **Priority:** P0 (Critical)
- **Description:** Enable teams to set and monitor budgets at multiple levels
- **Acceptance Criteria:**
  - Support budget hierarchies (organization > department > team > project)
  - Define budgets by time period (daily, weekly, monthly, quarterly, annual)
  - Track actual vs budgeted spend in real-time
  - Calculate burn rate and projected end-of-period spend
  - Support budget amendments and transfers

**FR-013: Integration with LLM-Governance-Core**
- **Priority:** P0 (Critical)
- **Description:** Enforce budget policies through governance framework
- **Acceptance Criteria:**
  - Register budget policies in governance engine
  - Trigger policy violations when budgets are exceeded
  - Support soft limits (warnings) and hard limits (blocking)
  - Generate audit logs for all budget-related actions
  - Enable governance-driven auto-scaling or throttling

**FR-014: Alerting and Notifications**
- **Priority:** P1 (High)
- **Description:** Notify stakeholders of budget events and anomalies
- **Acceptance Criteria:**
  - Support multiple notification channels (email, Slack, PagerDuty, webhooks)
  - Configure alert thresholds (50%, 75%, 90%, 100% of budget)
  - Detect spending anomalies using statistical methods
  - Rate limit notifications to prevent alert fatigue
  - Support escalation paths for critical budget violations

**FR-015: Budget Forecasting**
- **Priority:** P1 (High)
- **Description:** Predict future costs based on historical trends
- **Acceptance Criteria:**
  - Apply time series forecasting models (ARIMA, exponential smoothing, ML)
  - Generate forecasts at multiple time horizons (1 week, 1 month, 1 quarter)
  - Include confidence intervals in forecasts
  - Detect seasonality and trend changes
  - Adjust forecasts based on planned changes (new features, scaling events)

### 2.5 Cost-Aware Routing

**FR-016: Integration with LLM-Auto-Optimizer**
- **Priority:** P1 (High)
- **Description:** Provide cost data to routing and optimization engine
- **Acceptance Criteria:**
  - Expose real-time cost APIs for routing decisions
  - Calculate cost-per-request estimates for different providers/models
  - Support cost-constrained routing policies
  - Track cost savings from optimized routing
  - Provide feedback loop for routing algorithm tuning

**FR-017: Cost-Performance Pareto Frontier**
- **Priority:** P2 (Medium)
- **Description:** Identify optimal cost-performance tradeoff options
- **Acceptance Criteria:**
  - Calculate Pareto frontier for cost vs quality/latency
  - Recommend provider/model combinations for different use cases
  - Update frontier dynamically as prices and performance change
  - Visualize tradeoff curves in dashboards
  - Support constraint-based optimization (max cost, min quality)

**FR-018: Cost-Based Load Balancing**
- **Priority:** P2 (Medium)
- **Description:** Enable load balancing strategies that consider costs
- **Acceptance Criteria:**
  - Weight routing decisions by cost efficiency
  - Support cost-aware failover (prefer cheaper backup providers)
  - Balance cost optimization with latency and availability requirements
  - Track load balancing metrics and cost impact
  - Integrate with provider rate limits and quotas

---

## 3. Non-Functional Requirements

### 3.1 Performance Constraints

**NFR-001: Real-time Processing Latency**
- **Requirement:** Process incoming usage metrics with p99 latency < 100ms
- **Rationale:** Enable real-time cost monitoring and budget enforcement
- **Measurement:** Monitor metric ingestion pipeline latency using LLM-Observatory

**NFR-002: Batch Processing Throughput**
- **Requirement:** Process 1M+ cost records per minute for historical analysis
- **Rationale:** Support large-scale cost recalculation and reporting
- **Measurement:** Benchmark batch processing jobs with production-scale data

**NFR-003: Query Performance**
- **Requirement:** Return cost reports for 90 days of data in < 5 seconds
- **Rationale:** Enable interactive cost exploration and analysis
- **Measurement:** Monitor query response times for common report types

**NFR-004: API Response Time**
- **Requirement:** Cost estimation API responds in < 50ms at p95
- **Rationale:** Support real-time cost-aware routing decisions
- **Measurement:** Track API latency metrics in production

### 3.2 Scalability Requirements

**NFR-005: Horizontal Scalability**
- **Requirement:** Scale to handle 10x traffic increase without architecture changes
- **Rationale:** Support rapid growth in LLM usage across organization
- **Design:** Stateless services, distributed data stores, message queues

**NFR-006: Data Volume Capacity**
- **Requirement:** Store and analyze 1B+ cost records (5 years of history)
- **Rationale:** Enable long-term cost trend analysis and forecasting
- **Design:** Time-series optimized database, data archival policies

**NFR-007: Multi-Tenancy Support**
- **Requirement:** Support 1000+ tenants with isolated cost tracking
- **Rationale:** Enable SaaS deployment model for LLM DevOps platform
- **Design:** Tenant-aware data partitioning, resource isolation

### 3.3 Data Accuracy and Precision

**NFR-008: Token Counting Accuracy**
- **Requirement:** 100% accuracy in token counting (zero tolerance for errors)
- **Rationale:** Incorrect token counts lead to billing discrepancies and loss of trust
- **Validation:** Cross-validate against provider token counts for 10,000+ requests

**NFR-009: Cost Calculation Precision**
- **Requirement:** Currency calculations accurate to 6 decimal places
- **Rationale:** Prevent rounding errors in high-volume micro-transactions
- **Design:** Use fixed-point decimal arithmetic (not floating-point)

**NFR-010: Rate Card Freshness**
- **Requirement:** Provider rate cards updated within 24 hours of public announcement
- **Rationale:** Ensure cost estimates reflect current market pricing
- **Design:** Automated rate monitoring with manual override capability

**NFR-011: Data Consistency**
- **Requirement:** 99.99% consistency between cost calculations and source metrics
- **Rationale:** Enable reliable cost attribution and billing
- **Design:** Idempotent processing, reconciliation jobs, audit trails

### 3.4 Reliability and Availability

**NFR-012: Service Availability**
- **Requirement:** 99.9% uptime for cost tracking services (43 minutes downtime/month)
- **Rationale:** Cost tracking should not be a single point of failure for LLM operations
- **Design:** Active-passive failover, graceful degradation

**NFR-013: Data Durability**
- **Requirement:** 99.999999999% durability for cost records (11 nines)
- **Rationale:** Cost data is critical financial record that must never be lost
- **Design:** Replicated storage, automated backups, disaster recovery

**NFR-014: Fault Tolerance**
- **Requirement:** Continue operating during partial infrastructure failures
- **Rationale:** Maintain cost visibility even during incidents
- **Design:** Circuit breakers, fallback mechanisms, local caching

### 3.5 Security and Privacy

**NFR-015: Data Encryption**
- **Requirement:** Encrypt all cost data at rest and in transit
- **Rationale:** Cost information is financially sensitive and confidential
- **Design:** TLS 1.3 for transit, AES-256 for rest, key rotation policies

**NFR-016: Access Control**
- **Requirement:** Role-based access control (RBAC) for all cost data
- **Rationale:** Limit cost visibility based on organizational hierarchy
- **Design:** Integration with LLM-Governance-Core for policy enforcement

**NFR-017: Audit Logging**
- **Requirement:** Log all access and modifications to cost data
- **Rationale:** Support compliance audits and forensic investigation
- **Design:** Immutable audit logs, centralized log aggregation

**NFR-018: Data Anonymization**
- **Requirement:** Support PII removal from cost attribution metadata
- **Rationale:** Enable cost analysis while protecting user privacy
- **Design:** Configurable PII scrubbing, pseudonymization for analytics

**NFR-019: Compliance Requirements**
- **Requirement:** Meet SOC 2 Type II, ISO 27001, and GDPR requirements
- **Rationale:** Support enterprise deployment in regulated industries
- **Design:** Security controls framework, regular compliance audits

---

## 4. Integration Points

### 4.1 Module Dependencies

#### Intelligence Core

**INT-001: LLM-Observatory**
- **Dependency Type:** Critical (P0)
- **Data Flow Direction:** Inbound (Observatory → CostOps)
- **Integration Pattern:** Event streaming / Pub-Sub
- **Data Exchanged:**
  - Token usage metrics (input, output, total tokens)
  - Request metadata (timestamp, model, endpoint, user/tenant ID)
  - Response metadata (status codes, latency, error rates)
  - Streaming progress indicators
- **SLA Requirements:**
  - Metric delivery latency < 1 second
  - Guaranteed delivery (at-least-once semantics)
  - Metric schema versioning support

**INT-002: LLM-Test-Bench**
- **Dependency Type:** High (P1)
- **Data Flow Direction:** Bidirectional
- **Integration Pattern:** API calls + Async events
- **Data Exchanged:**
  - Outbound: Cost data for test runs, cost-per-benchmark metrics
  - Inbound: Performance benchmarks (quality scores, latency, throughput)
  - Test configurations and model parameters

#### Automation Core

**INT-003: LLM-Auto-Optimizer**
- **Dependency Type:** High (P1)
- **Data Flow Direction:** Bidirectional
- **Integration Pattern:** Synchronous API + Configuration updates
- **Data Exchanged:**
  - Outbound: Real-time cost estimates, cost-efficiency rankings
  - Inbound: Routing decisions, optimization requests
  - Cost-performance tradeoff curves
- **SLA Requirements:**
  - Cost estimate API response < 50ms (p95)
  - Cost rankings updated every 5 minutes

**INT-004: LLM-Edge-Agent**
- **Dependency Type:** Medium (P1)
- **Data Flow Direction:** Inbound (Edge → CostOps)
- **Integration Pattern:** Batch uploads + Periodic sync
- **Data Exchanged:**
  - Aggregated usage metrics from edge deployments
  - Edge-specific cost factors (local compute, bandwidth)
  - Offline operation logs

#### Governance Core

**INT-005: LLM-Governance-Core**
- **Dependency Type:** Critical (P0)
- **Data Flow Direction:** Bidirectional
- **Integration Pattern:** Policy engine integration + Event notifications
- **Data Exchanged:**
  - Outbound: Budget violation events, cost anomalies
  - Inbound: Budget policies, enforcement actions
  - Cost-based access control decisions
- **SLA Requirements:**
  - Policy evaluation < 100ms
  - Event delivery within 1 second
  - Policy updates propagate within 30 seconds

**INT-006: LLM-Registry**
- **Dependency Type:** High (P1)
- **Data Flow Direction:** Bidirectional
- **Integration Pattern:** API calls + Webhook subscriptions
- **Data Exchanged:**
  - Outbound: Cost data for provider comparison
  - Inbound: Provider metadata, model catalogs, rate cards
  - Provider status and availability updates

---

## 5. Success Criteria

### 5.1 Measurable Outcomes

#### Phase 1: Foundation (Months 1-3)

**SC-001: Core Functionality**
- ✅ Process 1M+ token metrics per day with 99.9% accuracy
- ✅ Support 5+ major LLM providers (OpenAI, Anthropic, Google, AWS, Azure)
- ✅ Calculate costs with 100% accuracy vs provider invoices
- ✅ Real-time metric ingestion with p99 latency < 100ms

**SC-002: Integration Completeness**
- ✅ Full integration with LLM-Observatory (metrics collection)
- ✅ Full integration with LLM-Governance-Core (budget enforcement)
- ✅ Basic integration with LLM-Registry (provider metadata)
- ✅ 95% API test coverage for all public endpoints

**SC-003: Operational Readiness**
- ✅ Deploy to staging environment with 99.5% uptime
- ✅ Comprehensive monitoring and alerting in place
- ✅ Runbooks for common operational scenarios
- ✅ Security audit completed with zero critical findings

#### Phase 2: Scale & Optimize (Months 4-6)

**SC-004: Performance & Scale**
- ✅ Handle 10M+ requests per day with linear cost scaling
- ✅ Query performance: p95 < 2 seconds for 90-day reports
- ✅ Support 100+ concurrent tenants with resource isolation
- ✅ Cost estimation API: p95 latency < 50ms

**SC-005: Advanced Features**
- ✅ Budget forecasting with 85%+ accuracy (30-day horizon)
- ✅ Cost anomaly detection with 90% precision, 80% recall
- ✅ Integration with LLM-Auto-Optimizer for cost-aware routing
- ✅ Custom report builder with 10+ customer-generated reports

**SC-006: Reliability**
- ✅ Production deployment with 99.9% uptime
- ✅ Zero data loss incidents
- ✅ Mean time to recovery (MTTR) < 1 hour for incidents
- ✅ Successful disaster recovery drill

#### Phase 3: Intelligence & Optimization (Months 7-12)

**SC-007: Business Value**
- ✅ Enable 20%+ cost savings through optimized routing
- ✅ Reduce budget overruns by 50% through proactive alerts
- ✅ 95%+ customer satisfaction score for cost visibility
- ✅ ROI tracking for 50+ LLM projects

**SC-008: Advanced Analytics**
- ✅ Cost-performance Pareto frontier for 20+ model combinations
- ✅ ML-based forecasting with 90%+ accuracy (90-day horizon)
- ✅ What-if analysis for cost optimization scenarios
- ✅ Commitment recommendation engine (80%+ utilization improvement)

**SC-009: Ecosystem Maturity**
- ✅ Full integration with all 8 core modules
- ✅ Support 10+ LLM providers with automated rate updates
- ✅ Plugin architecture with 5+ reference implementations
- ✅ Public API with 1,000+ external API calls per day

---

# SPARC Phase 2: Pseudocode

## Core Data Flow Algorithms

### 1. Usage Metric Ingestion Pipeline

```pseudocode
FUNCTION IngestUsageMetrics(source: MetricSource) -> Result<()>
    // Multi-source metric ingestion with validation and normalization

    MATCH source:
        CASE Observatory:
            stream = SUBSCRIBE_TO_EVENT_STREAM("llm.usage.metrics")
            FOR EACH event IN stream:
                metric = PARSE_METRIC(event.payload)
                VALIDATE_METRIC_SCHEMA(metric)
                ENQUEUE_FOR_PROCESSING(metric)

        CASE EdgeAgent:
            batch_endpoint = START_HTTP_SERVER("/api/v1/metrics/batch")
            ON POST TO batch_endpoint:
                metrics = DESERIALIZE_BATCH(request.body)
                FOR EACH metric IN metrics:
                    VALIDATE_METRIC_SCHEMA(metric)
                    DEDUPLICATE_CHECK(metric.id)
                    ENQUEUE_FOR_PROCESSING(metric)
                RETURN HTTP 202 ACCEPTED

        CASE DirectAPI:
            api_endpoint = START_HTTP_SERVER("/api/v1/metrics/single")
            ON POST TO api_endpoint:
                metric = DESERIALIZE(request.body)
                VALIDATE_METRIC_SCHEMA(metric)
                ENQUEUE_FOR_PROCESSING(metric)
                RETURN HTTP 202 ACCEPTED

    // Asynchronous processing worker
    SPAWN_WORKER_POOL(num_workers=8):
        LOOP:
            metric = DEQUEUE_FROM_PROCESSING_QUEUE()
            normalized = NORMALIZE_METRIC(metric)
            enriched = ENRICH_WITH_METADATA(normalized)
            PERSIST_TO_DATABASE(enriched)
            EMIT_EVENT("usage.metric.ingested", enriched)
END FUNCTION

FUNCTION NormalizeMetric(raw_metric: RawMetric) -> NormalizedMetric
    // Provider-specific normalization
    normalized = NEW NormalizedMetric()
    normalized.timestamp = raw_metric.timestamp
    normalized.provider = NORMALIZE_PROVIDER_NAME(raw_metric.provider)
    normalized.model = NORMALIZE_MODEL_NAME(raw_metric.model, normalized.provider)

    // Token normalization
    IF raw_metric.prompt_tokens IS SET:
        normalized.input_tokens = raw_metric.prompt_tokens
    ELSE IF raw_metric.text IS SET:
        // Estimate tokens from text if not provided
        normalized.input_tokens = ESTIMATE_TOKENS(raw_metric.text, normalized.model)

    IF raw_metric.completion_tokens IS SET:
        normalized.output_tokens = raw_metric.completion_tokens
    ELSE:
        normalized.output_tokens = raw_metric.total_tokens - normalized.input_tokens

    // Validate consistency
    IF normalized.input_tokens + normalized.output_tokens != raw_metric.total_tokens:
        LOG_WARNING("Token count mismatch", raw_metric)
        // Use provider's total as source of truth
        normalized.total_tokens = raw_metric.total_tokens

    // Provider-specific normalization factors
    normalization_factor = GET_PROVIDER_NORMALIZATION_FACTOR(normalized.provider)
    normalized.input_tokens *= normalization_factor
    normalized.output_tokens *= normalization_factor
    normalized.total_tokens *= normalization_factor

    RETURN normalized
END FUNCTION
```

### 2. Token Counting and Normalization

```pseudocode
FUNCTION EstimateTokens(text: String, model: ModelIdentifier) -> Int
    // Tokenizer-based estimation when actual counts unavailable

    tokenizer = GET_TOKENIZER_FOR_MODEL(model)

    MATCH tokenizer.type:
        CASE "tiktoken":  // OpenAI models
            tokens = TIKTOKEN_ENCODE(text, model.encoding)
            RETURN tokens.length

        CASE "sentencepiece":  // Google, Meta models
            tokens = SENTENCEPIECE_ENCODE(text, model.vocab_file)
            RETURN tokens.length

        CASE "byte_pair_encoding":  // Anthropic, others
            tokens = BPE_ENCODE(text, model.merges_file)
            RETURN tokens.length

        DEFAULT:
            // Fallback: character-based approximation
            char_count = text.length
            RETURN CEILING(char_count / 4)  // Conservative estimate
END FUNCTION

FUNCTION ValidateTokenConsistency(metric: NormalizedMetric) -> ValidationResult
    // Cross-validate token counts with provider APIs

    IF metric.provider.supports_token_validation:
        api_tokens = CALL_PROVIDER_API_FOR_TOKEN_COUNT(
            metric.provider,
            metric.input_text,
            metric.output_text
        )

        input_diff = ABS(metric.input_tokens - api_tokens.input)
        output_diff = ABS(metric.output_tokens - api_tokens.output)

        IF input_diff > 0 OR output_diff > 0:
            RETURN ValidationResult{
                valid: false,
                discrepancy: {
                    input: input_diff,
                    output: output_diff
                },
                recommended_action: "USE_PROVIDER_TOKENS"
            }

    RETURN ValidationResult{valid: true}
END FUNCTION
```

### 3. Cost Calculation Engine

```pseudocode
FUNCTION CalculateCost(metric: NormalizedMetric) -> CostRecord
    // Multi-provider cost calculation with tiered pricing support

    // Get applicable pricing model
    pricing_model = GET_PRICING_MODEL(
        provider: metric.provider,
        model: metric.model,
        effective_date: metric.timestamp
    )

    cost_record = NEW CostRecord()
    cost_record.usage_id = metric.id
    cost_record.timestamp = metric.timestamp
    cost_record.provider = metric.provider
    cost_record.model = metric.model
    cost_record.currency = pricing_model.currency

    // Calculate base costs
    MATCH pricing_model.structure:
        CASE PerTokenPricing:
            input_cost = (metric.input_tokens / 1_000_000) * pricing_model.input_price_per_million
            output_cost = (metric.output_tokens / 1_000_000) * pricing_model.output_price_per_million

            // Apply cached token discount if applicable
            IF metric.cached_tokens > 0 AND pricing_model.cached_discount IS SET:
                cached_cost = (metric.cached_tokens / 1_000_000) * pricing_model.input_price_per_million
                cached_discount = cached_cost * pricing_model.cached_discount
                input_cost -= cached_discount

            cost_record.input_cost = ROUND_TO_PRECISION(input_cost, 10)
            cost_record.output_cost = ROUND_TO_PRECISION(output_cost, 10)

        CASE PerRequestPricing:
            base_cost = pricing_model.price_per_request

            IF metric.total_tokens > pricing_model.included_tokens:
                overage_tokens = metric.total_tokens - pricing_model.included_tokens
                overage_cost = (overage_tokens / 1_000_000) * pricing_model.overage_price_per_million
                base_cost += overage_cost

            cost_record.input_cost = ROUND_TO_PRECISION(base_cost * 0.5, 10)  // Arbitrary split
            cost_record.output_cost = ROUND_TO_PRECISION(base_cost * 0.5, 10)

        CASE TieredPricing:
            tier = FIND_APPLICABLE_TIER(pricing_model.tiers, metric.total_tokens)
            input_cost = (metric.input_tokens / 1_000_000) * tier.input_price_per_million
            output_cost = (metric.output_tokens / 1_000_000) * tier.output_price_per_million

            cost_record.input_cost = ROUND_TO_PRECISION(input_cost, 10)
            cost_record.output_cost = ROUND_TO_PRECISION(output_cost, 10)

    // Apply volume discounts
    monthly_volume = GET_MONTHLY_VOLUME(metric.organization_id, metric.provider)
    IF monthly_volume > pricing_model.volume_discount_threshold:
        discount_rate = pricing_model.volume_discount_rate
        cost_record.input_cost *= (1 - discount_rate)
        cost_record.output_cost *= (1 - discount_rate)

    // Apply special surcharges
    IF metric.timestamp WITHIN PEAK_HOURS:
        cost_record.input_cost *= pricing_model.peak_surcharge
        cost_record.output_cost *= pricing_model.peak_surcharge

    IF metric.region != pricing_model.base_region:
        regional_multiplier = GET_REGIONAL_MULTIPLIER(metric.region)
        cost_record.input_cost *= regional_multiplier
        cost_record.output_cost *= regional_multiplier

    // Calculate total cost
    cost_record.total_cost = cost_record.input_cost + cost_record.output_cost
    cost_record.cost_model_id = pricing_model.id
    cost_record.calculated_at = CURRENT_TIMESTAMP()

    // Store and emit event
    PERSIST_COST_RECORD(cost_record)
    EMIT_EVENT("cost.calculated", cost_record)

    RETURN cost_record
END FUNCTION
```

### 4. Performance Correlation Algorithm

```pseudocode
FUNCTION CorrelateCostWithPerformance(usage_id: UUID) -> PerformanceEfficiency
    // Link cost with performance metrics from Test-Bench

    cost_record = GET_COST_RECORD(usage_id)
    performance_metric = GET_PERFORMANCE_METRIC(usage_id)

    IF performance_metric IS NULL:
        RETURN NULL  // No performance data available

    efficiency = NEW PerformanceEfficiency()
    efficiency.metric_id = performance_metric.id
    efficiency.cost_per_token = cost_record.total_cost / cost_record.total_tokens

    // Calculate cost per second of latency
    IF performance_metric.total_latency_ms > 0:
        latency_seconds = performance_metric.total_latency_ms / 1000
        efficiency.cost_per_second = cost_record.total_cost / latency_seconds

    // Quality-adjusted cost
    IF performance_metric.quality_score IS SET:
        efficiency.quality_adjusted_cost = cost_record.total_cost / performance_metric.quality_score

    // Calculate efficiency score (0-100 scale)
    // Lower cost per quality point = higher efficiency
    benchmark_cost_per_quality = GET_BENCHMARK_COST_PER_QUALITY(cost_record.provider, cost_record.model)

    IF benchmark_cost_per_quality > 0:
        relative_efficiency = benchmark_cost_per_quality / efficiency.quality_adjusted_cost
        efficiency.efficiency_score = MIN(100, relative_efficiency * 100)

    PERSIST_EFFICIENCY_RECORD(efficiency)

    RETURN efficiency
END FUNCTION
```

### 5. ROI Computation Logic

```pseudocode
FUNCTION CalculateROI(
    period_start: DateTime,
    period_end: DateTime,
    dimensions: ROIDimensions
) -> ROICalculation
    // Comprehensive ROI calculation for LLM deployments

    roi = NEW ROICalculation()
    roi.period_start = period_start
    roi.period_end = period_end

    // 1. Aggregate total costs
    cost_records = QUERY_COST_RECORDS(
        start: period_start,
        end: period_end,
        filters: dimensions
    )

    roi.total_cost = SUM(cost_records.total_cost)

    // 2. Cost breakdown by dimensions
    roi.cost_breakdown.by_provider = GROUP_AND_SUM(cost_records, "provider")
    roi.cost_breakdown.by_model = GROUP_AND_SUM(cost_records, "model")
    roi.cost_breakdown.by_project = GROUP_AND_SUM(cost_records, "project_id")
    roi.cost_breakdown.by_tag = GROUP_AND_SUM(cost_records, "tags")

    // 3. Aggregate value metrics
    roi.total_requests = COUNT(cost_records)

    performance_metrics = QUERY_PERFORMANCE_METRICS(
        usage_ids: cost_records.map(r => r.usage_id)
    )

    successful_metrics = FILTER(performance_metrics, m => m.task_success == true)
    roi.successful_requests = COUNT(successful_metrics)

    IF COUNT(performance_metrics) > 0:
        roi.average_quality_score = AVG(performance_metrics.quality_score)

    // 4. ROI metrics
    roi.cost_per_request = roi.total_cost / roi.total_requests

    IF roi.successful_requests > 0:
        roi.cost_per_successful_request = roi.total_cost / roi.successful_requests

    IF roi.average_quality_score > 0:
        roi.efficiency_ratio = roi.average_quality_score / roi.cost_per_request

    // 5. Generate optimization suggestions
    roi.optimization_suggestions = GENERATE_OPTIMIZATION_SUGGESTIONS(
        cost_records,
        performance_metrics,
        roi.cost_breakdown
    )

    // Calculate potential savings
    roi.potential_savings = SUM(roi.optimization_suggestions.map(s => s.potential_savings))

    PERSIST_ROI_CALCULATION(roi)

    RETURN roi
END FUNCTION

FUNCTION GenerateOptimizationSuggestions(
    cost_records: List<CostRecord>,
    performance_metrics: List<PerformanceMetric>,
    breakdown: CostBreakdown
) -> List<OptimizationSuggestion>
    // AI-driven cost optimization recommendations

    suggestions = []

    // 1. Identify model downgrade opportunities
    FOR EACH (provider, cost) IN breakdown.by_provider:
        FOR EACH (model, model_cost) IN breakdown.by_model WHERE provider == model.provider:
            IF model.is_premium:
                cheaper_alternatives = FIND_CHEAPER_MODELS(provider, model.capabilities)

                FOR EACH alt IN cheaper_alternatives:
                    quality_drop = ESTIMATE_QUALITY_DROP(model, alt)

                    IF quality_drop < 0.1:  // Less than 10% quality drop
                        potential_savings = model_cost * (1 - alt.price_ratio)

                        suggestions.ADD(OptimizationSuggestion{
                            type: ModelDowngrade{from: model, to: alt},
                            current_cost: model_cost,
                            optimized_cost: model_cost * alt.price_ratio,
                            potential_savings: potential_savings,
                            confidence: 0.8,
                            description: "Switch from {model} to {alt} with minimal quality impact"
                        })

    // 2. Identify prompt optimization opportunities
    long_prompts = FILTER(cost_records, r => r.input_tokens > 2000)

    IF COUNT(long_prompts) > 100:
        avg_input_tokens = AVG(long_prompts.input_tokens)
        estimated_reduction = avg_input_tokens * 0.3  // Conservative 30% reduction

        cost_per_token = AVG(long_prompts.input_cost / long_prompts.input_tokens)
        potential_savings = COUNT(long_prompts) * estimated_reduction * cost_per_token

        suggestions.ADD(OptimizationSuggestion{
            type: PromptOptimization{estimated_token_reduction: 0.3},
            current_cost: SUM(long_prompts.input_cost),
            potential_savings: potential_savings,
            confidence: 0.6,
            description: "Optimize prompts to reduce average input tokens by 30%"
        })

    // 3. Identify caching opportunities
    duplicate_inputs = FIND_DUPLICATE_INPUTS(cost_records)

    IF COUNT(duplicate_inputs) > 50:
        duplicate_cost = SUM(duplicate_inputs.input_cost)
        cache_hit_rate_potential = ESTIMATE_CACHE_HIT_RATE(duplicate_inputs)
        potential_savings = duplicate_cost * cache_hit_rate_potential * 0.9  // 90% cost reduction on cache hits

        suggestions.ADD(OptimizationSuggestion{
            type: CachingOpportunity{cache_hit_rate_potential: cache_hit_rate_potential},
            current_cost: duplicate_cost,
            potential_savings: potential_savings,
            confidence: 0.85,
            description: "Implement prompt caching to reduce duplicate processing costs"
        })

    // 4. Provider switching recommendations
    FOR EACH (provider_a, cost_a) IN breakdown.by_provider:
        FOR EACH provider_b IN ALL_PROVIDERS WHERE provider_b != provider_a:
            IF SUPPORTS_SAME_CAPABILITIES(provider_a, provider_b):
                price_difference = COMPARE_PRICING(provider_a, provider_b)

                IF price_difference > 0.2:  // More than 20% cheaper
                    potential_savings = cost_a * price_difference

                    suggestions.ADD(OptimizationSuggestion{
                        type: ProviderSwitch{from: provider_a, to: provider_b},
                        current_cost: cost_a,
                        potential_savings: potential_savings,
                        confidence: 0.7,
                        description: "Switch to {provider_b} for {price_difference}% cost reduction"
                    })

    // Sort by potential savings (descending)
    suggestions = SORT(suggestions, BY potential_savings DESC)

    RETURN suggestions
END FUNCTION
```

## Integration Workflows

### 1. LLM-Observatory → CostOps Metric Streaming

```pseudocode
FUNCTION SetupObservatoryIntegration() -> Result<()>
    // Event-driven metric streaming from Observatory

    kafka_config = KafkaConfig{
        brokers: ENV["KAFKA_BROKERS"],
        topic: "llm.usage.metrics",
        consumer_group: "costops-metrics-consumer",
        auto_offset_reset: "earliest"
    }

    consumer = CREATE_KAFKA_CONSUMER(kafka_config)

    SUBSCRIBE(consumer, ["llm.usage.metrics", "llm.usage.metrics.batch"])

    SPAWN_ASYNC_TASK:
        LOOP:
            message = CONSUME_MESSAGE(consumer)

            IF message IS NULL:
                CONTINUE

            // Deserialize metric
            TRY:
                metric = DESERIALIZE_METRIC(message.value, message.headers["schema-version"])
            CATCH DeserializationError as e:
                LOG_ERROR("Failed to deserialize metric", e)
                SEND_TO_DEAD_LETTER_QUEUE(message)
                CONTINUE

            // Validate schema
            IF NOT VALIDATE_SCHEMA(metric, EXPECTED_SCHEMA):
                LOG_WARNING("Schema validation failed", metric)
                SEND_TO_VALIDATION_ERROR_QUEUE(message)
                CONTINUE

            // Process metric asynchronously
            ENQUEUE_FOR_PROCESSING(metric)

            // Acknowledge message
            COMMIT_OFFSET(consumer, message.offset)

            // Emit acknowledgment event back to Observatory
            EMIT_EVENT("costops.metric.received", {
                metric_id: metric.id,
                received_at: CURRENT_TIMESTAMP()
            })
END FUNCTION
```

### 2. LLM-Edge-Agent → CostOps Usage Reporting

```pseudocode
FUNCTION SetupEdgeAgentIntegration() -> Result<()>
    // Batch reporting API for edge deployments

    api_server = CREATE_HTTP_SERVER(port: 8080)

    ROUTE POST "/api/v1/edge/metrics/batch":
        HANDLER(request, response):
            // Authenticate edge agent
            auth_token = request.headers["Authorization"]
            edge_agent_id = VALIDATE_EDGE_AGENT_TOKEN(auth_token)

            IF edge_agent_id IS NULL:
                RETURN response.status(401).json({error: "Unauthorized"})

            // Deserialize batch
            TRY:
                batch = DESERIALIZE_JSON(request.body)
            CATCH JsonError as e:
                RETURN response.status(400).json({error: "Invalid JSON"})

            // Validate batch structure
            IF NOT VALIDATE_BATCH_SCHEMA(batch):
                RETURN response.status(400).json({error: "Invalid batch schema"})

            // Process metrics in batch
            processed_count = 0
            failed_metrics = []

            FOR EACH metric IN batch.metrics:
                // Add edge agent metadata
                metric.source = EdgeAgent{id: edge_agent_id}
                metric.ingested_at = CURRENT_TIMESTAMP()

                // Deduplicate
                IF METRIC_EXISTS(metric.id):
                    LOG_INFO("Duplicate metric detected", metric.id)
                    CONTINUE

                // Enqueue for processing
                TRY:
                    ENQUEUE_FOR_PROCESSING(metric)
                    processed_count += 1
                CATCH ProcessingError as e:
                    failed_metrics.ADD({
                        metric_id: metric.id,
                        error: e.message
                    })

            // Update edge agent sync status
            UPDATE_EDGE_AGENT_SYNC_STATUS(edge_agent_id, {
                last_sync: CURRENT_TIMESTAMP(),
                metrics_received: processed_count
            })

            // Return batch processing summary
            RETURN response.status(202).json({
                accepted: processed_count,
                failed: failed_metrics.length,
                failures: failed_metrics
            })

    START_SERVER(api_server)
END FUNCTION
```

### 3. LLM-Governance-Core ↔ CostOps Budget Enforcement

```pseudocode
FUNCTION EnforceBudgetPolicy(usage_metric: NormalizedMetric) -> EnforcementDecision
    // Real-time budget enforcement with governance integration

    // Calculate projected cost for this request
    projected_cost = ESTIMATE_COST(usage_metric)

    // Get applicable budget policies
    policies = GET_ACTIVE_BUDGET_POLICIES(
        organization_id: usage_metric.organization_id,
        project_id: usage_metric.project_id,
        tags: usage_metric.tags
    )

    IF policies IS EMPTY:
        RETURN EnforcementDecision{
            allowed: true,
            reason: "No budget policies apply"
        }

    FOR EACH policy IN policies:
        // Get current spend for policy period
        current_spend = GET_PERIOD_SPEND(
            policy: policy,
            period_start: policy.current_period_start,
            period_end: CURRENT_TIMESTAMP()
        )

        // Calculate projected end-of-period spend
        days_remaining = DAYS_UNTIL(policy.current_period_end)
        burn_rate = current_spend / DAYS_SINCE(policy.current_period_start)
        projected_total = current_spend + (burn_rate * days_remaining)

        // Check threshold levels
        utilization = (current_spend + projected_cost) / policy.budget_amount

        FOR EACH threshold IN policy.alert_thresholds:
            IF utilization >= threshold.percentage / 100:
                // Emit alert
                EMIT_BUDGET_ALERT({
                    policy_id: policy.id,
                    threshold: threshold.percentage,
                    current_spend: current_spend,
                    projected_cost: projected_cost,
                    budget_amount: policy.budget_amount,
                    utilization: utilization
                })

                // Send notifications
                FOR EACH channel IN threshold.channels:
                    SEND_NOTIFICATION(channel, threshold.recipients, {
                        message: "Budget threshold {threshold.percentage}% reached",
                        policy: policy.name,
                        current_spend: current_spend,
                        budget: policy.budget_amount
                    })

        // Enforcement decision
        MATCH policy.enforcement_type:
            CASE SoftLimit:
                // Allow but warn
                IF utilization >= 1.0:
                    LOG_WARNING("Budget exceeded but SoftLimit - allowing", policy.id)

            CASE HardLimit:
                // Block if over budget
                IF utilization >= 1.0:
                    EMIT_EVENT("budget.exceeded", {
                        policy_id: policy.id,
                        overage: (current_spend + projected_cost) - policy.budget_amount
                    })

                    RETURN EnforcementDecision{
                        allowed: false,
                        reason: "Budget hard limit exceeded",
                        policy_id: policy.id,
                        current_spend: current_spend,
                        budget_limit: policy.budget_amount
                    }

            CASE Throttle:
                // Rate limit if approaching budget
                IF utilization >= 0.9:
                    throttle_factor = (utilization - 0.9) / 0.1  // 0.0 to 1.0
                    delay_ms = throttle_factor * 1000  // Up to 1 second delay

                    SLEEP(delay_ms)

                    LOG_INFO("Request throttled due to budget", {
                        delay_ms: delay_ms,
                        policy_id: policy.id
                    })

    RETURN EnforcementDecision{
        allowed: true,
        reason: "Within budget limits"
    }
END FUNCTION
```

## Forecasting Logic

### 1. Time-Series Prediction Model

```pseudocode
FUNCTION GenerateCostForecast(
    dimensions: ForecastDimensions,
    horizon_days: Int,
    granularity: ForecastGranularity
) -> ForecastResult
    // Multi-method time-series forecasting with ensemble

    // 1. Query historical data
    historical_data = QUERY_HISTORICAL_COSTS(
        dimensions: dimensions,
        start_date: CURRENT_DATE() - 90 DAYS,
        end_date: CURRENT_DATE(),
        granularity: granularity
    )

    IF COUNT(historical_data) < 14:
        RETURN ERROR("Insufficient historical data for forecasting")

    // 2. Prepare time series
    time_series = CONVERT_TO_TIME_SERIES(historical_data)
    time_series = HANDLE_MISSING_VALUES(time_series)  // Interpolation

    // 3. Apply multiple forecasting methods
    methods = []

    // Method 1: Linear Trend
    linear_forecast = FORECAST_LINEAR_TREND(time_series, horizon_days)
    methods.ADD({name: "linear", forecast: linear_forecast, weight: 0.15})

    // Method 2: Holt-Winters (Exponential Smoothing)
    hw_forecast = FORECAST_HOLT_WINTERS(
        time_series,
        horizon_days,
        seasonality_period: DETECT_SEASONALITY(time_series)
    )
    methods.ADD({name: "holt_winters", forecast: hw_forecast, weight: 0.25})

    // Method 3: ARIMA-like (if sufficient data)
    IF COUNT(historical_data) >= 30:
        arima_params = AUTO_DETECT_ARIMA_PARAMS(time_series)
        arima_forecast = FORECAST_ARIMA(time_series, horizon_days, arima_params)
        methods.ADD({name: "arima", forecast: arima_forecast, weight: 0.35})

    // Method 4: Seasonal Decomposition + Trend
    decomposed = SEASONAL_DECOMPOSE(time_series)
    trend_forecast = EXTRAPOLATE_TREND(decomposed.trend, horizon_days)
    seasonal_forecast = PROJECT_SEASONALITY(decomposed.seasonal, horizon_days)
    sd_forecast = COMBINE(trend_forecast, seasonal_forecast)
    methods.ADD({name: "seasonal_decomposition", forecast: sd_forecast, weight: 0.25})

    // 4. Ensemble: Weighted average
    ensemble_forecast = WEIGHTED_AVERAGE_FORECAST(methods)

    // 5. Calculate confidence intervals
    forecast_points = []
    FOR i = 1 TO horizon_days:
        forecast_date = CURRENT_DATE() + i DAYS

        point_forecasts = methods.map(m => m.forecast[i])
        mean_forecast = WEIGHTED_MEAN(point_forecasts, methods.map(m => m.weight))
        std_dev = STANDARD_DEVIATION(point_forecasts)

        // 95% confidence interval
        lower_bound = mean_forecast - (1.96 * std_dev)
        upper_bound = mean_forecast + (1.96 * std_dev)

        forecast_points.ADD(ForecastPoint{
            timestamp: forecast_date,
            predicted_value: mean_forecast,
            lower_bound: MAX(0, lower_bound),  // Cost can't be negative
            upper_bound: upper_bound,
            confidence_interval: 0.95
        })

    // 6. Calculate forecast accuracy metrics (backtesting)
    validation_split = SPLIT_TIME_SERIES(time_series, train_ratio: 0.8)
    validation_forecast = APPLY_ENSEMBLE(validation_split.train, COUNT(validation_split.test))

    mape = CALCULATE_MAPE(validation_split.test, validation_forecast)
    rmse = CALCULATE_RMSE(validation_split.test, validation_forecast)

    // 7. Build forecast result
    result = ForecastResult{
        forecast_id: GENERATE_UUID(),
        generated_at: CURRENT_TIMESTAMP(),
        model: Ensemble{models: methods.map(m => m.name)},
        confidence_level: 0.95,
        start_date: CURRENT_DATE() + 1 DAY,
        end_date: CURRENT_DATE() + horizon_days DAYS,
        granularity: granularity,
        predictions: forecast_points,
        mape: mape,
        rmse: rmse,
        dimensions: dimensions
    }

    PERSIST_FORECAST(result)

    RETURN result
END FUNCTION
```

### 2. Anomaly Detection

```pseudocode
FUNCTION DetectCostAnomalies(
    window_hours: Int = 24
) -> List<Anomaly>
    // Multi-method anomaly detection

    anomalies = []

    // Get recent cost data
    recent_costs = QUERY_HOURLY_COSTS(
        start: CURRENT_TIMESTAMP() - window_hours HOURS,
        end: CURRENT_TIMESTAMP()
    )

    // Get baseline (historical average)
    baseline_costs = QUERY_HOURLY_COSTS(
        start: CURRENT_TIMESTAMP() - 30 DAYS,
        end: CURRENT_TIMESTAMP() - window_hours HOURS
    )

    // Method 1: Z-score (statistical outlier detection)
    FOR EACH hour_cost IN recent_costs:
        comparable_baseline = FILTER(baseline_costs, c => c.hour_of_day == hour_cost.hour_of_day)
        mean = AVG(comparable_baseline.cost)
        std_dev = STANDARD_DEVIATION(comparable_baseline.cost)

        IF std_dev > 0:
            z_score = (hour_cost.cost - mean) / std_dev

            IF ABS(z_score) > 3.0:  // 3 standard deviations
                anomalies.ADD(Anomaly{
                    type: "statistical_outlier",
                    timestamp: hour_cost.timestamp,
                    actual_value: hour_cost.cost,
                    expected_value: mean,
                    severity: CALCULATE_SEVERITY(z_score),
                    detection_method: "z_score",
                    z_score: z_score
                })

    // Method 2: IQR (Interquartile Range)
    sorted_costs = SORT(baseline_costs.cost)
    q1 = PERCENTILE(sorted_costs, 25)
    q3 = PERCENTILE(sorted_costs, 75)
    iqr = q3 - q1
    lower_bound = q1 - (1.5 * iqr)
    upper_bound = q3 + (1.5 * iqr)

    FOR EACH hour_cost IN recent_costs:
        IF hour_cost.cost < lower_bound OR hour_cost.cost > upper_bound:
            anomalies.ADD(Anomaly{
                type: "iqr_outlier",
                timestamp: hour_cost.timestamp,
                actual_value: hour_cost.cost,
                expected_range: {lower: lower_bound, upper: upper_bound},
                severity: CALCULATE_SEVERITY_IQR(hour_cost.cost, lower_bound, upper_bound),
                detection_method: "iqr"
            })

    // Method 3: Moving average deviation
    window_size = 7  // 7 periods
    FOR i = window_size TO COUNT(recent_costs):
        current_cost = recent_costs[i].cost
        window = recent_costs[i-window_size:i]
        moving_avg = AVG(window.cost)

        deviation_pct = ABS(current_cost - moving_avg) / moving_avg

        IF deviation_pct > 0.5:  // 50% deviation
            anomalies.ADD(Anomaly{
                type: "moving_average_deviation",
                timestamp: recent_costs[i].timestamp,
                actual_value: current_cost,
                expected_value: moving_avg,
                deviation_percentage: deviation_pct,
                severity: CALCULATE_SEVERITY_DEVIATION(deviation_pct),
                detection_method: "moving_average"
            })

    // Method 4: Rate of change
    FOR i = 1 TO COUNT(recent_costs):
        current_cost = recent_costs[i].cost
        previous_cost = recent_costs[i-1].cost

        IF previous_cost > 0:
            rate_of_change = (current_cost - previous_cost) / previous_cost

            IF ABS(rate_of_change) > 2.0:  // 200% increase or 200% decrease
                anomalies.ADD(Anomaly{
                    type: "rapid_change",
                    timestamp: recent_costs[i].timestamp,
                    actual_value: current_cost,
                    previous_value: previous_cost,
                    rate_of_change: rate_of_change,
                    severity: "high",
                    detection_method: "rate_of_change"
                })

    // Method 5: Pattern-based (day-of-week, hour-of-day)
    FOR EACH hour_cost IN recent_costs:
        same_hour_baseline = FILTER(baseline_costs,
            c => c.day_of_week == hour_cost.day_of_week AND
                 c.hour_of_day == hour_cost.hour_of_day
        )

        IF COUNT(same_hour_baseline) >= 4:  // At least 4 weeks of data
            pattern_mean = AVG(same_hour_baseline.cost)
            pattern_std = STANDARD_DEVIATION(same_hour_baseline.cost)

            IF pattern_std > 0:
                pattern_z_score = (hour_cost.cost - pattern_mean) / pattern_std

                IF ABS(pattern_z_score) > 2.5:
                    anomalies.ADD(Anomaly{
                        type: "pattern_deviation",
                        timestamp: hour_cost.timestamp,
                        actual_value: hour_cost.cost,
                        expected_value: pattern_mean,
                        severity: CALCULATE_SEVERITY(pattern_z_score),
                        detection_method: "pattern_analysis",
                        pattern: {
                            day_of_week: hour_cost.day_of_week,
                            hour_of_day: hour_cost.hour_of_day
                        }
                    })

    // Deduplicate anomalies (same timestamp, different methods)
    deduplicated = DEDUPLICATE_ANOMALIES(anomalies)

    // Rank by severity
    ranked = SORT(deduplicated, BY severity DESC)

    // Persist and alert
    FOR EACH anomaly IN ranked:
        PERSIST_ANOMALY(anomaly)

        IF anomaly.severity IN ["high", "critical"]:
            EMIT_ANOMALY_ALERT(anomaly)

    RETURN ranked
END FUNCTION

FUNCTION CalculateSeverity(z_score: Float) -> String
    abs_z = ABS(z_score)

    IF abs_z >= 5.0:
        RETURN "critical"
    ELSE IF abs_z >= 4.0:
        RETURN "high"
    ELSE IF abs_z >= 3.0:
        RETURN "medium"
    ELSE:
        RETURN "low"
END FUNCTION
```

---

# SPARC Phase 3: Architecture

## System Architecture Overview

LLM-CostOps follows a modular, event-driven architecture designed for high-performance cost analysis and forecasting. The system is built around five core subsystems that communicate through well-defined interfaces.

### Core Architectural Principles

1. **Modularity**: Each component operates independently with clear boundaries
2. **Performance**: Leverage Rust's zero-cost abstractions and async runtime
3. **Extensibility**: Plugin-based provider system for easy integration
4. **Observability**: Built-in metrics, tracing, and logging at every layer
5. **Deployment Flexibility**: Support CLI, daemon, and API modes

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   INGESTION LAYER                           │
│  API Connectors → Stream Processor → Normalization Engine  │
│  File Importers ────────────┴─────────────┘                │
│  Webhook Receivers ──────────┘                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   PROCESSING LAYER                          │
│  Usage Aggregator → Cost Calculator → Performance Correlator│
│         │                    │                    │          │
│         └────────────────────┴────────────────────┘          │
│                         Metrics Collector                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    STORAGE LAYER                            │
│  TimescaleDB (Primary) │ RocksDB (Cache) │ Parquet (Archive)│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    ANALYSIS LAYER                           │
│  Statistical Engine → Forecasting → Anomaly Detection      │
│          │                │               │                 │
│          └────────────────┴───────────────┴─> ROI Calculator│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      API LAYER                              │
│  REST API (Axum) │ gRPC Service (Tonic) │ CLI Interface   │
└─────────────────────────────────────────────────────────────┘
```

## Rust Crate Recommendations

### Core Runtime & Async
```toml
[dependencies]
tokio = { version = "1.37", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec", "time"] }
async-stream = "0.3"
futures = "0.3"
async-trait = "0.1"
```

### Data Ingestion & Streaming
```toml
reqwest = { version = "0.12", features = ["json", "stream"] }
rdkafka = { version = "0.36", features = ["ssl", "sasl"] }
axum = { version = "0.7", features = ["ws", "macros"] }
tower = { version = "0.4", features = ["limit", "buffer"] }
```

### Serialization & Data Formats
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
prost = "0.12"  # Protocol Buffers
parquet = "51.0"
arrow = "51.0"
```

### Storage & Databases
```toml
sqlx = { version = "0.7", features = [
    "runtime-tokio-native-tls",
    "postgres",
    "chrono",
    "uuid",
    "json"
]}
rocksdb = "0.22"
redis = { version = "0.25", features = ["tokio-comp"] }
```

### Statistical Analysis & ML
```toml
ndarray = { version = "0.15", features = ["rayon", "serde"] }
statrs = "0.16"
polars = { version = "0.38", features = ["lazy", "temporal", "parquet"] }
augurs = { version = "0.2", features = ["forecasting"] }
```

### HTTP APIs & Services
```toml
axum = { version = "0.7", features = ["macros", "multipart"] }
tonic = { version = "0.11", features = ["gzip", "tls"] }
utoipa = { version = "4.2", features = ["axum_extras"] }
```

### Metrics & Observability
```toml
prometheus = "0.13"
opentelemetry = { version = "0.22", features = ["trace", "metrics"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

## Data Models

### Core Domain Models

```rust
// Usage Record
pub struct UsageRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub provider: Provider,
    pub model: ModelIdentifier,
    pub organization_id: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub latency_ms: Option<u64>,
    pub tags: Vec<String>,
}

// Cost Model
pub struct CostModel {
    pub provider: Provider,
    pub model: String,
    pub effective_date: DateTime<Utc>,
    pub pricing: PricingStructure,
    pub currency: Currency,
}

pub enum PricingStructure {
    PerToken {
        input_price_per_million: Decimal,
        output_price_per_million: Decimal,
    },
    Tiered {
        tiers: Vec<PricingTier>,
    },
    Custom {
        formula: String,
    },
}

// Forecast Result
pub struct ForecastResult {
    pub forecast_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub predictions: Vec<ForecastPoint>,
    pub mape: Option<f64>,
    pub rmse: Option<f64>,
    pub confidence_level: f64,
}
```

### Storage Schema (TimescaleDB)

```sql
-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Usage records (hypertable)
CREATE TABLE usage_records (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    prompt_tokens BIGINT NOT NULL,
    completion_tokens BIGINT NOT NULL,
    total_tokens BIGINT NOT NULL,
    latency_ms BIGINT,
    tags TEXT[]
);

SELECT create_hypertable('usage_records', 'timestamp');

-- Cost records (hypertable)
CREATE TABLE cost_records (
    id UUID PRIMARY KEY,
    usage_id UUID REFERENCES usage_records(id),
    timestamp TIMESTAMPTZ NOT NULL,
    input_cost DECIMAL(20, 10) NOT NULL,
    output_cost DECIMAL(20, 10) NOT NULL,
    total_cost DECIMAL(20, 10) NOT NULL,
    currency TEXT NOT NULL
);

SELECT create_hypertable('cost_records', 'timestamp');

-- Continuous aggregates for performance
CREATE MATERIALIZED VIEW hourly_cost_summary
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', timestamp) AS bucket,
    provider,
    model_name,
    organization_id,
    SUM(total_cost) AS total_cost,
    SUM(total_tokens) AS total_tokens,
    COUNT(*) AS request_count
FROM cost_records cr
JOIN usage_records ur ON cr.usage_id = ur.id
GROUP BY bucket, provider, model_name, organization_id;
```

## Deployment Architecture

### Deployment Modes

LLM-CostOps supports four deployment modes:

1. **Standalone Daemon**: Single-tenant, file-based configuration
2. **CLI Utility**: Ad-hoc analysis, batch processing
3. **API Microservice**: Multi-tenant SaaS, Kubernetes-ready
4. **Hybrid**: Combination of daemon + API + CLI

### Standalone Daemon Mode

```yaml
# config/daemon.yaml
daemon:
  mode: standalone
  workers:
    ingestion: 4
    processing: 8

  storage:
    timescaledb:
      url: postgresql://localhost/llm_costops
    rocksdb:
      path: /var/lib/llm-costops/cache

  api:
    enabled: true
    bind: 127.0.0.1:3000
```

### CLI Utility Mode

```bash
# Query costs
llm-costops query --range 24h --group-by model

# Generate forecast
llm-costops forecast --model arima --horizon 90

# Export data
llm-costops export --format parquet --period 2024-Q1
```

### API Microservice Mode (Kubernetes)

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-costops-api
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: api
        image: llm-costops:v1.0.0
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: costops-secrets
              key: database-url
        ports:
        - containerPort: 3000
        - containerPort: 50051  # gRPC
```

## Scalability & Performance

### Data Partitioning Strategies

**Time-based Partitioning (TimescaleDB)**
```sql
-- Automatic partitioning
SELECT create_hypertable('usage_records', 'timestamp',
    chunk_time_interval => INTERVAL '1 day');

-- Compression for old data
ALTER TABLE usage_records SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'provider,organization_id'
);

SELECT add_compression_policy('usage_records', INTERVAL '7 days');
```

### Caching Layers

```rust
// Multi-level cache
pub struct CacheLayer {
    // L1: In-memory LRU cache
    l1: Arc<RwLock<LruCache<String, CachedValue>>>,

    // L2: Redis distributed cache
    l2: RedisPool,

    // L3: RocksDB local persistent cache
    l3: Arc<rocksdb::DB>,
}
```

### Horizontal Scaling

```
Ingestion Tier (Stateless)
┌──────────┐ ┌──────────┐ ┌──────────┐
│Worker 1  │ │Worker 2  │ │Worker N  │
└────┬─────┘ └────┬─────┘ └────┬─────┘
     └────────────┴────────────┘
              │
    ┌─────────▼────────┐
    │  Message Queue   │
    └─────────┬────────┘
              │
Processing Tier (Stateless)
┌──────────┐ ┌──────────┐ ┌──────────┐
│Worker 1  │ │Worker 2  │ │Worker N  │
└────┬─────┘ └────┬─────┘ └────┬─────┘
     └────────────┴────────────┘
              │
Storage Tier (Sharded)
┌──────────┐ ┌──────────┐ ┌──────────┐
│Shard 1   │ │Shard 2   │ │Shard N   │
└──────────┘ └──────────┘ └──────────┘
```

### Performance Benchmarks

| Operation | Target | Strategy |
|-----------|--------|----------|
| Ingest rate | 100K records/sec | Stream processing, batching |
| Query latency (recent) | <100ms | L1/L2 cache, indexed queries |
| Query latency (historical) | <2s | Continuous aggregates |
| Forecast generation | <5s | Pre-computed models |
| API response (p95) | <200ms | Caching, connection pooling |

---

# SPARC Phase 4: Refinement

## Detailed Implementation Schemas

### 1. Usage Record Schema (Complete)

```rust
// File: src/domain/schemas/usage.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageRecord {
    /// Unique identifier for this usage record
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// Timestamp when the request was made
    pub timestamp: DateTime<Utc>,

    /// LLM provider (openai, anthropic, google, etc.)
    #[serde(deserialize_with = "deserialize_provider")]
    pub provider: Provider,

    /// Model identifier
    pub model: ModelIdentifier,

    /// Organization/tenant identifier
    pub organization_id: String,

    /// Optional project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    /// Optional user identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Token counts
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,

    /// Optional cached tokens (for prompt caching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u64>,

    /// Optional reasoning tokens (for o1-style models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,

    /// Performance metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to_first_token_ms: Option<u64>,

    /// Cost attribution tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Free-form metadata
    #[serde(default)]
    pub metadata: serde_json::Value,

    /// Ingestion tracking
    pub ingested_at: DateTime<Utc>,
    pub source: IngestionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[serde(alias = "OpenAI", alias = "openai")]
    OpenAI,

    #[serde(alias = "Anthropic", alias = "anthropic")]
    Anthropic,

    #[serde(alias = "Google", alias = "google", alias = "vertex")]
    GoogleVertexAI,

    #[serde(alias = "Azure", alias = "azure")]
    AzureOpenAI,

    #[serde(alias = "AWS", alias = "aws", alias = "bedrock")]
    AWSBedrock,

    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIdentifier {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub context_window: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum IngestionSource {
    Api { endpoint: String },
    File { path: String },
    Webhook { source: String },
    Stream { topic: String },
}

// Validation rules
impl UsageRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Token counts must be positive
        if self.total_tokens == 0 {
            return Err(ValidationError::InvalidTokenCount);
        }

        // Token sum must equal total (with tolerance)
        let calculated_total = self.prompt_tokens + self.completion_tokens;
        if calculated_total != self.total_tokens {
            return Err(ValidationError::TokenCountMismatch {
                calculated: calculated_total,
                reported: self.total_tokens,
            });
        }

        // Organization ID must not be empty
        if self.organization_id.is_empty() {
            return Err(ValidationError::MissingOrganizationId);
        }

        // Timestamp must not be in the future
        if self.timestamp > Utc::now() {
            return Err(ValidationError::FutureTimestamp);
        }

        Ok(())
    }
}
```

### 2. Pricing Table Schema

```rust
// File: src/domain/schemas/pricing.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTable {
    pub id: Uuid,
    pub provider: Provider,
    pub model: String,

    /// When this pricing becomes effective
    pub effective_date: DateTime<Utc>,

    /// When this pricing expires (None = no expiration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,

    /// Pricing structure
    pub pricing: PricingStructure,

    /// Currency (USD, EUR, etc.)
    pub currency: Currency,

    /// Geographic region this pricing applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Metadata about this pricing table
    #[serde(default)]
    pub metadata: PricingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PricingStructure {
    /// Per-token pricing (most common)
    PerToken {
        /// Price per 1 million input tokens
        input_price_per_million: Decimal,

        /// Price per 1 million output tokens
        output_price_per_million: Decimal,

        /// Optional cached input discount (0.0 to 1.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        cached_input_discount: Option<Decimal>,
    },

    /// Per-request pricing with included tokens
    PerRequest {
        price_per_request: Decimal,
        included_tokens: u64,
        overage_price_per_million: Decimal,
    },

    /// Tiered pricing based on volume
    Tiered {
        tiers: Vec<PricingTier>,
    },

    /// Custom pricing logic (stored as formula)
    Custom {
        formula: String,
        parameters: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub min_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,  // None = unlimited
    pub input_price_per_million: Decimal,
    pub output_price_per_million: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PricingMetadata {
    /// Source of this pricing information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,  // "official_api", "manual_entry", "scraped"

    /// Last verified date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_verified: Option<DateTime<Utc>>,

    /// Volume discount information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_discounts: Option<Vec<VolumeDiscount>>,

    /// Peak hour surcharges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_surcharge: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDiscount {
    pub min_monthly_spend: Decimal,
    pub discount_rate: Decimal,  // 0.0 to 1.0
}
```

### 3. ROI Computation Result Schema

```rust
// File: src/domain/schemas/roi.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROIComputationResult {
    pub computation_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub computed_at: DateTime<Utc>,

    /// Cost metrics
    pub costs: CostMetrics,

    /// Value metrics
    pub value: ValueMetrics,

    /// Computed ROI metrics
    pub roi_metrics: ROIMetrics,

    /// Cost breakdown by dimension
    pub breakdown: CostBreakdown,

    /// Optimization opportunities
    pub optimizations: Vec<OptimizationOpportunity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    /// Total cost for the period
    pub total_cost: Decimal,
    pub currency: Currency,

    /// Cost components
    pub input_cost: Decimal,
    pub output_cost: Decimal,

    /// Average costs
    pub avg_cost_per_request: Decimal,
    pub avg_cost_per_1k_tokens: Decimal,

    /// Period-over-period comparison
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_period_cost: Option<Decimal>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub period_over_period_change: Option<Decimal>,  // -1.0 to +inf
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueMetrics {
    /// Request counts
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,

    /// Quality metrics (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_quality_score: Option<f64>,

    /// Performance metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_latency_ms: Option<f64>,

    /// Business value metrics (custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_value_metrics: Option<HashMap<String, Decimal>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROIMetrics {
    /// Cost per successful outcome
    pub cost_per_success: Decimal,

    /// Quality-adjusted cost
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_adjusted_cost: Option<Decimal>,

    /// Efficiency score (0-100)
    pub efficiency_score: f64,

    /// ROI ratio (if revenue data available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roi_ratio: Option<Decimal>,

    /// Payback period in days (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payback_period_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub by_provider: HashMap<String, Decimal>,
    pub by_model: HashMap<String, Decimal>,
    pub by_project: HashMap<String, Decimal>,
    pub by_tag: HashMap<String, Decimal>,

    /// Top cost drivers
    pub top_cost_drivers: Vec<CostDriver>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDriver {
    pub dimension_type: String,  // "provider", "model", "project", "tag"
    pub dimension_value: String,
    pub cost: Decimal,
    pub percentage_of_total: f64,
    pub request_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub opportunity_id: Uuid,
    pub opportunity_type: OptimizationType,
    pub current_cost: Decimal,
    pub optimized_cost: Decimal,
    pub potential_savings: Decimal,
    pub savings_percentage: f64,
    pub confidence: f64,  // 0.0 to 1.0
    pub description: String,
    pub action_items: Vec<String>,
    pub estimated_implementation_effort: ImplementationEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OptimizationType {
    ModelDowngrade {
        from_model: String,
        to_model: String,
        estimated_quality_impact: f64,
    },
    PromptOptimization {
        estimated_token_reduction_pct: f64,
    },
    CachingOpportunity {
        cache_hit_rate_potential: f64,
    },
    ProviderSwitch {
        from_provider: String,
        to_provider: String,
    },
    BatchingOpportunity {
        current_batch_size: u32,
        recommended_batch_size: u32,
    },
    RateLimitOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImplementationEffort {
    Low,     // < 1 day
    Medium,  // 1-5 days
    High,    // > 5 days
}
```

## Forecasting Strategies (Detailed)

### Time-Series Modeling Approaches

```rust
// File: src/forecasting/strategies.rs

/// Ensemble forecasting strategy combining multiple models
pub struct EnsembleForecaster {
    models: Vec<Box<dyn ForecastModel>>,
    weights: Vec<f64>,
}

impl EnsembleForecaster {
    pub fn new() -> Self {
        Self {
            models: vec![
                Box::new(LinearTrendModel::new()),
                Box::new(ExponentialSmoothingModel::new()),
                Box::new(ARIMAModel::new()),
                Box::new(SeasonalDecompositionModel::new()),
            ],
            weights: vec![0.15, 0.25, 0.35, 0.25],  // Tuned weights
        }
    }

    pub async fn forecast(
        &self,
        historical_data: &TimeSeries,
        horizon: u32,
    ) -> Result<ForecastResult> {
        let mut forecasts = Vec::new();

        for model in &self.models {
            let forecast = model.predict(historical_data, horizon).await?;
            forecasts.push(forecast);
        }

        // Weighted ensemble
        let ensemble = self.combine_forecasts(&forecasts)?;

        Ok(ensemble)
    }

    fn combine_forecasts(&self, forecasts: &[Forecast]) -> Result<ForecastResult> {
        // Weighted average of predictions with confidence intervals
        // Implementation details...
    }
}

/// ARIMA-based forecasting with auto-parameter selection
pub struct ARIMAModel {
    params: Option<ARIMAParams>,
}

impl ARIMAModel {
    pub fn auto_fit(data: &TimeSeries) -> Result<Self> {
        // Auto-detect ARIMA(p,d,q) parameters
        let (p, d, q) = Self::auto_detect_params(data)?;

        Ok(Self {
            params: Some(ARIMAParams { p, d, q }),
        })
    }

    fn auto_detect_params(data: &TimeSeries) -> Result<(usize, usize, usize)> {
        // ACF/PACF analysis for parameter selection
        // AIC/BIC model selection
        // Implementation details...
    }
}
```

## Cost Anomaly Detection (Production-Ready)

```rust
// File: src/anomaly/detector.rs

pub struct MultiMethodAnomalyDetector {
    z_score_threshold: f64,
    iqr_multiplier: f64,
    moving_avg_window: usize,
}

impl MultiMethodAnomalyDetector {
    pub fn new() -> Self {
        Self {
            z_score_threshold: 3.0,
            iqr_multiplier: 1.5,
            moving_avg_window: 7,
        }
    }

    pub async fn detect_anomalies(
        &self,
        recent_data: &TimeSeries,
        baseline_data: &TimeSeries,
    ) -> Result<Vec<Anomaly>> {
        let mut anomalies = Vec::new();

        // Method 1: Z-score detection
        anomalies.extend(self.detect_z_score_anomalies(recent_data, baseline_data).await?);

        // Method 2: IQR detection
        anomalies.extend(self.detect_iqr_anomalies(recent_data, baseline_data).await?);

        // Method 3: Moving average deviation
        anomalies.extend(self.detect_moving_avg_anomalies(recent_data).await?);

        // Method 4: Rate of change
        anomalies.extend(self.detect_rapid_changes(recent_data).await?);

        // Deduplicate and rank
        let deduplicated = self.deduplicate_anomalies(anomalies);
        let ranked = self.rank_by_severity(deduplicated);

        Ok(ranked)
    }

    async fn detect_z_score_anomalies(
        &self,
        recent: &TimeSeries,
        baseline: &TimeSeries,
    ) -> Result<Vec<Anomaly>> {
        // Z-score calculation with hour-of-day normalization
        // Implementation...
    }
}
```

## Deployment Modes (Detailed)

### 1. Standalone Daemon

```toml
# /etc/llm-costops/config.toml
[daemon]
mode = "standalone"
pid_file = "/var/run/llm-costops.pid"
log_file = "/var/log/llm-costops.log"
log_level = "info"

[storage]
timescaledb_url = "postgresql://localhost/llm_costops"
rocksdb_path = "/var/lib/llm-costops/cache"

[api]
enabled = true
bind = "127.0.0.1:3000"
grpc_bind = "127.0.0.1:50051"
```

**Systemd Service**:
```ini
# /etc/systemd/system/llm-costops.service
[Unit]
Description=LLM-CostOps Daemon
After=network.target postgresql.service

[Service]
Type=forking
User=costops
Group=costops
ExecStart=/usr/local/bin/llm-costops daemon --config /etc/llm-costops/config.toml
PIDFile=/var/run/llm-costops.pid
Restart=on-failure
RestartSec=10s

[Install]
WantedBy=multi-user.target
```

### 2. CLI Utility

```bash
#!/bin/bash
# Example CLI usage script

# Query last 24 hours of costs
llm-costops query \
  --range "last-24-hours" \
  --group-by model,provider \
  --output json \
  > daily_costs.json

# Generate 90-day forecast
llm-costops forecast \
  --horizon 90 \
  --model ensemble \
  --confidence 0.95 \
  --output forecast_90d.json

# Calculate ROI for last month
llm-costops roi \
  --period "last-month" \
  --include-optimizations \
  --output roi_report.html

# Export to Parquet for data lake
llm-costops export \
  --format parquet \
  --period "2024-Q1" \
  --output s3://data-lake/llm-costs/2024-q1/
```

### 3. API Microservice

**REST API Endpoints**:
```
POST   /api/v1/usage/ingest         - Ingest usage metrics
GET    /api/v1/costs/summary         - Get cost summary
GET    /api/v1/costs/forecast        - Get cost forecast
POST   /api/v1/roi/calculate         - Calculate ROI
GET    /api/v1/anomalies             - Get cost anomalies
POST   /api/v1/budgets               - Create budget policy
GET    /api/v1/budgets/:id/status    - Get budget status
```

**Kubernetes Deployment**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-costops-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: llm-costops
  template:
    metadata:
      labels:
        app: llm-costops
    spec:
      containers:
      - name: api
        image: llm-costops:v1.0.0
        ports:
        - containerPort: 3000
          name: http
        - containerPort: 50051
          name: grpc
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: costops-secrets
              key: database-url
        - name: REDIS_URL
          value: "redis://redis:6379"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: llm-costops-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-costops-api
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

# SPARC Phase 5: Completion

## Phased Roadmap

### Phase 1: MVP (Weeks 1-8)

**Objective:** Deliver core cost tracking functionality with basic integrations.

**Core Features:**
- Basic cost calculation engine
- Token accounting system
- Essential integrations (Observatory, Edge-Agent)
- CLI interface

**Technical Stack:**
- Rust monolithic binary
- SQLite for local persistence
- File-based configuration

**Milestones:**
- M1.1: Cost Engine Foundation (Week 2)
- M1.2: Integration Layer (Week 4)
- M1.3: CLI & Storage (Week 6)
- M1.4: MVP Release (Week 8)

**Success Metrics:**
- Cost calculation accuracy: ±0.1%
- Unit test coverage: >70%
- CLI command success rate: >99%

### Phase 2: Beta (Weeks 9-16)

**Objective:** Add multi-provider support, forecasting, and API mode.

**Enhanced Features:**
- Multi-provider support (5+ providers)
- Time-series forecasting (ARIMA-based)
- ROI correlation engine
- RESTful API microservice
- Governance integration

**Technical Evolution:**
- Hybrid deployment: CLI + API server
- PostgreSQL for production
- Redis for caching
- gRPC for inter-module communication

**Milestones:**
- M2.1: Multi-Provider Foundation (Week 10)
- M2.2: Forecasting Engine (Week 12)
- M2.3: API Microservice (Week 14)
- M2.4: Governance Integration (Week 16)

**Success Metrics:**
- Forecast MAPE: <15% (7-day)
- API uptime: >99.5%
- Test coverage: >80%

### Phase 3: v1.0 Production (Weeks 17-24)

**Objective:** Complete SPARC implementation with advanced features.

**Production Features:**
- Advanced ML forecasting (LightGBM)
- Complete deployment modes (CLI, API, serverless, WASM)
- Enterprise features (multi-tenancy, RBAC, SSO)
- Full LLM DevOps stack integration
- Cloud provider billing integrations

**Milestones:**
- M3.1: Advanced ML Forecasting (Week 18)
- M3.2: Enterprise Features (Week 20)
- M3.3: Complete Deployment Modes (Week 21)
- M3.4: Full Stack Integration (Week 22)
- M3.5: Documentation & Examples (Week 23)
- M3.6: v1.0 Release (Week 24)

**Success Metrics:**
- Forecast MAPE: <10% (7-day)
- API uptime: >99.9% (SLA)
- Test coverage: >85%

**Total Timeline:** 24 weeks (6 months) from kickoff to v1.0
**Team Size:** 2-3 full-time Rust engineers
**Buffer:** 20% contingency per phase

## Dependencies

### LLM DevOps Modules
- `llm-observatory` (v0.1+ for MVP, v0.2+ for Beta, v1.0+ for Production)
- `llm-edge-agent` (v0.1+ for MVP, v0.2+ for Beta, v1.0+ for Production)
- `llm-governance` (v0.1+ for Beta, v1.0+ for Production)
- `llm-security` (v0.2+ for Production)
- `llm-testing` (v0.2+ for Production)

### External Dependencies
- **MVP:** SQLite (bundled)
- **Beta:** PostgreSQL v14+, TimescaleDB v2.11+, Redis v7+
- **v1.0:** Kubernetes v1.25+, S3-compatible storage, Cloud provider APIs

### Rust Crates
See complete `Cargo.toml` in Architecture section with 40+ dependencies organized by feature flags.

## Validation Metrics

### Cost Calculation Accuracy
- **Target:** ±0.05% accuracy vs provider billing
- **Measurement:** Monthly reconciliation against provider invoices
- **Formula:** `Accuracy = 1 - |Calculated - Actual| / Actual`

### Forecast Precision
- **MAPE:** <10% for 7-day, <15% for 30-day, <20% for 90-day
- **Confidence Intervals:** 95% coverage >90%
- **Anomaly Detection:** F1 score >95%

### API Performance
| Metric | p50 | p95 | p99 |
|--------|-----|-----|-----|
| Cost Query (simple) | <20ms | <50ms | <100ms |
| Forecast Generation | <1s | <3s | <5s |
| Real-time Stream | <10ms | <30ms | <50ms |

### Test Coverage
- Unit tests: >85%
- Integration tests: >80%
- End-to-end tests: >70%
- Total coverage: >80%

## Risk Assessment

### Technical Risks
1. **Provider API Changes** (Impact: High, Probability: Medium)
   - Mitigation: Abstraction layer, automated monitoring, version pinning
2. **Forecasting Accuracy Degradation** (Impact: Medium, Probability: Medium)
   - Mitigation: Continuous retraining, ensemble methods, confidence intervals
3. **Database Scalability** (Impact: High, Probability: Low)
   - Mitigation: TimescaleDB hypertables, read replicas, caching
4. **Memory Leaks** (Impact: High, Probability: Low)
   - Mitigation: Rust ownership model, memory profiling, soak tests

### Integration Challenges
1. **Observatory Metrics Schema Alignment**
   - Mitigation: Early schema agreement, versioned protocol
2. **Edge-Agent Hook Performance (<5ms overhead)**
   - Mitigation: Async processing, sampling, optimized algorithms
3. **Governance Policy Synchronization**
   - Mitigation: Event-driven architecture, eventual consistency
4. **Multi-Cloud Authentication**
   - Mitigation: Centralized secret management, IAM roles

### Scalability Concerns
1. **Time-Series Data Growth** (100GB → 1TB+ first year)
   - Mitigation: Retention policies, tiered storage, compression
2. **Real-Time Calculation at Scale** (50K+ RPS)
   - Mitigation: Horizontal scaling, distributed caching, rate limiting
3. **Forecast Computation Complexity**
   - Mitigation: Pre-computed forecasts, model optimization, caching

---

# References

## SPARC Methodology
1. **SPARC Framework Overview** - Software Engineering Institute
   - URL: https://resources.sei.cmu.edu/library/asset-view.cfm?assetid=513908
2. **Applying SPARC to Microservices** - Martin Fowler
   - URL: https://martinfowler.com/articles/microservices.html

## Rust Ecosystem
3. **The Rust Programming Language** - https://doc.rust-lang.org/book/
4. **Tokio Asynchronous Runtime** - https://tokio.rs/
5. **Axum Web Framework** - https://docs.rs/axum/latest/axum/
6. **SQLx Database Library** - https://docs.rs/sqlx/latest/sqlx/
7. **Rust Performance Book** - https://nnethercote.github.io/perf-book/

## Time-Series Forecasting
8. **Time Series Forecasting: Principles and Practice** - Hyndman & Athanasopoulos
   - URL: https://otexts.com/fpp3/
9. **Prophet: Forecasting at Scale** - Facebook Research
   - DOI: 10.7287/peerj.preprints.3190v2
10. **ARIMA Models** - Statistical methods documentation
11. **Anomaly Detection: A Survey** - Chandola et al.
    - DOI: 10.1145/1541880.1541882
12. **LightGBM for Time Series** - Microsoft Research
    - GitHub: https://github.com/microsoft/LightGBM

## LLM Cost Optimization
13. **The Economics of Large Language Models** - Bommasani et al. (Stanford HAI)
    - URL: https://arxiv.org/abs/2108.07258
14. **Optimizing LLM Inference Costs** - Pope et al. (Google Research)
    - URL: https://arxiv.org/abs/2211.05102
15. **Cost-Aware LLM Serving: A Survey**
    - URL: https://arxiv.org/abs/2308.10481
16. **OpenAI Pricing** - https://openai.com/api/pricing/
17. **Anthropic Claude Pricing** - https://www.anthropic.com/api
18. **Google Vertex AI Pricing** - https://cloud.google.com/vertex-ai/pricing
19. **AWS Bedrock Pricing** - https://aws.amazon.com/bedrock/pricing/

## Open-Source Projects
20. **Langfuse** - LLM Engineering Platform
    - GitHub: https://github.com/langfuse/langfuse
21. **LiteLLM** - Unified LLM API
    - GitHub: https://github.com/BerriAI/litellm
22. **OpenLLMetry** - LLM Observability
    - GitHub: https://github.com/traceloop/openllmetry
23. **PromptLayer** - LLM Middleware
    - GitHub: https://github.com/MagnivOrg/prompt-layer-library
24. **Helicone** - LLM Observability Platform
    - GitHub: https://github.com/Helicone/helicone

## Databases & Infrastructure
25. **TimescaleDB Documentation** - https://docs.timescale.com/
26. **PostgreSQL Performance Tuning** - https://www.postgresql.org/docs/current/performance-tips.html

## APIs & Microservices
27. **RESTful API Design Rulebook** - Mark Masse
28. **GraphQL Best Practices** - https://graphql.org/learn/best-practices/
29. **gRPC in Rust (Tonic)** - https://docs.rs/tonic/latest/tonic/

## Cloud Cost Management
30. **AWS Cost Optimization** - https://aws.amazon.com/aws-cost-management/
31. **Google Cloud Cost Management** - https://cloud.google.com/cost-management
32. **Azure Cost Management** - https://azure.microsoft.com/products/cost-management/

## Observability & Security
33. **Prometheus Documentation** - https://prometheus.io/docs/
34. **OpenTelemetry in Rust** - https://opentelemetry.io/docs/languages/rust/
35. **Grafana Dashboard Design** - https://grafana.com/docs/grafana/latest/dashboards/
36. **OWASP API Security Top 10** - https://owasp.org/www-project-api-security/
37. **Rust Security Guidelines** - https://anssi-fr.github.io/rust-guide/
38. **SOC 2 Compliance Guide** - AICPA

## DevOps & Testing
39. **GitHub Actions for Rust** - https://github.com/actions-rs
40. **Kubernetes Best Practices** - "Kubernetes: Up and Running" (Hightower et al.)
41. **Helm Chart Best Practices** - https://helm.sh/docs/chart_best_practices/
42. **Property-Based Testing (PropTest)** - https://docs.rs/proptest/
43. **Mutation Testing (cargo-mutants)** - https://github.com/sourcefrog/cargo-mutants
44. **Load Testing with k6** - https://k6.io/docs/

## Machine Learning & MLOps
45. **Feature Engineering for ML** - Zheng & Casari (O'Reilly)
46. **Introducing MLOps** - Treveil et al. (O'Reilly)
47. **Practical Time Series Forecasting** - Shmueli & Lichtendahl

---

## Appendix: Glossary

- **ARIMA**: AutoRegressive Integrated Moving Average - time-series forecasting model
- **MAPE**: Mean Absolute Percentage Error - forecast accuracy metric
- **RMSE**: Root Mean Square Error - forecast accuracy metric
- **ROI**: Return on Investment
- **SLA**: Service Level Agreement
- **SPARC**: Specification, Pseudocode, Architecture, Refinement, Completion
- **TimescaleDB**: PostgreSQL extension optimized for time-series data
- **gRPC**: High-performance RPC framework by Google
- **JWT**: JSON Web Token - authentication standard
- **RBAC**: Role-Based Access Control
- **SOC 2**: Security and compliance certification standard

---

**Document Status:** Complete
**Next Steps:** Technical review, stakeholder approval, implementation kickoff
**Maintained By:** LLM DevOps Platform Team
**Contact:** cost-ops@llm-devops.org
