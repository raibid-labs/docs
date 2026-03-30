/**
 * Main Spark configuration optimizer
 */

import {
  SparkConfig,
  SparkConfigRequest,
  OptimizationResult,
  ExecutorConfig,
  DriverConfig,
  ShuffleConfig,
  GPUConfig,
  DynamicAllocationConfig,
  OptimizationConfig,
} from '../types/spark-config.js';
import { calculateMemoryConfig, parseSize } from './memory.js';
import {
  calculateExecutorResources,
  calculateDynamicAllocation,
  calculatePartitionCount,
} from './executor.js';

/**
 * Generate optimal Spark configuration
 */
export async function generateConfig(
  request: SparkConfigRequest
): Promise<OptimizationResult> {
  const rationale: string[] = [];

  // Parse data size
  const dataSizeBytes = parseSize(request.dataSize);
  const dataSizeGB = dataSizeBytes / (1024 ** 3);

  rationale.push(`Optimizing for ${request.workloadType} workload with ${dataSizeGB.toFixed(2)}GB of data.`);

  // Determine hardware context
  const totalMemory = request.totalMemory ?? 512; // Default to 512GB for DGX
  const totalCores = request.totalCores ?? 96; // Default to 96 cores for DGX
  const gpuCount = request.gpuCount ?? 0;

  // Calculate executor resources
  const executorResources = await calculateExecutorResources({
    totalMemory,
    totalCores,
    gpuCount,
    workloadType: request.workloadType,
    constraints: request.constraints,
  });

  rationale.push(...executorResources.rationale);

  // Calculate partition count
  const partitionCount = calculatePartitionCount(
    dataSizeGB,
    executorResources.executorCount,
    executorResources.executorCores
  );

  // Calculate memory configuration
  const memoryConfig = await calculateMemoryConfig({
    dataSize: dataSizeBytes,
    partitionCount,
    executorCount: executorResources.executorCount,
    totalMemoryGB: totalMemory,
    workloadType: request.workloadType,
  });

  rationale.push(...memoryConfig.recommendations);

  // Build executor configuration
  const executorConfig: ExecutorConfig = {
    memory: `${memoryConfig.executorMemoryGB}g`,
    memoryOverhead: `${memoryConfig.memoryOverheadGB}g`,
    cores: executorResources.executorCores,
    instances: executorResources.executorCount,
    offHeapMemory: memoryConfig.offHeapMemoryGB
      ? `${memoryConfig.offHeapMemoryGB}g`
      : undefined,
  };

  // Build driver configuration
  const driverConfig: DriverConfig = {
    memory: `${memoryConfig.driverMemoryGB}g`,
    memoryOverhead: `${Math.ceil(memoryConfig.driverMemoryGB * 0.1)}g`,
    cores: Math.min(4, totalCores), // Driver usually doesn't need many cores
    maxResultSize: `${Math.ceil(memoryConfig.driverMemoryGB * 0.5)}g`,
  };

  // Build shuffle configuration
  const shuffleConfig = buildShuffleConfig(request.workloadType, partitionCount);

  // Build GPU configuration if GPUs are available
  const gpuConfig: GPUConfig | undefined = gpuCount > 0
    ? buildGPUConfig(request.workloadType, executorResources.gpuPerExecutor)
    : undefined;

  if (gpuConfig?.enabled) {
    rationale.push('GPU acceleration enabled with RAPIDS support.');
  }

  // Build dynamic allocation configuration
  const dynamicAllocation = await buildDynamicAllocationConfig(
    executorResources.executorCount,
    request.workloadType,
    request.constraints
  );

  // Build optimization configuration
  const optimizationConfig = buildOptimizationConfig(request.workloadType);

  // Assemble complete Spark configuration
  const config: SparkConfig = {
    executor: executorConfig,
    driver: driverConfig,
    shuffle: shuffleConfig,
    memory: memoryConfig.memoryConfig,
    gpu: gpuConfig,
    dynamicAllocation,
    optimization: optimizationConfig,
    serializer: 'org.apache.spark.serializer.KryoSerializer',
    kryo: {
      registrationRequired: false,
      referenceTracking: request.workloadType === 'ml-training',
    },
    locality: {
      wait: request.workloadType === 'streaming' ? '1s' : '3s',
    },
    speculation: {
      enabled: request.workloadType !== 'streaming',
      interval: '100ms',
      multiplier: 1.5,
    },
  };

  // Estimate performance
  const estimatedPerformance = estimatePerformance(
    config,
    dataSizeGB,
    executorResources,
    request.workloadType
  );

  rationale.push(
    `Estimated execution time: ${estimatedPerformance.executionTimeMinutes?.toFixed(1)} minutes.`
  );

  // Generate alternative configurations
  const alternatives = generateAlternatives(config, request);

  return {
    config,
    estimatedPerformance,
    rationale,
    alternatives,
  };
}

/**
 * Build shuffle configuration based on workload
 */
function buildShuffleConfig(workloadType: string, partitions: number): ShuffleConfig {
  return {
    partitions,
    compress: true,
    spill: true,
    spillCompress: true,
    memoryFraction: workloadType === 'ml-training' ? 0.3 : 0.2,
    fileBuffer: workloadType === 'streaming' ? '32k' : '64k',
    sort: {
      bypassMergeThreshold: 200,
    },
  };
}

/**
 * Build GPU configuration
 */
function buildGPUConfig(workloadType: string, gpuPerExecutor?: number): GPUConfig {
  const isMLWorkload = workloadType === 'ml-training' || workloadType === 'ml-inference';

  return {
    enabled: true,
    resourceName: 'nvidia.com/gpu',
    amount: gpuPerExecutor ?? 1,
    rapids: {
      enabled: isMLWorkload || workloadType === 'analytics',
      sqlEnabled: workloadType === 'analytics',
      memoryFraction: 0.9,
      pooling: {
        enabled: true,
        mode: 'ARENA',
      },
    },
  };
}

/**
 * Build dynamic allocation configuration
 */
async function buildDynamicAllocationConfig(
  baseExecutorCount: number,
  workloadType: string,
  constraints?: SparkConfigRequest['constraints']
): Promise<DynamicAllocationConfig> {
  const enableDynamicAllocation =
    constraints?.enableDynamicAllocation ??
    (workloadType !== 'streaming' && workloadType !== 'ml-training');

  if (!enableDynamicAllocation) {
    return { enabled: false };
  }

  const allocation = await calculateDynamicAllocation(baseExecutorCount, workloadType);

  return {
    enabled: true,
    initialExecutors: allocation.initialExecutors,
    minExecutors: allocation.minExecutors,
    maxExecutors: allocation.maxExecutors,
    executorIdleTimeout: '60s',
    cachedExecutorIdleTimeout: '300s',
    schedulerBacklogTimeout: '1s',
    sustainedSchedulerBacklogTimeout: '1s',
  };
}

/**
 * Build optimization configuration
 */
function buildOptimizationConfig(workloadType: string): OptimizationConfig {
  return {
    adaptiveExecution: {
      enabled: true,
      coalescePartitions: true,
      skewJoin: workloadType === 'analytics',
    },
    autoBroadcastJoinThreshold: workloadType === 'analytics' ? '10mb' : '5mb',
    broadcastTimeout: '300s',
    sql: {
      inMemoryColumnarStorageCompressed: true,
      inMemoryColumnarStorageBatchSize: 10000,
    },
  };
}

/**
 * Estimate performance metrics
 */
function estimatePerformance(
  config: SparkConfig,
  dataSizeGB: number,
  executorResources: any,
  workloadType: string
): OptimizationResult['estimatedPerformance'] {
  // Simple performance model
  // More sophisticated models would use historical data

  const totalCores = executorResources.totalExecutorCores;
  const hasGPU = config.gpu?.enabled ?? false;

  // Base throughput in MB/s per core
  let throughputPerCore = 50; // Conservative estimate

  if (hasGPU) {
    throughputPerCore *= 3; // GPUs can significantly accelerate certain workloads
  }

  if (workloadType === 'ml-training') {
    throughputPerCore *= 0.5; // ML training is compute-intensive
  } else if (workloadType === 'etl') {
    throughputPerCore *= 1.5; // ETL is often I/O bound, benefits from parallelism
  }

  const totalThroughputMBps = throughputPerCore * totalCores;
  const dataSizeMB = dataSizeGB * 1024;
  const executionTimeMinutes = (dataSizeMB / totalThroughputMBps) / 60;

  // Resource efficiency (0-1)
  const memoryEfficiency = Math.min(1, executorResources.totalExecutorMemoryGB / (dataSizeGB * 2));
  const coreEfficiency = Math.min(1, totalCores / (dataSizeGB / 10)); // Rough heuristic
  const resourceEfficiency = (memoryEfficiency + coreEfficiency) / 2;

  return {
    executionTimeMinutes,
    throughputMBps: totalThroughputMBps,
    resourceEfficiency,
  };
}

/**
 * Generate alternative configurations
 */
function generateAlternatives(
  baseConfig: SparkConfig,
  _request: SparkConfigRequest
): OptimizationResult['alternatives'] {
  const alternatives: OptimizationResult['alternatives'] = [];

  // Alternative 1: More executors, less memory each
  if (baseConfig.executor.instances && baseConfig.executor.instances > 2) {
    const moreExecutorsConfig = JSON.parse(JSON.stringify(baseConfig)) as SparkConfig;
    const currentInstances = baseConfig.executor.instances;
    moreExecutorsConfig.executor.instances = Math.ceil(currentInstances * 1.5);
    moreExecutorsConfig.executor.memory = reduceMemory(baseConfig.executor.memory, 0.67);

    alternatives.push({
      config: moreExecutorsConfig,
      tradeoff: 'More executors with less memory each - better for smaller tasks, may increase overhead',
    });
  }

  // Alternative 2: Disable dynamic allocation for predictable performance
  if (baseConfig.dynamicAllocation.enabled) {
    const staticConfig = JSON.parse(JSON.stringify(baseConfig)) as SparkConfig;
    staticConfig.dynamicAllocation.enabled = false;

    alternatives.push({
      config: staticConfig,
      tradeoff: 'Static allocation - more predictable performance, but less flexible resource usage',
    });
  }

  return alternatives;
}

/**
 * Helper to reduce memory string
 */
function reduceMemory(memory: string, factor: number): string {
  const match = memory.match(/^(\d+)([gmk])$/i);
  if (!match || !match[1] || !match[2]) return memory;

  const value = parseInt(match[1], 10);
  const unit = match[2];
  return `${Math.ceil(value * factor)}${unit}`;
}

/**
 * Convert Spark config to command-line arguments
 */
export function configToSparkSubmitArgs(config: SparkConfig): string[] {
  const args: string[] = [];

  // Executor configuration
  args.push(`--executor-memory`, config.executor.memory);
  args.push(`--executor-cores`, config.executor.cores.toString());
  if (config.executor.instances) {
    args.push(`--num-executors`, config.executor.instances.toString());
  }

  // Driver configuration
  args.push(`--driver-memory`, config.driver.memory);
  args.push(`--driver-cores`, config.driver.cores.toString());

  // Configuration properties
  const props: string[] = [];

  props.push(`spark.executor.memoryOverhead=${config.executor.memoryOverhead}`);
  props.push(`spark.driver.memoryOverhead=${config.driver.memoryOverhead}`);
  props.push(`spark.memory.fraction=${config.memory.fraction}`);
  props.push(`spark.memory.storageFraction=${config.memory.storageFraction}`);
  props.push(`spark.sql.shuffle.partitions=${config.shuffle.partitions}`);
  props.push(`spark.shuffle.compress=${config.shuffle.compress}`);

  if (config.gpu?.enabled) {
    props.push(`spark.rapids.sql.enabled=${config.gpu.rapids?.sqlEnabled ?? false}`);
  }

  props.forEach(prop => {
    args.push('--conf', prop);
  });

  return args;
}
