/**
 * Resource estimation type definitions
 */

import { WorkloadCharacteristics } from './workload.js';

export interface ResourceEstimationRequest {
  description?: string;
  dataSize?: string | number;
  operations?: string[];
  workloadCharacteristics?: WorkloadCharacteristics;
  hardware?: HardwareContext;
}

export interface HardwareContext {
  cpuCores: number;
  totalMemory: number; // GB
  gpuCount?: number;
  gpuMemoryPerDevice?: number; // GB
  networkBandwidth?: number; // Gbps
  storageBandwidth?: number; // MB/s
}

export interface ResourceEstimate {
  memory: MemoryEstimate;
  compute: ComputeEstimate;
  storage: StorageEstimate;
  time: TimeEstimate;
  cost?: CostEstimate;
  confidence: number; // 0-1
  assumptions: string[];
}

export interface MemoryEstimate {
  executorMemoryGB: number;
  driverMemoryGB: number;
  totalMemoryGB: number;
  overheadGB: number;
  peakMemoryGB: number;
  breakdown: {
    execution: number;
    storage: number;
    overhead: number;
    offHeap?: number;
  };
  spillLikelihood: 'unlikely' | 'possible' | 'likely' | 'certain';
}

export interface ComputeEstimate {
  executorCores: number;
  executorCount: number;
  totalCores: number;
  parallelism: number;
  gpuRequirement?: {
    count: number;
    memoryPerGPU: number; // GB
    utilizationPercent: number;
  };
  cpuUtilizationPercent: number;
}

export interface StorageEstimate {
  inputDataGB: number;
  intermediateDataGB: number;
  outputDataGB: number;
  shuffleDataGB: number;
  totalIOGB: number;
  ioBandwidthMBps: number;
  tmpStorageGB: number;
}

export interface TimeEstimate {
  estimatedMinutes: number;
  range: {
    min: number;
    max: number;
  };
  breakdown: {
    inputIO: number;
    computation: number;
    shuffle: number;
    outputIO: number;
  };
  bottleneck?: 'cpu' | 'memory' | 'io' | 'shuffle' | 'gpu' | 'network';
}

export interface CostEstimate {
  computeCost: number; // USD
  storageCost: number;
  networkCost: number;
  totalCost: number;
  currency: string;
  billingModel?: 'spot' | 'on-demand' | 'reserved';
}

export interface ScalingPrediction {
  currentResources: ResourceEstimate;
  scalingOptions: ScalingOption[];
  recommendations: string[];
}

export interface ScalingOption {
  scale: number; // multiplier (e.g., 2x, 4x)
  resources: ResourceEstimate;
  expectedSpeedup: number; // e.g., 1.8 for 1.8x faster
  efficiency: number; // 0-1, how well resources are utilized
  costIncrease?: number; // percentage
  rationale: string;
}

export interface PerformanceMetrics {
  throughputMBps: number;
  recordsPerSecond?: number;
  cpuEfficiency: number; // 0-1
  memoryEfficiency: number; // 0-1
  ioEfficiency: number; // 0-1
  gpuEfficiency?: number; // 0-1
  overallScore: number; // 0-100
}

export interface EstimationModel {
  name: string;
  version: string;
  accuracy?: number; // historical accuracy percentage
  lastCalibrated?: number; // timestamp
  parameters: Record<string, number>;
}
