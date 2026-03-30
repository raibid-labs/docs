/**
 * Impact estimation for recommendations
 */

import { Recommendation } from './engine.js';
import { SparkConfig } from '../types/spark-config.js';

export interface ImpactEstimate {
  performanceImprovement: {
    estimated: number; // percentage
    range: { min: number; max: number };
    metric: 'throughput' | 'latency' | 'resource-efficiency';
  };
  costImpact: {
    estimated: number; // percentage (negative = savings)
    range: { min: number; max: number };
    metric: 'resource-cost' | 'execution-cost' | 'infrastructure-cost';
  };
  reliabilityImprovement: {
    estimated: number; // percentage
    metric: 'failure-rate' | 'stability' | 'predictability';
  };
  implementationEffort: {
    timeEstimate: string; // e.g., "15 minutes", "2 hours"
    complexity: 'trivial' | 'simple' | 'moderate' | 'complex';
    riskLevel: 'low' | 'medium' | 'high';
  };
  confidence: number; // 0-1
}

/**
 * Estimate impact of applying a recommendation
 */
export async function estimateImpact(request: {
  recommendation: Recommendation;
  baseline: SparkConfig;
  workloadType?: string;
}): Promise<ImpactEstimate> {
  const { recommendation, baseline, workloadType = 'mixed' } = request;

  // Estimate performance improvement
  const performanceImprovement = estimatePerformanceImpact(
    recommendation,
    baseline,
    workloadType
  );

  // Estimate cost impact
  const costImpact = estimateCostImpact(recommendation, baseline);

  // Estimate reliability improvement
  const reliabilityImprovement = estimateReliabilityImpact(recommendation);

  // Estimate implementation effort
  const implementationEffort = estimateImplementationEffort(recommendation);

  // Calculate confidence based on recommendation type and available data
  const confidence = calculateConfidence(recommendation);

  return {
    performanceImprovement,
    costImpact,
    reliabilityImprovement,
    implementationEffort,
    confidence,
  };
}

/**
 * Estimate performance improvement
 */
function estimatePerformanceImpact(
  recommendation: Recommendation,
  _baseline: SparkConfig,
  _workloadType: string
): ImpactEstimate['performanceImprovement'] {
  let estimated = 0;
  let min = 0;
  let max = 0;
  let metric: 'throughput' | 'latency' | 'resource-efficiency' = 'throughput';

  // Parse impact description
  const impactDesc = recommendation.impact.performanceGain ?? '';

  // Extract percentage or multiplier
  const percentMatch = impactDesc.match(/(\d+)-?(\d+)?%/);
  const multMatch = impactDesc.match(/(\d+)-?(\d+)?x/);

  if (percentMatch && percentMatch[1]) {
    min = parseInt(percentMatch[1], 10);
    max = percentMatch[2] ? parseInt(percentMatch[2], 10) : min * 1.5;
    estimated = (min + max) / 2;
  } else if (multMatch && multMatch[1]) {
    const multMin = parseInt(multMatch[1], 10);
    const multMax = multMatch[2] ? parseInt(multMatch[2], 10) : multMin * 1.5;
    min = (multMin - 1) * 100;
    max = (multMax - 1) * 100;
    estimated = (min + max) / 2;
  } else {
    // Qualitative assessment
    if (impactDesc.toLowerCase().includes('very high')) {
      estimated = 60;
      min = 40;
      max = 100;
    } else if (impactDesc.toLowerCase().includes('high')) {
      estimated = 35;
      min = 20;
      max = 50;
    } else if (impactDesc.toLowerCase().includes('moderate')) {
      estimated = 15;
      min = 10;
      max = 25;
    } else {
      estimated = 8;
      min = 5;
      max = 15;
    }
  }

  // Adjust based on recommendation category
  if (recommendation.category === 'performance') {
    metric = 'throughput';
  } else if (recommendation.category === 'reliability') {
    metric = 'resource-efficiency';
  }

  return {
    estimated,
    range: { min, max },
    metric,
  };
}

/**
 * Estimate cost impact
 */
function estimateCostImpact(
  recommendation: Recommendation,
  _baseline: SparkConfig
): ImpactEstimate['costImpact'] {
  let estimated = 0;
  let min = 0;
  let max = 0;
  let metric: 'resource-cost' | 'execution-cost' | 'infrastructure-cost' = 'execution-cost';

  const costDesc = recommendation.impact.costSavings ?? '';

  if (costDesc) {
    // Parse savings
    const percentMatch = costDesc.match(/(\d+)-?(\d+)?%/);
    if (percentMatch && percentMatch[1]) {
      min = -parseInt(percentMatch[1], 10); // Negative = savings
      max = percentMatch[2] ? -parseInt(percentMatch[2], 10) : min;
      estimated = (min + max) / 2;
    } else if (costDesc.toLowerCase().includes('significant')) {
      estimated = -25;
      min = -40;
      max = -15;
    } else if (costDesc.toLowerCase().includes('moderate')) {
      estimated = -15;
      min = -25;
      max = -10;
    } else {
      estimated = -10;
      min = -15;
      max = -5;
    }

    metric = 'execution-cost';
  } else if (recommendation.category === 'cost') {
    // Infer cost impact from recommendation
    if (recommendation.title.toLowerCase().includes('resource')) {
      estimated = -12;
      min = -20;
      max = -5;
      metric = 'resource-cost';
    } else {
      estimated = -8;
      min = -15;
      max = -3;
      metric = 'execution-cost';
    }
  }

  return {
    estimated,
    range: { min, max },
    metric,
  };
}

/**
 * Estimate reliability improvement
 */
function estimateReliabilityImpact(
  recommendation: Recommendation
): ImpactEstimate['reliabilityImprovement'] {
  let estimated = 0;
  let metric: 'failure-rate' | 'stability' | 'predictability' = 'stability';

  const reliabilityDesc = recommendation.impact.reliabilityImprovement ?? '';

  if (reliabilityDesc) {
    if (reliabilityDesc.toLowerCase().includes('high')) {
      estimated = 40;
      metric = 'failure-rate';
    } else if (reliabilityDesc.toLowerCase().includes('moderate')) {
      estimated = 25;
      metric = 'stability';
    } else {
      estimated = 15;
      metric = 'predictability';
    }
  } else if (recommendation.category === 'reliability') {
    estimated = 20;
    metric = 'stability';
  }

  return {
    estimated,
    metric,
  };
}

/**
 * Estimate implementation effort
 */
function estimateImplementationEffort(
  recommendation: Recommendation
): ImpactEstimate['implementationEffort'] {
  let timeEstimate: string;
  let complexity: 'trivial' | 'simple' | 'moderate' | 'complex';
  let riskLevel: 'low' | 'medium' | 'high' = 'low';

  const stepCount = recommendation.implementation.steps.length;
  const hasConfigChanges = recommendation.implementation.configChanges !== undefined;

  switch (recommendation.implementation.difficulty) {
    case 'easy':
      timeEstimate = hasConfigChanges ? '5-10 minutes' : '15-30 minutes';
      complexity = hasConfigChanges ? 'trivial' : 'simple';
      riskLevel = 'low';
      break;

    case 'medium':
      timeEstimate = stepCount > 3 ? '1-2 hours' : '30-60 minutes';
      complexity = 'moderate';
      riskLevel = 'low';
      break;

    case 'hard':
      timeEstimate = '2-4 hours';
      complexity = 'complex';
      riskLevel = recommendation.priority === 'critical' ? 'medium' : 'low';
      break;
  }

  // Adjust risk based on priority
  if (recommendation.priority === 'critical') {
    riskLevel = 'medium'; // Higher risk if critical issue
  }

  return {
    timeEstimate,
    complexity,
    riskLevel,
  };
}

/**
 * Calculate confidence in impact estimate
 */
function calculateConfidence(recommendation: Recommendation): number {
  let confidence = 0.7; // Base confidence

  // Higher confidence for specific, measurable recommendations
  if (recommendation.implementation.configChanges) {
    confidence += 0.15; // We know exact changes
  }

  // Adjust based on impact description specificity
  const hasSpecificNumbers =
    (recommendation.impact.performanceGain?.match(/\d+/) !== null) ||
    (recommendation.impact.costSavings?.match(/\d+/) !== null);

  if (hasSpecificNumbers) {
    confidence += 0.1;
  }

  // Lower confidence for complex implementations
  if (recommendation.implementation.difficulty === 'hard') {
    confidence -= 0.15;
  }

  // Higher confidence for well-known best practices
  if (recommendation.category === 'best-practice') {
    confidence += 0.05;
  }

  return Math.min(Math.max(confidence, 0.4), 0.95);
}

/**
 * Compare multiple recommendations
 */
export async function compareRecommendations(
  recommendations: Recommendation[],
  baseline: SparkConfig,
  workloadType?: string
): Promise<Array<{
  recommendation: Recommendation;
  impact: ImpactEstimate;
  roi: number; // Return on investment score
}>> {
  const results = await Promise.all(
    recommendations.map(async rec => {
      const impact = await estimateImpact({
        recommendation: rec,
        baseline,
        workloadType,
      });

      // Calculate ROI (performance gain / implementation effort)
      const roi = calculateROI(impact);

      return {
        recommendation: rec,
        impact,
        roi,
      };
    })
  );

  // Sort by ROI (descending)
  results.sort((a, b) => b.roi - a.roi);

  return results;
}

/**
 * Calculate ROI score
 */
function calculateROI(impact: ImpactEstimate): number {
  const perfGain = impact.performanceImprovement.estimated;
  const costSavings = Math.abs(impact.costImpact.estimated);
  const reliabilityGain = impact.reliabilityImprovement.estimated;

  // Effort penalty
  const effortPenalty = impact.implementationEffort.complexity === 'trivial' ? 1.0 :
                       impact.implementationEffort.complexity === 'simple' ? 0.9 :
                       impact.implementationEffort.complexity === 'moderate' ? 0.7 : 0.5;

  // Weighted benefit
  const benefit = (perfGain * 0.5) + (costSavings * 0.3) + (reliabilityGain * 0.2);

  // ROI = benefit * confidence * effort_factor
  return benefit * impact.confidence * effortPenalty;
}
