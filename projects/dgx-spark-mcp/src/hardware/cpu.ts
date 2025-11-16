/**
 * CPU detection module
 */

import { CPU, CPUDetectionResult, CPUCore, NUMANode, CPUCache } from '../types/cpu.js';
import { parseCpuInfo, readSysFile, readNumericSysFile } from '../utils/proc-parser.js';
import { executeCommand } from '../utils/exec.js';

/**
 * Detect CPU information from /proc/cpuinfo
 */
export async function detectCPU(): Promise<CPUDetectionResult> {
  const startTime = Date.now();

  const processors = await parseCpuInfo();

  if (processors.length === 0) {
    throw new Error('Failed to parse /proc/cpuinfo');
  }

  // Get first processor for general info
  const first = processors[0]!;

  const modelName = first.get('model name') || 'Unknown';
  const vendor = first.get('vendor_id') || 'Unknown';
  const architecture = await detectArchitecture();
  const flags = (first.get('flags') || '').split(/\s+/).filter(f => f.length > 0);

  // Count physical cores and sockets
  const physicalIds = new Set(processors.map(p => p.get('physical id')).filter(id => id !== undefined));
  const coreIds = new Set(processors.map(p => p.get('core id')).filter(id => id !== undefined));

  const sockets = physicalIds.size || 1;
  const coresPerSocket = coreIds.size || processors.length;
  const physicalCores = sockets * coresPerSocket;
  const logicalCores = processors.length;

  // Parse cache information
  const cache = await detectCache(first);

  // Get frequency information
  const frequency = await detectFrequency(first);

  // Detect virtualization
  const virtualization = detectVirtualization(flags);

  // Detect NUMA
  const numaNodes = await detectNUMA();

  const cpu: CPU = {
    vendor,
    modelName,
    architecture,
    cores: {
      physical: physicalCores,
      logical: logicalCores,
    },
    threads: logicalCores,
    sockets,
    cache,
    frequency,
    flags,
    virtualization,
    numaNodes: numaNodes.length > 0 ? numaNodes : undefined,
  };

  const cores = processors.map((proc, idx) => {
    const core: CPUCore = {
      coreId: parseInt(proc.get('core id') || '0', 10),
      physicalId: parseInt(proc.get('physical id') || '0', 10),
      processor: parseInt(proc.get('processor') || String(idx), 10),
      modelName: proc.get('model name') || modelName,
      mhz: parseFloat(proc.get('cpu MHz') || '0'),
      cacheSize: parseInt((proc.get('cache size') || '0').replace(/\D/g, ''), 10) * 1024,
      flags: (proc.get('flags') || '').split(/\s+/).filter(f => f.length > 0),
    };
    return core;
  });

  const detectionTime = Date.now() - startTime;

  return {
    cpu,
    cores,
    timestamp: Date.now(),
    detectionTime,
  };
}

/**
 * Detect CPU architecture
 */
async function detectArchitecture(): Promise<string> {
  const arch = await readSysFile('/proc/sys/kernel/arch');
  if (arch) return arch;

  const result = await executeCommand('uname -m');
  return result.stdout || 'unknown';
}

/**
 * Detect CPU cache sizes
 */
async function detectCache(procInfo: Map<string, string>): Promise<CPUCache> {
  const cache: CPUCache = {};

  // Try to get from /proc/cpuinfo
  const cacheSizeStr = procInfo.get('cache size') || '';
  const match = cacheSizeStr.match(/(\d+)\s*KB/i);
  if (match) {
    cache.l3 = parseInt(match[1]!, 10) * 1024;
  }

  // Try to get from /sys/devices/system/cpu
  const l1d = await readNumericSysFile('/sys/devices/system/cpu/cpu0/cache/index0/size');
  const l1i = await readNumericSysFile('/sys/devices/system/cpu/cpu0/cache/index1/size');
  const l2 = await readNumericSysFile('/sys/devices/system/cpu/cpu0/cache/index2/size');
  const l3 = await readNumericSysFile('/sys/devices/system/cpu/cpu0/cache/index3/size');

  if (l1d !== null) cache.l1d = l1d;
  if (l1i !== null) cache.l1i = l1i;
  if (l2 !== null) cache.l2 = l2;
  if (l3 !== null) cache.l3 = l3;

  return cache;
}

/**
 * Detect CPU frequency
 */
async function detectFrequency(procInfo: Map<string, string>): Promise<{ min: number; max: number; current: number }> {
  const current = parseFloat(procInfo.get('cpu MHz') || '0');

  // Try to get min/max from /sys
  const minFreq = await readNumericSysFile('/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq');
  const maxFreq = await readNumericSysFile('/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq');

  return {
    min: minFreq !== null ? minFreq / 1000 : current, // Convert kHz to MHz
    max: maxFreq !== null ? maxFreq / 1000 : current,
    current,
  };
}

/**
 * Detect virtualization support
 */
function detectVirtualization(flags: string[]): string | undefined {
  if (flags.includes('vmx')) return 'Intel VT-x';
  if (flags.includes('svm')) return 'AMD-V';
  return undefined;
}

/**
 * Detect NUMA topology
 */
async function detectNUMA(): Promise<NUMANode[]> {
  const result = await executeCommand('numactl --hardware 2>/dev/null');

  if (result.exitCode !== 0) {
    return [];
  }

  const nodes: NUMANode[] = [];
  const lines = result.stdout.split('\n');

  let currentNode: Partial<NUMANode> | null = null;

  for (const line of lines) {
    const trimmed = line.trim();

    // Parse node lines
    const nodeMatch = trimmed.match(/^node (\d+) cpus: (.+)$/);
    if (nodeMatch) {
      const nodeId = parseInt(nodeMatch[1]!, 10);
      const cpusStr = nodeMatch[2]!;
      const cpus = cpusStr.split(/\s+/).map(c => parseInt(c, 10)).filter(c => !isNaN(c));

      currentNode = {
        nodeId,
        cpus,
        memoryTotal: 0,
        memoryFree: 0,
        distances: [],
      };
    }

    // Parse memory size
    const sizeMatch = trimmed.match(/^node (\d+) size: (\d+) MB$/);
    if (sizeMatch && currentNode && currentNode.nodeId === parseInt(sizeMatch[1]!, 10)) {
      currentNode.memoryTotal = parseInt(sizeMatch[2]!, 10) * 1024 * 1024;
    }

    // Parse free memory
    const freeMatch = trimmed.match(/^node (\d+) free: (\d+) MB$/);
    if (freeMatch && currentNode && currentNode.nodeId === parseInt(freeMatch[1]!, 10)) {
      currentNode.memoryFree = parseInt(freeMatch[2]!, 10) * 1024 * 1024;
    }

    // Parse distances
    const distMatch = trimmed.match(/^node (\d+) distances: (.+)$/);
    if (distMatch && currentNode && currentNode.nodeId === parseInt(distMatch[1]!, 10)) {
      const distances = distMatch[2]!.split(/\s+/).map(d => parseInt(d, 10)).filter(d => !isNaN(d));
      currentNode.distances = distances;

      // Complete current node
      if (currentNode.nodeId !== undefined && currentNode.cpus && currentNode.distances) {
        nodes.push(currentNode as NUMANode);
      }
      currentNode = null;
    }
  }

  return nodes;
}

/**
 * Get CPU count
 */
export async function getCPUCount(): Promise<{ physical: number; logical: number }> {
  const result = await detectCPU();
  return result.cpu.cores;
}

/**
 * Get CPU model name
 */
export async function getCPUModel(): Promise<string> {
  const result = await detectCPU();
  return result.cpu.modelName;
}

/**
 * Check if CPU supports virtualization
 */
export async function hasVirtualizationSupport(): Promise<boolean> {
  const result = await detectCPU();
  return result.cpu.virtualization !== undefined;
}

/**
 * Check if system has NUMA
 */
export async function hasNUMA(): Promise<boolean> {
  const result = await detectCPU();
  return result.cpu.numaNodes !== undefined && result.cpu.numaNodes.length > 1;
}
