/**
 * Memory hardware type definitions
 */

export interface MemoryInfo {
  total: number; // bytes
  available: number; // bytes
  used: number; // bytes
  free: number; // bytes
  shared: number; // bytes
  buffers: number; // bytes
  cached: number; // bytes
  swapTotal: number; // bytes
  swapFree: number; // bytes
  swapUsed: number; // bytes
}

export interface MemoryModule {
  locator: string;
  size: number; // bytes
  type: string;
  speed: number; // MHz
  manufacturer?: string;
  partNumber?: string;
  serialNumber?: string;
}

export interface Memory {
  info: MemoryInfo;
  modules?: MemoryModule[];
  hugepages?: {
    total: number;
    free: number;
    size: number; // bytes
  };
}

export interface MemoryDetectionResult {
  memory: Memory;
  timestamp: number;
  detectionTime: number; // milliseconds
}
