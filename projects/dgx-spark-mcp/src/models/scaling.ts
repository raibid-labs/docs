/**
 * Scaling prediction model for Spark workloads
 */

import {  ScalingOption } from '../types/estimation.js';
import { SparkConfig, WorkloadType } from '../types/spark-config.js';

export interface ScalingPredictionRequest {
  currentGPUs?: number;
  targetGPUs?: number;
  currentCores?: number;
  targetCores?: number;
  currentExecutors?: number;
  targetExecutors?: number;
  workload: {
    type: WorkloadType;
    dataSize: number; // bytes
    currentExecutionTimeMs?: number;
  };
  config?: SparkConfig;
}

export interface ScalingAnalysis {
  scalingFactor: number; // e.g., 2 for 2x resources
  expectedSpeedup: number; // Actual speedup expected (e.g., 1.7 for 1.7x faster)
  efficiency: number; // 0-1, how efficiently resources are used
  amdahlsLaw: {
    parallelFraction: number;
    serialFraction: number;
    theoreticalMaxSpeedup: number;
  };
  costEffectiveness: 'excellent' | 'good' | 'fair' | 'poor';
  recommendation: string;
}

/**
 * Predict scaling behavior for workload
 */
export async function predictScaling(
  request: ScalingPredictionRequest
): Promise<ScalingAnalysis> {
  // Determine scaling factor
  let scalingFactor = 1;
  let resourceType = 'cores';

  if (request.targetGPUs && request.currentGPUs) {
    scalingFactor = request.targetGPUs / request.currentGPUs;
    resourceType = 'GPUs';
  } else if (request.targetCores && request.currentCores) {
    scalingFactor = request.targetCores / request.currentCores;
    resourceType = 'cores';
  } else if (request.targetExecutors && request.currentExecutors) {
    scalingFactor = request.targetExecutors / request.currentExecutors;
    resourceType = 'executors';
  }

  // Estimate parallel fraction based on workload type
  const parallelFraction = estimateParallelFraction(request.workload.type);
  const serialFraction = 1 - parallelFraction;

  // Apply Amdahl's Law
  const theoreticalMaxSpeedup = 1 / (serialFraction + (parallelFraction / scalingFactor));

  // Apply practical efficiency factor
  const practicalEfficiency = calculatePracticalEfficiency(
    scalingFactor,
    request.workload.type,
    resourceType
  );

  const expectedSpeedup = theoreticalMaxSpeedup * practicalEfficiency;

  // Calculate efficiency (speedup / scaling factor)
  const efficiency = expectedSpeedup / scalingFactor;

  // Determine cost effectiveness
  const costEffectiveness = determineCostEffectiveness(efficiency);

  // Generate recommendation
  const recommendation = generateScalingRecommendation(
    scalingFactor,
    expectedSpeedup,
    efficiency,
    request.workload.type,
    resourceType
  );

  return {
    scalingFactor,
    expectedSpeedup,
    efficiency,
    amdahlsLaw: {
      parallelFraction,
      serialFraction,
      theoreticalMaxSpeedup,
    },
    costEffectiveness,
    recommendation,
  };
}

/**
 * Generate multiple scaling options
 */
export async function generateScalingOptions(
  currentConfig: SparkConfig,
  workloadType: WorkloadType,
  dataSizeGB: number
): Promise<ScalingOption[]> {
  const options: ScalingOption[] = [];

  const currentExecutors = currentConfig.executor.instances ?? 4;
  const currentCores = currentConfig.executor.cores;

  // Option 1: 2x executors
  const option2x = await predictScaling({
    currentExecutors,
    targetExecutors: currentExecutors * 2,
    workload: {
      type: workloadType,
      dataSize: dataSizeGB * (1024 ** 3),
    },
  });

  options.push({
    scale: 2,
    resources: {
      memory: {
        executorMemoryGB: parseMemory(currentConfig.executor.memory),
        driverMemoryGB: parseMemory(currentConfig.driver.memory),
        totalMemoryGB: parseMemory(currentConfig.executor.memory) * currentExecutors * 2,
        overheadGB: 0,
        peakMemoryGB: 0,
        breakdown: { execution: 0, storage: 0, overhead: 0 },
        spillLikelihood: 'unlikely',
      },
      compute: {
        executorCores: currentCores,
        executorCount: currentExecutors * 2,
        totalCores: currentCores * currentExecutors * 2,
        parallelism: currentCores * currentExecutors * 2 * 3,
        cpuUtilizationPercent: 80,
      },
      storage: {
        inputDataGB: dataSizeGB,
        intermediateDataGB: dataSizeGB * 0.5,
        outputDataGB: dataSizeGB,
        shuffleDataGB: dataSizeGB * 0.3,
        totalIOGB: dataSizeGB * 2.8,
        ioBandwidthMBps: 2000,
        tmpStorageGB: dataSizeGB * 0.5,
      },
      time: {
        estimatedMinutes: 60 / option2x.expectedSpeedup,
        range: { min: 40, max: 80 },
        breakdown: { inputIO: 10, computation: 30, shuffle: 10, outputIO: 10 },
      },
      confidence: 0.7,
      assumptions: ['2x executors scaling'],
    },
    expectedSpeedup: option2x.expectedSpeedup,
    efficiency: option2x.efficiency,
    costIncrease: 100,
    rationale: option2x.recommendation,
  });

  // Option 2: 4x executors
  const option4x = await predictScaling({
    currentExecutors,
    targetExecutors: currentExecutors * 4,
    workload: {
      type: workloadType,
      dataSize: dataSizeGB * (1024 ** 3),
    },
  });

  options.push({
    scale: 4,
    resources: {
      memory: {
        executorMemoryGB: parseMemory(currentConfig.executor.memory),
        driverMemoryGB: parseMemory(currentConfig.driver.memory),
        totalMemoryGB: parseMemory(currentConfig.executor.memory) * currentExecutors * 4,
        overheadGB: 0,
        peakMemoryGB: 0,
        breakdown: { execution: 0, storage: 0, overhead: 0 },
        spillLikelihood: 'unlikely',
      },
      compute: {
        executorCores: currentCores,
        executorCount: currentExecutors * 4,
        totalCores: currentCores * currentExecutors * 4,
        parallelism: currentCores * currentExecutors * 4 * 3,
        cpuUtilizationPercent: 75,
      },
      storage: {
        inputDataGB: dataSizeGB,
        intermediateDataGB: dataSizeGB * 0.5,
        outputDataGB: dataSizeGB,
        shuffleDataGB: dataSizeGB * 0.3,
        totalIOGB: dataSizeGB * 2.8,
        ioBandwidthMBps: 3000,
        tmpStorageGB: dataSizeGB * 0.5,
      },
      time: {
        estimatedMinutes: 60 / option4x.expectedSpeedup,
        range: { min: 30, max: 60 },
        breakdown: { inputIO: 8, computation: 20, shuffle: 8, outputIO: 8 },
      },
      confidence: 0.65,
      assumptions: ['4x executors scaling'],
    },
    expectedSpeedup: option4x.expectedSpeedup,
    efficiency: option4x.efficiency,
    costIncrease: 300,
    rationale: option4x.recommendation,
  });

  return options;
}

/**
 * Estimate parallel fraction for workload type
 */
function estimateParallelFraction(workloadType: WorkloadType): number {
  // Fraction of workload that can be parallelized
  const parallelFractions: Record<WorkloadType, number> = {
    'etl': 0.90, // Highly parallelizable
    'analytics': 0.80, // Some serial parts (final aggregations)
    'ml-training': 0.75, // Iterative algorithms have serial components
    'ml-inference': 0.95, // Highly parallelizable
    'streaming': 0.85, // Good parallelism
    'graph': 0.70, // Many dependencies
    'sql': 0.80,
    'mixed': 0.80,
  };

  return parallelFractions[workloadType] ?? 0.80;
}

/**
 * Calculate practical efficiency factor
 */
function calculatePracticalEfficiency(
  scalingFactor: number,
  workloadType: WorkloadType,
  resourceType: string
): number {
  // Base efficiency
  let efficiency = 0.90;

  // Diminishing returns with scale
  if (scalingFactor > 8) {
    efficiency *= 0.80; // Coordination overhead
  } else if (scalingFactor > 4) {
    efficiency *= 0.90;
  }

  // GPU scaling is more efficient for certain workloads
  if (resourceType === 'GPUs') {
    if (workloadType === 'ml-training' || workloadType === 'ml-inference') {
      efficiency *= 1.1; // GPUs scale well for ML
    }
  }

  // Some workloads scale better than others
  if (workloadType === 'etl' || workloadType === 'ml-inference') {
    efficiency *= 1.05; // Embarrassingly parallel
  } else if (workloadType === 'graph') {
    efficiency *= 0.85; // Harder to parallelize
  }

  return Math.min(efficiency, 1.0);
}

/**
 * Determine cost effectiveness
 */
function determineCostEffectiveness(
  efficiency: number
): 'excellent' | 'good' | 'fair' | 'poor' {
  if (efficiency >= 0.85) return 'excellent';
  if (efficiency >= 0.70) return 'good';
  if (efficiency >= 0.50) return 'fair';
  return 'poor';
}

/**
 * Generate scaling recommendation
 */
function generateScalingRecommendation(
  scalingFactor: number,
  expectedSpeedup: number,
  efficiency: number,
  workloadType: WorkloadType,
  resourceType: string
): string {
  const recommendations: string[] = [];

  recommendations.push(
    `Scaling ${resourceType} by ${scalingFactor}x will provide approximately ${expectedSpeedup.toFixed(2)}x speedup.`
  );

  if (efficiency >= 0.85) {
    recommendations.push(
      'Excellent scaling efficiency. This is a cost-effective scaling option.'
    );
  } else if (efficiency >= 0.70) {
    recommendations.push(
      'Good scaling efficiency. Recommended for production workloads.'
    );
  } else if (efficiency >= 0.50) {
    recommendations.push(
      'Fair scaling efficiency. Consider optimizing the workload before scaling further.'
    );
  } else {
    recommendations.push(
      'Poor scaling efficiency. Review workload for serial bottlenecks before adding more resources.'
    );
  }

  // Workload-specific recommendations
  if (workloadType === 'graph' && scalingFactor > 4) {
    recommendations.push(
      'Graph workloads often have diminishing returns beyond 4x scaling. Consider algorithmic optimizations.'
    );
  }

  if (workloadType === 'streaming' && resourceType === 'executors') {
    recommendations.push(
      'For streaming workloads, also consider tuning batch intervals and trigger mechanisms.'
    );
  }

  if (resourceType === 'GPUs' && (workloadType === 'ml-training' || workloadType === 'ml-inference')) {
    recommendations.push(
      'ML workloads scale well with GPUs. Ensure data pipeline can keep GPUs saturated.'
    );
  }

  return recommendations.join(' ');
}

/**
 * Helper to parse memory string
 */
function parseMemory(memory: string): number {
  const match = memory.match(/^(\d+)([gmk])$/i);
  if (!match) return 8; // Default

  const value = parseInt(match?.[1] ?? "0");
  const unit = match?.[2] ?? "g".toLowerCase();

  switch (unit) {
    case 'g': return value;
    case 'm': return value / 1024;
    case 'k': return value / (1024 * 1024);
    default: return value;
  }
}
