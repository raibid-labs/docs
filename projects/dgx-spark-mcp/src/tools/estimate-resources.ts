/**
 * Resource estimation tool
 * Estimates resource requirements for workloads
 */

import { getHardwareSnapshot } from '../hardware/topology.js';
import { analyzeCapabilities } from '../analyzers/capabilities.js';
import type { EstimateResourcesArgs, ToolCallResponse } from '../types/tools.js';
import { parseDataSize, formatBytes } from '../types/spark.js';

export interface ResourceEstimateResult {
  description: string;
  estimatedRequirements: {
    executors: number;
    coresPerExecutor: number;
    memoryPerExecutor: string;
    totalMemory: string;
    totalCores: number;
    gpus?: number;
  };
  feasibility: {
    canRun: boolean;
    reason: string;
    limitations?: string[];
  };
  recommendations: string[];
  estimatedDuration?: string;
}

/**
 * Estimate resources for a workload
 */
export async function estimateResources(args: EstimateResourcesArgs): Promise<ToolCallResponse> {
  try {
    const { description, dataSize, computeType = 'cpu' } = args;

    // Get system capabilities
    const snapshot = await getHardwareSnapshot({ useCache: true });
    const capabilities = await analyzeCapabilities();

    // Parse data size if provided
    let dataSizeBytes: number | undefined;
    if (dataSize) {
      try {
        dataSizeBytes = parseDataSize(dataSize);
      } catch (error) {
        // Invalid data size format, continue without it
      }
    }

    // Estimate based on description and data size
    const estimate = estimateFromDescription(
      description,
      dataSizeBytes,
      computeType,
      snapshot.topology,
      capabilities
    );

    // Check feasibility
    const feasibility = checkFeasibility(estimate, snapshot.topology, capabilities);

    // Generate recommendations
    const recommendations = generateRecommendations(estimate, feasibility, capabilities);

    const result: ResourceEstimateResult = {
      description,
      estimatedRequirements: estimate,
      feasibility,
      recommendations,
    };

    return {
      content: [{
        type: 'text',
        text: JSON.stringify(result, null, 2),
      }],
    };
  } catch (error) {
    return {
      content: [{
        type: 'text',
        text: JSON.stringify({
          error: 'Failed to estimate resources',
          message: error instanceof Error ? error.message : 'Unknown error',
        }, null, 2),
      }],
      isError: true,
    };
  }
}

/**
 * Estimate resources from description
 */
function estimateFromDescription(
  description: string,
  dataSizeBytes: number | undefined,
  computeType: string,
  _topology: any,
  capabilities: any
) {
  const lowerDesc = description.toLowerCase();

  // Detect workload characteristics from description
  const isMachineLearning = /\b(ml|machine learning|train|model|neural|deep learning)\b/.test(lowerDesc);
  const isLargeScale = /\b(large|massive|big|petabyte|terabyte|billion|million)\b/.test(lowerDesc);
  const isRealTime = /\b(real-time|streaming|live|online)\b/.test(lowerDesc);
  const requiresGPU = /\b(gpu|cuda|rapids|tensor)\b/.test(lowerDesc) || computeType === 'gpu';

  // Base estimates
  let executors: number;
  let coresPerExecutor: number;
  let memoryPerExecutorGB: number;
  let gpus: number | undefined;

  if (isMachineLearning) {
    // ML workloads typically need more memory and GPUs
    executors = requiresGPU ? Math.min(capabilities.hardware.gpuCount, 4) : 4;
    coresPerExecutor = 4;
    memoryPerExecutorGB = 16;
    gpus = requiresGPU ? executors : undefined;
  } else if (isLargeScale) {
    // Large-scale data processing
    executors = capabilities.spark.recommendedExecutors.cpuOnly;
    coresPerExecutor = capabilities.spark.recommendedExecutorCores;
    memoryPerExecutorGB = Math.max(8, capabilities.spark.recommendedExecutorMemoryGB);
  } else if (isRealTime) {
    // Streaming workloads
    executors = Math.ceil(capabilities.spark.recommendedExecutors.cpuOnly / 2);
    coresPerExecutor = 2;
    memoryPerExecutorGB = 4;
  } else {
    // General workload
    executors = Math.ceil(capabilities.spark.recommendedExecutors.cpuOnly / 2);
    coresPerExecutor = capabilities.spark.recommendedExecutorCores;
    memoryPerExecutorGB = 4;
  }

  // Adjust based on data size
  if (dataSizeBytes) {
    const dataGB = dataSizeBytes / (1024 * 1024 * 1024);

    // Ensure enough memory for data
    const totalDataMemoryGB = dataGB * 1.5; // 1.5x data size for overhead
    const memoryPerExecutor = totalDataMemoryGB / executors;

    if (memoryPerExecutor > memoryPerExecutorGB) {
      memoryPerExecutorGB = Math.ceil(memoryPerExecutor);
    }

    // Adjust executors if data is very large
    if (dataGB > 1000) { // > 1TB
      executors = Math.min(executors * 2, capabilities.spark.recommendedExecutors.cpuOnly);
    }
  }

  const totalMemory = memoryPerExecutorGB * executors;
  const totalCores = coresPerExecutor * executors;

  return {
    executors,
    coresPerExecutor,
    memoryPerExecutor: `${memoryPerExecutorGB}g`,
    totalMemory: formatBytes(totalMemory * 1024 * 1024 * 1024),
    totalCores,
    gpus,
  };
}

/**
 * Check if workload can run on current system
 */
function checkFeasibility(
  estimate: ReturnType<typeof estimateFromDescription>,
  _topology: any,
  capabilities: any
) {
  const limitations: string[] = [];
  let canRun = true;
  let reason = 'System has sufficient resources for this workload';

  // Check memory
  const requiredMemoryGB = estimate.executors * parseInt(estimate.memoryPerExecutor);
  const availableMemoryGB = capabilities.hardware.memoryGB;

  if (requiredMemoryGB > availableMemoryGB * 0.9) {
    canRun = false;
    limitations.push(`Insufficient memory: need ${requiredMemoryGB}GB, have ${availableMemoryGB}GB`);
    reason = 'System does not have enough memory';
  }

  // Check cores
  const requiredCores = estimate.totalCores;
  const availableCores = capabilities.hardware.cpuCores.physical;

  if (requiredCores > availableCores) {
    limitations.push(`Insufficient cores: need ${requiredCores}, have ${availableCores}`);
    if (canRun) {
      reason = 'May run but with reduced parallelism';
    }
  }

  // Check GPUs
  if (estimate.gpus) {
    const availableGPUs = capabilities.hardware.gpuCount;
    if (estimate.gpus > availableGPUs) {
      canRun = false;
      limitations.push(`Insufficient GPUs: need ${estimate.gpus}, have ${availableGPUs}`);
      reason = 'System does not have enough GPUs';
    }
  }

  return {
    canRun,
    reason,
    limitations: limitations.length > 0 ? limitations : undefined,
  };
}

/**
 * Generate recommendations
 */
function generateRecommendations(
  estimate: ReturnType<typeof estimateFromDescription>,
  feasibility: ReturnType<typeof checkFeasibility>,
  capabilities: any
): string[] {
  const recommendations: string[] = [];

  if (!feasibility.canRun) {
    recommendations.push('Consider reducing the workload size or splitting it into smaller jobs');
    recommendations.push('Use dynamic allocation to better manage resources');

    if (feasibility.limitations?.some(l => l.includes('memory'))) {
      recommendations.push('Reduce executor memory or number of executors');
      recommendations.push('Enable disk-based spilling for large datasets');
    }

    if (feasibility.limitations?.some(l => l.includes('GPU'))) {
      recommendations.push('Consider using CPU-only execution');
      recommendations.push('Schedule job when more GPUs are available');
    }
  } else {
    // System can run the workload
    recommendations.push('System resources are sufficient for this workload');

    if (estimate.gpus && capabilities.gpu.rapidsAcceleration) {
      recommendations.push('Enable RAPIDS for GPU acceleration');
    }

    if (capabilities.hardware.hasNVMe) {
      recommendations.push('Configure local storage on NVMe for optimal I/O performance');
    }

    if (capabilities.hardware.hasInfiniBand) {
      recommendations.push('Leverage InfiniBand for fast shuffle operations');
    }
  }

  return recommendations;
}
