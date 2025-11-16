/**
 * Documentation API
 * Public API for accessing documentation system
 * DGX-Spark MCP Server - Workstream 4
 */

import { DocsApiResponse, SearchOptions } from '../types/docs.js';
import { getIndex, buildIndex } from './indexer.js';
import { search as searchDocs, initializeSearch as initSearch } from './search.js';
import { parseMarkdown } from './parser.js';
import { fetchExternalDoc, getExternalSources } from './fetcher.js';
import { getCache } from './cache.js';

// Re-export for external use
export { buildIndex, initSearch as initializeSearch, searchDocs as search };

/**
 * Initialize documentation system
 */
export async function initialize(docsDir: string = 'docs'): Promise<DocsApiResponse> {
  try {
    // Build or load index
    const index = getIndex();
    await index.rebuildIfNeeded(docsDir, 3600000); // 1 hour

    // Initialize search engine
    await initSearch();

    const stats = index.getStats();

    return {
      success: true,
      data: {
        status: 'initialized',
        stats,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to initialize documentation system: ${error}`,
    };
  }
}

/**
 * Search documentation
 */
export async function searchDocumentation(
  query: string,
  options: Partial<SearchOptions> = {}
): Promise<DocsApiResponse> {
  try {
    const results = await searchDocs(query, options);

    return {
      success: true,
      data: results,
      metadata: {
        totalResults: results.length,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Search failed: ${error}`,
    };
  }
}

/**
 * Get document by ID
 */
export async function getDocument(id: string): Promise<DocsApiResponse> {
  try {
    const index = getIndex();
    const entry = index.getEntry(id);

    if (!entry) {
      return {
        success: false,
        error: `Document not found: ${id}`,
      };
    }

    // Parse full document
    const parsed = await parseMarkdown(entry.filePath);

    return {
      success: true,
      data: {
        ...parsed,
        url: `dgx://docs/${id}`,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to get document: ${error}`,
    };
  }
}

/**
 * Get document content by ID
 */
export async function getDocumentContent(id: string): Promise<DocsApiResponse> {
  try {
    const index = getIndex();
    const entry = index.getEntry(id);

    if (!entry) {
      return {
        success: false,
        error: `Document not found: ${id}`,
      };
    }

    const parsed = await parseMarkdown(entry.filePath);

    return {
      success: true,
      data: {
        id: parsed.id,
        title: parsed.metadata.title,
        content: parsed.content,
        metadata: parsed.metadata,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to get document content: ${error}`,
    };
  }
}

/**
 * List all documents
 */
export async function listAllDocs(): Promise<DocsApiResponse> {
  try {
    const index = getIndex();
    const entries = index.getAllEntries();

    const docs = entries.map(entry => ({
      id: entry.id,
      title: entry.title,
      description: entry.description,
      category: entry.category,
      tags: entry.tags,
      url: `dgx://docs/${entry.id}`,
    }));

    return {
      success: true,
      data: docs,
      metadata: {
        totalResults: docs.length,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to list documents: ${error}`,
    };
  }
}

/**
 * List documents by category
 */
export async function listDocsByCategory(category: string): Promise<DocsApiResponse> {
  try {
    const index = getIndex();
    const entries = index.getEntriesByCategory(category);

    const docs = entries.map(entry => ({
      id: entry.id,
      title: entry.title,
      description: entry.description,
      category: entry.category,
      url: `dgx://docs/${entry.id}`,
    }));

    return {
      success: true,
      data: docs,
      metadata: {
        totalResults: docs.length,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to list documents by category: ${error}`,
    };
  }
}

/**
 * List documents by tag
 */
export async function listDocsByTag(tag: string): Promise<DocsApiResponse> {
  try {
    const index = getIndex();
    const entries = index.getEntriesByTag(tag);

    const docs = entries.map(entry => ({
      id: entry.id,
      title: entry.title,
      description: entry.description,
      tags: entry.tags,
      url: `dgx://docs/${entry.id}`,
    }));

    return {
      success: true,
      data: docs,
      metadata: {
        totalResults: docs.length,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to list documents by tag: ${error}`,
    };
  }
}

/**
 * Get documentation statistics
 */
export async function getStats(): Promise<DocsApiResponse> {
  try {
    const index = getIndex();
    const indexStats = index.getStats();
    const cacheStats = await getCache().getStats();

    return {
      success: true,
      data: {
        index: indexStats,
        cache: cacheStats,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to get stats: ${error}`,
    };
  }
}

/**
 * Rebuild documentation index
 */
export async function rebuildIndex(docsDir: string = 'docs'): Promise<DocsApiResponse> {
  try {
    const stats = await buildIndex(docsDir);
    await initSearch();

    return {
      success: true,
      data: {
        status: 'rebuilt',
        stats,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to rebuild index: ${error}`,
    };
  }
}

/**
 * Get external documentation
 */
export async function getExternalDoc(
  source: string,
  path: string,
  useCache: boolean = true
): Promise<DocsApiResponse> {
  try {
    const result = await fetchExternalDoc(source, path, { useCache });

    return {
      success: true,
      data: {
        source,
        path,
        content: result.content,
        fromCache: result.fromCache,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to fetch external doc: ${error}`,
    };
  }
}

/**
 * List external documentation sources
 */
export async function listExternalSources(): Promise<DocsApiResponse> {
  try {
    const sources = getExternalSources();

    return {
      success: true,
      data: sources,
      metadata: {
        totalResults: sources.length,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to list external sources: ${error}`,
    };
  }
}

/**
 * Clear documentation cache
 */
export async function clearCache(): Promise<DocsApiResponse> {
  try {
    await getCache().clear();

    return {
      success: true,
      data: {
        status: 'cache cleared',
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to clear cache: ${error}`,
    };
  }
}

/**
 * Prune expired cache entries
 */
export async function pruneCache(): Promise<DocsApiResponse> {
  try {
    const prunedCount = await getCache().prune();

    return {
      success: true,
      data: {
        status: 'cache pruned',
        prunedCount,
      },
    };
  } catch (error) {
    return {
      success: false,
      error: `Failed to prune cache: ${error}`,
    };
  }
}
