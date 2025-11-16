/**
 * Memory detection module
 */

import { Memory, MemoryDetectionResult, MemoryInfo, MemoryModule } from '../types/memory.js';
import { parseMemInfo } from '../utils/proc-parser.js';
import { executeCommand, parseKeyValue } from '../utils/exec.js';

/**
 * Detect system memory information
 */
export async function detectMemory(includeModules: boolean = false): Promise<MemoryDetectionResult> {
  const startTime = Date.now();

  // Parse /proc/meminfo
  const memInfo = await parseMemInfo();

  const total = memInfo.get('MemTotal') || 0;
  const available = memInfo.get('MemAvailable') || 0;
  const free = memInfo.get('MemFree') || 0;
  const buffers = memInfo.get('Buffers') || 0;
  const cached = memInfo.get('Cached') || 0;
  const shared = memInfo.get('Shmem') || 0;
  const swapTotal = memInfo.get('SwapTotal') || 0;
  const swapFree = memInfo.get('SwapFree') || 0;

  const used = total - free - buffers - cached;
  const swapUsed = swapTotal - swapFree;

  const info: MemoryInfo = {
    total,
    available,
    used,
    free,
    shared,
    buffers,
    cached,
    swapTotal,
    swapFree,
    swapUsed,
  };

  // Detect memory modules if requested (requires root)
  let modules: MemoryModule[] | undefined;
  if (includeModules) {
    modules = await detectMemoryModules();
  }

  // Detect hugepages
  const hugepages = await detectHugepages(memInfo);

  const memory: Memory = {
    info,
    modules,
    hugepages,
  };

  const detectionTime = Date.now() - startTime;

  return {
    memory,
    timestamp: Date.now(),
    detectionTime,
  };
}

/**
 * Detect memory modules using dmidecode (requires root)
 */
async function detectMemoryModules(): Promise<MemoryModule[] | undefined> {
  const result = await executeCommand('sudo dmidecode --type memory 2>/dev/null');

  if (result.exitCode !== 0) {
    // Not running as root or dmidecode not available
    return undefined;
  }

  const modules: MemoryModule[] = [];
  const sections = result.stdout.split('\n\n');

  for (const section of sections) {
    if (!section.includes('Memory Device')) continue;

    const lines = parseKeyValue(section);

    const sizeStr = lines.get('Size') || '';
    const typeStr = lines.get('Type') || '';
    const speedStr = lines.get('Speed') || '';
    const locator = lines.get('Locator') || '';

    // Skip empty slots
    if (sizeStr.toLowerCase().includes('no module') || sizeStr === '') continue;

    // Parse size
    const sizeMatch = sizeStr.match(/(\d+)\s*(MB|GB)/i);
    let size = 0;
    if (sizeMatch && sizeMatch[1]) {
      const value = parseInt(sizeMatch[1], 10);
      const unit = sizeMatch[2] ? sizeMatch[2].toUpperCase() : 'MB';
      size = unit === 'GB' ? value * 1024 * 1024 * 1024 : value * 1024 * 1024;
    }

    // Parse speed
    const speedMatch = speedStr.match(/(\d+)\s*MHz/i);
    const speed = speedMatch && speedMatch[1] ? parseInt(speedMatch[1], 10) : 0;

    modules.push({
      locator,
      size,
      type: typeStr,
      speed,
      manufacturer: lines.get('Manufacturer'),
      partNumber: lines.get('Part Number'),
      serialNumber: lines.get('Serial Number'),
    });
  }

  return modules.length > 0 ? modules : undefined;
}

/**
 * Detect hugepages configuration
 */
async function detectHugepages(memInfo: Map<string, number>): Promise<{ total: number; free: number; size: number } | undefined> {
  const hugepagesTotal = memInfo.get('HugePages_Total') || 0;
  const hugepagesFree = memInfo.get('HugePages_Free') || 0;
  const hugepageSize = memInfo.get('Hugepagesize') || 0;

  if (hugepagesTotal === 0) {
    return undefined;
  }

  return {
    total: hugepagesTotal,
    free: hugepagesFree,
    size: hugepageSize,
  };
}

/**
 * Get total system memory
 */
export async function getTotalMemory(): Promise<number> {
  const result = await detectMemory(false);
  return result.memory.info.total;
}

/**
 * Get available system memory
 */
export async function getAvailableMemory(): Promise<number> {
  const result = await detectMemory(false);
  return result.memory.info.available;
}

/**
 * Get memory utilization percentage
 */
export async function getMemoryUtilization(): Promise<number> {
  const result = await detectMemory(false);
  const { total, available } = result.memory.info;
  if (total === 0) return 0;
  return ((total - available) / total) * 100;
}

/**
 * Check if system has swap configured
 */
export async function hasSwap(): Promise<boolean> {
  const result = await detectMemory(false);
  return result.memory.info.swapTotal > 0;
}

/**
 * Check if system has hugepages configured
 */
export async function hasHugepages(): Promise<boolean> {
  const result = await detectMemory(false);
  return result.memory.hugepages !== undefined && result.memory.hugepages.total > 0;
}
