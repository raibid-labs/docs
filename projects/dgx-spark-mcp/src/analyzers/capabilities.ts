/**
 * System capabilities analyzer
 * Analyzes hardware topology and determines system capabilities
 */

import { getHardwareSnapshot } from '../hardware/topology.js';
import type { SystemTopology } from '../types/topology.js';

export interface SystemCapabilitiesAnalysis {
  hardware: {
    cpuCores: {
      physical: number;
      logical: number;
    };
    memoryGB: number;
    gpuCount: number;
    gpuTotalMemoryGB: number;
    storageGB: number;
    hasNVLink: boolean;
    hasInfiniBand: boolean;
    hasNVMe: boolean;
  };
  spark: {
    maxConcurrentJobs: number;
    recommendedExecutors: {
      cpuOnly: number;
      withGPU: number;
    };
    recommendedExecutorCores: number;
    recommendedExecutorMemoryGB: number;
    maxShufflePartitions: number;
  };
  gpu: {
    available: boolean;
    rapidsAcceleration: boolean;
    recommendedGPUsPerExecutor: number;
    maxConcurrentGPUJobs: number;
  };
  frameworks: {
    spark: boolean;
    rapids: boolean;
    tensorflow: boolean;
    pytorch: boolean;
  };
  performance: {
    expectedDataThroughputGBps: number;
    expectedComputePerformanceTFLOPS?: number;
    networkBandwidthGbps: number;
    storageBandwidthGBps: number;
  };
  recommendations: string[];
}

/**
 * Analyze system capabilities based on hardware topology
 */
export async function analyzeCapabilities(): Promise<SystemCapabilitiesAnalysis> {
  const snapshot = await getHardwareSnapshot({ useCache: true });
  const { topology } = snapshot;

  // Hardware analysis
  const hardware = analyzeHardware(topology);

  // Spark capabilities
  const spark = analyzeSparkCapabilities(topology, hardware);

  // GPU capabilities
  const gpu = analyzeGPUCapabilities(topology);

  // Framework support
  const frameworks = analyzeFrameworkSupport(topology);

  // Performance estimates
  const performance = estimatePerformance(topology, hardware);

  // Generate recommendations
  const recommendations = generateRecommendations(topology, spark, gpu);

  return {
    hardware,
    spark,
    gpu,
    frameworks,
    performance,
    recommendations,
  };
}

/**
 * Analyze hardware specs
 */
function analyzeHardware(topology: SystemTopology) {
  const memoryGB = Math.round(topology.memory.info.total / (1024 * 1024 * 1024));
  const storageGB = Math.round(topology.storage.totalCapacity / (1024 * 1024 * 1024));

  const gpuCount = topology.gpus?.length || 0;
  const gpuTotalMemoryGB = topology.gpus?.reduce((sum, gpu) => {
    return sum + gpu.memory.total / (1024 * 1024 * 1024);
  }, 0) || 0;

  const hasNVLink = topology.gpus?.some(gpu =>
    gpu.nvlinks && gpu.nvlinks.length > 0
  ) || false;

  return {
    cpuCores: topology.cpu.cores,
    memoryGB,
    gpuCount,
    gpuTotalMemoryGB: Math.round(gpuTotalMemoryGB),
    storageGB,
    hasNVLink,
    hasInfiniBand: topology.capabilities.hasInfiniBand,
    hasNVMe: topology.capabilities.hasNVMe,
  };
}

/**
 * Analyze Spark capabilities
 */
function analyzeSparkCapabilities(topology: SystemTopology, hardware: ReturnType<typeof analyzeHardware>) {
  const physicalCores = topology.cpu.cores.physical;
  const memoryGB = hardware.memoryGB;
  const gpuCount = hardware.gpuCount;

  // Reserve 1 core and 2GB for system
  const availableCores = Math.max(1, physicalCores - 1);
  const availableMemoryGB = Math.max(4, memoryGB - 2);

  // Recommended executor cores (typically 4-5 for best performance)
  const recommendedExecutorCores = Math.min(5, Math.max(2, Math.floor(availableCores / 4)));

  // Calculate recommended executors
  const maxExecutorsCPU = Math.floor(availableCores / recommendedExecutorCores);
  const maxExecutorsMemory = Math.floor(availableMemoryGB / 4); // Assume 4GB per executor minimum
  const recommendedExecutorsCPU = Math.min(maxExecutorsCPU, maxExecutorsMemory);

  // With GPU, typically 1 executor per GPU
  const recommendedExecutorsGPU = gpuCount > 0 ? gpuCount : recommendedExecutorsCPU;

  // Memory per executor (leave 10% overhead)
  const recommendedExecutorMemoryGB = Math.floor((availableMemoryGB / recommendedExecutorsCPU) * 0.9);

  // Shuffle partitions (typically 2x cores)
  const maxShufflePartitions = availableCores * 2;

  // Max concurrent jobs (conservative estimate)
  const maxConcurrentJobs = Math.max(1, Math.floor(recommendedExecutorsCPU / 2));

  return {
    maxConcurrentJobs,
    recommendedExecutors: {
      cpuOnly: recommendedExecutorsCPU,
      withGPU: recommendedExecutorsGPU,
    },
    recommendedExecutorCores,
    recommendedExecutorMemoryGB,
    maxShufflePartitions,
  };
}

/**
 * Analyze GPU capabilities
 */
function analyzeGPUCapabilities(topology: SystemTopology) {
  const gpuCount = topology.gpus?.length || 0;
  const available = gpuCount > 0;

  // Check for RAPIDS acceleration support (compute capability >= 6.0)
  const rapidsAcceleration = topology.gpus?.every(gpu =>
    gpu.computeCapability.major >= 6
  ) || false;

  // Recommended GPUs per executor (typically 1)
  const recommendedGPUsPerExecutor = 1;

  // Max concurrent GPU jobs (one per GPU, conservative)
  const maxConcurrentGPUJobs = gpuCount;

  return {
    available,
    rapidsAcceleration,
    recommendedGPUsPerExecutor,
    maxConcurrentGPUJobs,
  };
}

/**
 * Analyze framework support
 */
function analyzeFrameworkSupport(topology: SystemTopology) {
  const hasGPU = (topology.gpus?.length || 0) > 0;
  const hasModernGPU = topology.gpus?.every(gpu =>
    gpu.computeCapability.major >= 6
  ) || false;

  return {
    spark: true, // Always supported
    rapids: hasGPU && hasModernGPU,
    tensorflow: hasGPU,
    pytorch: hasGPU,
  };
}

/**
 * Estimate performance characteristics
 */
function estimatePerformance(topology: SystemTopology, _hardware: ReturnType<typeof analyzeHardware>) {
  // Network bandwidth estimate
  const hasInfiniBand = topology.capabilities.hasInfiniBand;
  const networkBandwidthGbps = hasInfiniBand ? 200 : 10; // IB HDR or 10GbE

  // Storage bandwidth estimate
  const hasNVMe = topology.capabilities.hasNVMe;
  const storageBandwidthGBps = hasNVMe ? 3.5 : 0.5; // NVMe or SATA SSD

  // Data throughput (limited by slowest component)
  const expectedDataThroughputGBps = Math.min(
    networkBandwidthGbps / 8, // Convert Gbps to GBps
    storageBandwidthGBps
  );

  // Compute performance (if GPUs available)
  let expectedComputePerformanceTFLOPS: number | undefined;
  if (topology.gpus && topology.gpus.length > 0) {
    // Rough estimate: modern datacenter GPU ~100 TFLOPS FP16
    expectedComputePerformanceTFLOPS = topology.gpus.length * 100;
  }

  return {
    expectedDataThroughputGBps,
    expectedComputePerformanceTFLOPS,
    networkBandwidthGbps,
    storageBandwidthGBps,
  };
}

/**
 * Generate recommendations
 */
function generateRecommendations(
  topology: SystemTopology,
  spark: ReturnType<typeof analyzeSparkCapabilities>,
  gpu: ReturnType<typeof analyzeGPUCapabilities>
): string[] {
  const recommendations: string[] = [];

  // GPU recommendations
  if (gpu.available) {
    if (gpu.rapidsAcceleration) {
      recommendations.push('RAPIDS acceleration available - consider using GPU-accelerated Spark for ETL workloads');
    }
    recommendations.push(`Configure ${gpu.recommendedGPUsPerExecutor} GPU per Spark executor for optimal performance`);
  } else {
    recommendations.push('No GPUs detected - CPU-only Spark configuration recommended');
  }

  // Memory recommendations
  const memoryGB = Math.round(topology.memory.info.total / (1024 * 1024 * 1024));
  if (memoryGB < 64) {
    recommendations.push('Consider adding more RAM for larger datasets and better caching');
  }

  // Executor recommendations
  recommendations.push(`Use ${spark.recommendedExecutors.cpuOnly} executors with ${spark.recommendedExecutorCores} cores each for CPU workloads`);
  if (gpu.available) {
    recommendations.push(`Use ${spark.recommendedExecutors.withGPU} executors for GPU-accelerated workloads`);
  }

  // Network recommendations
  if (topology.capabilities.hasInfiniBand) {
    recommendations.push('InfiniBand detected - excellent for distributed shuffle operations');
  }

  // Storage recommendations
  if (topology.capabilities.hasNVMe) {
    recommendations.push('NVMe storage detected - configure Spark to use local storage for shuffle');
  }

  // NVLink recommendations
  const hasNVLink = topology.gpus?.some(gpu =>
    gpu.nvlinks && gpu.nvlinks.length > 0
  ) || false;

  if (hasNVLink) {
    recommendations.push('NVLink detected - enable GPU-to-GPU direct transfers for multi-GPU jobs');
  }

  return recommendations;
}
