/**
 * Spark configuration type definitions
 */

export type WorkloadType = 'etl' | 'ml-training' | 'ml-inference' | 'analytics' | 'streaming';

export interface SparkConfig {
  master: string;
  appName: string;
  executor: {
    instances: number;
    cores: number;
    memory: string;
    memoryOverhead: string;
  };
  driver: {
    cores: number;
    memory: string;
    memoryOverhead: string;
  };
  shuffle: {
    partitions: number;
    compress: boolean;
    spill: {
      compress: boolean;
    };
  };
  serializer: string;
  kryoRegistrationRequired?: boolean;
  dynamicAllocation: {
    enabled: boolean;
    minExecutors?: number;
    maxExecutors?: number;
    initialExecutors?: number;
  };
  sql?: {
    adaptive: {
      enabled: boolean;
      coalescePartitions: {
        enabled: boolean;
      };
    };
  };
  rapids?: {
    enabled: boolean;
    memory: {
      pinnedPoolSize: string;
    };
    sql: {
      enabled: boolean;
      concurrentGpuTasks: number;
    };
  };
  extraJavaOptions?: string[];
  extraLibraryPath?: string[];
}

export interface SparkConfigRecommendation {
  config: SparkConfig;
  explanation: string;
  warnings?: string[];
  estimatedResources: {
    totalMemory: string;
    totalCores: number;
    gpuCount?: number;
  };
  optimizationTips: string[];
}

export interface ResourceEstimate {
  recommendedExecutors: number;
  recommendedCoresPerExecutor: number;
  recommendedMemoryPerExecutor: string;
  recommendedDriverMemory: string;
  estimatedProcessingTime?: string;
  gpuRecommendation?: {
    useGPU: boolean;
    gpuCount: number;
    reason: string;
  };
}

/**
 * Workload characteristics
 */
export interface WorkloadCharacteristics {
  type: WorkloadType;
  dataSizeBytes: number;
  isGPUAccelerated: boolean;
  requiresHighMemory: boolean;
  requiresHighConcurrency: boolean;
  shuffleIntensive: boolean;
}

/**
 * Parse data size string to bytes
 */
export function parseDataSize(size: string): number {
  const units: Record<string, number> = {
    'B': 1,
    'KB': 1024,
    'MB': 1024 * 1024,
    'GB': 1024 * 1024 * 1024,
    'TB': 1024 * 1024 * 1024 * 1024,
    'PB': 1024 * 1024 * 1024 * 1024 * 1024,
  };

  const match = size.trim().match(/^(\d+(?:\.\d+)?)\s*([A-Z]+)$/i);
  if (!match) {
    throw new Error(`Invalid data size format: ${size}`);
  }

  const [, value, unit] = match;

  if (!value || !unit) {
    throw new Error(`Invalid data size format: ${size}`);
  }

  const multiplier = units[unit.toUpperCase()];

  if (!multiplier) {
    throw new Error(`Unknown unit: ${unit}`);
  }

  return parseFloat(value) * multiplier;
}

/**
 * Format bytes to human-readable string
 */
export function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  let value = bytes;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }

  return `${Math.round(value * 100) / 100}${units[unitIndex]}`;
}
