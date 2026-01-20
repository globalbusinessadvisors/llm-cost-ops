/**
 * Cost Attributor
 *
 * Attributes costs to dimensions (user, project, organization, etc.)
 */

import { AttributionDimensions, AttributionResult } from './types';

export class CostAttributor {
  /**
   * Attribute cost to dimensions
   */
  attribute(dimensions?: AttributionDimensions): AttributionResult {
    // Determine primary attribution dimension
    const primary = this.determinePrimary(dimensions);

    // Calculate confidence based on available dimensions
    const confidence = this.calculateConfidence(dimensions);

    // Build attribution result
    return {
      primary,
      dimensions: {
        userId: dimensions?.userId,
        projectId: dimensions?.projectId,
        organizationId: dimensions?.organizationId,
        environment: dimensions?.environment,
      },
      tags: dimensions?.tags || {},
      confidence,
    };
  }

  /**
   * Determine primary attribution dimension
   * Priority: organizationId > projectId > userId > environment
   */
  private determinePrimary(dimensions?: AttributionDimensions): string {
    if (!dimensions) {
      return 'unattributed';
    }

    if (dimensions.organizationId) {
      return `organization:${dimensions.organizationId}`;
    }

    if (dimensions.projectId) {
      return `project:${dimensions.projectId}`;
    }

    if (dimensions.userId) {
      return `user:${dimensions.userId}`;
    }

    if (dimensions.environment) {
      return `environment:${dimensions.environment}`;
    }

    return 'unattributed';
  }

  /**
   * Calculate confidence score based on available dimensions
   * More dimensions = higher confidence
   */
  private calculateConfidence(dimensions?: AttributionDimensions): number {
    if (!dimensions) {
      return 0.0;
    }

    let score = 0;
    let maxScore = 0;

    // Organization ID: 40 points (most important)
    maxScore += 40;
    if (dimensions.organizationId) {
      score += 40;
    }

    // Project ID: 30 points
    maxScore += 30;
    if (dimensions.projectId) {
      score += 30;
    }

    // User ID: 20 points
    maxScore += 20;
    if (dimensions.userId) {
      score += 20;
    }

    // Environment: 10 points
    maxScore += 10;
    if (dimensions.environment) {
      score += 10;
    }

    // Tags: up to 10 bonus points (2 points per tag, max 5 tags)
    if (dimensions.tags) {
      const tagCount = Object.keys(dimensions.tags).length;
      const tagBonus = Math.min(tagCount * 2, 10);
      score += tagBonus;
      maxScore += 10;
    }

    // Calculate confidence as percentage
    const confidence = maxScore > 0 ? score / maxScore : 0.0;

    // Round to 2 decimal places
    return Math.round(confidence * 100) / 100;
  }
}
