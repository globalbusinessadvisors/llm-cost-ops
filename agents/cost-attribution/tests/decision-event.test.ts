/**
 * Decision Event Tests
 *
 * Verifies DecisionEvent schema compliance with LLM-CostOps Constitution
 */

import { DecisionEventSchema } from '../src/contracts/schemas';
import { randomUUID } from 'crypto';

describe('DecisionEvent Schema Compliance', () => {
  const validEvent = {
    agent_id: 'cost-attribution-agent',
    agent_version: '1.0.0',
    decision_type: 'cost_attribution' as const,
    inputs_hash: 'sha256:abc123def456',
    outputs: {
      total_cost: '0.0015',
      currency: 'USD',
      attributions: [],
    },
    confidence: 0.95,
    constraints_applied: ['budget_cap:1000', 'rate_limit:100/min'],
    execution_ref: randomUUID(),
    timestamp: new Date().toISOString(),
  };

  describe('Schema Validation', () => {
    it('should accept valid DecisionEvent', () => {
      const result = DecisionEventSchema.safeParse(validEvent);
      expect(result.success).toBe(true);
    });

    it('should require agent_id', () => {
      const event = { ...validEvent, agent_id: undefined };
      const result = DecisionEventSchema.safeParse(event);
      expect(result.success).toBe(false);
    });

    it('should require agent_version', () => {
      const event = { ...validEvent, agent_version: undefined };
      const result = DecisionEventSchema.safeParse(event);
      expect(result.success).toBe(false);
    });

    it('should validate decision_type enum', () => {
      const validTypes = [
        'cost_attribution',
        'pricing_calculation',
        'scope_determination',
        'aggregation',
        'validation',
      ];

      for (const type of validTypes) {
        const event = { ...validEvent, decision_type: type };
        const result = DecisionEventSchema.safeParse(event);
        expect(result.success).toBe(true);
      }

      const invalidEvent = { ...validEvent, decision_type: 'invalid_type' };
      const result = DecisionEventSchema.safeParse(invalidEvent);
      expect(result.success).toBe(false);
    });

    it('should require inputs_hash', () => {
      const event = { ...validEvent, inputs_hash: undefined };
      const result = DecisionEventSchema.safeParse(event);
      expect(result.success).toBe(false);
    });

    it('should require outputs as object', () => {
      const event = { ...validEvent, outputs: undefined };
      const result = DecisionEventSchema.safeParse(event);
      expect(result.success).toBe(false);
    });

    it('should validate confidence range 0-1', () => {
      const lowConfidence = { ...validEvent, confidence: 0 };
      expect(DecisionEventSchema.safeParse(lowConfidence).success).toBe(true);

      const highConfidence = { ...validEvent, confidence: 1 };
      expect(DecisionEventSchema.safeParse(highConfidence).success).toBe(true);

      const midConfidence = { ...validEvent, confidence: 0.5 };
      expect(DecisionEventSchema.safeParse(midConfidence).success).toBe(true);

      const negativeConfidence = { ...validEvent, confidence: -0.1 };
      expect(DecisionEventSchema.safeParse(negativeConfidence).success).toBe(false);

      const overConfidence = { ...validEvent, confidence: 1.1 };
      expect(DecisionEventSchema.safeParse(overConfidence).success).toBe(false);
    });

    it('should require constraints_applied as array', () => {
      const event = { ...validEvent, constraints_applied: undefined };
      const result = DecisionEventSchema.safeParse(event);
      expect(result.success).toBe(false);

      const emptyConstraints = { ...validEvent, constraints_applied: [] };
      expect(DecisionEventSchema.safeParse(emptyConstraints).success).toBe(true);
    });

    it('should require execution_ref as UUID', () => {
      const event = { ...validEvent, execution_ref: 'not-a-uuid' };
      const result = DecisionEventSchema.safeParse(event);
      expect(result.success).toBe(false);

      const validUUID = { ...validEvent, execution_ref: randomUUID() };
      expect(DecisionEventSchema.safeParse(validUUID).success).toBe(true);
    });

    it('should require timestamp as ISO datetime', () => {
      const event = { ...validEvent, timestamp: 'not-a-date' };
      const result = DecisionEventSchema.safeParse(event);
      expect(result.success).toBe(false);

      const validTimestamp = { ...validEvent, timestamp: new Date().toISOString() };
      expect(DecisionEventSchema.safeParse(validTimestamp).success).toBe(true);
    });

    it('should allow optional metadata', () => {
      const eventWithMetadata = {
        ...validEvent,
        metadata: {
          processing_time_ms: 125,
          validation_status: 'passed' as const,
        },
      };
      expect(DecisionEventSchema.safeParse(eventWithMetadata).success).toBe(true);

      const eventWithoutMetadata = { ...validEvent };
      delete (eventWithoutMetadata as Record<string, unknown>).metadata;
      expect(DecisionEventSchema.safeParse(eventWithoutMetadata).success).toBe(true);
    });
  });

  describe('Constitution Compliance', () => {
    it('should have all required fields per constitution', () => {
      const requiredFields = [
        'agent_id',
        'agent_version',
        'decision_type',
        'inputs_hash',
        'outputs',
        'confidence',
        'constraints_applied',
        'execution_ref',
        'timestamp',
      ];

      const parsedEvent = DecisionEventSchema.parse(validEvent);

      for (const field of requiredFields) {
        expect(parsedEvent).toHaveProperty(field);
        expect(parsedEvent[field as keyof typeof parsedEvent]).toBeDefined();
      }
    });

    it('should support estimation certainty via confidence field', () => {
      // Constitution requirement: confidence (estimation certainty)
      const event = { ...validEvent, confidence: 0.87 };
      const result = DecisionEventSchema.parse(event);
      expect(result.confidence).toBe(0.87);
    });

    it('should support constraint tracking', () => {
      // Constitution requirement: constraints_applied (budget, ROI, or cost caps)
      const constraints = [
        'budget_cap:5000USD',
        'roi_threshold:1.5',
        'cost_cap_daily:100',
      ];
      const event = { ...validEvent, constraints_applied: constraints };
      const result = DecisionEventSchema.parse(event);
      expect(result.constraints_applied).toEqual(constraints);
    });
  });
});
