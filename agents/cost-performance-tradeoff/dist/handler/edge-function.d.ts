/**
 * Edge Function Handler
 *
 * Google Cloud Edge Function handler for the Cost-Performance Tradeoff Agent.
 *
 * Deployment:
 *   gcloud functions deploy cost-performance-tradeoff-agent \
 *     --runtime nodejs20 \
 *     --trigger-http \
 *     --allow-unauthenticated \
 *     --entry-point handler
 *
 * This handler:
 * - Receives HTTP requests with performance records
 * - Validates input against schemas
 * - Runs tradeoff analysis
 * - Emits DecisionEvent to ruvector-service
 * - Returns analysis results
 *
 * This handler MUST NOT:
 * - Intercept runtime execution
 * - Trigger retries
 * - Execute workflows
 * - Modify routing or execution behavior
 * - Apply optimizations automatically
 * - Enforce policies directly
 */
interface HttpRequest {
    method: string;
    headers: Record<string, string>;
    body: unknown;
    query: Record<string, string>;
}
interface HttpResponse {
    status: (code: number) => HttpResponse;
    json: (data: unknown) => void;
    set: (header: string, value: string) => HttpResponse;
}
/**
 * Main Edge Function handler
 */
export declare function handler(req: HttpRequest, res: HttpResponse): Promise<void>;
/**
 * Health check endpoint
 */
export declare function health(_req: HttpRequest, res: HttpResponse): Promise<void>;
export {};
//# sourceMappingURL=edge-function.d.ts.map