/**
 * Priority ranking for recommendations
 */

import { Recommendation } from './engine.js';

export interface RankedRecommendation extends Recommendation {
  score: number;
  rank: number;
}

/**
 * Rank recommendations by priority and impact
 */
export async function rankRecommendations(
  recommendations: Recommendation[]
): Promise<RankedRecommendation[]> {
  // Calculate scores for each recommendation
  const scored = recommendations.map(rec => ({
    ...rec,
    score: calculateScore(rec),
    rank: 0,
  }));

  // Sort by score (descending)
  scored.sort((a, b) => b.score - a.score);

  // Assign ranks
  scored.forEach((rec, index) => {
    rec.rank = index + 1;
  });

  return scored;
}

/**
 * Calculate priority score for a recommendation
 */
function calculateScore(recommendation: Recommendation): number {
  let score = 0;

  // Priority weight (40% of total score)
  const priorityWeights = {
    critical: 100,
    high: 70,
    medium: 40,
    low: 20,
  };
  score += priorityWeights[recommendation.priority] * 0.4;

  // Impact weight (40% of total score)
  let impactScore = 0;
  if (recommendation.impact.performanceGain) {
    impactScore += estimateImpactValue(recommendation.impact.performanceGain);
  }
  if (recommendation.impact.costSavings) {
    impactScore += estimateImpactValue(recommendation.impact.costSavings) * 0.8;
  }
  if (recommendation.impact.reliabilityImprovement) {
    impactScore += estimateImpactValue(recommendation.impact.reliabilityImprovement) * 0.9;
  }
  score += impactScore * 0.4;

  // Implementation difficulty (20% of total score - easier is better)
  const difficultyWeights = {
    easy: 20,
    medium: 12,
    hard: 5,
  };
  score += difficultyWeights[recommendation.implementation.difficulty] * 0.2;

  return Math.round(score);
}

/**
 * Estimate numeric value from impact description
 */
function estimateImpactValue(impactDescription: string): number {
  const desc = impactDescription.toLowerCase();

  // Parse percentage improvements
  const percentMatch = desc.match(/(\d+)-?(\d+)?%/);
  if (percentMatch && percentMatch[1]) {
    const low = parseInt(percentMatch[1], 10);
    const high = percentMatch[2] ? parseInt(percentMatch[2], 10) : low;
    return (low + high) / 2;
  }

  // Parse multiplier improvements (e.g., "3-10x")
  const multMatch = desc.match(/(\d+)-?(\d+)?x/);
  if (multMatch && multMatch[1]) {
    const low = parseInt(multMatch[1], 10);
    const high = multMatch[2] ? parseInt(multMatch[2], 10) : low;
    return ((low + high) / 2) * 20; // Convert to percentage-like score
  }

  // Qualitative assessments
  if (desc.includes('very high')) return 80;
  if (desc.includes('high')) return 60;
  if (desc.includes('moderate')) return 40;
  if (desc.includes('low')) return 20;
  if (desc.includes('minimal')) return 10;

  return 30; // Default moderate impact
}

/**
 * Filter recommendations by minimum priority
 */
export async function filterByPriority(
  recommendations: Recommendation[],
  minPriority: 'critical' | 'high' | 'medium' | 'low'
): Promise<Recommendation[]> {
  const priorityOrder = ['low', 'medium', 'high', 'critical'];
  const minIndex = priorityOrder.indexOf(minPriority);

  return recommendations.filter(rec => {
    const recIndex = priorityOrder.indexOf(rec.priority);
    return recIndex >= minIndex;
  });
}

/**
 * Group recommendations by category
 */
export async function groupByCategory(
  recommendations: Recommendation[]
): Promise<Record<string, Recommendation[]>> {
  const grouped: Record<string, Recommendation[]> = {
    performance: [],
    cost: [],
    reliability: [],
    'best-practice': [],
  };

  recommendations.forEach(rec => {
    const categoryArray = grouped[rec.category];
    if (categoryArray) {
      categoryArray.push(rec);
    }
  });

  return grouped;
}

/**
 * Get top N recommendations
 */
export async function getTopRecommendations(
  recommendations: Recommendation[],
  n: number
): Promise<Recommendation[]> {
  const ranked = await rankRecommendations(recommendations);
  return ranked.slice(0, n);
}

/**
 * Get quick wins (high impact, easy implementation)
 */
export async function getQuickWins(
  recommendations: Recommendation[]
): Promise<Recommendation[]> {
  return recommendations.filter(rec => {
    const hasHighImpact = rec.priority === 'critical' || rec.priority === 'high';
    const isEasy = rec.implementation.difficulty === 'easy';
    return hasHighImpact && isEasy;
  });
}
