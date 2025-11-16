/**
 * CPU hardware type definitions
 */

export interface CPUCache {
  l1d?: number; // bytes
  l1i?: number; // bytes
  l2?: number; // bytes
  l3?: number; // bytes
}

export interface CPUCore {
  coreId: number;
  physicalId: number;
  processor: number;
  modelName: string;
  mhz: number;
  cacheSize: number;
  flags: string[];
}

export interface NUMANode {
  nodeId: number;
  cpus: number[];
  memoryTotal: number; // bytes
  memoryFree: number; // bytes
  distances: number[]; // relative distances to other NUMA nodes
}

export interface CPU {
  vendor: string;
  modelName: string;
  architecture: string;
  cores: {
    physical: number;
    logical: number;
  };
  threads: number;
  sockets: number;
  cache: CPUCache;
  frequency: {
    min: number; // MHz
    max: number; // MHz
    current: number; // MHz
  };
  flags: string[];
  virtualization?: string;
  numaNodes?: NUMANode[];
}

export interface CPUDetectionResult {
  cpu: CPU;
  cores?: CPUCore[];
  timestamp: number;
  detectionTime: number; // milliseconds
}
