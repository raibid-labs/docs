/**
 * Documentation Indexer
 * Builds and maintains search index for documentation
 * DGX-Spark MCP Server - Workstream 4
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { DocumentIndexEntry, IndexStats } from '../types/docs.js';
import { parseMarkdown } from './parser.js';
import { scanDirectory } from './scanner.js';
import { stripMarkdown } from './parser.js';

const INDEX_FILE = 'data/docs-index.json';
const INDEX_VERSION = '1.0.0';

/**
 * Document index
 */
export class DocumentIndex {
  private entries: Map<string, DocumentIndexEntry> = new Map();
  private stats: IndexStats = {
    totalDocuments: 0,
    totalSize: 0,
    categories: {},
    tags: {},
    lastBuilt: 0,
    buildDuration: 0,
  };

  /**
   * Build index from directory
   */
  async buildFromDirectory(dirPath: string): Promise<IndexStats> {
    const startTime = Date.now();
    this.entries.clear();

    // Scan for markdown files
    const scanResult = await scanDirectory(dirPath, true);

    if (scanResult.errors.length > 0) {
      console.warn(`Scan completed with ${scanResult.errors.length} errors`);
    }

    // Parse and index each file
    const indexPromises = scanResult.files.map(file => this.indexFile(file));
    await Promise.all(indexPromises);

    // Update stats
    this.stats.totalDocuments = this.entries.size;
    this.stats.totalSize = scanResult.totalSize;
    this.stats.lastBuilt = Date.now();
    this.stats.buildDuration = Date.now() - startTime;

    return this.stats;
  }

  /**
   * Index a single file
   */
  private async indexFile(filePath: string): Promise<void> {
    try {
      const parsed = await parseMarkdown(filePath);
      const stats = await fs.stat(filePath);

      const entry: DocumentIndexEntry = {
        id: parsed.id,
        title: parsed.metadata.title,
        description: parsed.metadata.description || '',
        category: parsed.metadata.category || 'general',
        tags: parsed.metadata.tags || [],
        filePath,
        content: stripMarkdown(parsed.content),
        lastModified: stats.mtimeMs,
        size: stats.size,
      };

      this.entries.set(entry.id, entry);

      // Update category stats
      this.stats.categories[entry.category] = (this.stats.categories[entry.category] || 0) + 1;

      // Update tag stats
      for (const tag of entry.tags) {
        this.stats.tags[tag] = (this.stats.tags[tag] || 0) + 1;
      }
    } catch (error) {
      console.error(`Failed to index file ${filePath}:`, error);
    }
  }

  /**
   * Get entry by ID
   */
  getEntry(id: string): DocumentIndexEntry | undefined {
    return this.entries.get(id);
  }

  /**
   * Get all entries
   */
  getAllEntries(): DocumentIndexEntry[] {
    return Array.from(this.entries.values());
  }

  /**
   * Get entries by category
   */
  getEntriesByCategory(category: string): DocumentIndexEntry[] {
    return this.getAllEntries().filter(entry => entry.category === category);
  }

  /**
   * Get entries by tag
   */
  getEntriesByTag(tag: string): DocumentIndexEntry[] {
    return this.getAllEntries().filter(entry => entry.tags.includes(tag));
  }

  /**
   * Get index statistics
   */
  getStats(): IndexStats {
    return { ...this.stats };
  }

  /**
   * Save index to disk
   */
  async save(filePath: string = INDEX_FILE): Promise<void> {
    const data = {
      version: INDEX_VERSION,
      stats: this.stats,
      entries: Array.from(this.entries.entries()),
    };

    const dir = path.dirname(filePath);
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(filePath, JSON.stringify(data, null, 2));
  }

  /**
   * Load index from disk
   */
  async load(filePath: string = INDEX_FILE): Promise<void> {
    try {
      const content = await fs.readFile(filePath, 'utf-8');
      const data = JSON.parse(content);

      if (data.version !== INDEX_VERSION) {
        console.warn(`Index version mismatch: ${data.version} vs ${INDEX_VERSION}`);
      }

      this.stats = data.stats;
      this.entries = new Map(data.entries);
    } catch (error) {
      console.error(`Failed to load index from ${filePath}:`, error);
      throw error;
    }
  }

  /**
   * Check if index exists
   */
  static async exists(filePath: string = INDEX_FILE): Promise<boolean> {
    try {
      await fs.access(filePath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Rebuild index if needed
   */
  async rebuildIfNeeded(dirPath: string, maxAge: number = 3600000): Promise<boolean> {
    const indexExists = await DocumentIndex.exists(INDEX_FILE);

    if (!indexExists) {
      await this.buildFromDirectory(dirPath);
      await this.save();
      return true;
    }

    await this.load();

    const age = Date.now() - this.stats.lastBuilt;
    if (age > maxAge) {
      await this.buildFromDirectory(dirPath);
      await this.save();
      return true;
    }

    return false;
  }
}

/**
 * Global index instance
 */
let globalIndex: DocumentIndex | null = null;

/**
 * Get or create global index
 */
export function getIndex(): DocumentIndex {
  if (!globalIndex) {
    globalIndex = new DocumentIndex();
  }
  return globalIndex;
}

/**
 * Build index for a directory
 */
export async function buildIndex(dirPath: string = 'docs'): Promise<IndexStats> {
  const index = getIndex();
  const stats = await index.buildFromDirectory(dirPath);
  await index.save();
  return stats;
}

/**
 * Load existing index
 */
export async function loadIndex(): Promise<DocumentIndex> {
  const index = getIndex();
  await index.load();
  return index;
}
