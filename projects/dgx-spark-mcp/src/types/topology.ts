/**
 * System topology type definitions
 */

import { GPU, GPUTopology } from './gpu.js';
import { CPU } from './cpu.js';
import { Memory } from './memory.js';
import { Storage } from './storage.js';
import { Network } from './network.js';

export interface SystemCapabilities {
  hasNVIDIA: boolean;
  hasInfiniBand: boolean;
  hasNVMe: boolean;
  hasRAID: boolean;
  hasNUMA: boolean;
  hasVirtualization: boolean;
}

export interface SystemTopology {
  cpu: CPU;
  memory: Memory;
  gpus?: GPU[];
  gpuTopology?: GPUTopology;
  storage: Storage;
  network: Network;
  capabilities: SystemCapabilities;
  hostname: string;
  kernel: string;
  os: string;
  uptime: number; // seconds
}

export interface HardwareSnapshot {
  topology: SystemTopology;
  timestamp: number;
  detectionTime: number; // milliseconds
  cached: boolean;
}

export interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number; // milliseconds
}

export interface DetectionOptions {
  includeGPU?: boolean;
  includeCPU?: boolean;
  includeMemory?: boolean;
  includeStorage?: boolean;
  includeNetwork?: boolean;
  useCache?: boolean;
  cacheTTL?: number; // milliseconds
}
