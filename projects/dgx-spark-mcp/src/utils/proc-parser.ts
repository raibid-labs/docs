/**
 * Utilities for parsing /proc filesystem files
 */

import * as fs from 'fs/promises';

/**
 * Parse /proc/cpuinfo into structured data
 */
export async function parseCpuInfo(): Promise<Map<string, string>[]> {
  const content = await fs.readFile('/proc/cpuinfo', 'utf-8');
  const processors: Map<string, string>[] = [];
  let current = new Map<string, string>();

  for (const line of content.split('\n')) {
    const trimmed = line.trim();

    if (trimmed === '') {
      if (current.size > 0) {
        processors.push(current);
        current = new Map<string, string>();
      }
      continue;
    }

    const colonIndex = trimmed.indexOf(':');
    if (colonIndex > 0) {
      const key = trimmed.substring(0, colonIndex).trim();
      const value = trimmed.substring(colonIndex + 1).trim();
      current.set(key, value);
    }
  }

  if (current.size > 0) {
    processors.push(current);
  }

  return processors;
}

/**
 * Parse /proc/meminfo into key-value map
 */
export async function parseMemInfo(): Promise<Map<string, number>> {
  const content = await fs.readFile('/proc/meminfo', 'utf-8');
  const memInfo = new Map<string, number>();

  for (const line of content.split('\n')) {
    const trimmed = line.trim();
    if (trimmed === '') continue;

    const colonIndex = trimmed.indexOf(':');
    if (colonIndex > 0) {
      const key = trimmed.substring(0, colonIndex).trim();
      const valuePart = trimmed.substring(colonIndex + 1).trim();

      // Extract numeric value (remove 'kB' or other units)
      const match = valuePart.match(/^(\d+)/);
      if (match && match[1]) {
        const value = parseInt(match[1], 10);
        // Convert kB to bytes (most values in meminfo are in kB)
        memInfo.set(key, valuePart.includes('kB') ? value * 1024 : value);
      }
    }
  }

  return memInfo;
}

/**
 * Parse key-value file format (common in /proc and /sys)
 */
export async function parseKeyValueFile(path: string, separator: string = ':'): Promise<Map<string, string>> {
  try {
    const content = await fs.readFile(path, 'utf-8');
    const data = new Map<string, string>();

    for (const line of content.split('\n')) {
      const trimmed = line.trim();
      if (trimmed === '') continue;

      const sepIndex = trimmed.indexOf(separator);
      if (sepIndex > 0) {
        const key = trimmed.substring(0, sepIndex).trim();
        const value = trimmed.substring(sepIndex + 1).trim();
        data.set(key, value);
      }
    }

    return data;
  } catch (error) {
    return new Map();
  }
}

/**
 * Read a single-line /sys or /proc file
 */
export async function readSysFile(path: string): Promise<string | null> {
  try {
    const content = await fs.readFile(path, 'utf-8');
    return content.trim();
  } catch (error) {
    return null;
  }
}

/**
 * Parse numeric value from file
 */
export async function readNumericSysFile(path: string): Promise<number | null> {
  const value = await readSysFile(path);
  if (value === null) return null;

  const num = parseFloat(value);
  return isNaN(num) ? null : num;
}
