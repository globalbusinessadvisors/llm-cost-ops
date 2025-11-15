# SPARC Phase 2: Pseudocode

## Table of Contents
1. [Core Data Flow Algorithms](#1-core-data-flow-algorithms)
2. [Integration Workflows](#2-integration-workflows)
3. [Forecasting Logic](#3-forecasting-logic)
4. [Query & Reporting](#4-query--reporting)
5. [Supporting Algorithms](#5-supporting-algorithms)

---

## 1. CORE DATA FLOW ALGORITHMS

### 1.1 Usage Metric Ingestion Pipeline

#### Algorithm: IngestUsageMetrics
```
FUNCTION IngestUsageMetrics(source_type, raw_metric_data):
    // Source types: 'observatory', 'edge-agent', 'direct-api'

    // Step 1: Validate incoming data
    validation_result = ValidateMetricSchema(raw_metric_data, source_type)
    IF NOT validation_result.is_valid:
        LOG_ERROR("Invalid metric schema", validation_result.errors)
        SEND_TO_DLQ(raw_metric_data, validation_result.errors)
        RETURN {status: "failed", reason: "validation_error"}
    END IF

    // Step 2: Normalize metric format
    normalized_metric = NormalizeMetric(raw_metric_data, source_type)

    // Step 3: Enrich with metadata
    enriched_metric = EnrichMetric(normalized_metric)

    // Step 4: Deduplicate
    IF IsDuplicate(enriched_metric):
        LOG_INFO("Duplicate metric detected, skipping")
        RETURN {status: "skipped", reason: "duplicate"}
    END IF

    // Step 5: Store in raw metrics buffer
    buffer_id = StoreInBuffer(enriched_metric)

    // Step 6: Trigger async processing
    ENQUEUE_FOR_PROCESSING(buffer_id, enriched_metric)

    // Step 7: Emit ingestion event
    EMIT_EVENT("metric_ingested", {
        source: source_type,
        metric_id: enriched_metric.id,
        timestamp: enriched_metric.timestamp
    })

    RETURN {status: "success", metric_id: enriched_metric.id}
END FUNCTION


FUNCTION NormalizeMetric(raw_data, source_type):
    normalized = {
        id: GENERATE_UUID(),
        timestamp: PARSE_TIMESTAMP(raw_data.timestamp),
        source: source_type,
        model_id: NULL,
        provider: NULL,
        tokens_input: 0,
        tokens_output: 0,
        tokens_total: 0,
        request_count: 1,
        latency_ms: NULL,
        error_occurred: false,
        metadata: {}
    }

    SWITCH source_type:
        CASE 'observatory':
            normalized.model_id = raw_data.model_identifier
            normalized.provider = raw_data.provider_name
            normalized.tokens_input = raw_data.prompt_tokens
            normalized.tokens_output = raw_data.completion_tokens
            normalized.tokens_total = raw_data.total_tokens
            normalized.latency_ms = raw_data.response_time_ms
            normalized.metadata = raw_data.observability_context

        CASE 'edge-agent':
            normalized.model_id = raw_data.model
            normalized.provider = ExtractProviderFromModel(raw_data.model)
            normalized.tokens_input = raw_data.usage.prompt_tokens
            normalized.tokens_output = raw_data.usage.completion_tokens
            normalized.tokens_total = raw_data.usage.total_tokens
            normalized.metadata = {
                edge_agent_id: raw_data.agent_id,
                request_id: raw_data.request_id
            }

        CASE 'direct-api':
            normalized.model_id = raw_data.model
            normalized.provider = raw_data.provider
            normalized.tokens_input = raw_data.input_tokens
            normalized.tokens_output = raw_data.output_tokens
            normalized.tokens_total = raw_data.input_tokens + raw_data.output_tokens

        DEFAULT:
            THROW InvalidSourceTypeException(source_type)
    END SWITCH

    RETURN normalized
END FUNCTION


FUNCTION EnrichMetric(metric):
    // Add organizational context
    metric.project_id = DeriveProjectId(metric.metadata)
    metric.team_id = DeriveTeamId(metric.project_id)
    metric.cost_center = DeriveCostCenter(metric.team_id)

    // Add provider pricing tier information
    pricing_tier = GetPricingTier(metric.provider, metric.model_id, metric.timestamp)
    metric.pricing_tier = pricing_tier

    // Add time-based partitioning keys
    metric.partition_date = ExtractDate(metric.timestamp)
    metric.partition_hour = ExtractHour(metric.timestamp)

    // Add geographic region if available
    IF metric.metadata.region:
        metric.region = metric.metadata.region
    ELSE:
        metric.region = "unknown"
    END IF

    RETURN metric
END FUNCTION
```

**Flowchart Description: Ingestion Pipeline**
```
[Raw Metric]
    → [Validate Schema]
        → {Valid?}
            YES → [Normalize Format]
                → [Enrich Metadata]
                → [Check Duplicate]
                    → {Duplicate?}
                        NO → [Store in Buffer]
                            → [Enqueue Processing]
                            → [Emit Event]
                            → [Success]
                        YES → [Log & Skip]
                            → [End]
            NO → [Log Error]
                → [Send to DLQ]
                → [Failure]
```

---

### 1.2 Token Counting and Normalization

#### Algorithm: NormalizeTokenCounts
```
FUNCTION NormalizeTokenCounts(metric):
    // Handle missing or zero token counts
    IF metric.tokens_total == 0 OR metric.tokens_total IS NULL:
        IF metric.tokens_input > 0 OR metric.tokens_output > 0:
            metric.tokens_total = metric.tokens_input + metric.tokens_output
        ELSE:
            // Estimate tokens from request/response if available
            IF metric.metadata.request_text AND metric.metadata.response_text:
                metric.tokens_input = EstimateTokens(metric.metadata.request_text, metric.model_id)
                metric.tokens_output = EstimateTokens(metric.metadata.response_text, metric.model_id)
                metric.tokens_total = metric.tokens_input + metric.tokens_output
                metric.token_count_estimated = true
            ELSE:
                LOG_WARNING("Cannot determine token count", metric.id)
                metric.token_count_estimated = false
                metric.tokens_total = 0
            END IF
        END IF
    END IF

    // Validate token count consistency
    IF metric.tokens_total != (metric.tokens_input + metric.tokens_output):
        discrepancy = ABS(metric.tokens_total - (metric.tokens_input + metric.tokens_output))
        IF discrepancy > THRESHOLD_TOKEN_DISCREPANCY:
            LOG_WARNING("Token count mismatch", {
                metric_id: metric.id,
                reported_total: metric.tokens_total,
                calculated_total: metric.tokens_input + metric.tokens_output
            })
            // Use provider's total as source of truth
            metric.token_count_discrepancy = discrepancy
        END IF
    END IF

    // Normalize to standard token units (some providers use different counting)
    normalized_tokens = ApplyTokenNormalizationFactor(
        metric.tokens_total,
        metric.provider,
        metric.model_id
    )

    metric.tokens_normalized = normalized_tokens

    RETURN metric
END FUNCTION


FUNCTION EstimateTokens(text, model_id):
    // Get tokenizer for specific model
    tokenizer = GetTokenizer(model_id)

    IF tokenizer IS NOT NULL:
        // Use actual tokenizer
        tokens = tokenizer.encode(text)
        RETURN LENGTH(tokens)
    ELSE:
        // Fallback to approximation (1 token ≈ 4 characters for English)
        char_count = LENGTH(text)
        estimated_tokens = CEILING(char_count / 4)
        RETURN estimated_tokens
    END IF
END FUNCTION


FUNCTION ApplyTokenNormalizationFactor(token_count, provider, model_id):
    // Some providers count tokens differently
    normalization_factor = GetNormalizationFactor(provider, model_id)

    // Default factor is 1.0 (no normalization)
    IF normalization_factor IS NULL:
        normalization_factor = 1.0
    END IF

    normalized = token_count * normalization_factor

    RETURN ROUND(normalized, 0)
END FUNCTION
```

---

### 1.3 Cost Calculation Engine

#### Algorithm: CalculateCost
```
FUNCTION CalculateCost(metric):
    // Step 1: Retrieve pricing data
    pricing = GetPricingData(
        provider: metric.provider,
        model: metric.model_id,
        timestamp: metric.timestamp,
        region: metric.region
    )

    IF pricing IS NULL:
        LOG_ERROR("Pricing data not found", {
            provider: metric.provider,
            model: metric.model_id
        })
        RETURN NULL
    END IF

    // Step 2: Calculate base costs
    input_cost = CalculateInputCost(metric.tokens_input, pricing)
    output_cost = CalculateOutputCost(metric.tokens_output, pricing)
    base_cost = input_cost + output_cost

    // Step 3: Apply volume discounts
    discount = CalculateVolumeDiscount(
        metric.project_id,
        metric.provider,
        metric.partition_date
    )

    discounted_cost = base_cost * (1 - discount)

    // Step 4: Add any surcharges
    surcharges = CalculateSurcharges(metric, pricing)

    total_cost = discounted_cost + surcharges

    // Step 5: Create cost record
    cost_record = {
        metric_id: metric.id,
        timestamp: metric.timestamp,
        provider: metric.provider,
        model_id: metric.model_id,
        tokens_input: metric.tokens_input,
        tokens_output: metric.tokens_output,
        tokens_total: metric.tokens_total,
        input_cost_usd: input_cost,
        output_cost_usd: output_cost,
        base_cost_usd: base_cost,
        discount_rate: discount,
        discount_amount_usd: base_cost * discount,
        surcharges_usd: surcharges,
        total_cost_usd: total_cost,
        pricing_tier: pricing.tier,
        pricing_version: pricing.version,
        currency: "USD",
        project_id: metric.project_id,
        cost_center: metric.cost_center
    }

    // Step 6: Store cost record
    StoreCostRecord(cost_record)

    // Step 7: Update running totals
    UpdateCostAggregates(cost_record)

    RETURN cost_record
END FUNCTION


FUNCTION CalculateInputCost(tokens, pricing):
    // Pricing typically per 1M tokens
    cost_per_token = pricing.input_price_per_1m / 1_000_000
    total_cost = tokens * cost_per_token
    RETURN ROUND(total_cost, 6)  // Round to 6 decimal places for precision
END FUNCTION


FUNCTION CalculateOutputCost(tokens, pricing):
    cost_per_token = pricing.output_price_per_1m / 1_000_000
    total_cost = tokens * cost_per_token
    RETURN ROUND(total_cost, 6)
END FUNCTION


FUNCTION CalculateVolumeDiscount(project_id, provider, date):
    // Get monthly usage for this project and provider
    monthly_usage = GetMonthlyTokenUsage(project_id, provider, date)

    // Get discount tiers
    discount_tiers = GetDiscountTiers(provider)

    // Find applicable discount tier
    discount_rate = 0.0
    FOR EACH tier IN discount_tiers:
        IF monthly_usage >= tier.min_tokens:
            discount_rate = tier.discount_rate
        ELSE:
            BREAK
        END IF
    END FOR

    RETURN discount_rate
END FUNCTION


FUNCTION CalculateSurcharges(metric, pricing):
    total_surcharges = 0.0

    // Peak time surcharge
    IF IsPeakTime(metric.timestamp, pricing.peak_hours):
        peak_surcharge = metric.base_cost * pricing.peak_multiplier - metric.base_cost
        total_surcharges += peak_surcharge
    END IF

    // Regional surcharge
    IF pricing.regional_surcharges.CONTAINS(metric.region):
        regional_surcharge = metric.base_cost * pricing.regional_surcharges[metric.region]
        total_surcharges += regional_surcharge
    END IF

    // Feature-based surcharges (e.g., function calling, vision)
    IF metric.metadata.features:
        FOR EACH feature IN metric.metadata.features:
            IF pricing.feature_surcharges.CONTAINS(feature):
                feature_surcharge = pricing.feature_surcharges[feature]
                total_surcharges += feature_surcharge
            END IF
        END FOR
    END IF

    RETURN total_surcharges
END FUNCTION
```

**Decision Tree: Cost Calculation**
```
[Metric]
    → {Pricing Data Available?}
        YES → [Calculate Input Cost]
            → [Calculate Output Cost]
            → [Sum Base Cost]
            → {Volume Discount Applicable?}
                YES → [Apply Discount]
                NO → [Skip Discount]
            → {Surcharges Apply?}
                YES → [Add Surcharges]
                NO → [Skip Surcharges]
            → [Store Cost Record]
            → [Return Cost]
        NO → [Log Error]
            → [Return NULL]
```

---

### 1.4 Performance Correlation Algorithm

#### Algorithm: CorrelatePerformanceMetrics
```
FUNCTION CorrelatePerformanceMetrics(cost_record):
    // Step 1: Fetch corresponding performance data from Test-Bench
    performance_data = FetchPerformanceData(
        model_id: cost_record.model_id,
        time_window: {
            start: cost_record.timestamp - TIME_WINDOW_BEFORE,
            end: cost_record.timestamp + TIME_WINDOW_AFTER
        }
    )

    IF performance_data IS EMPTY:
        // No performance data available for correlation
        RETURN NULL
    END IF

    // Step 2: Calculate performance metrics
    performance_metrics = {
        avg_accuracy: AVERAGE(performance_data.accuracy_scores),
        avg_latency_ms: AVERAGE(performance_data.latency_values),
        avg_quality_score: AVERAGE(performance_data.quality_scores),
        error_rate: COUNT(performance_data.errors) / COUNT(performance_data.total),
        benchmark_tasks: UNIQUE(performance_data.task_types)
    }

    // Step 3: Create correlation record
    correlation = {
        cost_record_id: cost_record.id,
        model_id: cost_record.model_id,
        provider: cost_record.provider,
        timestamp: cost_record.timestamp,
        total_cost_usd: cost_record.total_cost_usd,
        tokens_total: cost_record.tokens_total,
        performance_metrics: performance_metrics,
        cost_per_quality_point: cost_record.total_cost_usd / performance_metrics.avg_quality_score,
        cost_efficiency_score: CalculateCostEfficiencyScore(cost_record, performance_metrics)
    }

    // Step 4: Store correlation data
    StoreCorrelation(correlation)

    // Step 5: Update performance-cost indexes
    UpdatePerformanceCostIndex(correlation)

    RETURN correlation
END FUNCTION


FUNCTION CalculateCostEfficiencyScore(cost_record, performance_metrics):
    // Normalize all metrics to 0-1 scale
    normalized_quality = performance_metrics.avg_quality_score / 100.0
    normalized_speed = 1 / (1 + performance_metrics.avg_latency_ms / 1000.0)
    normalized_reliability = 1 - performance_metrics.error_rate

    // Weighted performance composite
    performance_composite = (
        normalized_quality * WEIGHT_QUALITY +
        normalized_speed * WEIGHT_SPEED +
        normalized_reliability * WEIGHT_RELIABILITY
    )

    // Normalize cost (lower is better)
    // Using log scale for cost to handle wide ranges
    normalized_cost = 1 / (1 + LOG(1 + cost_record.total_cost_usd * 1000))

    // Cost efficiency = Performance / Cost
    // Higher score = better efficiency
    efficiency_score = performance_composite / (1 - normalized_cost + 0.01)

    RETURN ROUND(efficiency_score, 4)
END FUNCTION
```

---

### 1.5 ROI Computation Logic

#### Algorithm: ComputeROI
```
FUNCTION ComputeROI(use_case_id, time_period):
    // Step 1: Gather all costs for the use case
    total_costs = GetUseCaseCosts(use_case_id, time_period)

    // Step 2: Gather business value metrics
    value_metrics = GetUseCaseValueMetrics(use_case_id, time_period)

    // Step 3: Calculate direct benefits
    direct_benefits = CalculateDirectBenefits(value_metrics)

    // Step 4: Calculate indirect benefits (time saved, automation, etc.)
    indirect_benefits = CalculateIndirectBenefits(value_metrics)

    total_benefits = direct_benefits + indirect_benefits

    // Step 5: Calculate ROI
    roi_percentage = ((total_benefits - total_costs.total_cost_usd) / total_costs.total_cost_usd) * 100

    // Step 6: Calculate payback period
    monthly_benefit = total_benefits / MonthsInPeriod(time_period)
    monthly_cost = total_costs.total_cost_usd / MonthsInPeriod(time_period)
    net_monthly_benefit = monthly_benefit - monthly_cost

    IF net_monthly_benefit > 0:
        // Calculate how many months to break even (if there was initial investment)
        payback_months = total_costs.initial_setup_cost / net_monthly_benefit
    ELSE:
        payback_months = INFINITY  // Never pays back
    END IF

    // Step 7: Create ROI report
    roi_report = {
        use_case_id: use_case_id,
        time_period: time_period,
        total_costs: total_costs.total_cost_usd,
        breakdown_costs: {
            llm_usage: total_costs.llm_cost,
            infrastructure: total_costs.infra_cost,
            development: total_costs.dev_cost,
            maintenance: total_costs.maintenance_cost
        },
        total_benefits: total_benefits,
        breakdown_benefits: {
            direct: direct_benefits,
            indirect: indirect_benefits
        },
        net_benefit: total_benefits - total_costs.total_cost_usd,
        roi_percentage: roi_percentage,
        payback_period_months: payback_months,
        cost_benefit_ratio: total_benefits / total_costs.total_cost_usd,
        generated_at: CURRENT_TIMESTAMP()
    }

    // Step 8: Store ROI report
    StoreROIReport(roi_report)

    RETURN roi_report
END FUNCTION


FUNCTION CalculateDirectBenefits(value_metrics):
    benefits = 0.0

    // Revenue generated
    IF value_metrics.revenue_generated:
        benefits += value_metrics.revenue_generated
    END IF

    // Cost savings from automation
    IF value_metrics.manual_tasks_automated:
        labor_hours_saved = value_metrics.manual_tasks_automated * value_metrics.hours_per_task
        cost_per_hour = value_metrics.average_hourly_rate
        automation_savings = labor_hours_saved * cost_per_hour
        benefits += automation_savings
    END IF

    // Reduced error costs
    IF value_metrics.errors_prevented:
        error_cost = value_metrics.errors_prevented * value_metrics.average_error_cost
        benefits += error_cost
    END IF

    RETURN benefits
END FUNCTION


FUNCTION CalculateIndirectBenefits(value_metrics):
    benefits = 0.0

    // Improved customer satisfaction → retention
    IF value_metrics.customer_satisfaction_improvement:
        retention_improvement = value_metrics.customer_satisfaction_improvement * 0.15  // Industry factor
        revenue_impact = value_metrics.customer_lifetime_value * retention_improvement
        benefits += revenue_impact
    END IF

    // Faster time to market
    IF value_metrics.time_to_market_reduction_days:
        opportunity_cost_per_day = value_metrics.daily_revenue_potential
        benefits += value_metrics.time_to_market_reduction_days * opportunity_cost_per_day
    END IF

    // Quality improvements
    IF value_metrics.quality_improvement_score:
        // Quality improvements can reduce rework and support costs
        quality_benefit = value_metrics.quality_improvement_score * value_metrics.rework_cost_factor
        benefits += quality_benefit
    END IF

    RETURN benefits
END FUNCTION
```

---

## 2. INTEGRATION WORKFLOWS

### 2.1 LLM-Observatory → CostOps Metric Streaming

#### Workflow: ObservatoryStreamingIntegration
```
// Observatory publishes metrics to message queue/stream
FUNCTION HandleObservatoryStream():
    // Subscribe to Observatory metrics topic
    SUBSCRIBE_TO_TOPIC("llm.observatory.metrics", OnObservatoryMetric)
END FUNCTION


FUNCTION OnObservatoryMetric(message):
    TRY:
        // Step 1: Parse message
        metric_data = PARSE_JSON(message.payload)

        // Step 2: Validate message format
        IF NOT ValidateObservatorySchema(metric_data):
            LOG_ERROR("Invalid Observatory metric schema", message.id)
            NACK_MESSAGE(message)
            RETURN
        END IF

        // Step 3: Transform to CostOps format
        costops_metric = TransformObservatoryToCostOps(metric_data)

        // Step 4: Ingest into CostOps pipeline
        result = IngestUsageMetrics("observatory", costops_metric)

        IF result.status == "success":
            ACK_MESSAGE(message)

            // Step 5: Send acknowledgment back to Observatory
            PublishAck("llm.costops.ack", {
                original_id: message.id,
                costops_metric_id: result.metric_id,
                timestamp: CURRENT_TIMESTAMP()
            })
        ELSE:
            NACK_MESSAGE(message)
        END IF

    CATCH exception:
        LOG_ERROR("Error processing Observatory metric", exception)
        NACK_MESSAGE(message)
        SEND_ALERT("Observatory integration error", exception)
    END TRY
END FUNCTION


FUNCTION TransformObservatoryToCostOps(obs_data):
    costops_data = {
        timestamp: obs_data.timestamp,
        model_identifier: obs_data.model_name,
        provider_name: obs_data.provider,
        prompt_tokens: obs_data.token_usage.input,
        completion_tokens: obs_data.token_usage.output,
        total_tokens: obs_data.token_usage.total,
        response_time_ms: obs_data.latency_ms,
        observability_context: {
            trace_id: obs_data.trace_id,
            span_id: obs_data.span_id,
            request_id: obs_data.request_id,
            user_id: obs_data.user_context.user_id,
            session_id: obs_data.user_context.session_id,
            tags: obs_data.tags
        }
    }

    RETURN costops_data
END FUNCTION
```

**Sequence Diagram Description: Observatory Integration**
```
Observatory → Message Queue: Publish metric
Message Queue → CostOps: Deliver message
CostOps → CostOps: Validate schema
CostOps → CostOps: Transform format
CostOps → CostOps: Ingest metric
CostOps → CostOps: Calculate cost
CostOps → Message Queue: ACK message
CostOps → Observatory: Publish acknowledgment
```

---

### 2.2 LLM-Edge-Agent → CostOps Usage Reporting

#### Workflow: EdgeAgentReporting
```
// Edge Agent calls CostOps API to report usage
FUNCTION ReceiveEdgeAgentReport(request):
    // Step 1: Authenticate Edge Agent
    auth_result = AuthenticateEdgeAgent(request.headers.authorization)
    IF NOT auth_result.valid:
        RETURN {status: 401, error: "Unauthorized"}
    END IF

    edge_agent_id = auth_result.agent_id

    // Step 2: Parse batch of usage records
    usage_batch = PARSE_JSON(request.body)

    // Step 3: Validate batch
    IF NOT ValidateEdgeAgentBatch(usage_batch):
        RETURN {status: 400, error: "Invalid batch format"}
    END IF

    // Step 4: Process each usage record
    results = []
    FOR EACH usage_record IN usage_batch.records:
        // Add agent identification
        usage_record.agent_id = edge_agent_id
        usage_record.agent_version = usage_batch.agent_version
        usage_record.edge_location = usage_batch.edge_location

        // Ingest the metric
        result = IngestUsageMetrics("edge-agent", usage_record)

        results.APPEND({
            request_id: usage_record.request_id,
            status: result.status,
            metric_id: result.metric_id
        })
    END FOR

    // Step 5: Update Edge Agent statistics
    UpdateEdgeAgentStats(edge_agent_id, {
        last_report_time: CURRENT_TIMESTAMP(),
        records_reported: LENGTH(usage_batch.records),
        successful_ingestions: COUNT(results WHERE status == "success")
    })

    // Step 6: Return results
    RETURN {
        status: 200,
        batch_id: GENERATE_UUID(),
        results: results,
        timestamp: CURRENT_TIMESTAMP()
    }
END FUNCTION


// Edge Agent batch reporting (called periodically by agent)
FUNCTION EdgeAgentBatchReporter(agent_config):
    // Runs on Edge Agent side

    WHILE true:
        // Step 1: Collect buffered usage records
        buffered_records = GetBufferedUsageRecords(agent_config.buffer_size)

        IF LENGTH(buffered_records) == 0:
            SLEEP(agent_config.reporting_interval)
            CONTINUE
        END IF

        // Step 2: Create batch payload
        batch = {
            agent_version: VERSION,
            edge_location: agent_config.location,
            records: buffered_records
        }

        // Step 3: Send to CostOps
        TRY:
            response = HTTP_POST(
                url: agent_config.costops_endpoint + "/api/v1/usage/edge-agent",
                headers: {
                    "Authorization": "Bearer " + agent_config.api_key,
                    "Content-Type": "application/json"
                },
                body: JSON_STRINGIFY(batch),
                timeout: agent_config.request_timeout
            )

            IF response.status == 200:
                // Step 4: Remove successfully reported records from buffer
                batch_results = PARSE_JSON(response.body).results
                FOR EACH result IN batch_results:
                    IF result.status == "success":
                        RemoveFromBuffer(result.request_id)
                    END IF
                END FOR
            ELSE:
                LOG_ERROR("Failed to report to CostOps", response)
            END IF

        CATCH exception:
            LOG_ERROR("Error reporting to CostOps", exception)
            // Records remain in buffer for retry
        END TRY

        // Step 5: Wait for next reporting interval
        SLEEP(agent_config.reporting_interval)
    END WHILE
END FUNCTION
```

---

### 2.3 LLM-Test-Bench → CostOps Performance Correlation

#### Workflow: TestBenchIntegration
```
FUNCTION ReceiveTestBenchResults(test_results):
    // Called when Test-Bench completes a benchmark run

    // Step 1: Validate test results
    IF NOT ValidateTestBenchSchema(test_results):
        RETURN {status: "error", message: "Invalid test results schema"}
    END IF

    // Step 2: Extract performance metrics
    performance_data = {
        benchmark_id: test_results.benchmark_id,
        model_id: test_results.model_tested,
        provider: test_results.provider,
        timestamp: test_results.completed_at,
        accuracy_score: test_results.metrics.accuracy,
        quality_score: test_results.metrics.quality,
        latency_p50: test_results.metrics.latency.p50,
        latency_p95: test_results.metrics.latency.p95,
        latency_p99: test_results.metrics.latency.p99,
        error_rate: test_results.metrics.error_rate,
        task_type: test_results.task_category,
        test_dataset: test_results.dataset_name
    }

    // Step 3: Store performance data
    performance_id = StorePerformanceData(performance_data)

    // Step 4: Find corresponding cost data
    cost_records = FindCostRecords(
        model_id: performance_data.model_id,
        time_range: {
            start: test_results.started_at,
            end: test_results.completed_at
        }
    )

    // Step 5: Create correlations
    correlations = []
    FOR EACH cost_record IN cost_records:
        correlation = CorrelatePerformanceMetrics(cost_record)
        correlations.APPEND(correlation)
    END FOR

    // Step 6: Update model performance-cost profile
    UpdateModelProfile(performance_data.model_id, performance_data, cost_records)

    // Step 7: Send results back to Test-Bench
    PublishToTestBench("llm.testbench.costops-correlation", {
        benchmark_id: test_results.benchmark_id,
        performance_id: performance_id,
        correlations: correlations,
        cost_summary: SummarizeCosts(cost_records)
    })

    RETURN {status: "success", performance_id: performance_id}
END FUNCTION


FUNCTION UpdateModelProfile(model_id, performance_data, cost_records):
    // Aggregate historical performance-cost data
    profile = GetOrCreateModelProfile(model_id)

    // Update performance statistics
    profile.performance_stats.accuracy_history.APPEND(performance_data.accuracy_score)
    profile.performance_stats.quality_history.APPEND(performance_data.quality_score)
    profile.performance_stats.latency_history.APPEND(performance_data.latency_p50)

    // Update cost statistics
    total_cost = SUM(cost_records.total_cost_usd)
    total_tokens = SUM(cost_records.tokens_total)

    profile.cost_stats.total_cost_accumulated += total_cost
    profile.cost_stats.total_tokens_accumulated += total_tokens
    profile.cost_stats.avg_cost_per_1m_tokens =
        (profile.cost_stats.total_cost_accumulated / profile.cost_stats.total_tokens_accumulated) * 1_000_000

    // Update composite scores
    profile.efficiency_score = CalculateModelEfficiencyScore(profile)
    profile.value_score = CalculateModelValueScore(profile)

    // Update timestamp
    profile.last_updated = CURRENT_TIMESTAMP()

    SaveModelProfile(profile)
END FUNCTION
```

---

### 2.4 LLM-Governance-Core ↔ CostOps Budget Enforcement

#### Workflow: GovernanceBudgetEnforcement
```
FUNCTION EnforceBudgetLimits(cost_record):
    // Called after each cost calculation

    // Step 1: Get budget configuration from Governance
    budget_config = FetchGovernanceBudget(
        project_id: cost_record.project_id,
        cost_center: cost_record.cost_center
    )

    IF budget_config IS NULL:
        // No budget limits configured
        RETURN {enforced: false, reason: "no_budget_configured"}
    END IF

    // Step 2: Calculate current spending
    current_period_spend = GetCurrentPeriodSpending(
        cost_center: cost_record.cost_center,
        period_type: budget_config.period_type
    )

    new_total_spend = current_period_spend + cost_record.total_cost_usd

    // Step 3: Check budget thresholds
    budget_status = CheckBudgetThresholds(new_total_spend, budget_config)

    // Step 4: Take action based on threshold
    SWITCH budget_status.level:
        CASE "NORMAL":
            // Under threshold, no action needed
            RETURN {enforced: false, status: "normal"}

        CASE "WARNING":
            // Approaching limit (e.g., 80%)
            SEND_ALERT("budget_warning", {
                cost_center: cost_record.cost_center,
                current_spend: new_total_spend,
                budget_limit: budget_config.limit,
                percentage_used: budget_status.percentage,
                period: budget_config.period_type
            })

            // Log to Governance
            LogToGovernance("budget_warning", budget_status)

            RETURN {enforced: false, status: "warning", details: budget_status}

        CASE "CRITICAL":
            // Near limit (e.g., 95%)
            SEND_ALERT("budget_critical", budget_status)
            LogToGovernance("budget_critical", budget_status)

            // Request throttling policy from Governance
            throttle_policy = RequestGovernancePolicy(
                policy_type: "throttle",
                cost_center: cost_record.cost_center
            )

            IF throttle_policy:
                ApplyThrottling(cost_record.project_id, throttle_policy)
            END IF

            RETURN {enforced: true, status: "critical", action: "throttled"}

        CASE "EXCEEDED":
            // Over limit
            SEND_ALERT("budget_exceeded", budget_status)
            LogToGovernance("budget_exceeded", budget_status)

            // Check hard vs soft limit
            IF budget_config.hard_limit:
                // Block further requests
                BlockProject(cost_record.project_id, "Budget exceeded")

                RETURN {enforced: true, status: "exceeded", action: "blocked"}
            ELSE:
                // Soft limit - log but allow
                RETURN {enforced: false, status: "exceeded", action: "logged"}
            END IF
    END SWITCH
END FUNCTION


FUNCTION CheckBudgetThresholds(current_spend, budget_config):
    percentage_used = (current_spend / budget_config.limit) * 100

    IF percentage_used >= 100:
        level = "EXCEEDED"
    ELSE IF percentage_used >= budget_config.critical_threshold:
        level = "CRITICAL"
    ELSE IF percentage_used >= budget_config.warning_threshold:
        level = "WARNING"
    ELSE:
        level = "NORMAL"
    END IF

    RETURN {
        level: level,
        percentage: percentage_used,
        current_spend: current_spend,
        budget_limit: budget_config.limit,
        remaining: budget_config.limit - current_spend,
        period: budget_config.period_type
    }
END FUNCTION


// Sync budget updates from Governance to CostOps
FUNCTION SyncBudgetsFromGovernance():
    // Periodic sync job

    // Step 1: Fetch all budget configurations from Governance
    governance_budgets = FetchAllGovernanceBudgets()

    // Step 2: Update local budget cache
    FOR EACH budget IN governance_budgets:
        local_budget = GetLocalBudgetConfig(budget.cost_center)

        IF local_budget IS NULL OR budget.version > local_budget.version:
            // New or updated budget
            UpdateLocalBudgetConfig(budget)

            LOG_INFO("Budget updated", {
                cost_center: budget.cost_center,
                new_limit: budget.limit,
                version: budget.version
            })
        END IF
    END FOR

    // Step 3: Clean up stale budgets
    CleanupStaleBudgets(governance_budgets)
END FUNCTION
```

**Integration Flow: Budget Enforcement**
```
[New Cost Record]
    → [Fetch Budget Config from Governance]
    → [Calculate Current Period Spend]
    → [Add New Cost]
    → {Check Threshold}
        → [< 80%: NORMAL]
            → [Allow]
        → [80-95%: WARNING]
            → [Send Alert]
            → [Log to Governance]
            → [Allow]
        → [95-100%: CRITICAL]
            → [Send Alert]
            → [Request Throttle Policy]
            → [Apply Throttling]
            → [Allow with Throttle]
        → [> 100%: EXCEEDED]
            → [Send Alert]
            → {Hard Limit?}
                → YES: [Block Project]
                → NO: [Log & Allow]
```

---

### 2.5 LLM-Auto-Optimizer ↔ CostOps Routing Decisions

#### Workflow: OptimizerCostFeedback
```
FUNCTION ProvideCostDataToOptimizer(optimization_request):
    // Auto-Optimizer requests cost data to make routing decisions

    // Step 1: Extract request parameters
    models_to_compare = optimization_request.candidate_models
    time_window = optimization_request.analysis_window
    use_case = optimization_request.use_case

    // Step 2: Gather cost data for each model
    cost_analysis = []
    FOR EACH model IN models_to_compare:
        model_costs = GetModelCostStats(model, time_window)
        performance_data = GetModelPerformanceData(model, time_window)

        analysis = {
            model_id: model,
            provider: model_costs.provider,
            avg_cost_per_request: model_costs.avg_cost_per_request,
            avg_cost_per_1k_tokens: model_costs.avg_cost_per_1k_tokens,
            total_requests: model_costs.total_requests,
            total_cost: model_costs.total_cost,
            cost_trend: CalculateCostTrend(model, time_window),
            performance_metrics: performance_data,
            efficiency_score: model_costs.efficiency_score
        }

        cost_analysis.APPEND(analysis)
    END FOR

    // Step 3: Rank models by cost-efficiency
    ranked_models = RankModelsByCostEfficiency(cost_analysis, optimization_request.optimization_goal)

    // Step 4: Generate recommendations
    recommendations = GenerateRoutingRecommendations(ranked_models, use_case)

    // Step 5: Return analysis to Auto-Optimizer
    RETURN {
        request_id: optimization_request.id,
        timestamp: CURRENT_TIMESTAMP(),
        cost_analysis: cost_analysis,
        ranked_models: ranked_models,
        recommendations: recommendations
    }
END FUNCTION


FUNCTION RankModelsByCostEfficiency(cost_analysis, optimization_goal):
    // Different ranking strategies based on goal

    SWITCH optimization_goal:
        CASE "minimize_cost":
            // Rank by lowest cost
            ranked = SORT(cost_analysis, BY: avg_cost_per_request, ORDER: ASC)

        CASE "maximize_performance":
            // Rank by highest performance (cost is secondary)
            ranked = SORT(cost_analysis, BY: [
                performance_metrics.quality_score DESC,
                avg_cost_per_request ASC
            ])

        CASE "optimize_efficiency":
            // Rank by best cost-performance ratio
            ranked = SORT(cost_analysis, BY: efficiency_score, ORDER: DESC)

        CASE "balance":
            // Weighted composite score
            FOR EACH analysis IN cost_analysis:
                // Normalize metrics to 0-1
                norm_cost = 1 - (analysis.avg_cost_per_request / MAX(cost_analysis.avg_cost_per_request))
                norm_quality = analysis.performance_metrics.quality_score / 100
                norm_speed = 1 / (1 + analysis.performance_metrics.avg_latency_ms / 1000)

                // Weighted score
                analysis.composite_score = (
                    norm_cost * WEIGHT_COST +
                    norm_quality * WEIGHT_QUALITY +
                    norm_speed * WEIGHT_SPEED
                )
            END FOR

            ranked = SORT(cost_analysis, BY: composite_score, ORDER: DESC)

        DEFAULT:
            ranked = cost_analysis
    END SWITCH

    RETURN ranked
END FUNCTION


FUNCTION GenerateRoutingRecommendations(ranked_models, use_case):
    recommendations = []

    // Best overall choice
    best_model = ranked_models[0]
    recommendations.APPEND({
        recommendation_type: "primary",
        model_id: best_model.model_id,
        provider: best_model.provider,
        reason: "Best cost-efficiency score",
        expected_cost_per_request: best_model.avg_cost_per_request,
        confidence: 0.95
    })

    // Budget-friendly alternative
    cheapest_model = FIND_MIN(ranked_models, BY: avg_cost_per_request)
    IF cheapest_model != best_model:
        recommendations.APPEND({
            recommendation_type: "budget_alternative",
            model_id: cheapest_model.model_id,
            provider: cheapest_model.provider,
            reason: "Lowest cost option",
            expected_cost_per_request: cheapest_model.avg_cost_per_request,
            cost_savings: best_model.avg_cost_per_request - cheapest_model.avg_cost_per_request,
            performance_tradeoff: best_model.performance_metrics.quality_score - cheapest_model.performance_metrics.quality_score
        })
    END IF

    // High-performance alternative (for critical requests)
    best_performance = FIND_MAX(ranked_models, BY: performance_metrics.quality_score)
    IF best_performance != best_model:
        recommendations.APPEND({
            recommendation_type: "premium_alternative",
            model_id: best_performance.model_id,
            provider: best_performance.provider,
            reason: "Highest quality/performance",
            expected_cost_per_request: best_performance.avg_cost_per_request,
            additional_cost: best_performance.avg_cost_per_request - best_model.avg_cost_per_request,
            performance_gain: best_performance.performance_metrics.quality_score - best_model.performance_metrics.quality_score
        })
    END IF

    RETURN recommendations
END FUNCTION


// Receive routing decision from Auto-Optimizer for tracking
FUNCTION TrackRoutingDecision(routing_event):
    // Track which model was selected and why

    tracking_record = {
        timestamp: routing_event.timestamp,
        request_id: routing_event.request_id,
        selected_model: routing_event.selected_model,
        selection_reason: routing_event.reason,
        candidate_models: routing_event.candidates,
        expected_cost: routing_event.expected_cost,
        actual_cost: NULL,  // Will be filled in when cost is calculated
        cost_prediction_error: NULL,
        use_case: routing_event.use_case
    }

    StoreRoutingTracking(tracking_record)

    // Later, when actual cost comes in
    REGISTER_CALLBACK(routing_event.request_id, UpdateActualCost)
END FUNCTION


FUNCTION UpdateActualCost(request_id, actual_cost_record):
    tracking_record = GetRoutingTracking(request_id)

    tracking_record.actual_cost = actual_cost_record.total_cost_usd
    tracking_record.cost_prediction_error =
        ABS(tracking_record.actual_cost - tracking_record.expected_cost)

    // Update prediction model accuracy
    UpdateCostPredictionModel(tracking_record)

    SaveRoutingTracking(tracking_record)
END FUNCTION
```

---

### 2.6 LLM-Registry → CostOps Rate Updates

#### Workflow: RegistryPricingSync
```
FUNCTION SyncPricingFromRegistry():
    // Periodic job to sync latest pricing from Registry

    // Step 1: Fetch latest pricing data from Registry
    TRY:
        registry_pricing = FetchRegistryPricing()
    CATCH exception:
        LOG_ERROR("Failed to fetch pricing from Registry", exception)
        SEND_ALERT("Registry sync failed", exception)
        RETURN
    END TRY

    // Step 2: Process each provider's pricing
    updates_made = 0
    FOR EACH provider_pricing IN registry_pricing:
        FOR EACH model_pricing IN provider_pricing.models:
            // Check if pricing has changed
            current_pricing = GetCurrentPricing(
                provider: provider_pricing.provider_id,
                model: model_pricing.model_id
            )

            IF HasPricingChanged(current_pricing, model_pricing):
                // Step 3: Create new pricing version
                new_pricing = {
                    provider: provider_pricing.provider_id,
                    model_id: model_pricing.model_id,
                    effective_date: model_pricing.effective_date,
                    input_price_per_1m: model_pricing.input_cost_per_1m_tokens,
                    output_price_per_1m: model_pricing.output_cost_per_1m_tokens,
                    currency: model_pricing.currency,
                    region: model_pricing.region,
                    tier: model_pricing.pricing_tier,
                    version: GENERATE_VERSION_ID(),
                    source: "registry",
                    last_updated: CURRENT_TIMESTAMP()
                }

                // Step 4: Store new pricing
                StorePricingVersion(new_pricing)

                // Step 5: Update active pricing pointer
                UpdateActivePricing(
                    provider: provider_pricing.provider_id,
                    model: model_pricing.model_id,
                    pricing_version: new_pricing.version,
                    effective_date: new_pricing.effective_date
                )

                updates_made++

                // Step 6: Log pricing change
                LOG_INFO("Pricing updated", {
                    provider: provider_pricing.provider_id,
                    model: model_pricing.model_id,
                    old_input_price: current_pricing.input_price_per_1m,
                    new_input_price: new_pricing.input_price_per_1m,
                    change_percentage: CalculatePriceChange(current_pricing, new_pricing)
                })

                // Step 7: Emit pricing change event
                EMIT_EVENT("pricing_updated", {
                    provider: provider_pricing.provider_id,
                    model: model_pricing.model_id,
                    effective_date: new_pricing.effective_date,
                    pricing_version: new_pricing.version
                })
            END IF
        END FOR
    END FOR

    LOG_INFO("Pricing sync completed", {updates_made: updates_made})
END FUNCTION


FUNCTION HasPricingChanged(current_pricing, new_pricing):
    IF current_pricing IS NULL:
        RETURN true  // New pricing entry
    END IF

    // Check if prices have changed
    IF current_pricing.input_price_per_1m != new_pricing.input_cost_per_1m_tokens:
        RETURN true
    END IF

    IF current_pricing.output_price_per_1m != new_pricing.output_cost_per_1m_tokens:
        RETURN true
    END IF

    // Check if other attributes changed
    IF current_pricing.tier != new_pricing.pricing_tier:
        RETURN true
    END IF

    RETURN false
END FUNCTION


// Handle manual pricing override (when Registry data is incorrect/delayed)
FUNCTION OverridePricing(override_request):
    // Validate authorization
    IF NOT HasPricingAdminRole(override_request.user):
        RETURN {status: "error", message: "Unauthorized"}
    END IF

    // Create override pricing entry
    override_pricing = {
        provider: override_request.provider,
        model_id: override_request.model,
        effective_date: override_request.effective_date,
        input_price_per_1m: override_request.input_price,
        output_price_per_1m: override_request.output_price,
        currency: override_request.currency,
        region: override_request.region,
        tier: override_request.tier,
        version: GENERATE_VERSION_ID(),
        source: "manual_override",
        override_reason: override_request.reason,
        overridden_by: override_request.user,
        last_updated: CURRENT_TIMESTAMP()
    }

    // Store override
    StorePricingVersion(override_pricing)
    UpdateActivePricing(
        provider: override_request.provider,
        model: override_request.model,
        pricing_version: override_pricing.version,
        effective_date: override_pricing.effective_date
    )

    // Log override
    LOG_AUDIT("pricing_manual_override", {
        user: override_request.user,
        provider: override_request.provider,
        model: override_request.model,
        reason: override_request.reason
    })

    RETURN {status: "success", pricing_version: override_pricing.version}
END FUNCTION
```

---

## 3. FORECASTING LOGIC

### 3.1 Time-Series Prediction Model

#### Algorithm: ForecastCosts
```
FUNCTION ForecastCosts(forecast_request):
    // Predict future costs based on historical data

    // Step 1: Gather historical data
    historical_data = GetHistoricalCostData(
        entity_type: forecast_request.entity_type,  // project, cost_center, model, etc.
        entity_id: forecast_request.entity_id,
        lookback_period: forecast_request.lookback_period,
        granularity: forecast_request.granularity  // hourly, daily, weekly, monthly
    )

    IF LENGTH(historical_data) < MINIMUM_DATA_POINTS:
        RETURN {
            status: "insufficient_data",
            message: "Need at least " + MINIMUM_DATA_POINTS + " data points for forecasting"
        }
    END IF

    // Step 2: Prepare time series
    time_series = PrepareTimeSeries(historical_data, forecast_request.granularity)

    // Step 3: Detect and handle anomalies
    cleaned_series = DetectAndCleanAnomalies(time_series)

    // Step 4: Decompose time series (trend, seasonality, residual)
    decomposition = DecomposeTimeSeries(cleaned_series)

    // Step 5: Select forecasting method based on data characteristics
    forecast_method = SelectForecastingMethod(decomposition)

    // Step 6: Generate forecast
    forecast = GenerateForecast(
        cleaned_series,
        decomposition,
        forecast_method,
        forecast_request.forecast_horizon
    )

    // Step 7: Calculate confidence intervals
    confidence_intervals = CalculateConfidenceIntervals(
        forecast,
        cleaned_series,
        confidence_level: 0.95
    )

    // Step 8: Package results
    forecast_result = {
        entity_type: forecast_request.entity_type,
        entity_id: forecast_request.entity_id,
        forecast_method: forecast_method,
        forecast_horizon: forecast_request.forecast_horizon,
        granularity: forecast_request.granularity,
        historical_periods: LENGTH(cleaned_series),
        forecast_data: forecast,
        confidence_intervals: confidence_intervals,
        trend: decomposition.trend,
        seasonality_detected: decomposition.has_seasonality,
        forecast_accuracy_estimate: EstimateForecastAccuracy(cleaned_series, forecast_method),
        generated_at: CURRENT_TIMESTAMP()
    }

    // Step 9: Store forecast
    StoreForecast(forecast_result)

    RETURN forecast_result
END FUNCTION


FUNCTION PrepareTimeSeries(historical_data, granularity):
    // Group data by time period
    time_series = []

    // Create continuous time periods
    start_date = MIN(historical_data.timestamp)
    end_date = MAX(historical_data.timestamp)

    current_period = start_date
    WHILE current_period <= end_date:
        period_end = AdvanceTime(current_period, granularity)

        // Aggregate costs for this period
        period_costs = FILTER(historical_data, WHERE:
            timestamp >= current_period AND timestamp < period_end
        )

        total_cost = SUM(period_costs.total_cost_usd)
        total_tokens = SUM(period_costs.tokens_total)
        request_count = COUNT(period_costs)

        time_series.APPEND({
            period_start: current_period,
            period_end: period_end,
            total_cost: total_cost,
            total_tokens: total_tokens,
            request_count: request_count,
            avg_cost_per_request: IF request_count > 0 THEN total_cost / request_count ELSE 0
        })

        current_period = period_end
    END WHILE

    RETURN time_series
END FUNCTION


FUNCTION DecomposeTimeSeries(time_series):
    // Extract trend, seasonality, and residual components

    costs = EXTRACT(time_series, field: "total_cost")

    // Step 1: Calculate trend using moving average
    window_size = CalculateOptimalWindowSize(LENGTH(costs))
    trend = CalculateMovingAverage(costs, window_size)

    // Step 2: Remove trend to find seasonality + residual
    detrended = costs - trend

    // Step 3: Detect seasonality
    seasonality_result = DetectSeasonality(detrended)

    IF seasonality_result.has_seasonality:
        seasonal_component = ExtractSeasonalComponent(detrended, seasonality_result.period)
        residual = detrended - seasonal_component
    ELSE:
        seasonal_component = ARRAY_FILLED_WITH_ZEROS(LENGTH(costs))
        residual = detrended
    END IF

    RETURN {
        original: costs,
        trend: trend,
        seasonal: seasonal_component,
        residual: residual,
        has_seasonality: seasonality_result.has_seasonality,
        seasonal_period: seasonality_result.period
    }
END FUNCTION


FUNCTION SelectForecastingMethod(decomposition):
    // Choose appropriate forecasting method based on data characteristics

    // Calculate variation in components
    trend_strength = VARIANCE(decomposition.trend) / VARIANCE(decomposition.original)
    seasonal_strength = IF decomposition.has_seasonality THEN
        VARIANCE(decomposition.seasonal) / VARIANCE(decomposition.original)
    ELSE 0

    residual_strength = VARIANCE(decomposition.residual) / VARIANCE(decomposition.original)

    // Decision logic
    IF trend_strength > 0.6 AND seasonal_strength < 0.2:
        // Strong trend, weak seasonality → Linear regression
        RETURN "linear_trend"

    ELSE IF trend_strength > 0.4 AND seasonal_strength > 0.3:
        // Both trend and seasonality → ARIMA or Holt-Winters
        RETURN "holt_winters"

    ELSE IF seasonal_strength > 0.5:
        // Strong seasonality → Seasonal decomposition
        RETURN "seasonal_decomposition"

    ELSE IF residual_strength > 0.7:
        // High residual (noisy data) → Simple exponential smoothing
        RETURN "exponential_smoothing"

    ELSE:
        // Default to moving average for stable data
        RETURN "moving_average"
    END IF
END FUNCTION


FUNCTION GenerateForecast(time_series, decomposition, method, horizon):
    costs = EXTRACT(time_series, field: "total_cost")

    SWITCH method:
        CASE "linear_trend":
            RETURN ForecastLinearTrend(costs, horizon)

        CASE "holt_winters":
            RETURN ForecastHoltWinters(costs, decomposition, horizon)

        CASE "seasonal_decomposition":
            RETURN ForecastSeasonalDecomposition(decomposition, horizon)

        CASE "exponential_smoothing":
            RETURN ForecastExponentialSmoothing(costs, horizon)

        CASE "moving_average":
            RETURN ForecastMovingAverage(costs, horizon)

        DEFAULT:
            RETURN ForecastMovingAverage(costs, horizon)
    END SWITCH
END FUNCTION


FUNCTION ForecastLinearTrend(costs, horizon):
    // Simple linear regression forecast

    n = LENGTH(costs)
    x = RANGE(1, n)  // Time indices

    // Calculate linear regression coefficients
    x_mean = MEAN(x)
    y_mean = MEAN(costs)

    numerator = SUM((x - x_mean) * (costs - y_mean))
    denominator = SUM((x - x_mean)^2)

    slope = numerator / denominator
    intercept = y_mean - slope * x_mean

    // Generate forecast
    forecast = []
    FOR i FROM n+1 TO n+horizon:
        predicted_value = intercept + slope * i
        forecast.APPEND(predicted_value)
    END FOR

    RETURN forecast
END FUNCTION


FUNCTION ForecastHoltWinters(costs, decomposition, horizon):
    // Triple exponential smoothing (Holt-Winters)

    // Smoothing parameters (can be optimized)
    alpha = 0.2  // Level
    beta = 0.1   // Trend
    gamma = 0.1  // Seasonality

    seasonal_period = decomposition.seasonal_period

    // Initialize components
    level = costs[0]
    trend = (costs[seasonal_period] - costs[0]) / seasonal_period
    seasonal = decomposition.seasonal[0..seasonal_period-1]

    // Update components through historical data
    FOR t FROM 1 TO LENGTH(costs)-1:
        previous_level = level

        level = alpha * (costs[t] - seasonal[t MOD seasonal_period]) +
                (1 - alpha) * (previous_level + trend)

        trend = beta * (level - previous_level) + (1 - beta) * trend

        seasonal[t MOD seasonal_period] =
            gamma * (costs[t] - level) +
            (1 - gamma) * seasonal[t MOD seasonal_period]
    END FOR

    // Generate forecast
    forecast = []
    FOR h FROM 1 TO horizon:
        predicted = level + h * trend + seasonal[(LENGTH(costs) + h - 1) MOD seasonal_period]
        forecast.APPEND(MAX(0, predicted))  // Costs can't be negative
    END FOR

    RETURN forecast
END FUNCTION


FUNCTION ForecastMovingAverage(costs, horizon):
    // Simple moving average forecast

    window_size = MIN(30, LENGTH(costs))  // Use last 30 periods or all available

    recent_costs = costs[LENGTH(costs) - window_size .. LENGTH(costs) - 1]
    average = MEAN(recent_costs)

    // Flat forecast at the average
    forecast = ARRAY_FILLED_WITH(average, horizon)

    RETURN forecast
END FUNCTION


FUNCTION CalculateConfidenceIntervals(forecast, historical_data, confidence_level):
    // Calculate prediction intervals

    // Estimate forecast error from historical residuals
    historical_costs = EXTRACT(historical_data, field: "total_cost")

    // Use last N periods to estimate error
    validation_window = MIN(30, LENGTH(historical_costs) / 3)
    validation_start = LENGTH(historical_costs) - validation_window

    errors = []
    FOR i FROM validation_start TO LENGTH(historical_costs) - 1:
        // Make one-step ahead prediction
        prediction = ForecastOneStep(historical_costs[0..i-1])
        error = ABS(historical_costs[i] - prediction)
        errors.APPEND(error)
    END FOR

    // Calculate standard error
    mean_absolute_error = MEAN(errors)
    std_error = STANDARD_DEVIATION(errors)

    // Calculate z-score for confidence level
    z_score = GET_Z_SCORE(confidence_level)  // e.g., 1.96 for 95%

    // Generate confidence intervals
    intervals = []
    FOR i FROM 0 TO LENGTH(forecast) - 1:
        // Error grows with forecast horizon
        horizon_factor = SQRT(i + 1)
        margin = z_score * std_error * horizon_factor

        intervals.APPEND({
            point_forecast: forecast[i],
            lower_bound: MAX(0, forecast[i] - margin),
            upper_bound: forecast[i] + margin,
            confidence_level: confidence_level
        })
    END FOR

    RETURN intervals
END FUNCTION
```

---

### 3.2 Trend Detection Algorithm

#### Algorithm: DetectTrends
```
FUNCTION DetectTrends(entity_type, entity_id, analysis_window):
    // Detect upward/downward trends in cost data

    // Step 1: Get time series data
    time_series = GetCostTimeSeries(entity_type, entity_id, analysis_window)

    IF LENGTH(time_series) < MINIMUM_TREND_POINTS:
        RETURN {trend: "insufficient_data"}
    END IF

    costs = EXTRACT(time_series, field: "total_cost")

    // Step 2: Calculate trend using linear regression
    trend_line = FitLinearTrend(costs)

    // Step 3: Statistical significance test
    significance = TestTrendSignificance(costs, trend_line)

    // Step 4: Classify trend
    IF NOT significance.is_significant:
        trend_type = "stable"
        trend_strength = 0
    ELSE:
        slope_normalized = trend_line.slope / MEAN(costs)

        IF slope_normalized > THRESHOLD_STRONG_UPWARD:
            trend_type = "strong_upward"
            trend_strength = slope_normalized
        ELSE IF slope_normalized > THRESHOLD_MODERATE_UPWARD:
            trend_type = "moderate_upward"
            trend_strength = slope_normalized
        ELSE IF slope_normalized > THRESHOLD_SLIGHT_UPWARD:
            trend_type = "slight_upward"
            trend_strength = slope_normalized
        ELSE IF slope_normalized < -THRESHOLD_STRONG_UPWARD:
            trend_type = "strong_downward"
            trend_strength = ABS(slope_normalized)
        ELSE IF slope_normalized < -THRESHOLD_MODERATE_UPWARD:
            trend_type = "moderate_downward"
            trend_strength = ABS(slope_normalized)
        ELSE IF slope_normalized < -THRESHOLD_SLIGHT_UPWARD:
            trend_type = "slight_downward"
            trend_strength = ABS(slope_normalized)
        ELSE:
            trend_type = "stable"
            trend_strength = ABS(slope_normalized)
        END IF
    END IF

    // Step 5: Detect change points (abrupt changes in trend)
    change_points = DetectChangePoints(costs)

    // Step 6: Project future costs based on trend
    IF trend_type != "stable":
        projected_30d = ProjectFutureCost(trend_line, 30)
        projected_90d = ProjectFutureCost(trend_line, 90)
    ELSE:
        current_avg = MEAN(costs[LENGTH(costs) - 7 .. LENGTH(costs) - 1])
        projected_30d = current_avg
        projected_90d = current_avg
    END IF

    // Step 7: Package trend analysis
    trend_analysis = {
        entity_type: entity_type,
        entity_id: entity_id,
        analysis_window: analysis_window,
        trend_type: trend_type,
        trend_strength: trend_strength,
        trend_slope: trend_line.slope,
        trend_direction: IF trend_line.slope > 0 THEN "increasing" ELSE "decreasing",
        statistical_significance: significance.p_value,
        is_significant: significance.is_significant,
        r_squared: trend_line.r_squared,
        change_points: change_points,
        current_avg_cost: MEAN(costs[LENGTH(costs) - 7 .. LENGTH(costs) - 1]),
        projected_cost_30d: projected_30d,
        projected_cost_90d: projected_90d,
        analyzed_at: CURRENT_TIMESTAMP()
    }

    // Step 8: Generate alerts if needed
    IF trend_type CONTAINS "strong" OR LENGTH(change_points) > 0:
        GenerateTrendAlert(trend_analysis)
    END IF

    RETURN trend_analysis
END FUNCTION


FUNCTION DetectChangePoints(time_series):
    // Detect points where trend significantly changes

    change_points = []
    window_size = MAX(7, LENGTH(time_series) / 10)

    FOR i FROM window_size TO LENGTH(time_series) - window_size:
        // Compare trend before and after this point
        before = time_series[i - window_size .. i - 1]
        after = time_series[i .. i + window_size - 1]

        trend_before = FitLinearTrend(before)
        trend_after = FitLinearTrend(after)

        // Check if slopes are significantly different
        slope_change = ABS(trend_after.slope - trend_before.slope)
        slope_threshold = STANDARD_DEVIATION(time_series) * 0.5

        IF slope_change > slope_threshold:
            change_points.APPEND({
                index: i,
                timestamp: time_series[i].timestamp,
                slope_before: trend_before.slope,
                slope_after: trend_after.slope,
                magnitude: slope_change
            })
        END IF
    END FOR

    // Merge nearby change points
    change_points = MergeNearbyChangePoints(change_points, window_size / 2)

    RETURN change_points
END FUNCTION


FUNCTION TestTrendSignificance(costs, trend_line):
    // Perform statistical test to determine if trend is significant

    n = LENGTH(costs)

    // Calculate residuals
    residuals = []
    FOR i FROM 0 TO n - 1:
        predicted = trend_line.intercept + trend_line.slope * i
        residual = costs[i] - predicted
        residuals.APPEND(residual)
    END FOR

    // Calculate standard error
    sse = SUM(residuals^2)
    mse = sse / (n - 2)
    se = SQRT(mse)

    // Calculate t-statistic for slope
    x = RANGE(0, n - 1)
    x_mean = MEAN(x)
    sxx = SUM((x - x_mean)^2)
    se_slope = se / SQRT(sxx)

    t_stat = trend_line.slope / se_slope

    // Degrees of freedom
    df = n - 2

    // Calculate p-value (two-tailed test)
    p_value = 2 * (1 - T_DISTRIBUTION_CDF(ABS(t_stat), df))

    // Significant if p < 0.05
    is_significant = p_value < 0.05

    RETURN {
        t_statistic: t_stat,
        p_value: p_value,
        is_significant: is_significant,
        degrees_of_freedom: df
    }
END FUNCTION
```

---

### 3.3 Anomaly Detection Algorithm

#### Algorithm: DetectAnomalies
```
FUNCTION DetectAnomalies(time_series_data, sensitivity):
    // Detect cost spikes, usage outliers, and abnormal patterns

    costs = EXTRACT(time_series_data, field: "total_cost")
    tokens = EXTRACT(time_series_data, field: "total_tokens")
    requests = EXTRACT(time_series_data, field: "request_count")

    anomalies = []

    // Method 1: Statistical outliers (Z-score)
    cost_anomalies = DetectStatisticalOutliers(costs, "cost", sensitivity)
    anomalies.EXTEND(cost_anomalies)

    // Method 2: Inter-quartile range (IQR) method
    iqr_anomalies = DetectIQRAnomalies(costs, "cost")
    anomalies.EXTEND(iqr_anomalies)

    // Method 3: Moving average deviation
    ma_anomalies = DetectMovingAverageAnomalies(costs, "cost")
    anomalies.EXTEND(ma_anomalies)

    // Method 4: Rate of change anomalies
    rate_anomalies = DetectRateChangeAnomalies(costs, "cost")
    anomalies.EXTEND(rate_anomalies)

    // Method 5: Pattern anomalies (unexpected patterns)
    pattern_anomalies = DetectPatternAnomalies(time_series_data)
    anomalies.EXTEND(pattern_anomalies)

    // Deduplicate and rank anomalies
    unique_anomalies = DeduplicateAnomalies(anomalies)
    ranked_anomalies = RankAnomaliesBySeverity(unique_anomalies)

    // Package results
    anomaly_report = {
        time_period: {
            start: time_series_data[0].period_start,
            end: time_series_data[LENGTH(time_series_data) - 1].period_end
        },
        total_data_points: LENGTH(time_series_data),
        anomalies_detected: LENGTH(ranked_anomalies),
        anomalies: ranked_anomalies,
        detection_methods: ["z_score", "iqr", "moving_average", "rate_change", "pattern"],
        sensitivity: sensitivity,
        generated_at: CURRENT_TIMESTAMP()
    }

    // Generate alerts for high-severity anomalies
    high_severity = FILTER(ranked_anomalies, WHERE: severity >= THRESHOLD_HIGH_SEVERITY)
    FOR EACH anomaly IN high_severity:
        GenerateAnomalyAlert(anomaly)
    END FOR

    RETURN anomaly_report
END FUNCTION


FUNCTION DetectStatisticalOutliers(data, metric_name, sensitivity):
    // Z-score method for outlier detection

    mean_value = MEAN(data)
    std_dev = STANDARD_DEVIATION(data)

    // Sensitivity determines z-score threshold
    // Low sensitivity: z > 3 (99.7%)
    // Medium sensitivity: z > 2.5 (98.8%)
    // High sensitivity: z > 2 (95.4%)
    threshold = SWITCH sensitivity:
        CASE "low": 3.0
        CASE "medium": 2.5
        CASE "high": 2.0
        DEFAULT: 2.5
    END SWITCH

    outliers = []
    FOR i FROM 0 TO LENGTH(data) - 1:
        z_score = ABS(data[i] - mean_value) / std_dev

        IF z_score > threshold:
            outliers.APPEND({
                index: i,
                value: data[i],
                metric: metric_name,
                z_score: z_score,
                deviation_from_mean: data[i] - mean_value,
                deviation_percentage: ((data[i] - mean_value) / mean_value) * 100,
                detection_method: "z_score",
                severity: CalculateSeverity(z_score, threshold)
            })
        END IF
    END FOR

    RETURN outliers
END FUNCTION


FUNCTION DetectIQRAnomalies(data, metric_name):
    // Inter-Quartile Range method

    sorted_data = SORT(data)
    n = LENGTH(sorted_data)

    q1_index = FLOOR(n * 0.25)
    q3_index = FLOOR(n * 0.75)

    q1 = sorted_data[q1_index]
    q3 = sorted_data[q3_index]
    iqr = q3 - q1

    lower_bound = q1 - 1.5 * iqr
    upper_bound = q3 + 1.5 * iqr

    anomalies = []
    FOR i FROM 0 TO LENGTH(data) - 1:
        IF data[i] < lower_bound OR data[i] > upper_bound:
            distance_from_bound = IF data[i] < lower_bound THEN
                lower_bound - data[i]
            ELSE
                data[i] - upper_bound

            anomalies.APPEND({
                index: i,
                value: data[i],
                metric: metric_name,
                lower_bound: lower_bound,
                upper_bound: upper_bound,
                distance_from_bound: distance_from_bound,
                detection_method: "iqr",
                severity: CalculateSeverityIQR(distance_from_bound, iqr)
            })
        END IF
    END FOR

    RETURN anomalies
END FUNCTION


FUNCTION DetectMovingAverageAnomalies(data, metric_name):
    // Detect deviations from moving average

    window_size = MIN(7, LENGTH(data) / 4)
    anomalies = []

    FOR i FROM window_size TO LENGTH(data) - 1:
        window = data[i - window_size .. i - 1]
        moving_avg = MEAN(window)
        moving_std = STANDARD_DEVIATION(window)

        deviation = ABS(data[i] - moving_avg)
        threshold = 2 * moving_std

        IF deviation > threshold:
            anomalies.APPEND({
                index: i,
                value: data[i],
                metric: metric_name,
                moving_average: moving_avg,
                deviation: deviation,
                threshold: threshold,
                detection_method: "moving_average",
                severity: CalculateSeverity(deviation / moving_std, 2.0)
            })
        END IF
    END FOR

    RETURN anomalies
END FUNCTION


FUNCTION DetectRateChangeAnomalies(data, metric_name):
    // Detect sudden rate of change

    anomalies = []

    FOR i FROM 1 TO LENGTH(data) - 1:
        IF data[i-1] == 0:
            CONTINUE
        END IF

        rate_of_change = (data[i] - data[i-1]) / data[i-1]

        // Flag if rate of change exceeds thresholds
        IF ABS(rate_of_change) > THRESHOLD_RATE_CHANGE:
            anomalies.APPEND({
                index: i,
                value: data[i],
                previous_value: data[i-1],
                metric: metric_name,
                rate_of_change: rate_of_change,
                percentage_change: rate_of_change * 100,
                detection_method: "rate_change",
                severity: CalculateSeverityRateChange(ABS(rate_of_change))
            })
        END IF
    END FOR

    RETURN anomalies
END FUNCTION


FUNCTION DetectPatternAnomalies(time_series_data):
    // Detect unusual patterns

    anomalies = []

    // Pattern 1: Sustained spike (high cost for multiple periods)
    sustained_spike = DetectSustainedSpike(time_series_data)
    IF sustained_spike:
        anomalies.APPEND(sustained_spike)
    END IF

    // Pattern 2: Unusual time-of-day pattern
    time_pattern = DetectUnusualTimePattern(time_series_data)
    IF time_pattern:
        anomalies.APPEND(time_pattern)
    END IF

    // Pattern 3: Cost without corresponding usage
    cost_usage_mismatch = DetectCostUsageMismatch(time_series_data)
    anomalies.EXTEND(cost_usage_mismatch)

    RETURN anomalies
END FUNCTION


FUNCTION DetectSustainedSpike(time_series_data):
    // Detect periods of sustained high cost

    costs = EXTRACT(time_series_data, field: "total_cost")
    baseline = PERCENTILE(costs, 75)  // Use 75th percentile as baseline

    spike_threshold = baseline * 1.5
    consecutive_threshold = 3

    consecutive_high = 0
    spike_start_index = -1

    FOR i FROM 0 TO LENGTH(costs) - 1:
        IF costs[i] > spike_threshold:
            IF consecutive_high == 0:
                spike_start_index = i
            END IF
            consecutive_high++
        ELSE:
            IF consecutive_high >= consecutive_threshold:
                // Found sustained spike
                RETURN {
                    type: "sustained_spike",
                    start_index: spike_start_index,
                    end_index: i - 1,
                    duration: consecutive_high,
                    avg_cost_during_spike: MEAN(costs[spike_start_index .. i-1]),
                    baseline_cost: baseline,
                    detection_method: "pattern",
                    severity: "high"
                }
            END IF
            consecutive_high = 0
        END IF
    END FOR

    RETURN NULL
END FUNCTION


FUNCTION DetectCostUsageMismatch(time_series_data):
    // Detect cases where cost doesn't match usage

    anomalies = []

    FOR i FROM 0 TO LENGTH(time_series_data) - 1:
        cost = time_series_data[i].total_cost
        tokens = time_series_data[i].total_tokens

        IF tokens == 0 AND cost > THRESHOLD_MIN_COST:
            // Cost without usage
            anomalies.APPEND({
                type: "cost_without_usage",
                index: i,
                cost: cost,
                tokens: tokens,
                detection_method: "pattern",
                severity: "medium"
            })
        ELSE IF tokens > 0:
            // Calculate expected cost per token
            historical_avg_rate = CalculateHistoricalCostPerToken(time_series_data[0..i-1])
            expected_cost = tokens * historical_avg_rate
            actual_cost = cost

            deviation = ABS(actual_cost - expected_cost) / expected_cost

            IF deviation > THRESHOLD_COST_USAGE_DEVIATION:
                anomalies.APPEND({
                    type: "cost_usage_mismatch",
                    index: i,
                    actual_cost: actual_cost,
                    expected_cost: expected_cost,
                    tokens: tokens,
                    deviation_percentage: deviation * 100,
                    detection_method: "pattern",
                    severity: IF deviation > 0.5 THEN "high" ELSE "medium"
                })
            END IF
        END IF
    END FOR

    RETURN anomalies
END FUNCTION


FUNCTION CalculateSeverity(z_score, threshold):
    // Map z-score to severity level

    IF z_score > threshold * 2:
        RETURN "critical"
    ELSE IF z_score > threshold * 1.5:
        RETURN "high"
    ELSE IF z_score > threshold * 1.2:
        RETURN "medium"
    ELSE:
        RETURN "low"
    END IF
END FUNCTION
```

---

### 3.4 Budget Projection Calculations

#### Algorithm: ProjectBudget
```
FUNCTION ProjectBudget(cost_center, current_period, projection_window):
    // Project budget usage for remaining period

    // Step 1: Get budget configuration
    budget = GetBudgetConfig(cost_center, current_period)

    IF budget IS NULL:
        RETURN {error: "No budget configured"}
    END IF

    // Step 2: Calculate time progress
    period_info = CalculatePeriodProgress(current_period, budget.period_type)

    // Step 3: Get actual spending to date
    actual_spend = GetPeriodSpending(cost_center, current_period)

    // Step 4: Get historical spending patterns
    historical_patterns = GetHistoricalSpendingPatterns(
        cost_center,
        budget.period_type,
        lookback_periods: 6
    )

    // Step 5: Project remaining spending
    remaining_days = period_info.days_remaining
    days_elapsed = period_info.days_elapsed

    // Method 1: Linear extrapolation
    daily_run_rate = actual_spend / days_elapsed
    linear_projection = actual_spend + (daily_run_rate * remaining_days)

    // Method 2: Historical pattern matching
    similar_period = FindMostSimilarHistoricalPeriod(
        current_spend: actual_spend,
        days_elapsed: days_elapsed,
        historical: historical_patterns
    )

    IF similar_period:
        pattern_projection = ProjectFromPattern(
            actual_spend,
            days_elapsed,
            remaining_days,
            similar_period
        )
    ELSE:
        pattern_projection = linear_projection
    END IF

    // Method 3: Trend-adjusted projection
    trend_analysis = DetectTrends(
        entity_type: "cost_center",
        entity_id: cost_center,
        analysis_window: period_info.start_date .. CURRENT_DATE()
    )

    trend_projection = ProjectWithTrend(
        actual_spend,
        daily_run_rate,
        remaining_days,
        trend_analysis.trend_slope
    )

    // Step 6: Combine projections with weights
    final_projection = (
        linear_projection * WEIGHT_LINEAR +
        pattern_projection * WEIGHT_PATTERN +
        trend_projection * WEIGHT_TREND
    )

    // Step 7: Calculate budget variance
    projected_variance = final_projection - budget.limit
    variance_percentage = (projected_variance / budget.limit) * 100

    // Step 8: Calculate burn rate
    burn_rate = actual_spend / days_elapsed
    budget_exhaustion_date = CalculateExhaustionDate(
        current_spend: actual_spend,
        burn_rate: burn_rate,
        budget_limit: budget.limit,
        current_date: CURRENT_DATE()
    )

    // Step 9: Generate projection report
    projection_report = {
        cost_center: cost_center,
        period: current_period,
        budget_limit: budget.limit,
        actual_spend_to_date: actual_spend,
        days_elapsed: days_elapsed,
        days_remaining: remaining_days,
        percent_time_elapsed: period_info.percent_elapsed,
        percent_budget_used: (actual_spend / budget.limit) * 100,

        projections: {
            linear: linear_projection,
            pattern_based: pattern_projection,
            trend_adjusted: trend_projection,
            final: final_projection
        },

        projected_variance: projected_variance,
        projected_variance_percentage: variance_percentage,

        burn_rate: {
            daily: burn_rate,
            weekly: burn_rate * 7,
            monthly: burn_rate * 30
        },

        budget_exhaustion_date: budget_exhaustion_date,
        days_until_exhaustion: IF budget_exhaustion_date THEN
            DAYS_BETWEEN(CURRENT_DATE(), budget_exhaustion_date)
        ELSE NULL,

        is_on_track: variance_percentage <= THRESHOLD_ON_TRACK_VARIANCE,
        risk_level: CalculateRiskLevel(variance_percentage, days_remaining),

        recommendations: GenerateBudgetRecommendations(
            actual_spend,
            final_projection,
            budget.limit,
            period_info
        ),

        generated_at: CURRENT_TIMESTAMP()
    }

    // Step 10: Send alerts if projected to exceed
    IF variance_percentage > 0:
        SEND_ALERT("budget_projection_exceeded", projection_report)
    END IF

    RETURN projection_report
END FUNCTION


FUNCTION CalculatePeriodProgress(current_period, period_type):
    start_date = current_period.start
    end_date = current_period.end
    current_date = CURRENT_DATE()

    total_days = DAYS_BETWEEN(start_date, end_date)
    days_elapsed = DAYS_BETWEEN(start_date, current_date)
    days_remaining = DAYS_BETWEEN(current_date, end_date)

    percent_elapsed = (days_elapsed / total_days) * 100

    RETURN {
        start_date: start_date,
        end_date: end_date,
        current_date: current_date,
        total_days: total_days,
        days_elapsed: days_elapsed,
        days_remaining: days_remaining,
        percent_elapsed: percent_elapsed,
        period_type: period_type
    }
END FUNCTION


FUNCTION ProjectFromPattern(actual_spend, days_elapsed, days_remaining, similar_period):
    // Use similar historical period to project

    // Find where we are in the similar period
    similar_days_elapsed = similar_period.days_elapsed
    similar_total_spend = similar_period.total_spend

    // Calculate what percentage was spent in remaining portion
    similar_spend_at_current_point = similar_period.spend_at_day[days_elapsed]
    similar_remaining_spend = similar_total_spend - similar_spend_at_current_point

    // Calculate the ratio
    ratio = similar_remaining_spend / similar_spend_at_current_point

    // Apply ratio to current spend
    projected_remaining = actual_spend * ratio
    total_projection = actual_spend + projected_remaining

    RETURN total_projection
END FUNCTION


FUNCTION ProjectWithTrend(actual_spend, daily_rate, days_remaining, trend_slope):
    // Adjust projection based on detected trend

    // trend_slope is change per day
    projection = actual_spend

    FOR day FROM 1 TO days_remaining:
        // Apply trend adjustment to daily rate
        adjusted_rate = daily_rate + (trend_slope * day)
        projection += MAX(0, adjusted_rate)
    END FOR

    RETURN projection
END FUNCTION


FUNCTION CalculateExhaustionDate(current_spend, burn_rate, budget_limit, current_date):
    remaining_budget = budget_limit - current_spend

    IF remaining_budget <= 0:
        // Already exceeded
        RETURN current_date
    END IF

    IF burn_rate <= 0:
        // Not burning budget
        RETURN NULL
    END IF

    days_until_exhaustion = remaining_budget / burn_rate
    exhaustion_date = current_date + days_until_exhaustion

    RETURN exhaustion_date
END FUNCTION


FUNCTION CalculateRiskLevel(variance_percentage, days_remaining):
    IF variance_percentage > 20:
        RETURN "critical"
    ELSE IF variance_percentage > 10:
        RETURN "high"
    ELSE IF variance_percentage > 0:
        RETURN "medium"
    ELSE IF variance_percentage > -10:
        RETURN "low"
    ELSE:
        RETURN "minimal"
    END IF
END FUNCTION


FUNCTION GenerateBudgetRecommendations(actual_spend, projected_spend, budget_limit, period_info):
    recommendations = []

    variance = projected_spend - budget_limit

    IF variance > 0:
        // Over budget
        daily_reduction_needed = variance / period_info.days_remaining
        percentage_reduction = (daily_reduction_needed / (actual_spend / period_info.days_elapsed)) * 100

        recommendations.APPEND({
            type: "reduce_spending",
            message: "Reduce daily spending by $" + ROUND(daily_reduction_needed, 2) +
                     " (" + ROUND(percentage_reduction, 1) + "%) to stay within budget",
            priority: "high"
        })

        // Suggest specific actions
        recommendations.APPEND({
            type: "optimize_usage",
            message: "Review and optimize high-cost models or switch to more cost-efficient alternatives",
            priority: "high"
        })

        recommendations.APPEND({
            type: "implement_quotas",
            message: "Consider implementing usage quotas or rate limiting for projects",
            priority: "medium"
        })
    ELSE IF variance > -budget_limit * 0.1:
        // Close to budget
        recommendations.APPEND({
            type: "monitor_closely",
            message: "Continue monitoring spending closely. On track but with little margin",
            priority: "medium"
        })
    ELSE:
        // Under budget
        available_budget = ABS(variance)
        recommendations.APPEND({
            type: "budget_available",
            message: "Projected to have $" + ROUND(available_budget, 2) + " remaining budget available",
            priority: "low"
        })

        recommendations.APPEND({
            type: "consider_expansion",
            message: "Consider expanding usage for high-value use cases or experimentation",
            priority: "low"
        })
    END IF

    RETURN recommendations
END FUNCTION
```

---

## 4. QUERY & REPORTING

### 4.1 Cost Aggregation

#### Algorithm: AggregateCosts
```
FUNCTION AggregateCosts(aggregation_request):
    // Aggregate costs by various dimensions

    // Step 1: Parse aggregation parameters
    dimensions = aggregation_request.group_by  // e.g., ["project", "model", "date"]
    time_range = aggregation_request.time_range
    filters = aggregation_request.filters
    metrics = aggregation_request.metrics  // e.g., ["total_cost", "avg_cost", "request_count"]

    // Step 2: Build query
    query = BuildAggregationQuery(dimensions, time_range, filters, metrics)

    // Step 3: Execute query
    raw_results = ExecuteQuery(query)

    // Step 4: Post-process results
    aggregated_data = ProcessAggregationResults(raw_results, dimensions, metrics)

    // Step 5: Calculate additional derived metrics
    aggregated_data = CalculateDerivedMetrics(aggregated_data)

    // Step 6: Sort and limit results
    IF aggregation_request.sort_by:
        aggregated_data = SORT(aggregated_data, BY: aggregation_request.sort_by)
    END IF

    IF aggregation_request.limit:
        aggregated_data = aggregated_data[0 .. aggregation_request.limit - 1]
    END IF

    // Step 7: Package response
    response = {
        aggregation_key: dimensions,
        time_range: time_range,
        filters: filters,
        total_records: LENGTH(aggregated_data),
        data: aggregated_data,
        summary: CalculateOverallSummary(aggregated_data, metrics),
        generated_at: CURRENT_TIMESTAMP()
    }

    RETURN response
END FUNCTION


FUNCTION BuildAggregationQuery(dimensions, time_range, filters, metrics):
    // Build SQL-like query structure

    query = {
        select: [],
        from: "cost_records",
        where: [],
        group_by: dimensions,
        order_by: NULL,
        limit: NULL
    }

    // Add dimension fields to SELECT
    FOR EACH dimension IN dimensions:
        query.select.APPEND(dimension)
    END FOR

    // Add metric aggregations to SELECT
    FOR EACH metric IN metrics:
        SWITCH metric:
            CASE "total_cost":
                query.select.APPEND("SUM(total_cost_usd) AS total_cost")
            CASE "avg_cost":
                query.select.APPEND("AVG(total_cost_usd) AS avg_cost")
            CASE "min_cost":
                query.select.APPEND("MIN(total_cost_usd) AS min_cost")
            CASE "max_cost":
                query.select.APPEND("MAX(total_cost_usd) AS max_cost")
            CASE "total_tokens":
                query.select.APPEND("SUM(tokens_total) AS total_tokens")
            CASE "avg_tokens":
                query.select.APPEND("AVG(tokens_total) AS avg_tokens")
            CASE "request_count":
                query.select.APPEND("COUNT(*) AS request_count")
            CASE "unique_models":
                query.select.APPEND("COUNT(DISTINCT model_id) AS unique_models")
        END SWITCH
    END FOR

    // Add time range filter
    query.where.APPEND("timestamp >= " + time_range.start)
    query.where.APPEND("timestamp <= " + time_range.end)

    // Add custom filters
    FOR EACH filter IN filters:
        query.where.APPEND(BuildFilterCondition(filter))
    END FOR

    RETURN query
END FUNCTION


FUNCTION ProcessAggregationResults(raw_results, dimensions, metrics):
    // Transform raw query results into structured format

    processed = []

    FOR EACH row IN raw_results:
        record = {}

        // Extract dimension values
        FOR EACH dimension IN dimensions:
            record[dimension] = row[dimension]
        END FOR

        // Extract metric values
        FOR EACH metric IN metrics:
            record[metric] = row[metric]
        END FOR

        processed.APPEND(record)
    END FOR

    RETURN processed
END FUNCTION


FUNCTION CalculateDerivedMetrics(aggregated_data):
    // Add calculated metrics

    FOR EACH record IN aggregated_data:
        // Cost per token
        IF record.total_cost AND record.total_tokens:
            record.cost_per_1k_tokens = (record.total_cost / record.total_tokens) * 1000
            record.cost_per_1m_tokens = (record.total_cost / record.total_tokens) * 1_000_000
        END IF

        // Cost per request
        IF record.total_cost AND record.request_count:
            record.cost_per_request = record.total_cost / record.request_count
        END IF

        // Average tokens per request
        IF record.total_tokens AND record.request_count:
            record.avg_tokens_per_request = record.total_tokens / record.request_count
        END IF
    END FOR

    RETURN aggregated_data
END FUNCTION


FUNCTION CalculateOverallSummary(aggregated_data, metrics):
    // Calculate summary statistics across all groups

    summary = {}

    // Total across all groups
    IF "total_cost" IN metrics:
        summary.grand_total_cost = SUM(aggregated_data.total_cost)
    END IF

    IF "total_tokens" IN metrics:
        summary.grand_total_tokens = SUM(aggregated_data.total_tokens)
    END IF

    IF "request_count" IN metrics:
        summary.grand_total_requests = SUM(aggregated_data.request_count)
    END IF

    // Averages
    IF "avg_cost" IN metrics:
        summary.overall_avg_cost = MEAN(aggregated_data.avg_cost)
    END IF

    // Top contributors
    IF "total_cost" IN metrics:
        sorted_by_cost = SORT(aggregated_data, BY: total_cost, ORDER: DESC)
        summary.top_cost_contributors = sorted_by_cost[0..4]  // Top 5
    END IF

    RETURN summary
END FUNCTION
```

---

### 4.2 ROI Calculation Per Use Case

#### Algorithm: CalculateUseCaseROI
```
FUNCTION CalculateUseCaseROI(use_case_id, time_period):
    // Detailed ROI calculation for specific use case
    // (This builds on the ComputeROI function from section 1.5)

    // Step 1: Gather cost data
    cost_data = GatherUseCaseCosts(use_case_id, time_period)

    // Step 2: Gather value metrics
    value_data = GatherUseCaseValue(use_case_id, time_period)

    // Step 3: Calculate ROI components
    roi_components = {
        // Cost components
        llm_costs: cost_data.llm_usage_cost,
        infrastructure_costs: cost_data.infrastructure_cost,
        development_costs: cost_data.development_cost,
        maintenance_costs: cost_data.maintenance_cost,
        total_costs: cost_data.total_cost,

        // Value components
        revenue_generated: value_data.revenue,
        cost_savings: value_data.automation_savings + value_data.error_reduction_savings,
        efficiency_gains: value_data.time_savings_value,
        quality_improvements: value_data.quality_impact_value,
        total_value: value_data.total_value,

        // ROI metrics
        net_value: value_data.total_value - cost_data.total_cost,
        roi_percentage: ((value_data.total_value - cost_data.total_cost) / cost_data.total_cost) * 100,
        value_to_cost_ratio: value_data.total_value / cost_data.total_cost,
        payback_period_months: CalculatePaybackPeriod(cost_data, value_data),

        // Efficiency metrics
        value_per_dollar_spent: value_data.total_value / cost_data.total_cost,
        value_per_request: value_data.total_value / cost_data.total_requests,
        llm_cost_as_percentage_of_value: (cost_data.llm_usage_cost / value_data.total_value) * 100
    }

    // Step 4: Break down by time period
    time_series_roi = CalculateTimeSeriesROI(use_case_id, time_period)

    // Step 5: Comparative analysis
    comparative = CompareToSimilarUseCases(use_case_id, roi_components)

    // Step 6: Create comprehensive report
    roi_report = {
        use_case_id: use_case_id,
        use_case_name: GetUseCaseName(use_case_id),
        time_period: time_period,
        components: roi_components,
        time_series: time_series_roi,
        comparative_analysis: comparative,
        generated_at: CURRENT_TIMESTAMP()
    }

    RETURN roi_report
END FUNCTION


FUNCTION GatherUseCaseCosts(use_case_id, time_period):
    // Collect all costs associated with use case

    // Direct LLM usage costs
    llm_costs = AggregateCosts({
        group_by: ["use_case_id"],
        time_range: time_period,
        filters: [{field: "use_case_id", operator: "=", value: use_case_id}],
        metrics: ["total_cost", "total_tokens", "request_count"]
    })

    // Infrastructure costs (if tracked)
    infra_costs = GetInfrastructureCosts(use_case_id, time_period)

    // Development costs (amortized)
    dev_costs = GetDevelopmentCosts(use_case_id, time_period)

    // Maintenance costs
    maintenance_costs = GetMaintenanceCosts(use_case_id, time_period)

    total_cost = (
        llm_costs.data[0].total_cost +
        infra_costs.total +
        dev_costs.amortized_cost +
        maintenance_costs.total
    )

    RETURN {
        llm_usage_cost: llm_costs.data[0].total_cost,
        total_tokens: llm_costs.data[0].total_tokens,
        total_requests: llm_costs.data[0].request_count,
        infrastructure_cost: infra_costs.total,
        development_cost: dev_costs.amortized_cost,
        maintenance_cost: maintenance_costs.total,
        total_cost: total_cost,
        cost_breakdown: {
            llm_percentage: (llm_costs.data[0].total_cost / total_cost) * 100,
            infra_percentage: (infra_costs.total / total_cost) * 100,
            dev_percentage: (dev_costs.amortized_cost / total_cost) * 100,
            maintenance_percentage: (maintenance_costs.total / total_cost) * 100
        }
    }
END FUNCTION


FUNCTION GatherUseCaseValue(use_case_id, time_period):
    // Collect all value metrics for use case

    use_case_metrics = GetUseCaseMetrics(use_case_id, time_period)

    // Revenue generated (if applicable)
    revenue = use_case_metrics.revenue_generated OR 0

    // Automation savings
    automation_savings = CalculateAutomationSavings(use_case_metrics)

    // Error reduction savings
    error_savings = CalculateErrorReductionSavings(use_case_metrics)

    // Time savings value
    time_value = CalculateTimeSavingsValue(use_case_metrics)

    // Quality improvement value
    quality_value = CalculateQualityImprovementValue(use_case_metrics)

    total_value = revenue + automation_savings + error_savings + time_value + quality_value

    RETURN {
        revenue: revenue,
        automation_savings: automation_savings,
        error_reduction_savings: error_savings,
        time_savings_value: time_value,
        quality_impact_value: quality_value,
        total_value: total_value,
        value_breakdown: {
            revenue_percentage: (revenue / total_value) * 100,
            automation_percentage: (automation_savings / total_value) * 100,
            error_reduction_percentage: (error_savings / total_value) * 100,
            time_savings_percentage: (time_value / total_value) * 100,
            quality_percentage: (quality_value / total_value) * 100
        }
    }
END FUNCTION


FUNCTION CalculateTimeSeriesROI(use_case_id, time_period):
    // Calculate ROI over time to show trend

    // Break period into smaller intervals (e.g., monthly)
    intervals = SplitIntoPeriods(time_period, "monthly")

    time_series = []
    cumulative_cost = 0
    cumulative_value = 0

    FOR EACH interval IN intervals:
        interval_cost = GatherUseCaseCosts(use_case_id, interval).total_cost
        interval_value = GatherUseCaseValue(use_case_id, interval).total_value

        cumulative_cost += interval_cost
        cumulative_value += interval_value

        time_series.APPEND({
            period: interval,
            interval_cost: interval_cost,
            interval_value: interval_value,
            interval_net: interval_value - interval_cost,
            interval_roi: IF interval_cost > 0 THEN ((interval_value - interval_cost) / interval_cost) * 100 ELSE 0,
            cumulative_cost: cumulative_cost,
            cumulative_value: cumulative_value,
            cumulative_net: cumulative_value - cumulative_cost,
            cumulative_roi: IF cumulative_cost > 0 THEN ((cumulative_value - cumulative_cost) / cumulative_cost) * 100 ELSE 0
        })
    END FOR

    RETURN time_series
END FUNCTION
```

---

### 4.3 Comparative Analysis

#### Algorithm: CompareProviders
```
FUNCTION CompareProviders(comparison_request):
    // Compare costs and performance across providers

    providers = comparison_request.providers
    time_range = comparison_request.time_range
    workload_type = comparison_request.workload_type  // Optional filter

    comparison_data = []

    // Step 1: Gather data for each provider
    FOR EACH provider IN providers:
        provider_stats = GatherProviderStats(provider, time_range, workload_type)
        comparison_data.APPEND(provider_stats)
    END FOR

    // Step 2: Calculate relative metrics
    FOR EACH provider_data IN comparison_data:
        provider_data.relative_metrics = CalculateRelativeMetrics(
            provider_data,
            comparison_data
        )
    END FOR

    // Step 3: Rank providers
    ranked = RankProviders(comparison_data, comparison_request.optimization_goal)

    // Step 4: Generate insights
    insights = GenerateComparisonInsights(ranked)

    // Step 5: Create comparison report
    report = {
        providers_compared: providers,
        time_range: time_range,
        workload_type: workload_type,
        comparison_data: ranked,
        insights: insights,
        summary: {
            cheapest: FIND_MIN(ranked, BY: avg_cost_per_request),
            fastest: FIND_MIN(ranked, BY: avg_latency_ms),
            highest_quality: FIND_MAX(ranked, BY: avg_quality_score),
            best_value: FIND_MAX(ranked, BY: value_score)
        },
        generated_at: CURRENT_TIMESTAMP()
    }

    RETURN report
END FUNCTION


FUNCTION GatherProviderStats(provider, time_range, workload_type):
    // Collect comprehensive stats for a provider

    filters = [{field: "provider", operator: "=", value: provider}]
    IF workload_type:
        filters.APPEND({field: "workload_type", operator: "=", value: workload_type})
    END IF

    // Cost statistics
    cost_stats = AggregateCosts({
        group_by: ["provider"],
        time_range: time_range,
        filters: filters,
        metrics: ["total_cost", "avg_cost", "min_cost", "max_cost", "total_tokens", "request_count"]
    })

    // Performance statistics
    perf_stats = GetPerformanceStats(provider, time_range, workload_type)

    // Model variety
    models_used = GetUniqueModels(provider, time_range, workload_type)

    RETURN {
        provider: provider,
        total_cost: cost_stats.data[0].total_cost,
        avg_cost_per_request: cost_stats.data[0].avg_cost,
        total_requests: cost_stats.data[0].request_count,
        total_tokens: cost_stats.data[0].total_tokens,
        cost_per_1m_tokens: (cost_stats.data[0].total_cost / cost_stats.data[0].total_tokens) * 1_000_000,

        avg_latency_ms: perf_stats.avg_latency,
        p95_latency_ms: perf_stats.p95_latency,
        avg_quality_score: perf_stats.avg_quality,
        error_rate: perf_stats.error_rate,

        models_available: LENGTH(models_used),
        models_used: models_used,

        value_score: CalculateProviderValueScore(cost_stats.data[0], perf_stats)
    }
END FUNCTION


FUNCTION CalculateRelativeMetrics(provider_data, all_providers):
    // Calculate how this provider compares to others

    all_costs = EXTRACT(all_providers, field: "avg_cost_per_request")
    all_latencies = EXTRACT(all_providers, field: "avg_latency_ms")
    all_quality = EXTRACT(all_providers, field: "avg_quality_score")

    avg_market_cost = MEAN(all_costs)
    avg_market_latency = MEAN(all_latencies)
    avg_market_quality = MEAN(all_quality)

    RETURN {
        cost_vs_average: ((provider_data.avg_cost_per_request - avg_market_cost) / avg_market_cost) * 100,
        latency_vs_average: ((provider_data.avg_latency_ms - avg_market_latency) / avg_market_latency) * 100,
        quality_vs_average: ((provider_data.avg_quality_score - avg_market_quality) / avg_market_quality) * 100,

        cost_rank: RANK(all_providers, BY: avg_cost_per_request, ORDER: ASC),
        latency_rank: RANK(all_providers, BY: avg_latency_ms, ORDER: ASC),
        quality_rank: RANK(all_providers, BY: avg_quality_score, ORDER: DESC),
        value_rank: RANK(all_providers, BY: value_score, ORDER: DESC)
    }
END FUNCTION


FUNCTION RankProviders(comparison_data, optimization_goal):
    // Rank providers based on optimization goal

    SWITCH optimization_goal:
        CASE "minimize_cost":
            RETURN SORT(comparison_data, BY: avg_cost_per_request, ORDER: ASC)

        CASE "maximize_performance":
            RETURN SORT(comparison_data, BY: [
                avg_quality_score DESC,
                avg_latency_ms ASC
            ])

        CASE "best_value":
            RETURN SORT(comparison_data, BY: value_score, ORDER: DESC)

        DEFAULT:
            RETURN SORT(comparison_data, BY: value_score, ORDER: DESC)
    END SWITCH
END FUNCTION


FUNCTION GenerateComparisonInsights(ranked_providers):
    // Generate human-readable insights from comparison

    insights = []

    best_provider = ranked_providers[0]
    worst_provider = ranked_providers[LENGTH(ranked_providers) - 1]

    // Cost insights
    cost_spread = worst_provider.avg_cost_per_request - best_provider.avg_cost_per_request
    cost_savings_percentage = (cost_spread / worst_provider.avg_cost_per_request) * 100

    insights.APPEND({
        type: "cost",
        message: "Switching from " + worst_provider.provider + " to " + best_provider.provider +
                 " could reduce costs by " + ROUND(cost_savings_percentage, 1) + "%",
        impact: "high"
    })

    // Performance insights
    fastest = FIND_MIN(ranked_providers, BY: avg_latency_ms)
    slowest = FIND_MAX(ranked_providers, BY: avg_latency_ms)

    latency_improvement = slowest.avg_latency_ms - fastest.avg_latency_ms
    latency_improvement_pct = (latency_improvement / slowest.avg_latency_ms) * 100

    insights.APPEND({
        type: "performance",
        message: fastest.provider + " is " + ROUND(latency_improvement_pct, 1) +
                 "% faster than " + slowest.provider + " (avg latency: " +
                 ROUND(fastest.avg_latency_ms, 0) + "ms vs " + ROUND(slowest.avg_latency_ms, 0) + "ms)",
        impact: "medium"
    })

    // Value insights
    best_value = FIND_MAX(ranked_providers, BY: value_score)

    insights.APPEND({
        type: "value",
        message: best_value.provider + " offers the best overall value (quality-adjusted cost efficiency)",
        impact: "high"
    })

    // Model variety insight
    most_models = FIND_MAX(ranked_providers, BY: models_available)

    insights.APPEND({
        type: "flexibility",
        message: most_models.provider + " offers the most model options (" +
                 most_models.models_available + " models)",
        impact: "low"
    })

    RETURN insights
END FUNCTION


FUNCTION CalculateProviderValueScore(cost_stats, perf_stats):
    // Composite score combining cost and performance

    // Normalize metrics (0-1 scale)
    // For cost: lower is better, so invert
    norm_cost = 1 / (1 + cost_stats.avg_cost)

    // For quality: higher is better
    norm_quality = perf_stats.avg_quality / 100

    // For latency: lower is better, so invert
    norm_speed = 1 / (1 + perf_stats.avg_latency / 1000)

    // For reliability: higher is better (1 - error_rate)
    norm_reliability = 1 - perf_stats.error_rate

    // Weighted composite
    value_score = (
        norm_cost * WEIGHT_COST +
        norm_quality * WEIGHT_QUALITY +
        norm_speed * WEIGHT_SPEED +
        norm_reliability * WEIGHT_RELIABILITY
    )

    RETURN ROUND(value_score * 100, 2)  // Scale to 0-100
END FUNCTION
```

---

## 5. SUPPORTING ALGORITHMS

### 5.1 Data Validation

#### Algorithm: ValidateMetricSchema
```
FUNCTION ValidateMetricSchema(data, source_type):
    errors = []

    // Required fields based on source
    required_fields = GetRequiredFields(source_type)

    FOR EACH field IN required_fields:
        IF NOT data.CONTAINS(field):
            errors.APPEND({
                field: field,
                error: "Missing required field",
                severity: "error"
            })
        ELSE IF data[field] IS NULL:
            errors.APPEND({
                field: field,
                error: "Field value is null",
                severity: "error"
            })
        END IF
    END FOR

    // Validate data types
    IF data.CONTAINS("timestamp"):
        IF NOT IsValidTimestamp(data.timestamp):
            errors.APPEND({
                field: "timestamp",
                error: "Invalid timestamp format",
                severity: "error"
            })
        END IF
    END IF

    IF data.CONTAINS("tokens_total"):
        IF NOT IsNumeric(data.tokens_total) OR data.tokens_total < 0:
            errors.APPEND({
                field: "tokens_total",
                error: "Token count must be a non-negative number",
                severity: "error"
            })
        END IF
    END IF

    // Validate ranges
    IF data.CONTAINS("tokens_total"):
        IF data.tokens_total > MAX_REASONABLE_TOKENS:
            errors.APPEND({
                field: "tokens_total",
                error: "Token count exceeds reasonable maximum",
                severity: "warning"
            })
        END IF
    END IF

    RETURN {
        is_valid: LENGTH(errors WHERE severity == "error") == 0,
        errors: errors,
        warnings: errors WHERE severity == "warning"
    }
END FUNCTION
```

### 5.2 Deduplication

#### Algorithm: IsDuplicate
```
FUNCTION IsDuplicate(metric):
    // Check if this metric was already ingested

    // Generate fingerprint
    fingerprint = GenerateMetricFingerprint(metric)

    // Check in dedup cache (time-windowed cache)
    cache_key = "dedup:" + fingerprint
    cached_entry = GET_FROM_CACHE(cache_key)

    IF cached_entry:
        // Found duplicate
        LOG_INFO("Duplicate detected", {
            original_id: cached_entry.metric_id,
            duplicate_id: metric.id,
            fingerprint: fingerprint
        })
        RETURN true
    ELSE:
        // Not a duplicate, cache this fingerprint
        SET_IN_CACHE(cache_key, {
            metric_id: metric.id,
            timestamp: metric.timestamp
        }, TTL: DEDUP_WINDOW_SECONDS)

        RETURN false
    END IF
END FUNCTION


FUNCTION GenerateMetricFingerprint(metric):
    // Create unique identifier for metric

    fingerprint_data = CONCAT([
        metric.source,
        metric.model_id,
        metric.provider,
        ROUND(metric.timestamp, PRECISION_SECONDS),
        metric.tokens_total,
        metric.metadata.request_id  // If available
    ], separator: "|")

    fingerprint = HASH(fingerprint_data, algorithm: "SHA256")

    RETURN fingerprint
END FUNCTION
```

### 5.3 Pricing Data Management

#### Algorithm: GetPricingData
```
FUNCTION GetPricingData(provider, model, timestamp, region):
    // Retrieve applicable pricing for given parameters

    // Step 1: Check cache first
    cache_key = "pricing:" + provider + ":" + model + ":" + region
    cached_pricing = GET_FROM_CACHE(cache_key)

    IF cached_pricing AND cached_pricing.effective_date <= timestamp:
        RETURN cached_pricing
    END IF

    // Step 2: Query pricing database
    pricing = QUERY_DATABASE({
        table: "pricing",
        where: [
            {field: "provider", operator: "=", value: provider},
            {field: "model_id", operator: "=", value: model},
            {field: "region", operator: "=", value: region},
            {field: "effective_date", operator: "<=", value: timestamp}
        ],
        order_by: "effective_date DESC",
        limit: 1
    })

    IF pricing IS NULL:
        // Try without region filter (use default pricing)
        pricing = QUERY_DATABASE({
            table: "pricing",
            where: [
                {field: "provider", operator: "=", value: provider},
                {field: "model_id", operator: "=", value: model},
                {field: "region", operator: "=", value: "default"},
                {field: "effective_date", operator: "<=", value: timestamp}
            ],
            order_by: "effective_date DESC",
            limit: 1
        })
    END IF

    IF pricing IS NULL:
        // No pricing found
        RETURN NULL
    END IF

    // Step 3: Cache for future use
    SET_IN_CACHE(cache_key, pricing, TTL: PRICING_CACHE_TTL)

    RETURN pricing
END FUNCTION
```

---

## APPENDIX: Constants and Configuration

```
// Thresholds
CONSTANT MINIMUM_DATA_POINTS = 30
CONSTANT MINIMUM_TREND_POINTS = 10
CONSTANT THRESHOLD_TOKEN_DISCREPANCY = 100
CONSTANT THRESHOLD_RATE_CHANGE = 0.5  // 50% change
CONSTANT THRESHOLD_HIGH_SEVERITY = 0.8
CONSTANT THRESHOLD_ON_TRACK_VARIANCE = 5.0  // 5%
CONSTANT MAX_REASONABLE_TOKENS = 1_000_000

// Weights for scoring
CONSTANT WEIGHT_COST = 0.35
CONSTANT WEIGHT_QUALITY = 0.40
CONSTANT WEIGHT_SPEED = 0.15
CONSTANT WEIGHT_RELIABILITY = 0.10

// Weights for forecasting methods
CONSTANT WEIGHT_LINEAR = 0.3
CONSTANT WEIGHT_PATTERN = 0.4
CONSTANT WEIGHT_TREND = 0.3

// Trend thresholds (as percentage of mean)
CONSTANT THRESHOLD_STRONG_UPWARD = 0.15
CONSTANT THRESHOLD_MODERATE_UPWARD = 0.08
CONSTANT THRESHOLD_SLIGHT_UPWARD = 0.03

// Time windows
CONSTANT TIME_WINDOW_BEFORE = 300_000  // 5 minutes in ms
CONSTANT TIME_WINDOW_AFTER = 300_000
CONSTANT DEDUP_WINDOW_SECONDS = 3600  // 1 hour
CONSTANT PRICING_CACHE_TTL = 86400  // 24 hours

// Cost thresholds
CONSTANT THRESHOLD_MIN_COST = 0.001  // $0.001
CONSTANT THRESHOLD_COST_USAGE_DEVIATION = 0.3  // 30%
```

---

## Summary

This pseudocode document provides detailed algorithms for:

1. **Core Data Flows**: Ingesting metrics from multiple sources, normalizing token counts, calculating costs with provider-specific pricing, correlating with performance data, and computing ROI.

2. **Integration Workflows**: Seamless data exchange with LLM-Observatory, Edge-Agent, Test-Bench, Governance-Core, Auto-Optimizer, and Registry through event-driven and API-based patterns.

3. **Forecasting Logic**: Time-series prediction using multiple methods (linear, Holt-Winters, moving average), trend detection with statistical significance testing, anomaly detection using multiple algorithms, and budget projection with confidence intervals.

4. **Query & Reporting**: Flexible cost aggregation across multiple dimensions, detailed ROI calculations per use case, and comprehensive provider comparisons.

All algorithms include error handling, caching strategies, and optimization considerations for production deployment.
