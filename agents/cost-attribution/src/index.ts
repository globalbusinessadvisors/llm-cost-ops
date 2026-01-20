/**
 * Cost Attribution Agent - Main Entry Point
 *
 * Exports handler for Google Cloud Functions and all types for SDK usage
 */

import { http } from '@google-cloud/functions-framework';
import { costAttributionHandler } from './handler';

// Register HTTP function handler
http('costAttributionHandler', costAttributionHandler);

// Export handler for direct usage
export { costAttributionHandler } from './handler';

// Export types for SDK usage
export type {
  CostAttributionInput,
  CostAttributionOutput,
  UsageData,
  PricingContext,
  AttributionDimensions,
  CostBreakdown,
  AttributionResult,
  DecisionEventMetadata,
  TelemetryMetadata,
  ValidationError,
  ErrorResponse,
} from './types';

// Export schemas for SDK validation
export {
  costAttributionInputSchema,
  costAttributionOutputSchema,
  usageDataSchema,
  pricingContextSchema,
  attributionDimensionsSchema,
  costBreakdownSchema,
  attributionResultSchema,
  decisionEventMetadataSchema,
  telemetryMetadataSchema,
  validationErrorSchema,
  errorResponseSchema,
} from './schemas';

// Export service classes for SDK usage
export { CostCalculator } from './calculator';
export { CostAttributor } from './attributor';
