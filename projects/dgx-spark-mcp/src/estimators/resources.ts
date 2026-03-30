/**
 * Resource estimation engine for Spark jobs
 */

import {
  ResourceEstimationRequest,
  ResourceEstimate,
  MemoryEstimate,
  ComputeEstimate,
  StorageEstimate,
  TimeEstimate,
} from '../types/estimation.js';
import { classifyWorkload } from '../analyzers/workload.js';
// import { analyzeIOPattern } from '../analyzers/io-pattern.js';
import { parseSize } from '../optimizers/memory.js';

/**
 * Estimate resources required for a Spark job
 */
export async function estimateResources(
  request: ResourceEstimationRequest
): Promise<ResourceEstimate> {
  const assumptions: string[] = [];

  // Parse data size
  let dataSizeBytes = 0;
  if (request.dataSize) {
    dataSizeBytes = typeof request.dataSize === 'number'
      ? request.dataSize
      : parseSize(request.dataSize);
    assumptions.push(`Data size: ${(dataSizeBytes / (1024 ** 3)).toFixed(2)} GB`);
  }

  // Classify workload if not already done
  let workloadCharacteristics = request.workloadCharacteristics;
  if (!workloadCharacteristics && request.description) {
    const classification = await classifyWorkload({
      description: request.description,
      dataSize: dataSizeBytes,
      operations: request.operations,
    });
    workloadCharacteristics = classification.characteristics;
    assumptions.push(`Workload type: ${workloadCharacteristics.type}`);
  }

  // Get hardware context
  const hardware = request.hardware ?? {
    cpuCores: 96,
    totalMemory: 512,
    gpuCount: 8,
    gpuMemoryPerDevice: 80,
    networkBandwidth: 100,
    storageBandwidth: 10000,
  };

  assumptions.push(
    `Hardware: ${hardware.cpuCores} cores, ${hardware.totalMemory}GB RAM, ${hardware.gpuCount ?? 0} GPUs`
  );

  // Estimate memory requirements
  const memory = estimateMemory(
    dataSizeBytes,
    workloadCharacteristics,
    hardware
  );

  // Estimate compute requirements
  const compute = estimateCompute(
    dataSizeBytes,
    workloadCharacteristics,
    hardware
  );

  // Estimate storage requirements
  const storage = estimateStorage(
    dataSizeBytes,
    workloadCharacteristics,
    request.operations ?? []
  );

  // Estimate execution time
  const time = estimateTime(
    dataSizeBytes,
    workloadCharacteristics,
    compute,
    storage,
    hardware
  );

  // Calculate confidence based on available information
  const confidence = calculateConfidence(request, workloadCharacteristics);

  return {
    memory,
    compute,
    storage,
    time,
    confidence,
    assumptions,
  };
}

/**
 * Estimate memory requirements
 */
function estimateMemory(
  dataSizeBytes: number,
  workloadCharacteristics: any,
  hardware: any
): MemoryEstimate {
  const dataSizeGB = dataSizeBytes / (1024 ** 3);

  // Memory multiplier based on workload type
  const memoryMultipliers: Record<string, number> = {
    'ml-training': 4.0,
    'ml-inference': 2.0,
    'analytics': 2.5,
    'etl': 2.0,
    'streaming': 1.5,
    'graph': 3.5,
    'sql': 2.5,
    'mixed': 2.5,
  };

  const workloadType = workloadCharacteristics?.type ?? 'mixed';
  const multiplier = memoryMultipliers[workloadType] ?? 2.5;

  const peakMemoryGB = dataSizeGB * multiplier;

  // Distribute memory between executors and driver
  const driverMemoryGB = Math.min(
    Math.max(8, peakMemoryGB * 0.1),
    64 // Cap driver at 64GB
  );

  const executorTotalGB = peakMemoryGB - driverMemoryGB;

  // Assume 4-8 executors by default
  const executorCount = Math.min(
    Math.max(4, Math.ceil(hardware.cpuCores / 12)),
    16
  );

  const executorMemoryGB = executorTotalGB / executorCount;

  // Memory overhead (10-20%)
  const overheadPercent = workloadType === 'ml-training' ? 0.2 : 0.15;
  const overheadGB = (executorMemoryGB + driverMemoryGB) * overheadPercent;

  const totalMemoryGB = peakMemoryGB + overheadGB;

  // Breakdown
  const breakdown = {
    execution: peakMemoryGB * 0.6,
    storage: peakMemoryGB * 0.3,
    overhead: overheadGB,
    offHeap: peakMemoryGB * 0.1,
  };

  // Determine spill likelihood
  let spillLikelihood: MemoryEstimate['spillLikelihood'];
  if (totalMemoryGB > hardware.totalMemory * 0.9) {
    spillLikelihood = 'certain';
  } else if (totalMemoryGB > hardware.totalMemory * 0.75) {
    spillLikelihood = 'likely';
  } else if (totalMemoryGB > hardware.totalMemory * 0.5) {
    spillLikelihood = 'possible';
  } else {
    spillLikelihood = 'unlikely';
  }

  return {
    executorMemoryGB: Math.ceil(executorMemoryGB),
    driverMemoryGB: Math.ceil(driverMemoryGB),
    totalMemoryGB: Math.ceil(totalMemoryGB),
    overheadGB: Math.ceil(overheadGB),
    peakMemoryGB: Math.ceil(peakMemoryGB),
    breakdown,
    spillLikelihood,
  };
}

/**
 * Estimate compute requirements
 */
function estimateCompute(
  dataSizeBytes: number,
  workloadCharacteristics: any,
  hardware: any
): ComputeEstimate {
  const dataSizeGB = dataSizeBytes / (1024 ** 3);
  const workloadType = workloadCharacteristics?.type ?? 'mixed';

  // Determine executor cores based on workload
  const executorCores = workloadType === 'ml-training' ? 8 :
                       workloadType === 'streaming' ? 4 : 5;

  // Calculate executor count
  const maxExecutors = Math.floor(hardware.cpuCores / executorCores);
  const minExecutors = Math.max(2, Math.floor(dataSizeGB / 100));
  const executorCount = Math.min(maxExecutors, Math.max(minExecutors, 4));

  const totalCores = executorCount * executorCores;

  // Parallelism (2-3x cores)
  const parallelism = totalCores * 3;

  // GPU requirements
  let gpuRequirement;
  const gpuUtilization = workloadCharacteristics?.gpuUtilization ?? 'none';

  if (gpuUtilization === 'high' || gpuUtilization === 'full') {
    const gpuCount = Math.min(
      Math.ceil(executorCount / 2),
      hardware.gpuCount ?? 0
    );

    if (gpuCount > 0) {
      gpuRequirement = {
        count: gpuCount,
        memoryPerGPU: hardware.gpuMemoryPerDevice ?? 80,
        utilizationPercent: gpuUtilization === 'full' ? 90 : 70,
      };
    }
  } else if (gpuUtilization === 'medium' && hardware.gpuCount) {
    gpuRequirement = {
      count: Math.min(2, hardware.gpuCount),
      memoryPerGPU: hardware.gpuMemoryPerDevice ?? 80,
      utilizationPercent: 50,
    };
  }

  // Estimate CPU utilization
  const computeIntensity = workloadCharacteristics?.computeIntensity ?? 'medium';
  const cpuUtilizationPercent =
    computeIntensity === 'very-high' ? 95 :
    computeIntensity === 'high' ? 85 :
    computeIntensity === 'medium' ? 70 : 50;

  return {
    executorCores,
    executorCount,
    totalCores,
    parallelism,
    gpuRequirement,
    cpuUtilizationPercent,
  };
}

/**
 * Estimate storage requirements
 */
function estimateStorage(
  dataSizeBytes: number,
  workloadCharacteristics: any,
  operations: string[]
): StorageEstimate {
  const dataSizeGB = dataSizeBytes / (1024 ** 3);

  const inputDataGB = dataSizeGB;

  // Estimate intermediate data (transformations create intermediate datasets)
  const transformationCount = operations.filter(op =>
    ['map', 'filter', 'flatmap', 'transform'].some(t => op.toLowerCase().includes(t))
  ).length;

  const intermediateDataGB = dataSizeGB * Math.min(transformationCount * 0.5, 2);

  // Estimate shuffle data
  const shuffleOps = operations.filter(op =>
    ['join', 'groupby', 'aggregate', 'distinct'].some(s => op.toLowerCase().includes(s))
  ).length;

  const shuffleDataGB = dataSizeGB * Math.min(shuffleOps * 0.7, 3);

  // Estimate output data (assume similar to input unless significant filtering)
  const hasFiltering = operations.some(op => op.toLowerCase().includes('filter'));
  const outputDataGB = hasFiltering ? dataSizeGB * 0.5 : dataSizeGB;

  const totalIOGB = inputDataGB + intermediateDataGB + shuffleDataGB + outputDataGB;

  // Estimate I/O bandwidth requirement (MB/s)
  const ioIntensity = workloadCharacteristics?.ioPattern === 'streaming' ? 'high' :
                     workloadCharacteristics?.ioPattern === 'random' ? 'medium' : 'low';

  const ioBandwidthMBps = ioIntensity === 'high' ? 5000 :
                         ioIntensity === 'medium' ? 2000 : 1000;

  // Temporary storage for spills and checkpoints
  const tmpStorageGB = Math.ceil(shuffleDataGB * 1.5);

  return {
    inputDataGB,
    intermediateDataGB,
    outputDataGB,
    shuffleDataGB,
    totalIOGB,
    ioBandwidthMBps,
    tmpStorageGB,
  };
}

/**
 * Estimate execution time
 */
function estimateTime(
  dataSizeBytes: number,
  workloadCharacteristics: any,
  compute: ComputeEstimate,
  storage: StorageEstimate,
  hardware: any
): TimeEstimate {
  const dataSizeMB = dataSizeBytes / (1024 ** 2);

  // Base throughput per core (MB/s)
  let throughputPerCoreMBps = 50;

  // Adjust for workload type
  const workloadType = workloadCharacteristics?.type ?? 'mixed';
  if (workloadType === 'ml-training') {
    throughputPerCoreMBps *= 0.3; // Compute-intensive
  } else if (workloadType === 'etl') {
    throughputPerCoreMBps *= 1.5; // I/O-optimized
  }

  // GPU acceleration
  if (compute.gpuRequirement) {
    throughputPerCoreMBps *= 3;
  }

  const totalThroughputMBps = throughputPerCoreMBps * compute.totalCores;

  // Time breakdown
  const inputIOMinutes = (storage.inputDataGB * 1024) / totalThroughputMBps / 60;
  const computationMinutes = (dataSizeMB / totalThroughputMBps / 60) * (workloadType === 'ml-training' ? 3 : 1);
  const shuffleMinutes = (storage.shuffleDataGB * 1024) / (hardware.networkBandwidth ?? 1000) / 60;
  const outputIOMinutes = (storage.outputDataGB * 1024) / totalThroughputMBps / 60;

  const estimatedMinutes = inputIOMinutes + computationMinutes + shuffleMinutes + outputIOMinutes;

  // Range (±30%)
  const range = {
    min: estimatedMinutes * 0.7,
    max: estimatedMinutes * 1.3,
  };

  // Determine bottleneck
  const times = {
    cpu: computationMinutes,
    memory: 0, // Estimated separately
    io: inputIOMinutes + outputIOMinutes,
    shuffle: shuffleMinutes,
    gpu: compute.gpuRequirement ? computationMinutes * 0.3 : 0,
    network: shuffleMinutes,
  };

  const bottleneck = Object.entries(times).reduce((a, b) =>
// @ts-ignore
    times[(a[0] ?? "cpu")] > times[(b[0] ?? "cpu")] ? a : b
  )[0] as TimeEstimate['bottleneck'];

  return {
    estimatedMinutes,
    range,
    breakdown: {
      inputIO: inputIOMinutes,
      computation: computationMinutes,
      shuffle: shuffleMinutes,
      outputIO: outputIOMinutes,
    },
    bottleneck,
  };
}

/**
 * Calculate confidence in estimates
 */
function calculateConfidence(
  request: ResourceEstimationRequest,
  workloadCharacteristics: any
): number {
  let confidence = 0.5; // Base confidence

  // Increase confidence with more information
  if (request.description) confidence += 0.1;
  if (request.dataSize) confidence += 0.15;
  if (request.operations && request.operations.length > 0) confidence += 0.1;
  if (request.hardware) confidence += 0.1;
  if (workloadCharacteristics) confidence += 0.05;

  return Math.min(confidence, 1.0);
}
