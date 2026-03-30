/**
 * GPU hardware type definitions
 */

export interface GPUMemory {
  total: number; // bytes
  used: number; // bytes
  free: number; // bytes
}

export interface GPUUtilization {
  gpu: number; // percentage
  memory: number; // percentage
}

export interface GPUPower {
  current: number; // watts
  limit: number; // watts
  default: number; // watts
}

export interface GPUTemperature {
  current: number; // celsius
  max: number; // celsius
  slowdown: number; // celsius
  shutdown: number; // celsius
}

export interface GPUClocks {
  graphics: number; // MHz
  sm: number; // MHz
  memory: number; // MHz
  video: number; // MHz
}

export interface NVLinkConnection {
  gpu: number;
  link: number;
  connected: boolean;
  bandwidth: number; // GB/s
}

export interface GPU {
  id: number;
  uuid: string;
  name: string;
  busId: string;
  memory: GPUMemory;
  utilization: GPUUtilization;
  temperature: GPUTemperature;
  power: GPUPower;
  clocks: GPUClocks;
  computeCapability: {
    major: number;
    minor: number;
  };
  driverVersion: string;
  cudaVersion: string;
  nvlinks?: NVLinkConnection[];
}

export interface GPUTopology {
  gpus: GPU[];
  nvlinkMatrix: number[][]; // GPU-to-GPU NVLink bandwidth matrix
  pcieTopology: PCIeTopology[];
}

export interface PCIeTopology {
  busId: string;
  gpuId: number;
  generation: number;
  width: number;
  maxWidth: number;
}

export interface GPUDetectionResult {
  gpus: GPU[];
  topology?: GPUTopology;
  timestamp: number;
  detectionTime: number; // milliseconds
}
