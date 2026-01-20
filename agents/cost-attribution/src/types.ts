/**
 * Cost Attribution Agent Types
 *
 * Type definitions for cost attribution analysis and decision events
 */

export interface CostAttributionInput {
  /** Unique request identifier */
  requestId: string;
  /** Timestamp of the request (ISO 8601) */
  timestamp: string;
  /** Usage data for cost calculation */
  usage: UsageData;
  /** Optional pricing context */
  pricingContext?: PricingContext;
  /** Optional attribution dimensions */
  dimensions?: AttributionDimensions;
}

export interface UsageData {
  /** LLM provider (e.g., "openai", "anthropic", "google") */
  provider: string;
  /** Model identifier */
  model: string;
  /** Number of input tokens */
  inputTokens: number;
  /** Number of output tokens */
  outputTokens: number;
  /** Optional: Number of cached tokens */
  cachedTokens?: number;
  /** Optional: Request latency in milliseconds */
  latencyMs?: number;
}

export interface PricingContext {
  /** Pricing tier or plan */
  tier?: string;
  /** Currency code (default: "USD") */
  currency?: string;
  /** Optional custom pricing overrides */
  customPricing?: {
    inputPricePerToken?: number;
    outputPricePerToken?: number;
    cachedPricePerToken?: number;
  };
}

export interface AttributionDimensions {
  /** User identifier */
  userId?: string;
  /** Project or team identifier */
  projectId?: string;
  /** Organization identifier */
  organizationId?: string;
  /** Environment (e.g., "production", "staging") */
  environment?: string;
  /** Custom tags for attribution */
  tags?: Record<string, string>;
}

export interface CostAttributionOutput {
  /** Request identifier (echoed from input) */
  requestId: string;
  /** Timestamp when analysis was completed */
  analysisTimestamp: string;
  /** Calculated cost breakdown */
  costs: CostBreakdown;
  /** Attribution results */
  attribution: AttributionResult;
  /** Decision event metadata */
  decisionEvent: DecisionEventMetadata;
  /** Telemetry for LLM-Observatory */
  telemetry: TelemetryMetadata;
}

export interface CostBreakdown {
  /** Total cost in specified currency */
  totalCost: number;
  /** Cost for input tokens */
  inputCost: number;
  /** Cost for output tokens */
  outputCost: number;
  /** Cost for cached tokens (if applicable) */
  cachedCost?: number;
  /** Currency code */
  currency: string;
  /** Cost per 1K tokens (blended rate) */
  costPer1kTokens: number;
}

export interface AttributionResult {
  /** Primary attribution dimension */
  primary: string;
  /** Attribution breakdown by dimension */
  dimensions: {
    userId?: string;
    projectId?: string;
    organizationId?: string;
    environment?: string;
  };
  /** Custom tags applied */
  tags: Record<string, string>;
  /** Confidence score (0-1) */
  confidence: number;
}

export interface DecisionEventMetadata {
  /** Unique event identifier */
  eventId: string;
  /** Event type (always "cost_attribution") */
  eventType: "cost_attribution";
  /** Timestamp when event was created */
  timestamp: string;
  /** Agent identifier */
  agentId: string;
  /** Decision made by the agent */
  decision: {
    action: "attribute_cost";
    result: "success" | "error";
    confidence: number;
  };
  /** Context for the decision */
  context: {
    provider: string;
    model: string;
    totalTokens: number;
    totalCost: number;
  };
}

export interface TelemetryMetadata {
  /** Telemetry event identifier */
  telemetryId: string;
  /** Agent identifier */
  agentId: string;
  /** Timestamp */
  timestamp: string;
  /** Metrics collected */
  metrics: {
    /** Processing duration in milliseconds */
    processingDurationMs: number;
    /** Cost calculation accuracy (if validation data available) */
    accuracy?: number;
    /** Number of attribution dimensions */
    dimensionCount: number;
  };
  /** Trace context for distributed tracing */
  trace?: {
    traceId: string;
    spanId: string;
  };
}

export interface ValidationError {
  field: string;
  message: string;
  value?: unknown;
}

export interface ErrorResponse {
  error: {
    code: string;
    message: string;
    details?: ValidationError[];
  };
  requestId?: string;
  timestamp: string;
}
