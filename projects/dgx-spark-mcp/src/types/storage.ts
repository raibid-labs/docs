/**
 * Storage hardware type definitions
 */

export interface BlockDevice {
  name: string;
  type: string;
  size: number; // bytes
  mountpoint?: string;
  fstype?: string;
  model?: string;
  serial?: string;
  uuid?: string;
  children?: BlockDevice[];
}

export interface MountPoint {
  device: string;
  mountpoint: string;
  fstype: string;
  size: number; // bytes
  used: number; // bytes
  available: number; // bytes
  usedPercent: number;
}

export interface NVMeDevice {
  device: string;
  model: string;
  serial: string;
  firmware: string;
  totalCapacity: number; // bytes
  usedCapacity: number; // bytes
  temperatureCelsius?: number;
  criticalWarning?: number;
}

export interface RAIDArray {
  name: string;
  level: string;
  devices: string[];
  state: string;
  size: number; // bytes
  usedSize: number; // bytes
}

export interface Storage {
  blockDevices: BlockDevice[];
  mountPoints: MountPoint[];
  nvmeDevices?: NVMeDevice[];
  raidArrays?: RAIDArray[];
  totalCapacity: number; // bytes
  totalUsed: number; // bytes
  totalAvailable: number; // bytes
}

export interface StorageDetectionResult {
  storage: Storage;
  timestamp: number;
  detectionTime: number; // milliseconds
}
