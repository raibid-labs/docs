/**
 * Workload analysis type definitions
 */

import { WorkloadType } from './spark-config.js';

export interface WorkloadCharacteristics {
  type: WorkloadType;
  dataSize: number; // bytes
  partitionCount?: number;
  computeIntensity: ComputeIntensity;
  ioPattern: IOPattern;
  gpuUtilization: GPUUtilization;
  memoryFootprint: MemoryFootprint;
  shuffleIntensity: ShuffleIntensity;
  confidence: number; // 0-1
}

export type ComputeIntensity = 'low' | 'medium' | 'high' | 'very-high';
export type IOPattern = 'sequential' | 'random' | 'mixed' | 'streaming';
export type GPUUtilization = 'none' | 'low' | 'medium' | 'high' | 'full';
export type ShuffleIntensity = 'none' | 'light' | 'moderate' | 'heavy' | 'extreme';

export interface MemoryFootprint {
  estimatedPeakGB: number;
  cacheRequirementGB?: number;
  spillRisk: 'low' | 'medium' | 'high';
}

export interface WorkloadAnalysisRequest {
  description?: string;
  operations?: string[];
  dataSize?: string | number;
  sqlQuery?: string;
  codeSnippet?: string;
  historicalMetrics?: HistoricalWorkloadMetrics;
}

export interface HistoricalWorkloadMetrics {
  previousExecutionTimeMs?: number;
  previousDataSize?: number;
  previousResourceUsage?: {
    memoryGB: number;
    cpuCores: number;
    gpuCount?: number;
  };
  shuffleReadMB?: number;
  shuffleWriteMB?: number;
  inputRecords?: number;
  outputRecords?: number;
}

export interface WorkloadClassificationResult {
  characteristics: WorkloadCharacteristics;
  recommendedResources: {
    executorMemoryGB: number;
    executorCores: number;
    executorCount: number;
    gpuCount?: number;
  };
  optimizationHints: string[];
}

export interface OperationProfile {
  operation: string;
  category: 'transformation' | 'action' | 'io' | 'shuffle';
  computeCost: number; // relative cost 1-10
  memoryImpact: number; // relative impact 1-10
  gpuAccelerable: boolean;
  ioIntensive: boolean;
}

export interface DatasetProfile {
  sizeBytes: number;
  recordCount?: number;
  schema?: {
    columns: number;
    complexTypes: number;
    nestedDepth: number;
  };
  format?: 'parquet' | 'csv' | 'json' | 'avro' | 'orc' | 'text' | 'binary';
  compression?: 'none' | 'gzip' | 'snappy' | 'lz4' | 'zstd';
  partitioned: boolean;
  partitionCount?: number;
}

export interface WorkloadPattern {
  name: string;
  description: string;
  indicators: string[];
  recommendedType: WorkloadType;
  typicalCharacteristics: Partial<WorkloadCharacteristics>;
}
