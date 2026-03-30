/**
 * Performance prediction model for Spark jobs
 */

import { SparkConfig } from '../types/spark-config.js';
import { HardwareContext, PerformanceMetrics } from '../types/estimation.js';

export interface PerformancePredictionRequest {
  config: SparkConfig;
  hardware: HardwareContext;
  dataSize?: number; // bytes
  workloadType?: string;
}

export interface PerformancePredictionResult {
  metrics: PerformanceMetrics;
  predictions: {
    estimatedThroughputMBps: number;
    estimatedLatencyMs: number;
    resourceUtilization: {
      cpu: number;
      memory: number;
      gpu?: number;
      io: number;
    };
  };
  bottlenecks: string[];
  recommendations: string[];
  confidence: number;
}

/**
 * Predict performance for a given Spark configuration
 */
export async function predictPerformance(
  request: PerformancePredictionRequest
): Promise<PerformancePredictionResult> {
  const { config, hardware, dataSize = 0, workloadType = 'mixed' } = request;

  // Calculate resource utilization
  const resourceUtilization = calculateResourceUtilization(config, hardware);

  // Calculate efficiencies
  const cpuEfficiency = calculateCPUEfficiency(config, hardware, workloadType);
  const memoryEfficiency = calculateMemoryEfficiency(config, hardware, dataSize);
  const ioEfficiency = calculateIOEfficiency(config, workloadType);
  const gpuEfficiency = config.gpu?.enabled
    ? calculateGPUEfficiency(config, workloadType)
    : undefined;

  // Calculate throughput
  const estimatedThroughputMBps = calculateThroughput(
    config,
    hardware,
    workloadType,
    cpuEfficiency,
    gpuEfficiency
  );

  // Calculate latency (inverse of throughput, adjusted)
  const estimatedLatencyMs = dataSize > 0
    ? (dataSize / (1024 ** 2)) / estimatedThroughputMBps * 1000
    : 0;

  // Calculate overall score
  const overallScore = calculateOverallScore(
    cpuEfficiency,
    memoryEfficiency,
    ioEfficiency,
    gpuEfficiency
  );

  const metrics: PerformanceMetrics = {
    throughputMBps: estimatedThroughputMBps,
    recordsPerSecond: estimatedThroughputMBps * 1000, // Rough estimate
    cpuEfficiency,
    memoryEfficiency,
    ioEfficiency,
    gpuEfficiency,
    overallScore,
  };

  // Detect bottlenecks
  const bottlenecks = detectBottlenecks(
    config,
    hardware,
    resourceUtilization,
    metrics
  );

  // Generate recommendations
  const recommendations = generatePerformanceRecommendations(
    config,
    metrics,
    bottlenecks
  );

  // Calculate confidence
  const confidence = 0.7; // Base confidence for model-based predictions

  return {
    metrics,
    predictions: {
      estimatedThroughputMBps,
      estimatedLatencyMs,
      resourceUtilization,
    },
    bottlenecks,
    recommendations,
    confidence,
  };
}

/**
 * Calculate resource utilization percentages
 */
function calculateResourceUtilization(
  config: SparkConfig,
  hardware: HardwareContext
): PerformancePredictionResult['predictions']['resourceUtilization'] {
  // CPU utilization
  const requestedCores = (config.executor.instances ?? 1) * config.executor.cores + config.driver.cores;
  const cpu = Math.min((requestedCores / hardware.cpuCores) * 100, 100);

  // Memory utilization
  const executorMemGB = parseMemory(config.executor.memory);
  const driverMemGB = parseMemory(config.driver.memory);
  const totalRequestedMem = (executorMemGB * (config.executor.instances ?? 1)) + driverMemGB;
  const memory = Math.min((totalRequestedMem / hardware.totalMemory) * 100, 100);

  // GPU utilization
  const gpu = config.gpu?.enabled && hardware.gpuCount
    ? Math.min(((config.gpu.amount ?? 1) * (config.executor.instances ?? 1) / hardware.gpuCount) * 100, 100)
    : undefined;

  // I/O utilization (estimated based on workload)
  const io = 50; // Placeholder - would need actual I/O metrics

  return { cpu, memory, gpu, io };
}

/**
 * Calculate CPU efficiency
 */
function calculateCPUEfficiency(
  config: SparkConfig,
  hardware: HardwareContext,
  workloadType: string
): number {
  const executorCores = config.executor.cores;
  const executorCount = config.executor.instances ?? 1;

  // Ideal executor cores: 4-6 for most workloads
  const idealCores = workloadType === 'ml-training' ? 8 : 5;
  const coreEfficiency = 1 - Math.abs(executorCores - idealCores) / idealCores * 0.3;

  // Parallelism efficiency
  const totalCores = executorCores * executorCount;
  const parallelismEfficiency = Math.min(totalCores / hardware.cpuCores, 1);

  // Dynamic allocation efficiency
  const dynamicEfficiency = config.dynamicAllocation.enabled ? 1.1 : 1.0;

  return Math.min(coreEfficiency * parallelismEfficiency * dynamicEfficiency, 1);
}

/**
 * Calculate memory efficiency
 */
function calculateMemoryEfficiency(
  config: SparkConfig,
  _hardware: HardwareContext,
  dataSize: number
): number {
  const executorMemGB = parseMemory(config.executor.memory);
  const totalExecutorMem = executorMemGB * (config.executor.instances ?? 1);

  const dataSizeGB = dataSize / (1024 ** 3);

  // Memory should be 2-4x data size for optimal performance
  const idealMemory = dataSizeGB * 3;

  if (totalExecutorMem < dataSizeGB) {
    return 0.3; // Severe under-provisioning
  } else if (totalExecutorMem < idealMemory * 0.5) {
    return 0.6; // Under-provisioned
  } else if (totalExecutorMem > idealMemory * 2) {
    return 0.7; // Over-provisioned (waste)
  } else {
    return 0.95; // Good balance
  }
}

/**
 * Calculate I/O efficiency
 */
function calculateIOEfficiency(config: SparkConfig, workloadType: string): number {
  let efficiency = 0.8; // Base efficiency

  // Shuffle configuration impact
  if (config.shuffle.compress) efficiency += 0.05;
  if (config.shuffle.spill) efficiency += 0.05;

  // Adaptive execution
  if (config.optimization.adaptiveExecution?.enabled) {
    efficiency += 0.1;
  }

  // Workload-specific adjustments
  if (workloadType === 'etl' || workloadType === 'streaming') {
    efficiency *= 1.1; // These workloads benefit from good I/O config
  }

  return Math.min(efficiency, 1);
}

/**
 * Calculate GPU efficiency
 */
function calculateGPUEfficiency(config: SparkConfig, workloadType: string): number {
  if (!config.gpu?.enabled) return 0;

  let efficiency = 0.7; // Base GPU efficiency

  // RAPIDS optimization
  if (config.gpu.rapids?.enabled) {
    efficiency += 0.15;
  }

  if (config.gpu.rapids?.pooling?.enabled) {
    efficiency += 0.1;
  }

  // Workload suitability
  if (workloadType === 'ml-training' || workloadType === 'ml-inference') {
    efficiency *= 1.2;
  } else if (workloadType === 'analytics') {
    efficiency *= 1.1;
  }

  return Math.min(efficiency, 1);
}

/**
 * Calculate throughput
 */
function calculateThroughput(
  config: SparkConfig,
  _hardware: HardwareContext,
  workloadType: string,
  cpuEfficiency: number,
  gpuEfficiency?: number
): number {
  const totalCores = config.executor.cores * (config.executor.instances ?? 1);

  // Base throughput per core (MB/s)
  let throughputPerCore = 50;

  // Adjust for workload
  const workloadMultipliers: Record<string, number> = {
    'etl': 1.5,
    'analytics': 1.0,
    'ml-training': 0.4,
    'ml-inference': 1.2,
    'streaming': 1.3,
  };

  throughputPerCore *= workloadMultipliers[workloadType] ?? 1.0;

  // Apply CPU efficiency
  throughputPerCore *= cpuEfficiency;

  // GPU acceleration
  if (gpuEfficiency && gpuEfficiency > 0) {
    throughputPerCore *= (1 + gpuEfficiency * 2);
  }

  return throughputPerCore * totalCores;
}

/**
 * Calculate overall performance score
 */
function calculateOverallScore(
  cpuEfficiency: number,
  memoryEfficiency: number,
  ioEfficiency: number,
  gpuEfficiency?: number
): number {
  const weights = {
    cpu: 0.35,
    memory: 0.3,
    io: 0.25,
    gpu: 0.1,
  };

  let score =
    cpuEfficiency * weights.cpu +
    memoryEfficiency * weights.memory +
    ioEfficiency * weights.io;

  if (gpuEfficiency !== undefined) {
    score += gpuEfficiency * weights.gpu;
  }

  return Math.round(score * 100);
}

/**
 * Detect performance bottlenecks
 */
function detectBottlenecks(
  config: SparkConfig,
  _hardware: HardwareContext,
  utilization: any,
  metrics: PerformanceMetrics
): string[] {
  const bottlenecks: string[] = [];

  // CPU bottleneck
  if (utilization.cpu > 90) {
    bottlenecks.push('CPU: High CPU utilization (>90%). Consider adding more executors or reducing cores per executor.');
  }

  // Memory bottleneck
  if (utilization.memory > 85) {
    bottlenecks.push('Memory: High memory utilization (>85%). Risk of OOM errors. Increase executor memory or add more executors.');
  } else if (metrics.memoryEfficiency < 0.5) {
    bottlenecks.push('Memory: Under-provisioned memory. Performance may suffer from excessive spilling.');
  }

  // GPU bottleneck
  if (utilization.gpu && utilization.gpu > 95) {
    bottlenecks.push('GPU: GPU fully utilized. Consider adding more GPUs or optimizing GPU operations.');
  }

  // I/O bottleneck
  if (metrics.ioEfficiency < 0.6) {
    bottlenecks.push('I/O: Poor I/O efficiency. Consider using better file formats, enabling compression, or increasing I/O parallelism.');
  }

  // Shuffle bottleneck
  if (config.shuffle.partitions < (config.executor.cores * (config.executor.instances ?? 1)) * 2) {
    bottlenecks.push('Shuffle: Too few shuffle partitions. Increase to at least 2-3x total cores.');
  }

  return bottlenecks;
}

/**
 * Generate performance recommendations
 */
function generatePerformanceRecommendations(
  config: SparkConfig,
  metrics: PerformanceMetrics,
  _bottlenecks: string[]
): string[] {
  const recommendations: string[] = [];

  // Overall score recommendations
  if (metrics.overallScore < 60) {
    recommendations.push('Overall performance score is low. Review all configuration parameters.');
  }

  // Efficiency-based recommendations
  if (metrics.cpuEfficiency < 0.7) {
    recommendations.push('CPU efficiency is low. Adjust executor cores and count for better CPU utilization.');
  }

  if (metrics.memoryEfficiency < 0.7) {
    recommendations.push('Memory efficiency is low. Review memory allocation and consider caching frequently used data.');
  }

  if (metrics.ioEfficiency < 0.7) {
    recommendations.push('I/O efficiency is low. Use columnar formats (Parquet), enable compression, and optimize shuffle operations.');
  }

  if (metrics.gpuEfficiency && metrics.gpuEfficiency < 0.6) {
    recommendations.push('GPU efficiency is low. Enable RAPIDS acceleration and optimize data transfer to/from GPU.');
  }

  // Configuration-specific recommendations
  if (!config.optimization.adaptiveExecution?.enabled) {
    recommendations.push('Enable Adaptive Query Execution (AQE) for better query optimization.');
  }

  if (config.serializer !== 'org.apache.spark.serializer.KryoSerializer') {
    recommendations.push('Use Kryo serialization for better performance.');
  }

  if (!config.memory.offHeap.enabled) {
    recommendations.push('Consider enabling off-heap memory to reduce GC pressure.');
  }

  return recommendations;
}

/**
 * Parse memory string to GB
 */
function parseMemory(memory: string): number {
  const match = memory.match(/^(\d+)([gmk])$/i);
  if (!match) return 0;

  const value = parseInt(match?.[1] ?? "0");
  const unit = match?.[2] ?? "g".toLowerCase();

  switch (unit) {
    case 'g': return value;
    case 'm': return value / 1024;
    case 'k': return value / (1024 * 1024);
    default: return value;
  }
}
