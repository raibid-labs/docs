/**
 * System topology orchestrator
 */

import { SystemTopology, SystemCapabilities, HardwareSnapshot, DetectionOptions } from '../types/topology.js';
import { detectGPUs, hasNVIDIAGPUs } from './gpu.js';
import { detectCPU, hasNUMA, hasVirtualizationSupport } from './cpu.js';
import { detectMemory } from './memory.js';
import { detectStorage, hasNVMe, hasRAID } from './storage.js';
import { detectNetwork, hasInfiniBand } from './network.js';
import { executeCommand } from '../utils/exec.js';
import { hardwareCache, CacheKeys } from './cache.js';

/**
 * Build complete system topology
 */
export async function buildSystemTopology(options?: DetectionOptions): Promise<SystemTopology> {
  const includeGPU = options?.includeGPU !== false;
  const includeCPU = options?.includeCPU !== false;
  const includeMemory = options?.includeMemory !== false;
  const includeStorage = options?.includeStorage !== false;
  const includeNetwork = options?.includeNetwork !== false;

  // Detect CPU
  const cpuResult = includeCPU ? await detectCPU() : null;
  if (!cpuResult) {
    throw new Error('Failed to detect CPU');
  }

  // Detect memory
  const memoryResult = includeMemory ? await detectMemory(false) : null;
  if (!memoryResult) {
    throw new Error('Failed to detect memory');
  }

  // Detect GPUs (optional)
  let gpuResult = null;
  if (includeGPU) {
    try {
      gpuResult = await detectGPUs(true);
    } catch (error) {
      // GPU detection failed, continue without GPUs
      gpuResult = null;
    }
  }

  // Detect storage
  const storageResult = includeStorage ? await detectStorage(true, true) : null;
  if (!storageResult) {
    throw new Error('Failed to detect storage');
  }

  // Detect network
  const networkResult = includeNetwork ? await detectNetwork(true) : null;
  if (!networkResult) {
    throw new Error('Failed to detect network');
  }

  // Detect system capabilities
  const capabilities = await detectCapabilities();

  // Get system information
  const hostname = await getHostname();
  const kernel = await getKernelVersion();
  const os = await getOSInfo();
  const uptime = await getUptime();

  const topology: SystemTopology = {
    cpu: cpuResult.cpu,
    memory: memoryResult.memory,
    gpus: gpuResult?.gpus,
    gpuTopology: gpuResult?.topology,
    storage: storageResult.storage,
    network: networkResult.network,
    capabilities,
    hostname,
    kernel,
    os,
    uptime,
  };

  return topology;
}

/**
 * Detect system capabilities
 */
async function detectCapabilities(): Promise<SystemCapabilities> {
  const [hasGPU, hasIB, hasNVMeDev, hasRAIDDev, hasNUMADev, hasVirt] = await Promise.all([
    hasNVIDIAGPUs(),
    hasInfiniBand(),
    hasNVMe(),
    hasRAID(),
    hasNUMA(),
    hasVirtualizationSupport(),
  ]);

  return {
    hasNVIDIA: hasGPU,
    hasInfiniBand: hasIB,
    hasNVMe: hasNVMeDev,
    hasRAID: hasRAIDDev,
    hasNUMA: hasNUMADev,
    hasVirtualization: hasVirt,
  };
}

/**
 * Get system hostname
 */
async function getHostname(): Promise<string> {
  const result = await executeCommand('hostname');
  return result.stdout || 'unknown';
}

/**
 * Get kernel version
 */
async function getKernelVersion(): Promise<string> {
  const result = await executeCommand('uname -r');
  return result.stdout || 'unknown';
}

/**
 * Get OS information
 */
async function getOSInfo(): Promise<string> {
  const result = await executeCommand('cat /etc/os-release | grep PRETTY_NAME | cut -d= -f2 | tr -d \'"\'');
  if (result.exitCode === 0 && result.stdout) {
    return result.stdout;
  }

  // Fallback
  const fallback = await executeCommand('uname -o');
  return fallback.stdout || 'Linux';
}

/**
 * Get system uptime in seconds
 */
async function getUptime(): Promise<number> {
  const result = await executeCommand('cat /proc/uptime | cut -d. -f1');
  if (result.exitCode === 0 && result.stdout) {
    return parseInt(result.stdout, 10) || 0;
  }
  return 0;
}

/**
 * Get hardware snapshot with caching
 */
export async function getHardwareSnapshot(options?: DetectionOptions): Promise<HardwareSnapshot> {
  const startTime = Date.now();
  const useCache = options?.useCache !== false;
  const cacheTTL = options?.cacheTTL;

  // Check cache
  if (useCache) {
    const cached = hardwareCache.get<SystemTopology>(CacheKeys.SYSTEM_TOPOLOGY);
    if (cached) {
      const detectionTime = Date.now() - startTime;
      return {
        topology: cached,
        timestamp: Date.now(),
        detectionTime,
        cached: true,
      };
    }
  }

  // Build topology
  const topology = await buildSystemTopology(options);

  // Cache result
  if (useCache) {
    hardwareCache.set(CacheKeys.SYSTEM_TOPOLOGY, topology, cacheTTL);
  }

  const detectionTime = Date.now() - startTime;

  return {
    topology,
    timestamp: Date.now(),
    detectionTime,
    cached: false,
  };
}

/**
 * Refresh hardware snapshot (invalidate cache and re-detect)
 */
export async function refreshHardwareSnapshot(options?: DetectionOptions): Promise<HardwareSnapshot> {
  // Invalidate cache
  hardwareCache.invalidate(CacheKeys.SYSTEM_TOPOLOGY);

  // Get fresh snapshot
  return getHardwareSnapshot(options);
}

/**
 * Get specific hardware component with caching
 */
export async function getHardwareComponent<T>(
  _component: 'gpu' | 'cpu' | 'memory' | 'storage' | 'network',
  detectFn: () => Promise<T>,
  cacheKey: string,
  useCache: boolean = true,
  cacheTTL?: number
): Promise<T> {
  // Check cache
  if (useCache) {
    const cached = hardwareCache.get<T>(cacheKey);
    if (cached) {
      return cached;
    }
  }

  // Detect
  const result = await detectFn();

  // Cache result
  if (useCache) {
    hardwareCache.set(cacheKey, result, cacheTTL);
  }

  return result;
}

/**
 * Clear all hardware detection cache
 */
export function clearHardwareCache(): void {
  hardwareCache.clear();
}

/**
 * Get cache statistics
 */
export function getHardwareCacheStats() {
  return hardwareCache.getStats();
}

/**
 * Set default cache TTL
 */
export function setHardwareCacheTTL(ttl: number): void {
  hardwareCache.setDefaultTTL(ttl);
}
