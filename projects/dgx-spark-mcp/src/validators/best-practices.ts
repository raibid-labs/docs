/**
 * Best practices checker for Spark configurations
 */

import { SparkConfig } from '../types/spark-config.js';

export interface BestPracticeViolation {
  category: 'performance' | 'reliability' | 'cost' | 'security';
  severity: 'critical' | 'high' | 'medium' | 'low';
  pattern: string;
  description: string;
  recommendation: string;
  autoFix?: Partial<SparkConfig>;
}

export interface BestPracticesResult {
  passed: number;
  failed: number;
  violations: BestPracticeViolation[];
  score: number; // 0-100
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
}

/**
 * Check configuration against best practices
 */
export async function checkAntiPatterns(config: SparkConfig): Promise<BestPracticesResult> {
  const violations: BestPracticeViolation[] = [];

  // Check for anti-patterns
  checkMemoryAntiPatterns(config, violations);
  checkExecutorAntiPatterns(config, violations);
  checkShuffleAntiPatterns(config, violations);
  checkGPUAntiPatterns(config, violations);
  checkSerializationAntiPatterns(config, violations);
  checkOptimizationAntiPatterns(config, violations);

  // Calculate score
  const totalChecks = 20; // Number of best practices we check
  const failed = violations.length;
  const passed = totalChecks - failed;
  const score = Math.max(0, Math.round((passed / totalChecks) * 100));

  // Determine grade
  const grade = score >= 90 ? 'A' :
               score >= 80 ? 'B' :
               score >= 70 ? 'C' :
               score >= 60 ? 'D' : 'F';

  return {
    passed,
    failed,
    violations,
    score,
    grade,
  };
}

/**
 * Check memory anti-patterns
 */
function checkMemoryAntiPatterns(
  config: SparkConfig,
  violations: BestPracticeViolation[]
): void {
  const executorMemGB = parseMemory(config.executor.memory);
  const driverMemGB = parseMemory(config.driver.memory);

  // Giant executor anti-pattern
  if (executorMemGB > 64) {
    violations.push({
      category: 'performance',
      severity: 'high',
      pattern: 'Giant Executor',
      description: `Executor memory (${executorMemGB}GB) exceeds 64GB, leading to excessive GC overhead`,
      recommendation: 'Use multiple smaller executors (8-32GB each) instead of few large ones',
      autoFix: {
        executor: {
          ...config.executor,
          memory: '32g',
          instances: config.executor.instances ? config.executor.instances * 2 : undefined,
        },
      },
    });
  }

  // Tiny executor anti-pattern
  if (executorMemGB < 2) {
    violations.push({
      category: 'performance',
      severity: 'medium',
      pattern: 'Tiny Executor',
      description: `Executor memory (${executorMemGB}GB) is too small, causing frequent spilling`,
      recommendation: 'Increase executor memory to at least 4-8GB',
      autoFix: {
        executor: {
          ...config.executor,
          memory: '8g',
        },
      },
    });
  }

  // Driver-heavy anti-pattern
  if (driverMemGB > executorMemGB * 3) {
    violations.push({
      category: 'cost',
      severity: 'low',
      pattern: 'Driver-Heavy Configuration',
      description: `Driver memory (${driverMemGB}GB) is disproportionately large compared to executors`,
      recommendation: 'Driver should typically be 1-2x executor memory unless collecting large results',
    });
  }

  // No off-heap memory
  if (!config.memory.offHeap.enabled && executorMemGB > 16) {
    violations.push({
      category: 'performance',
      severity: 'low',
      pattern: 'No Off-Heap Memory',
      description: 'Off-heap memory disabled for large executors',
      recommendation: 'Enable off-heap memory to reduce GC pressure for large memory allocations',
      autoFix: {
        memory: {
          ...config.memory,
          offHeap: {
            enabled: true,
            size: `${Math.ceil(executorMemGB * 0.2)}g`,
          },
        },
      },
    });
  }
}

/**
 * Check executor anti-patterns
 */
function checkExecutorAntiPatterns(
  config: SparkConfig,
  violations: BestPracticeViolation[]
): void {
  // Too many cores per executor
  if (config.executor.cores > 8) {
    violations.push({
      category: 'performance',
      severity: 'high',
      pattern: 'Fat Executor',
      description: `Executor cores (${config.executor.cores}) exceeds recommended maximum of 8`,
      recommendation: 'Use 4-6 cores per executor for optimal HDFS throughput and reduced GC',
      autoFix: {
        executor: {
          ...config.executor,
          cores: 5,
        },
      },
    });
  }

  // Single core executor (unless streaming)
  if (config.executor.cores === 1) {
    violations.push({
      category: 'performance',
      severity: 'medium',
      pattern: 'Single-Core Executor',
      description: 'Single-core executors limit parallelism within executor',
      recommendation: 'Use at least 2-4 cores per executor for better task parallelism',
    });
  }

  // Too few executors
  if (config.executor.instances && config.executor.instances < 2) {
    violations.push({
      category: 'reliability',
      severity: 'medium',
      pattern: 'Insufficient Executors',
      description: 'Too few executors reduce fault tolerance and parallelism',
      recommendation: 'Use at least 2-4 executors for better resource utilization',
    });
  }
}

/**
 * Check shuffle anti-patterns
 */
function checkShuffleAntiPatterns(
  config: SparkConfig,
  violations: BestPracticeViolation[]
): void {
  const totalCores = config.executor.cores * (config.executor.instances ?? 1);

  // Too few shuffle partitions
  if (config.shuffle.partitions < totalCores * 2) {
    violations.push({
      category: 'performance',
      severity: 'high',
      pattern: 'Insufficient Shuffle Partitions',
      description: `Only ${config.shuffle.partitions} shuffle partitions for ${totalCores} cores`,
      recommendation: `Increase to at least ${totalCores * 2} (2-3x cores)`,
      autoFix: {
        shuffle: {
          ...config.shuffle,
          partitions: totalCores * 3,
        },
      },
    });
  }

  // Too many shuffle partitions
  if (config.shuffle.partitions > totalCores * 20) {
    violations.push({
      category: 'performance',
      severity: 'medium',
      pattern: 'Excessive Shuffle Partitions',
      description: `${config.shuffle.partitions} shuffle partitions creates too many small tasks`,
      recommendation: `Reduce to ${totalCores * 3}-${totalCores * 5}`,
    });
  }

  // Shuffle compression disabled
  if (!config.shuffle.compress) {
    violations.push({
      category: 'performance',
      severity: 'medium',
      pattern: 'No Shuffle Compression',
      description: 'Shuffle compression disabled increases I/O',
      recommendation: 'Enable shuffle compression to reduce network and disk I/O',
      autoFix: {
        shuffle: {
          ...config.shuffle,
          compress: true,
        },
      },
    });
  }
}

/**
 * Check GPU anti-patterns
 */
function checkGPUAntiPatterns(
  config: SparkConfig,
  violations: BestPracticeViolation[]
): void {
  if (!config.gpu?.enabled) return;

  // GPU without RAPIDS
  if (!config.gpu.rapids?.enabled) {
    violations.push({
      category: 'performance',
      severity: 'medium',
      pattern: 'GPU Without RAPIDS',
      description: 'GPU enabled but RAPIDS acceleration not configured',
      recommendation: 'Enable RAPIDS for GPU-accelerated DataFrame operations',
      autoFix: {
        gpu: {
          ...config.gpu,
          rapids: {
            enabled: true,
            sqlEnabled: true,
            memoryFraction: 0.9,
            pooling: {
              enabled: true,
              mode: 'ARENA',
            },
          },
        },
      },
    });
  }

  // No GPU pooling
  if (config.gpu.rapids?.enabled && !config.gpu.rapids.pooling?.enabled) {
    violations.push({
      category: 'performance',
      severity: 'low',
      pattern: 'No GPU Memory Pooling',
      description: 'GPU memory pooling disabled increases allocation overhead',
      recommendation: 'Enable GPU memory pooling for better performance',
    });
  }
}

/**
 * Check serialization anti-patterns
 */
function checkSerializationAntiPatterns(
  config: SparkConfig,
  violations: BestPracticeViolation[]
): void {
  // Using Java serialization
  if (!config.serializer || config.serializer.includes('JavaSerializer')) {
    violations.push({
      category: 'performance',
      severity: 'high',
      pattern: 'Java Serialization',
      description: 'Using default Java serialization instead of Kryo',
      recommendation: 'Switch to Kryo serialization for 10x performance improvement',
      autoFix: {
        serializer: 'org.apache.spark.serializer.KryoSerializer',
        kryo: {
          registrationRequired: false,
          referenceTracking: true,
        },
      },
    });
  }
}

/**
 * Check optimization anti-patterns
 */
function checkOptimizationAntiPatterns(
  config: SparkConfig,
  violations: BestPracticeViolation[]
): void {
  // AQE disabled
  if (!config.optimization.adaptiveExecution?.enabled) {
    violations.push({
      category: 'performance',
      severity: 'medium',
      pattern: 'Adaptive Execution Disabled',
      description: 'Adaptive Query Execution (AQE) not enabled',
      recommendation: 'Enable AQE for runtime query optimizations (coalescing, skew joins)',
      autoFix: {
        optimization: {
          ...config.optimization,
          adaptiveExecution: {
            enabled: true,
            coalescePartitions: true,
            skewJoin: true,
          },
        },
      },
    });
  }

  // Speculation disabled
  if (!config.speculation?.enabled) {
    violations.push({
      category: 'reliability',
      severity: 'low',
      pattern: 'Speculation Disabled',
      description: 'Speculative execution disabled can lead to stragglers',
      recommendation: 'Enable speculation to mitigate slow tasks',
      autoFix: {
        speculation: {
          enabled: true,
          interval: '100ms',
          multiplier: 1.5,
        },
      },
    });
  }

  // Low broadcast threshold
  const broadcastThreshold = parseSizeString(config.optimization.autoBroadcastJoinThreshold ?? '10mb');
  if (broadcastThreshold < 5 * 1024 * 1024) {
    violations.push({
      category: 'performance',
      severity: 'low',
      pattern: 'Low Broadcast Threshold',
      description: 'Broadcast join threshold may be too conservative',
      recommendation: 'Increase broadcast threshold to 10-20MB for better join performance',
    });
  }
}

/**
 * Helper functions
 */
function parseMemory(memory: string): number {
  const match = memory.match(/^(\d+)([gmk])$/i);
  if (!match || !match[1] || !match[2]) return 0;

  const value = parseInt(match[1], 10);
  const unit = match[2].toLowerCase();

  switch (unit) {
    case 'g': return value;
    case 'm': return value / 1024;
    case 'k': return value / (1024 * 1024);
    default: return value;
  }
}

function parseSizeString(size: string): number {
  const units: Record<string, number> = {
    b: 1,
    kb: 1024,
    mb: 1024 ** 2,
    gb: 1024 ** 3,
  };

  const match = size.toLowerCase().match(/^(\d+)([kmg]?b)$/);
  if (!match || !match[1] || !match[2]) return 0;

  const [, value, unit] = match;
  return parseInt(value, 10) * (units[unit] || 1);
}
