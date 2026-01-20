/**
 * Cost-Performance Tradeoff Agent
 *
 * Main entry point for the Cost-Performance Tradeoff Agent.
 *
 * Classification: TRADEOFF ANALYSIS
 *
 * Purpose:
 *   Evaluate tradeoffs between cost, latency, and quality to inform decision-making.
 *
 * Scope:
 *   - Analyze cost vs latency vs quality metrics
 *   - Identify diminishing returns
 *   - Emit tradeoff recommendations (non-executing)
 *
 * Decision Type: "cost_performance_tradeoff"
 *
 * This agent MAY:
 *   - Analyze cost vs latency vs quality metrics
 *   - Identify diminishing returns in cost/quality curves
 *   - Compute Pareto frontier (efficient options)
 *   - Generate model recommendations
 *   - Emit tradeoff recommendations (non-executing)
 *
 * This agent MUST NOT:
 *   - Intercept runtime execution
 *   - Trigger retries
 *   - Execute workflows
 *   - Modify routing or execution behavior
 *   - Apply optimizations automatically
 *   - Enforce policies directly (only emit advisories)
 *
 * Deployment:
 *   - Google Cloud Edge Function
 *   - Stateless execution
 *   - Deterministic behavior
 *   - Persistence via ruvector-service ONLY
 */
export * from './contracts/index.js';
export { TradeoffAnalyzer } from './engine/index.js';
export { RuvectorServiceClient, TelemetryEmitter } from './services/index.js';
export { DecisionEventEmitter, decisionEventEmitter } from './types/index.js';
export type { DecisionEventOptions } from './types/index.js';
export { InputReader, OutputFormatter } from './utils/index.js';
export type { OutputFormat } from './utils/index.js';
export { handler, health } from './handler/index.js';
export declare const AGENT_ID = "cost-performance-tradeoff-agent";
export declare const AGENT_VERSION = "1.0.0";
export declare const AGENT_CLASSIFICATION = "TRADEOFF_ANALYSIS";
export declare const DECISION_TYPE = "cost_performance_tradeoff";
//# sourceMappingURL=index.d.ts.map