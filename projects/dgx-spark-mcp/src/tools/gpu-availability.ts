/**
 * GPU availability checker tool
 * Checks current GPU availability and provides recommendations
 */

import { detectGPUs } from '../hardware/detector.js';
import type { CheckGPUAvailabilityArgs, ToolCallResponse } from '../types/tools.js';
import type { GPU } from '../types/gpu.js';

export interface SimplifiedGPU {
  id: number;
  name: string;
  uuid: string;
  memory: {
    total: number;
    totalGB: number;
    free: number;
    freeGB: string;
    used: number;
    usedGB: string;
  };
  utilization: {
    gpu: number;
    memory: number;
  };
  temperature: {
    current: number;
    max: number;
  };
  power: {
    current: number;
    limit: number;
  };
}

export interface GPUAvailabilityResult {
  totalGPUs: number;
  availableGPUs: SimplifiedGPU[];
  busyGPUs: SimplifiedGPU[];
  recommendations: {
    availableForJob: number;
    recommendedGPUs: number[];
    reason: string;
  };
  summary: string;
}
/**
 * Check GPU availability
 */
export async function checkGPUAvailability(args?: CheckGPUAvailabilityArgs): Promise<ToolCallResponse> {
  try {
    // Detect GPUs
    const gpuResult = await detectGPUs(true);
    const gpus = gpuResult.gpus;

    if (!gpus || gpus.length === 0) {
      return {
        content: [{
          type: 'text',
          text: JSON.stringify({
            error: 'No GPUs detected on this system',
            totalGPUs: 0,
            availableGPUs: [],
            busyGPUs: [],
          }, null, 2),
        }],
      };
    }

    // Determine availability based on utilization and memory
    const minMemoryGB = args?.minMemoryGB || 0;
    const minUtilization = args?.minUtilization ?? 20; // Default: GPUs with < 20% util are available

    const availableGPUs: GPU[] = [];
    const busyGPUs: GPU[] = [];

    for (const gpu of gpus) {
      const memoryGB = gpu.memory.free / (1024 * 1024 * 1024);
      const isAvailable =
        gpu.utilization.gpu < minUtilization &&
        memoryGB >= minMemoryGB;

      if (isAvailable) {
        availableGPUs.push(gpu);
      } else {
        busyGPUs.push(gpu);
      }
    }

    // Generate recommendations
    const recommendations = generateRecommendations(
      availableGPUs,
      busyGPUs,
      minMemoryGB
    );

    const summary = `${availableGPUs.length} of ${gpus.length} GPUs available for new jobs. ` +
      `Recommended to use ${recommendations.recommendedGPUs.length} GPU(s) for optimal performance.`;

    const result: GPUAvailabilityResult = {
      totalGPUs: gpus.length,
      availableGPUs: availableGPUs.map(simplifyGPU),
      busyGPUs: busyGPUs.map(simplifyGPU),
      recommendations,
      summary,
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
          error: 'Failed to check GPU availability',
          message: error instanceof Error ? error.message : 'Unknown error',
        }, null, 2),
      }],
      isError: true,
    };
  }
}

/**
 * Simplify GPU object for output
 */
function simplifyGPU(gpu: GPU) {
  return {
    id: gpu.id,
    name: gpu.name,
    uuid: gpu.uuid,
    memory: {
      total: gpu.memory.total,
      totalGB: Math.round(gpu.memory.total / (1024 * 1024 * 1024)),
      free: gpu.memory.free,
      freeGB: (gpu.memory.free / (1024 * 1024 * 1024)).toFixed(2),
      used: gpu.memory.used,
      usedGB: (gpu.memory.used / (1024 * 1024 * 1024)).toFixed(2),
    },
    utilization: gpu.utilization,
    temperature: {
      current: gpu.temperature.current,
      max: gpu.temperature.max,
    },
    power: {
      current: gpu.power.current,
      limit: gpu.power.limit,
    },
  };
}

/**
 * Generate GPU recommendations
 */
function generateRecommendations(
  availableGPUs: GPU[],
  busyGPUs: GPU[],
  minMemoryGB: number
): GPUAvailabilityResult['recommendations'] {
  if (availableGPUs.length === 0) {
    // Try to find least busy GPU
    if (busyGPUs.length > 0) {
      const leastBusy = busyGPUs.reduce((prev, curr) =>
        curr.utilization.gpu < prev.utilization.gpu ? curr : prev
      );

      return {
        availableForJob: 0,
        recommendedGPUs: [leastBusy.id],
        reason: `No fully available GPUs. GPU ${leastBusy.id} is least busy with ${leastBusy.utilization.gpu}% utilization. ` +
          `Consider waiting or using this GPU with caution.`,
      };
    }

    return {
      availableForJob: 0,
      recommendedGPUs: [],
      reason: 'No GPUs available',
    };
  }

  // Sort available GPUs by free memory (descending)
  const sortedByMemory = [...availableGPUs].sort((a, b) =>
    b.memory.free - a.memory.free
  );

  // Recommend GPUs based on memory requirements
  const recommendedGPUs: number[] = [];
  for (const gpu of sortedByMemory) {
    const freeGB = gpu.memory.free / (1024 * 1024 * 1024);
    if (freeGB >= minMemoryGB) {
      recommendedGPUs.push(gpu.id);
    }
  }

  // If no GPUs meet memory requirement, recommend those with most memory
  if (recommendedGPUs.length === 0 && minMemoryGB > 0) {
    if (sortedByMemory[0]) {
      recommendedGPUs.push(sortedByMemory[0].id);
    }
    return {
      availableForJob: availableGPUs.length,
      recommendedGPUs,
      reason: sortedByMemory[0] 
        ? `No GPUs have ${minMemoryGB}GB free memory. GPU ${sortedByMemory[0].id} has the most free memory (${(sortedByMemory[0].memory.free / (1024 * 1024 * 1024)).toFixed(2)}GB).`
        : "No GPUs available with sufficient memory",
    };
  }

  // Default: recommend all available GPUs
  const reason = minMemoryGB > 0
    ? `${recommendedGPUs.length} GPU(s) available with at least ${minMemoryGB}GB free memory`
    : `${availableGPUs.length} GPU(s) available with low utilization`;

  return {
    availableForJob: availableGPUs.length,
    recommendedGPUs: recommendedGPUs.length > 0 ? recommendedGPUs : availableGPUs.map(g => g.id),
    reason,
  };
}
