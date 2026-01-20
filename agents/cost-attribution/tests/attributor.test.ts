/**
 * Cost Attributor Tests
 */

import { CostAttributor } from '../src/attributor';
import type { AttributionDimensions } from '../src/types';

describe('CostAttributor', () => {
  let attributor: CostAttributor;

  beforeEach(() => {
    attributor = new CostAttributor();
  });

  describe('attribute', () => {
    it('should return unattributed when no dimensions provided', () => {
      const result = attributor.attribute();

      expect(result.primary).toBe('unattributed');
      expect(result.confidence).toBe(0.0);
      expect(result.tags).toEqual({});
    });

    it('should prioritize organization ID as primary dimension', () => {
      const dimensions: AttributionDimensions = {
        userId: 'user-123',
        projectId: 'proj-456',
        organizationId: 'org-789',
        environment: 'production',
      };

      const result = attributor.attribute(dimensions);

      expect(result.primary).toBe('organization:org-789');
      expect(result.dimensions.organizationId).toBe('org-789');
    });

    it('should use project ID when organization ID not available', () => {
      const dimensions: AttributionDimensions = {
        userId: 'user-123',
        projectId: 'proj-456',
        environment: 'production',
      };

      const result = attributor.attribute(dimensions);

      expect(result.primary).toBe('project:proj-456');
    });

    it('should use user ID when higher priority dimensions not available', () => {
      const dimensions: AttributionDimensions = {
        userId: 'user-123',
        environment: 'production',
      };

      const result = attributor.attribute(dimensions);

      expect(result.primary).toBe('user:user-123');
    });

    it('should use environment as fallback', () => {
      const dimensions: AttributionDimensions = {
        environment: 'production',
      };

      const result = attributor.attribute(dimensions);

      expect(result.primary).toBe('environment:production');
    });

    it('should calculate confidence correctly with all dimensions', () => {
      const dimensions: AttributionDimensions = {
        userId: 'user-123',
        projectId: 'proj-456',
        organizationId: 'org-789',
        environment: 'production',
        tags: {
          team: 'ml-platform',
          feature: 'chatbot',
        },
      };

      const result = attributor.attribute(dimensions);

      // 40 (org) + 30 (project) + 20 (user) + 10 (env) + 4 (2 tags) = 104 / 110 = 0.945
      expect(result.confidence).toBeCloseTo(0.95, 2);
    });

    it('should calculate confidence correctly with partial dimensions', () => {
      const dimensions: AttributionDimensions = {
        projectId: 'proj-456',
        environment: 'production',
      };

      const result = attributor.attribute(dimensions);

      // 30 (project) + 10 (env) = 40 / 100 = 0.4
      expect(result.confidence).toBe(0.4);
    });

    it('should include tags in result', () => {
      const dimensions: AttributionDimensions = {
        userId: 'user-123',
        tags: {
          team: 'ml-platform',
          feature: 'chatbot',
          version: 'v2',
        },
      };

      const result = attributor.attribute(dimensions);

      expect(result.tags).toEqual({
        team: 'ml-platform',
        feature: 'chatbot',
        version: 'v2',
      });
    });

    it('should limit tag bonus to 10 points', () => {
      const dimensions: AttributionDimensions = {
        organizationId: 'org-123',
        tags: {
          tag1: 'value1',
          tag2: 'value2',
          tag3: 'value3',
          tag4: 'value4',
          tag5: 'value5',
          tag6: 'value6', // 6th tag should not add more points
        },
      };

      const result = attributor.attribute(dimensions);

      // 40 (org) + 10 (max tag bonus) = 50 / 50 = 1.0
      expect(result.confidence).toBe(1.0);
    });
  });
});
