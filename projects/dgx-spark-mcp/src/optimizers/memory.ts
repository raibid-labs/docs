/**
 * Memory configuration optimizer for Spark
 */

import { MemoryConfig } from '../types/spark-config.js';

export interface MemoryCalculationRequest {
  dataSize: string | number; // e.g., "500GB" or bytes
  partitionCount: number;
  executorCount?: number;
  totalMemoryGB?: number;
  workloadType?: 'etl' | 'analytics' | 'ml-training' | 'ml-inference' | 'streaming' | 'graph' | 'sql' | 'mixed';
  enableOffHeap?: boolean;
}

export interface MemoryCalculationResult {
  executorMemoryGB: number;
  driverMemoryGB: number;
  memoryOverheadGB: number;
  offHeapMemoryGB?: number;
  memoryConfig: MemoryConfig;
  recommendations: string[];
}

/**
 * Calculate optimal memory configuration for Spark
 */
export async function calculateMemoryConfig(
  request: MemoryCalculationRequest
): Promise<MemoryCalculationResult> {
  const dataSizeBytes = parseSize(request.dataSize);
  const dataSizeGB = dataSizeBytes / (1024 ** 3);

  // Calculate memory per partition
  const avgPartitionSizeGB = dataSizeGB / request.partitionCount;

  // Determine memory fraction based on workload
  const memoryFraction = getMemoryFraction(request.workloadType);
  const storageFraction = getStorageFraction(request.workloadType);

  // Calculate executor memory
  // Rule of thumb: 3-4x partition size for compute-heavy, 2-3x for I/O heavy
  const memoryMultiplier = request.workloadType === 'ml-training' ? 4 :
                          request.workloadType === 'analytics' ? 3 : 2.5;

  let executorMemoryGB = Math.max(
    avgPartitionSizeGB * memoryMultiplier,
    4 // minimum 4GB per executor
  );

  // Round to nearest GB for cleaner configs
  executorMemoryGB = Math.ceil(executorMemoryGB);

  // Calculate memory overhead (10-20% depending on workload)
  const overheadPercent = request.workloadType === 'ml-training' ? 0.2 : 0.1;
  const memoryOverheadGB = Math.ceil(executorMemoryGB * overheadPercent);

  // Driver memory (usually 2-4x executor or 10% of total data, whichever is smaller)
  let driverMemoryGB = Math.min(
    executorMemoryGB * 2,
    Math.max(dataSizeGB * 0.1, 8)
  );
  driverMemoryGB = Math.ceil(driverMemoryGB);

  // Off-heap memory for improved GC performance
  let offHeapMemoryGB: number | undefined;
  let offHeapEnabled = request.enableOffHeap ?? false;

  if (request.workloadType === 'ml-training' || request.workloadType === 'analytics') {
    offHeapEnabled = true;
    offHeapMemoryGB = Math.ceil(executorMemoryGB * 0.2);
  }

  const memoryConfig: MemoryConfig = {
    fraction: memoryFraction,
    storageFraction: storageFraction,
    offHeap: {
      enabled: offHeapEnabled,
      size: offHeapMemoryGB ? `${offHeapMemoryGB}g` : undefined,
    },
  };

  const recommendations: string[] = [];

  // Generate recommendations
  if (avgPartitionSizeGB > 1) {
    recommendations.push(
      `Large partition size (${avgPartitionSizeGB.toFixed(2)}GB). Consider increasing partition count for better parallelism.`
    );
  }

  if (executorMemoryGB > 32) {
    recommendations.push(
      'Large executor memory (>32GB) may lead to GC issues. Consider using more executors with less memory each.'
    );
  }

  if (request.totalMemoryGB) {
    const executorCount = request.executorCount ?? 1;
    const totalRequiredGB = (executorMemoryGB + memoryOverheadGB) * executorCount + driverMemoryGB;
    if (totalRequiredGB > request.totalMemoryGB * 0.9) {
      recommendations.push(
        `Total memory requirement (${totalRequiredGB}GB) exceeds 90% of available memory (${request.totalMemoryGB}GB). Risk of OOM errors.`
      );
    }
  }

  if (offHeapEnabled) {
    recommendations.push(
      'Off-heap memory enabled for improved GC performance. Monitor for direct memory issues.'
    );
  }

  return {
    executorMemoryGB,
    driverMemoryGB,
    memoryOverheadGB,
    offHeapMemoryGB,
    memoryConfig,
    recommendations,
  };
}

/**
 * Get optimal memory fraction for workload type
 */
function getMemoryFraction(workloadType?: string): number {
  switch (workloadType) {
    case 'ml-training':
      return 0.5; // More for execution
    case 'analytics':
      return 0.6;
    case 'streaming':
      return 0.5;
    case 'etl':
      return 0.4; // More for storage/caching
    default:
      return 0.6;
  }
}

/**
 * Get optimal storage fraction for workload type
 */
function getStorageFraction(workloadType?: string): number {
  switch (workloadType) {
    case 'ml-training':
      return 0.3;
    case 'analytics':
      return 0.5; // Cache intermediate results
    case 'streaming':
      return 0.2;
    case 'etl':
      return 0.5;
    default:
      return 0.5;
  }
}

/**
 * Parse size string to bytes
 */
export function parseSize(size: string | number): number {
  if (typeof size === 'number') {
    return size;
  }

  const units: Record<string, number> = {
    b: 1,
    kb: 1024,
    mb: 1024 ** 2,
    gb: 1024 ** 3,
    tb: 1024 ** 4,
    pb: 1024 ** 5,
  };

  const match = size.toLowerCase().match(/^(\d+(?:\.\d+)?)\s*([kmgtp]?b)$/);
  if (!match || !match[1] || !match[2]) {
    throw new Error(`Invalid size format: ${size}`);
  }

  const value = match[1];
  const unit = match[2];
  return parseFloat(value) * (units[unit] || 1);
}

/**
 * Format bytes to human-readable string
 */
export function formatSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)}${units[unitIndex]}`;
}
