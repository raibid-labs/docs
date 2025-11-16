/**
 * Main hardware detection module
 *
 * This module provides the primary API for detecting all hardware components
 * and orchestrates the complete hardware detection process.
 */

import { DetectionOptions, HardwareSnapshot } from '../types/topology.js';
import { GPUDetectionResult } from '../types/gpu.js';
import { CPUDetectionResult } from '../types/cpu.js';
import { MemoryDetectionResult } from '../types/memory.js';
import { StorageDetectionResult } from '../types/storage.js';
import { NetworkDetectionResult } from '../types/network.js';

import { detectGPUs } from './gpu.js';
import { detectCPU } from './cpu.js';
import { detectMemory } from './memory.js';
import { detectStorage } from './storage.js';
import { detectNetwork } from './network.js';
import { getHardwareSnapshot, refreshHardwareSnapshot, clearHardwareCache, getHardwareCacheStats, setHardwareCacheTTL } from './topology.js';

/**
 * Detection results for all hardware components
 */
export interface AllHardwareDetectionResult {
  gpu?: GPUDetectionResult;
  cpu: CPUDetectionResult;
  memory: MemoryDetectionResult;
  storage: StorageDetectionResult;
  network: NetworkDetectionResult;
  timestamp: number;
  totalDetectionTime: number;
}

/**
 * Detect all hardware components
 */
export async function detectAll(options?: DetectionOptions): Promise<AllHardwareDetectionResult> {
  const startTime = Date.now();

  const includeGPU = options?.includeGPU !== false;
  const includeCPU = options?.includeCPU !== false;
  const includeMemory = options?.includeMemory !== false;
  const includeStorage = options?.includeStorage !== false;
  const includeNetwork = options?.includeNetwork !== false;

  // Run detections in parallel where possible
  const detectionPromises: Promise<any>[] = [];

  let gpuPromise: Promise<GPUDetectionResult> | null = null;
  let cpuPromise: Promise<CPUDetectionResult> | null = null;
  let memoryPromise: Promise<MemoryDetectionResult> | null = null;
  let storagePromise: Promise<StorageDetectionResult> | null = null;
  let networkPromise: Promise<NetworkDetectionResult> | null = null;

  if (includeGPU) {
    gpuPromise = detectGPUs(true).catch(() => null as any);
    detectionPromises.push(gpuPromise);
  }

  if (includeCPU) {
    cpuPromise = detectCPU();
    detectionPromises.push(cpuPromise);
  }

  if (includeMemory) {
    memoryPromise = detectMemory(false);
    detectionPromises.push(memoryPromise);
  }

  if (includeStorage) {
    storagePromise = detectStorage(true, true);
    detectionPromises.push(storagePromise);
  }

  if (includeNetwork) {
    networkPromise = detectNetwork(true);
    detectionPromises.push(networkPromise);
  }

  // Wait for all detections
  await Promise.all(detectionPromises);

  const totalDetectionTime = Date.now() - startTime;

  return {
    gpu: gpuPromise ? await gpuPromise : undefined,
    cpu: cpuPromise ? await cpuPromise : null as any,
    memory: memoryPromise ? await memoryPromise : null as any,
    storage: storagePromise ? await storagePromise : null as any,
    network: networkPromise ? await networkPromise : null as any,
    timestamp: Date.now(),
    totalDetectionTime,
  };
}

/**
 * Get complete system topology with caching
 */
export async function getTopology(options?: DetectionOptions): Promise<HardwareSnapshot> {
  return getHardwareSnapshot(options);
}

/**
 * Refresh system topology (bypass cache)
 */
export async function refreshTopology(options?: DetectionOptions): Promise<HardwareSnapshot> {
  return refreshHardwareSnapshot(options);
}

/**
 * Clear hardware detection cache
 */
export function clearCache(): void {
  clearHardwareCache();
}

/**
 * Get cache statistics
 */
export function getCacheStats() {
  return getHardwareCacheStats();
}

/**
 * Set cache TTL
 */
export function setCacheTTL(ttl: number): void {
  setHardwareCacheTTL(ttl);
}

/**
 * Quick hardware summary
 */
export async function getHardwareSummary(): Promise<{
  hostname: string;
  cpu: string;
  cpuCores: { physical: number; logical: number };
  memoryGB: number;
  gpuCount: number;
  gpuModel?: string;
  storageGB: number;
  networkInterfaces: number;
  hasInfiniBand: boolean;
  hasNVMe: boolean;
}> {
  const snapshot = await getTopology({ useCache: true });
  const { topology } = snapshot;

  const memoryGB = Math.round(topology.memory.info.total / (1024 * 1024 * 1024));
  const storageGB = Math.round(topology.storage.totalCapacity / (1024 * 1024 * 1024));

  return {
    hostname: topology.hostname,
    cpu: topology.cpu.modelName,
    cpuCores: topology.cpu.cores,
    memoryGB,
    gpuCount: topology.gpus?.length || 0,
    gpuModel: topology.gpus?.[0]?.name,
    storageGB,
    networkInterfaces: topology.network.totalInterfaces,
    hasInfiniBand: topology.capabilities.hasInfiniBand,
    hasNVMe: topology.capabilities.hasNVMe,
  };
}

// Export all detection functions
export { detectGPUs } from './gpu.js';
export { detectCPU } from './cpu.js';
export { detectMemory } from './memory.js';
export { detectStorage } from './storage.js';
export { detectNetwork } from './network.js';
