/**
 * Bottleneck detection for Spark configurations
 */

import { SparkConfig } from '../types/spark-config.js';
import { HardwareContext } from '../types/estimation.js';

export interface BottleneckAnalysis {
  bottlenecks: Bottleneck[];
  severity: 'critical' | 'high' | 'medium' | 'low' | 'none';
  primaryBottleneck?: Bottleneck;
  recommendations: string[];
}

export interface Bottleneck {
  type: 'cpu' | 'memory' | 'gpu' | 'io' | 'shuffle' | 'network' | 'configuration';
  severity: 'critical' | 'high' | 'medium' | 'low';
  description: string;
  impact: string;
  suggestedFix: string;
  metrics?: Record<string, number>;
}

/**
 * Detect bottlenecks in Spark configuration
 */
export async function detectBottlenecks(request: {
  config: SparkConfig;
  hardware: HardwareContext;
  workloadType?: string;
}): Promise<BottleneckAnalysis> {
  const { config, hardware, workloadType = 'mixed' } = request;
  const bottlenecks: Bottleneck[] = [];

  // Check CPU bottlenecks
  const cpuBottlenecks = checkCPUBottlenecks(config, hardware);
  bottlenecks.push(...cpuBottlenecks);

  // Check memory bottlenecks
  const memoryBottlenecks = checkMemoryBottlenecks(config, hardware, workloadType);
  bottlenecks.push(...memoryBottlenecks);

  // Check GPU bottlenecks
  const gpuBottlenecks = checkGPUBottlenecks(config, hardware, workloadType);
  bottlenecks.push(...gpuBottlenecks);

  // Check I/O bottlenecks
  const ioBottlenecks = checkIOBottlenecks(config, workloadType);
  bottlenecks.push(...ioBottlenecks);

  // Check shuffle bottlenecks
  const shuffleBottlenecks = checkShuffleBottlenecks(config, hardware);
  bottlenecks.push(...shuffleBottlenecks);

  // Check configuration bottlenecks
  const configBottlenecks = checkConfigurationBottlenecks(config);
  bottlenecks.push(...configBottlenecks);

  // Determine overall severity
  const severity = determineOverallSeverity(bottlenecks);

  // Find primary bottleneck
  const primaryBottleneck = bottlenecks.sort((a, b) => {
    const severityOrder = { critical: 4, high: 3, medium: 2, low: 1 };
    return severityOrder[b.severity] - severityOrder[a.severity];
  })[0];

  // Generate recommendations
  const recommendations = generateBottleneckRecommendations(bottlenecks, workloadType);

  return {
    bottlenecks,
    severity,
    primaryBottleneck,
    recommendations,
  };
}

/**
 * Check CPU bottlenecks
 */
function checkCPUBottlenecks(config: SparkConfig, hardware: HardwareContext): Bottleneck[] {
  const bottlenecks: Bottleneck[] = [];

  const totalCores = config.executor.cores * (config.executor.instances ?? 1) + config.driver.cores;
  const coreUtilization = totalCores / hardware.cpuCores;

  // Over-subscription
  if (coreUtilization > 1.0) {
    bottlenecks.push({
      type: 'cpu',
      severity: 'critical',
      description: `CPU over-subscribed: requesting ${totalCores} cores but only ${hardware.cpuCores} available`,
      impact: 'Severe performance degradation due to CPU contention',
      suggestedFix: `Reduce executor count or cores per executor to fit within ${hardware.cpuCores} cores`,
      metrics: { requested: totalCores, available: hardware.cpuCores },
    });
  }
  // Heavy utilization
  else if (coreUtilization > 0.90) {
    bottlenecks.push({
      type: 'cpu',
      severity: 'medium',
      description: `High CPU utilization: using ${(coreUtilization * 100).toFixed(0)}% of available cores`,
      impact: 'Limited headroom for system processes and driver overhead',
      suggestedFix: 'Reserve 10-20% of cores for system overhead',
    });
  }

  // Suboptimal executor cores
  if (config.executor.cores > 8) {
    bottlenecks.push({
      type: 'cpu',
      severity: 'medium',
      description: `Executor cores (${config.executor.cores}) exceeds recommended maximum of 8`,
      impact: 'May lead to increased GC overhead and reduced parallelism',
      suggestedFix: 'Use 4-8 cores per executor for optimal performance',
    });
  } else if (config.executor.cores < 2) {
    bottlenecks.push({
      type: 'cpu',
      severity: 'low',
      description: `Very low cores per executor (${config.executor.cores})`,
      impact: 'May underutilize resources and increase overhead',
      suggestedFix: 'Increase to at least 4 cores per executor',
    });
  }

  return bottlenecks;
}

/**
 * Check memory bottlenecks
 */
function checkMemoryBottlenecks(
  config: SparkConfig,
  hardware: HardwareContext,
  workloadType: string
): Bottleneck[] {
  const bottlenecks: Bottleneck[] = [];

  const executorMemGB = parseMemory(config.executor.memory);
  const driverMemGB = parseMemory(config.driver.memory);
  const totalMemGB = executorMemGB * (config.executor.instances ?? 1) + driverMemGB;

  const memUtilization = totalMemGB / hardware.totalMemory;

  // Over-subscription
  if (memUtilization > 1.0) {
    bottlenecks.push({
      type: 'memory',
      severity: 'critical',
      description: `Memory over-allocated: requesting ${totalMemGB}GB but only ${hardware.totalMemory}GB available`,
      impact: 'Will cause OOM errors and job failures',
      suggestedFix: `Reduce executor memory or count to fit within ${hardware.totalMemory}GB`,
      metrics: { requested: totalMemGB, available: hardware.totalMemory },
    });
  }
  // High utilization
  else if (memUtilization > 0.90) {
    bottlenecks.push({
      type: 'memory',
      severity: 'high',
      description: `High memory utilization: ${(memUtilization * 100).toFixed(0)}% of available memory`,
      impact: 'High risk of OOM errors and memory spilling',
      suggestedFix: 'Reserve at least 10% memory for OS and overhead',
    });
  }

  // Executor memory too large
  if (executorMemGB > 64) {
    bottlenecks.push({
      type: 'memory',
      severity: 'high',
      description: `Executor memory (${executorMemGB}GB) exceeds recommended maximum of 64GB`,
      impact: 'Will cause long GC pauses and poor performance',
      suggestedFix: 'Use more executors with less memory each (8-32GB per executor)',
    });
  }

  // Executor memory too small
  if (executorMemGB < 4 && workloadType !== 'streaming') {
    bottlenecks.push({
      type: 'memory',
      severity: 'medium',
      description: `Executor memory (${executorMemGB}GB) is very low`,
      impact: 'May cause frequent spilling and poor performance',
      suggestedFix: 'Increase executor memory to at least 4-8GB',
    });
  }

  // Driver memory issues
  if (driverMemGB > executorMemGB * 2) {
    bottlenecks.push({
      type: 'memory',
      severity: 'low',
      description: `Driver memory (${driverMemGB}GB) is much larger than executor memory (${executorMemGB}GB)`,
      impact: 'Possible inefficient resource allocation',
      suggestedFix: 'Driver memory should typically be 1-2x executor memory',
    });
  }

  return bottlenecks;
}

/**
 * Check GPU bottlenecks
 */
function checkGPUBottlenecks(
  config: SparkConfig,
  hardware: HardwareContext,
  workloadType: string
): Bottleneck[] {
  const bottlenecks: Bottleneck[] = [];

  if (!hardware.gpuCount || hardware.gpuCount === 0) {
    return bottlenecks; // No GPUs, no GPU bottlenecks
  }

  // GPU requested but not configured
  if (workloadType === 'ml-training' || workloadType === 'ml-inference') {
    if (!config.gpu?.enabled) {
      bottlenecks.push({
        type: 'gpu',
        severity: 'medium',
        description: 'ML workload but GPU acceleration not enabled',
        impact: 'Missing significant performance improvements from GPU acceleration',
        suggestedFix: 'Enable GPU acceleration and RAPIDS for ML workloads',
      });
    }
  }

  if (config.gpu?.enabled) {
    const gpusPerExecutor = config.gpu.amount ?? 1;
    const totalGPUs = gpusPerExecutor * (config.executor.instances ?? 1);

    // GPU over-subscription
    if (totalGPUs > hardware.gpuCount) {
      bottlenecks.push({
        type: 'gpu',
        severity: 'critical',
        description: `Requesting ${totalGPUs} GPUs but only ${hardware.gpuCount} available`,
        impact: 'GPU sharing will severely degrade performance',
        suggestedFix: `Reduce GPU allocation to ${hardware.gpuCount} or fewer`,
      });
    }

    // RAPIDS not enabled for suitable workloads
    if (!config.gpu.rapids?.enabled && (workloadType === 'ml-training' || workloadType === 'analytics')) {
      bottlenecks.push({
        type: 'gpu',
        severity: 'low',
        description: 'GPU enabled but RAPIDS not configured',
        impact: 'Missing RAPIDS-specific optimizations',
        suggestedFix: 'Enable RAPIDS for GPU-accelerated DataFrame operations',
      });
    }
  }

  return bottlenecks;
}

/**
 * Check I/O bottlenecks
 */
function checkIOBottlenecks(config: SparkConfig, workloadType: string): Bottleneck[] {
  const bottlenecks: Bottleneck[] = [];

  // No compression
  if (!config.shuffle.compress) {
    bottlenecks.push({
      type: 'io',
      severity: 'medium',
      description: 'Shuffle compression disabled',
      impact: 'Higher I/O and network traffic during shuffles',
      suggestedFix: 'Enable shuffle compression to reduce I/O',
    });
  }

  // Adaptive execution disabled
  if (!config.optimization.adaptiveExecution?.enabled) {
    bottlenecks.push({
      type: 'io',
      severity: 'low',
      description: 'Adaptive Query Execution (AQE) disabled',
      impact: 'Missing runtime optimizations for shuffle and join operations',
      suggestedFix: 'Enable AQE for better query optimization',
    });
  }

  // Broadcast threshold too low
  const broadcastThreshold = parseSizeString(config.optimization.autoBroadcastJoinThreshold ?? '10mb');
  if (broadcastThreshold < 10 * 1024 * 1024 && workloadType === 'analytics') {
    bottlenecks.push({
      type: 'io',
      severity: 'low',
      description: 'Broadcast join threshold may be too conservative',
      impact: 'May use shuffle joins where broadcast would be faster',
      suggestedFix: 'Consider increasing broadcast threshold to 10-20MB',
    });
  }

  return bottlenecks;
}

/**
 * Check shuffle bottlenecks
 */
function checkShuffleBottlenecks(config: SparkConfig, _hardware: HardwareContext): Bottleneck[] {
  const bottlenecks: Bottleneck[] = [];

  const totalCores = config.executor.cores * (config.executor.instances ?? 1);
  const shufflePartitions = config.shuffle.partitions;

  // Too few partitions
  if (shufflePartitions < totalCores * 2) {
    bottlenecks.push({
      type: 'shuffle',
      severity: 'high',
      description: `Shuffle partitions (${shufflePartitions}) less than 2x cores (${totalCores})`,
      impact: 'Underutilized parallelism and poor shuffle performance',
      suggestedFix: `Increase shuffle partitions to at least ${totalCores * 2}`,
    });
  }

  // Too many partitions
  if (shufflePartitions > totalCores * 10) {
    bottlenecks.push({
      type: 'shuffle',
      severity: 'medium',
      description: `Shuffle partitions (${shufflePartitions}) is very high relative to cores`,
      impact: 'Excessive overhead from too many small tasks',
      suggestedFix: `Reduce shuffle partitions to ${totalCores * 3}-${totalCores * 5}`,
    });
  }

  return bottlenecks;
}

/**
 * Check configuration bottlenecks
 */
function checkConfigurationBottlenecks(config: SparkConfig): Bottleneck[] {
  const bottlenecks: Bottleneck[] = [];

  // Using default Java serialization
  if (!config.serializer || config.serializer.includes('JavaSerializer')) {
    bottlenecks.push({
      type: 'configuration',
      severity: 'medium',
      description: 'Using default Java serialization',
      impact: 'Slower serialization and larger shuffle data',
      suggestedFix: 'Use Kryo serialization for better performance',
    });
  }

  // Off-heap memory not enabled
  if (!config.memory.offHeap.enabled) {
    bottlenecks.push({
      type: 'configuration',
      severity: 'low',
      description: 'Off-heap memory disabled',
      impact: 'May experience GC pressure for large in-memory datasets',
      suggestedFix: 'Enable off-heap memory to reduce GC overhead',
    });
  }

  // Speculation disabled for long-running jobs
  if (!config.speculation?.enabled) {
    bottlenecks.push({
      type: 'configuration',
      severity: 'low',
      description: 'Speculative execution disabled',
      impact: 'Slow tasks can delay job completion',
      suggestedFix: 'Enable speculation for better tail latency',
    });
  }

  return bottlenecks;
}

/**
 * Determine overall severity
 */
function determineOverallSeverity(bottlenecks: Bottleneck[]): BottleneckAnalysis['severity'] {
  if (bottlenecks.length === 0) return 'none';

  const hasCritical = bottlenecks.some(b => b.severity === 'critical');
  const hasHigh = bottlenecks.some(b => b.severity === 'high');
  const hasMedium = bottlenecks.some(b => b.severity === 'medium');

  if (hasCritical) return 'critical';
  if (hasHigh) return 'high';
  if (hasMedium) return 'medium';
  return 'low';
}

/**
 * Generate recommendations
 */
function generateBottleneckRecommendations(
  bottlenecks: Bottleneck[],
  workloadType: string
): string[] {
  const recommendations: string[] = [];

  if (bottlenecks.length === 0) {
    recommendations.push('No major bottlenecks detected. Configuration looks good.');
    return recommendations;
  }

  // Prioritize critical and high severity bottlenecks
  const critical = bottlenecks.filter(b => b.severity === 'critical' || b.severity === 'high');

  if (critical.length > 0) {
    recommendations.push('Address critical bottlenecks immediately:');
    critical.forEach(b => {
      recommendations.push(`- ${b.suggestedFix}`);
    });
  }

  // General recommendations
  const types = new Set(bottlenecks.map(b => b.type));

  if (types.has('memory') && types.has('cpu')) {
    recommendations.push('Both CPU and memory are bottlenecks. Consider increasing overall cluster resources.');
  }

  if (types.has('shuffle')) {
    recommendations.push('Shuffle operations are a bottleneck. Consider optimizing join strategies and data partitioning.');
  }

  if (types.has('gpu') && workloadType.includes('ml')) {
    recommendations.push('GPU configuration issues detected. Review GPU allocation and RAPIDS settings for ML workloads.');
  }

  return recommendations;
}

/**
 * Helper functions
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

function parseSizeString(size: string): number {
  const units: Record<string, number> = {
    b: 1,
    kb: 1024,
    mb: 1024 ** 2,
    gb: 1024 ** 3,
  };

  const match = size.toLowerCase().match(/^(\d+)([kmg]?b)$/);
  if (!match || !match[1] || !match[2]) return 0;

  const value = match[1];
  const unit = match[2];
  return parseInt(value, 10) * (units[unit] || 1);
}
