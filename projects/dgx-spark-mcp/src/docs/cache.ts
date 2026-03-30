/**
 * Documentation Cache
 * Caching layer for external documentation with TTL
 * DGX-Spark MCP Server - Workstream 4
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { CachedDocument } from '../types/docs.js';

const CACHE_DIR = 'data/cache/docs';

/**
 * Document cache with TTL support
 */
export class DocumentCache {
  private memoryCache: Map<string, CachedDocument> = new Map();

  /**
   * Get cached document
   */
  async get(url: string): Promise<CachedDocument | null> {
    // Check memory cache first
    const memCached = this.memoryCache.get(url);
    if (memCached && !this.isExpired(memCached)) {
      return memCached;
    }

    // Check disk cache
    try {
      const cacheFile = this.getCacheFilePath(url);
      const content = await fs.readFile(cacheFile, 'utf-8');
      const cached: CachedDocument = JSON.parse(content);

      if (!this.isExpired(cached)) {
        // Update memory cache
        this.memoryCache.set(url, cached);
        return cached;
      }

      // Expired, remove from disk
      await this.remove(url);
    } catch (error) {
      // Cache miss
    }

    return null;
  }

  /**
   * Set cached document
   */
  async set(url: string, content: string, ttl: number, etag?: string): Promise<void> {
    const cached: CachedDocument = {
      url,
      content,
      fetchedAt: Date.now(),
      ttl,
      etag,
    };

    // Store in memory
    this.memoryCache.set(url, cached);

    // Store on disk
    try {
      const cacheFile = this.getCacheFilePath(url);
      const cacheDir = path.dirname(cacheFile);
      await fs.mkdir(cacheDir, { recursive: true });
      await fs.writeFile(cacheFile, JSON.stringify(cached, null, 2));
    } catch (error) {
      console.error(`Failed to write cache for ${url}:`, error);
    }
  }

  /**
   * Remove cached document
   */
  async remove(url: string): Promise<void> {
    this.memoryCache.delete(url);

    try {
      const cacheFile = this.getCacheFilePath(url);
      await fs.unlink(cacheFile);
    } catch (error) {
      // Ignore errors if file doesn't exist
    }
  }

  /**
   * Clear all cache
   */
  async clear(): Promise<void> {
    this.memoryCache.clear();

    try {
      await fs.rm(CACHE_DIR, { recursive: true, force: true });
    } catch (error) {
      console.error('Failed to clear cache directory:', error);
    }
  }

  /**
   * Get cache statistics
   */
  async getStats(): Promise<{
    memoryEntries: number;
    diskEntries: number;
    totalSize: number;
  }> {
    const memoryEntries = this.memoryCache.size;
    let diskEntries = 0;
    let totalSize = 0;

    try {
      await fs.mkdir(CACHE_DIR, { recursive: true });
      const files = await this.getAllCacheFiles(CACHE_DIR);
      diskEntries = files.length;

      for (const file of files) {
        const stats = await fs.stat(file);
        totalSize += stats.size;
      }
    } catch (error) {
      console.error('Failed to get cache stats:', error);
    }

    return {
      memoryEntries,
      diskEntries,
      totalSize,
    };
  }

  /**
   * Get all cache files recursively
   */
  private async getAllCacheFiles(dir: string): Promise<string[]> {
    const files: string[] = [];

    try {
      const entries = await fs.readdir(dir, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
          const subFiles = await this.getAllCacheFiles(fullPath);
          files.push(...subFiles);
        } else if (entry.isFile() && entry.name.endsWith('.json')) {
          files.push(fullPath);
        }
      }
    } catch (error) {
      // Directory doesn't exist or not readable
    }

    return files;
  }

  /**
   * Check if cached document is expired
   */
  private isExpired(cached: CachedDocument): boolean {
    const age = Date.now() - cached.fetchedAt;
    return age > cached.ttl;
  }

  /**
   * Get cache file path for URL
   */
  private getCacheFilePath(url: string): string {
    // Create safe filename from URL
    const hash = this.hashString(url);
    const filename = `${hash}.json`;
    return path.join(CACHE_DIR, filename);
  }

  /**
   * Simple string hash function
   */
  private hashString(str: string): string {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(36);
  }

  /**
   * Prune expired entries
   */
  async prune(): Promise<number> {
    let prunedCount = 0;

    // Prune memory cache
    for (const [url, cached] of this.memoryCache.entries()) {
      if (this.isExpired(cached)) {
        this.memoryCache.delete(url);
        prunedCount++;
      }
    }

    // Prune disk cache
    try {
      const files = await this.getAllCacheFiles(CACHE_DIR);

      for (const file of files) {
        try {
          const content = await fs.readFile(file, 'utf-8');
          const cached: CachedDocument = JSON.parse(content);

          if (this.isExpired(cached)) {
            await fs.unlink(file);
            prunedCount++;
          }
        } catch (error) {
          // Invalid cache file, remove it
          await fs.unlink(file);
          prunedCount++;
        }
      }
    } catch (error) {
      console.error('Failed to prune cache:', error);
    }

    return prunedCount;
  }
}

/**
 * Global cache instance
 */
let globalCache: DocumentCache | null = null;

/**
 * Get or create global cache
 */
export function getCache(): DocumentCache {
  if (!globalCache) {
    globalCache = new DocumentCache();
  }
  return globalCache;
}
