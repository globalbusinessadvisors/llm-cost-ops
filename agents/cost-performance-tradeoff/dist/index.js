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
// Export contracts (schemas and types)
export * from './contracts/index.js';
// Export engine
export { TradeoffAnalyzer } from './engine/index.js';
// Export services
export { RuvectorServiceClient, TelemetryEmitter } from './services/index.js';
// Export types (decision event)
export { DecisionEventEmitter, decisionEventEmitter } from './types/index.js';
// Export utilities
export { InputReader, OutputFormatter } from './utils/index.js';
// Export Edge Function handler
export { handler, health } from './handler/index.js';
// Agent metadata
export const AGENT_ID = 'cost-performance-tradeoff-agent';
export const AGENT_VERSION = '1.0.0';
export const AGENT_CLASSIFICATION = 'TRADEOFF_ANALYSIS';
export const DECISION_TYPE = 'cost_performance_tradeoff';
//# sourceMappingURL=index.js.map