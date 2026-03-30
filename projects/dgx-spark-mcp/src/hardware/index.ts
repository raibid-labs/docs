/**
 * Hardware detection module exports
 */

// Main detector API
export {
  detectAll,
  getTopology,
  refreshTopology,
  clearCache,
  getCacheStats,
  setCacheTTL,
  getHardwareSummary,
} from './detector.js';

// Individual detection modules
export {
  detectGPUs,
  getGPU,
  getGPUCount,
  hasNVIDIAGPUs,
  getTotalGPUMemory,
  getAvailableGPUMemory,
  getAverageGPUUtilization,
} from './gpu.js';

export {
  detectCPU,
  getCPUCount,
  getCPUModel,
  hasVirtualizationSupport,
  hasNUMA,
} from './cpu.js';

export {
  detectMemory,
  getTotalMemory,
  getAvailableMemory,
  getMemoryUtilization,
  hasSwap,
  hasHugepages,
} from './memory.js';

export {
  detectStorage,
  getTotalStorageCapacity,
  getAvailableStorage,
  getStorageUtilization,
  hasNVMe,
  hasRAID,
} from './storage.js';

export {
  detectNetwork,
  hasInfiniBand,
  getActiveInterfaces,
  getInterface,
} from './network.js';

// Topology and caching
export {
  buildSystemTopology,
  getHardwareSnapshot,
  refreshHardwareSnapshot,
  clearHardwareCache,
  getHardwareCacheStats,
  setHardwareCacheTTL,
  getHardwareComponent,
} from './topology.js';

export { HardwareCache, hardwareCache, CacheKeys } from './cache.js';

// nvidia-smi utilities
export {
  isNvidiaSmiAvailable,
  getDriverVersion,
  getCudaVersion,
  queryGPUs,
  getNVLinkTopology,
  getPCIeTopology,
  buildGPUTopology,
} from './nvidia-smi.js';
