/**
 * I/O pattern analyzer for Spark workloads
 */

import { IOPattern } from '../types/workload.js';
import { parseSize } from '../optimizers/memory.js';

export interface IOPatternRequest {
  dataSize: string | number;
  operations: string[];
  fileFormat?: string;
  partitionCount?: number;
  readPattern?: 'full-scan' | 'filtered' | 'sampled';
  writePattern?: 'append' | 'overwrite' | 'partitioned';
}

export interface IOPatternAnalysis {
  pattern: IOPattern;
  estimatedReadMB: number;
  estimatedWriteMB: number;
  estimatedShuffleMB: number;
  totalIOMB: number;
  ioIntensity: 'low' | 'medium' | 'high';
  bottleneck: boolean;
  recommendations: string[];
}

/**
 * Analyze I/O pattern for workload
 */
export async function analyzeIOPattern(
  request: IOPatternRequest
): Promise<IOPatternAnalysis> {
  const dataSizeBytes = typeof request.dataSize === 'number'
    ? request.dataSize
    : parseSize(request.dataSize);
  const dataSizeMB = dataSizeBytes / (1024 ** 2);

  // Determine I/O pattern
  const pattern = determinePattern(request.operations);

  // Estimate read volume
  const readMultiplier = getReadMultiplier(request.readPattern, request.fileFormat);
  const estimatedReadMB = dataSizeMB * readMultiplier;

  // Estimate shuffle volume
  const shuffleOps = countShuffleOperations(request.operations);
  const shuffleMultiplier = Math.min(shuffleOps * 0.5, 3);
  const estimatedShuffleMB = dataSizeMB * shuffleMultiplier;

  // Estimate write volume
  const writeMultiplier = getWriteMultiplier(request.writePattern, request.fileFormat);
  const estimatedWriteMB = dataSizeMB * writeMultiplier;

  const totalIOMB = estimatedReadMB + estimatedWriteMB + estimatedShuffleMB;

  // Determine I/O intensity
  const ioIntensity = totalIOMB > dataSizeMB * 5 ? 'high' :
                      totalIOMB > dataSizeMB * 2 ? 'medium' : 'low';

  // Check if I/O is a bottleneck
  // Rule of thumb: I/O becomes a bottleneck when total I/O > 3x data size
  const bottleneck = totalIOMB > dataSizeMB * 3;

  // Generate recommendations
  const recommendations = generateIORecommendations(
    pattern,
    ioIntensity,
    bottleneck,
    request
  );

  return {
    pattern,
    estimatedReadMB,
    estimatedWriteMB,
    estimatedShuffleMB,
    totalIOMB,
    ioIntensity,
    bottleneck,
    recommendations,
  };
}

/**
 * Determine I/O pattern from operations
 */
function determinePattern(operations: string[]): IOPattern {
  const streamingOps = ['stream', 'readStream', 'writeStream', 'foreachBatch'];
  const randomOps = ['join', 'lookup', 'sample', 'randomSplit'];
  const sequentialOps = ['map', 'filter', 'select', 'flatMap'];

  const hasStreaming = operations.some(op =>
    streamingOps.some(s => op.toLowerCase().includes(s))
  );
  const hasRandom = operations.some(op =>
    randomOps.some(r => op.toLowerCase().includes(r))
  );
  const hasSequential = operations.some(op =>
    sequentialOps.some(s => op.toLowerCase().includes(s))
  );

  if (hasStreaming) return 'streaming';
  if (hasRandom && hasSequential) return 'mixed';
  if (hasRandom) return 'random';
  return 'sequential';
}

/**
 * Count shuffle operations
 */
function countShuffleOperations(operations: string[]): number {
  const shuffleOps = [
    'join',
    'groupby',
    'groupbykey',
    'aggregate',
    'distinct',
    'repartition',
    'coalesce',
    'sortby',
    'reducebykey',
  ];

  return operations.filter(op =>
    shuffleOps.some(shuffle => op.toLowerCase().includes(shuffle))
  ).length;
}

/**
 * Get read multiplier based on pattern
 */
function getReadMultiplier(
  readPattern?: string,
  fileFormat?: string
): number {
  let multiplier = 1.0;

  // Read pattern adjustments
  if (readPattern === 'filtered') {
    multiplier *= 0.3; // Assume 30% of data read with filtering
  } else if (readPattern === 'sampled') {
    multiplier *= 0.1; // Sample reads 10% of data
  }

  // File format adjustments
  if (fileFormat === 'parquet' || fileFormat === 'orc') {
    multiplier *= 0.7; // Columnar formats with compression
  } else if (fileFormat === 'csv' || fileFormat === 'json') {
    multiplier *= 1.2; // Text formats are larger
  }

  return multiplier;
}

/**
 * Get write multiplier based on pattern
 */
function getWriteMultiplier(
  writePattern?: string,
  fileFormat?: string
): number {
  let multiplier = 1.0;

  // Write pattern adjustments
  if (writePattern === 'partitioned') {
    multiplier *= 1.1; // Slightly more overhead for partitioning
  }

  // File format adjustments
  if (fileFormat === 'parquet' || fileFormat === 'orc') {
    multiplier *= 0.5; // Efficient compression
  } else if (fileFormat === 'csv' || fileFormat === 'json') {
    multiplier *= 1.3; // Text formats
  }

  return multiplier;
}

/**
 * Generate I/O recommendations
 */
function generateIORecommendations(
  pattern: IOPattern,
  intensity: string,
  bottleneck: boolean,
  request: IOPatternRequest
): string[] {
  const recommendations: string[] = [];

  // General I/O recommendations
  if (bottleneck) {
    recommendations.push(
      'I/O is a bottleneck. Consider faster storage (NVMe), parallel reads, or data caching.'
    );
  }

  if (intensity === 'high') {
    recommendations.push(
      'High I/O intensity detected. Use efficient file formats (Parquet/ORC) with compression.'
    );
  }

  // Pattern-specific recommendations
  switch (pattern) {
    case 'sequential':
      recommendations.push(
        'Sequential I/O pattern. Optimize for throughput with large block sizes.'
      );
      if (request.partitionCount && request.partitionCount < 10) {
        recommendations.push(
          'Increase partition count for better parallelism in sequential reads.'
        );
      }
      break;

    case 'random':
      recommendations.push(
        'Random I/O pattern. Use columnar formats with predicate pushdown and partitioning.'
      );
      recommendations.push(
        'Consider caching frequently accessed data in memory.'
      );
      break;

    case 'streaming':
      recommendations.push(
        'Streaming I/O. Use checkpointing and adjust trigger intervals for throughput vs. latency.'
      );
      recommendations.push(
        'Monitor watermark and state size for streaming jobs.'
      );
      break;

    case 'mixed':
      recommendations.push(
        'Mixed I/O pattern. Profile individual operations to identify bottlenecks.'
      );
      break;
  }

  // File format recommendations
  if (!request.fileFormat || request.fileFormat === 'csv' || request.fileFormat === 'json') {
    recommendations.push(
      'Consider using Parquet or ORC format for better compression and query performance.'
    );
  }

  // Shuffle recommendations
  const shuffleOps = countShuffleOperations(request.operations);
  if (shuffleOps > 3) {
    recommendations.push(
      `High number of shuffle operations (${shuffleOps}). Consider optimizing joins and aggregations.`
    );
    recommendations.push(
      'Use broadcast joins for small tables and adjust shuffle partition count.'
    );
  }

  return recommendations;
}

/**
 * Estimate optimal block size for I/O
 */
export function estimateOptimalBlockSize(
  pattern: IOPattern,
  _dataSizeMB?: number
): number {
  // Return block size in MB
  switch (pattern) {
    case 'sequential':
      return 128; // Large blocks for sequential reads

    case 'random':
      return 64; // Smaller blocks for random access

    case 'streaming':
      return 32; // Smaller blocks for low latency

    case 'mixed':
      return 64; // Balanced approach

    default:
      return 64;
  }
}

/**
 * Estimate optimal partition count for I/O
 */
export function estimateOptimalPartitionCount(
  availableCores: number,
  pattern: IOPattern,
  dataSizeMB?: number
): number {
  // Target partition size in MB
  const targetPartitionSize = estimateOptimalBlockSize(pattern, dataSizeMB);

  // Calculate partitions from data size
  const partitionsFromSize = dataSizeMB ? Math.ceil(dataSizeMB / targetPartitionSize) : availableCores * 3;

  // Calculate partitions from parallelism (2-3x cores)
  const partitionsFromCores = availableCores * 3;

  // Use the larger value
  return Math.max(partitionsFromSize, partitionsFromCores);
}
