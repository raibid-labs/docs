/**
 * Hardware detection caching module
 */

import { CacheEntry } from '../types/topology.js';

/**
 * Simple in-memory cache for hardware detection results
 */
export class HardwareCache {
  private cache: Map<string, CacheEntry<any>> = new Map();
  private defaultTTL: number = 60000; // 60 seconds

  /**
   * Set cache TTL in milliseconds
   */
  setDefaultTTL(ttl: number): void {
    this.defaultTTL = ttl;
  }

  /**
   * Get cached value if not expired
   */
  get<T>(key: string): T | null {
    const entry = this.cache.get(key);

    if (!entry) {
      return null;
    }

    const now = Date.now();
    const age = now - entry.timestamp;

    if (age > entry.ttl) {
      // Cache expired
      this.cache.delete(key);
      return null;
    }

    return entry.data as T;
  }

  /**
   * Set cache value with optional custom TTL
   */
  set<T>(key: string, data: T, ttl?: number): void {
    const entry: CacheEntry<T> = {
      data,
      timestamp: Date.now(),
      ttl: ttl !== undefined ? ttl : this.defaultTTL,
    };

    this.cache.set(key, entry);
  }

  /**
   * Check if key exists and is not expired
   */
  has(key: string): boolean {
    return this.get(key) !== null;
  }

  /**
   * Invalidate specific cache key
   */
  invalidate(key: string): void {
    this.cache.delete(key);
  }

  /**
   * Clear all cache
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get cache size
   */
  size(): number {
    return this.cache.size;
  }

  /**
   * Get cache statistics
   */
  getStats(): { size: number; keys: string[]; oldestAge: number; newestAge: number } {
    const keys = Array.from(this.cache.keys());
    const now = Date.now();

    let oldestAge = 0;
    let newestAge = Number.MAX_SAFE_INTEGER;

    for (const entry of this.cache.values()) {
      const age = now - entry.timestamp;
      if (age > oldestAge) oldestAge = age;
      if (age < newestAge) newestAge = age;
    }

    return {
      size: this.cache.size,
      keys,
      oldestAge,
      newestAge: newestAge === Number.MAX_SAFE_INTEGER ? 0 : newestAge,
    };
  }

  /**
   * Prune expired entries
   */
  prune(): number {
    const now = Date.now();
    let prunedCount = 0;

    for (const [key, entry] of this.cache.entries()) {
      const age = now - entry.timestamp;
      if (age > entry.ttl) {
        this.cache.delete(key);
        prunedCount++;
      }
    }

    return prunedCount;
  }
}

/**
 * Global cache instance
 */
export const hardwareCache = new HardwareCache();

/**
 * Cache key constants
 */
export const CacheKeys = {
  GPU_DETECTION: 'gpu_detection',
  GPU_TOPOLOGY: 'gpu_topology',
  CPU_DETECTION: 'cpu_detection',
  MEMORY_DETECTION: 'memory_detection',
  STORAGE_DETECTION: 'storage_detection',
  NETWORK_DETECTION: 'network_detection',
  SYSTEM_TOPOLOGY: 'system_topology',
} as const;
