/**
 * Spark configuration type definitions
 */

export interface ExecutorConfig {
  memory: string; // e.g., "8g", "16g"
  memoryOverhead: string; // e.g., "2g"
  cores: number;
  instances?: number;
  offHeapMemory?: string;
}

export interface DriverConfig {
  memory: string;
  memoryOverhead: string;
  cores: number;
  maxResultSize?: string;
}

export interface ShuffleConfig {
  partitions: number;
  compress: boolean;
  spill: boolean;
  spillCompress: boolean;
  memoryFraction: number;
  fileBuffer: string; // e.g., "64k"
  sort: {
    bypassMergeThreshold: number;
  };
}

export interface MemoryConfig {
  fraction: number;
  storageFraction: number;
  offHeap: {
    enabled: boolean;
    size?: string;
  };
}

export interface GPUConfig {
  enabled: boolean;
  resourceName?: string; // e.g., "nvidia.com/gpu"
  amount?: number;
  rapids?: {
    enabled: boolean;
    sqlEnabled: boolean;
    memoryFraction?: number;
    pooling?: {
      enabled: boolean;
      mode?: 'ARENA' | 'ASYNC';
    };
  };
}

export interface DynamicAllocationConfig {
  enabled: boolean;
  initialExecutors?: number;
  minExecutors?: number;
  maxExecutors?: number;
  executorIdleTimeout?: string; // e.g., "60s"
  cachedExecutorIdleTimeout?: string;
  schedulerBacklogTimeout?: string;
  sustainedSchedulerBacklogTimeout?: string;
}

export interface OptimizationConfig {
  adaptiveExecution?: {
    enabled: boolean;
    coalescePartitions?: boolean;
    skewJoin?: boolean;
  };
  autoBroadcastJoinThreshold?: string; // e.g., "10mb"
  broadcastTimeout?: string; // e.g., "300s"
  sql?: {
    inMemoryColumnarStorageCompressed: boolean;
    inMemoryColumnarStorageBatchSize: number;
  };
}

export interface SparkConfig {
  executor: ExecutorConfig;
  driver: DriverConfig;
  shuffle: ShuffleConfig;
  memory: MemoryConfig;
  gpu?: GPUConfig;
  dynamicAllocation: DynamicAllocationConfig;
  optimization: OptimizationConfig;
  serializer?: string; // e.g., "org.apache.spark.serializer.KryoSerializer"
  kryo?: {
    registrationRequired: boolean;
    referenceTracking: boolean;
  };
  locality?: {
    wait: string; // e.g., "3s"
  };
  speculation?: {
    enabled: boolean;
    interval?: string;
    multiplier?: number;
  };
}

export interface SparkConfigRequest {
  workloadType: WorkloadType;
  dataSize: string | number; // e.g., "1TB" or bytes
  gpuCount?: number;
  totalMemory?: number; // GB
  totalCores?: number;
  description?: string;
  constraints?: ConfigConstraints;
}

export interface ConfigConstraints {
  maxExecutorMemory?: number;
  maxDriverMemory?: number;
  maxExecutors?: number;
  minExecutors?: number;
  preferredExecutorCores?: number;
  enableGPU?: boolean;
  enableDynamicAllocation?: boolean;
}

export type WorkloadType =
  | 'etl'
  | 'analytics'
  | 'ml-training'
  | 'ml-inference'
  | 'streaming'
  | 'graph'
  | 'sql'
  | 'mixed';

export interface ConfigValidationResult {
  valid: boolean;
  errors: ConfigValidationError[];
  warnings: ConfigValidationWarning[];
  suggestions: string[];
}

export interface ConfigValidationError {
  field: string;
  message: string;
  severity: 'error';
  fix?: string;
}

export interface ConfigValidationWarning {
  field: string;
  message: string;
  severity: 'warning';
  suggestion?: string;
}

export interface OptimizationResult {
  config: SparkConfig;
  estimatedPerformance: {
    executionTimeMinutes?: number;
    throughputMBps?: number;
    resourceEfficiency?: number; // 0-1
  };
  rationale: string[];
  alternatives?: Array<{
    config: SparkConfig;
    tradeoff: string;
  }>;
}
