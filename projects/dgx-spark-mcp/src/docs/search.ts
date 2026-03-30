/**
 * Documentation Search Engine
 * Full-text search with ranking using lunr.js
 * DGX-Spark MCP Server - Workstream 4
 */

import { SearchResult, SearchOptions, DocumentIndexEntry } from '../types/docs.js';
import { getIndex } from './indexer.js';
import { extractExcerpt } from './parser.js';

/**
 * Simple search implementation without external dependencies
 * Uses TF-IDF-like scoring for relevance ranking
 */
export class SearchEngine {
  private index: Map<string, DocumentIndexEntry> = new Map();
  private termIndex: Map<string, Set<string>> = new Map();

  /**
   * Initialize search engine with documents
   */
  initialize(documents: DocumentIndexEntry[]): void {
    this.index.clear();
    this.termIndex.clear();

    for (const doc of documents) {
      this.index.set(doc.id, doc);
      this.indexDocument(doc);
    }
  }

  /**
   * Index a document for search
   */
  private indexDocument(doc: DocumentIndexEntry): void {
    const text = [
      doc.title,
      doc.description,
      doc.content,
      doc.category,
      ...doc.tags,
    ].join(' ');

    const terms = this.tokenize(text);

    for (const term of terms) {
      if (!this.termIndex.has(term)) {
        this.termIndex.set(term, new Set());
      }
      this.termIndex.get(term)!.add(doc.id);
    }
  }

  /**
   * Tokenize text into search terms
   */
  private tokenize(text: string): string[] {
    return text
      .toLowerCase()
      .replace(/[^\w\s]/g, ' ')
      .split(/\s+/)
      .filter(term => term.length > 2);
  }

  /**
   * Search for documents
   */
  search(options: SearchOptions): SearchResult[] {
    const { query, limit = 10, offset = 0, categories, tags, minScore = 0 } = options;

    const queryTerms = this.tokenize(query);
    if (queryTerms.length === 0) {
      return [];
    }

    // Find documents matching query terms
    const docScores = new Map<string, number>();
    const matchedFields = new Map<string, Set<string>>();

    for (const term of queryTerms) {
      const matchingDocs = this.termIndex.get(term);
      if (!matchingDocs) continue;

      for (const docId of matchingDocs) {
        const doc = this.index.get(docId);
        if (!doc) continue;

        // Calculate term frequency
        const tf = this.calculateTermFrequency(term, doc);

        // Calculate inverse document frequency
        const idf = Math.log(this.index.size / matchingDocs.size);

        // TF-IDF score
        const score = tf * idf;

        docScores.set(docId, (docScores.get(docId) || 0) + score);

        // Track which fields matched
        if (!matchedFields.has(docId)) {
          matchedFields.set(docId, new Set());
        }
        const fields = this.getMatchingFields(term, doc);
        fields.forEach(field => matchedFields.get(docId)!.add(field));
      }
    }

    // Filter by category and tags
    let results = Array.from(docScores.entries())
      .map(([id, score]) => {
        const doc = this.index.get(id)!;
        return { id, score, doc, matchedFields: Array.from(matchedFields.get(id) || []) };
      })
      .filter(result => {
        if (categories && categories.length > 0 && !categories.includes(result.doc.category)) {
          return false;
        }
        if (tags && tags.length > 0 && !tags.some(tag => result.doc.tags.includes(tag))) {
          return false;
        }
        return result.score >= minScore;
      });

    // Sort by relevance score
    results.sort((a, b) => b.score - a.score);

    // Apply pagination
    results = results.slice(offset, offset + limit);

    // Format results
    return results.map(result => ({
      id: result.doc.id,
      title: result.doc.title,
      description: result.doc.description,
      category: result.doc.category,
      score: result.score,
      matchedFields: result.matchedFields,
      excerpt: this.generateExcerpt(result.doc, queryTerms),
      filePath: result.doc.filePath,
      url: `dgx://docs/${result.doc.id}`,
    }));
  }

  /**
   * Calculate term frequency in document
   */
  private calculateTermFrequency(term: string, doc: DocumentIndexEntry): number {
    const text = [
      doc.title,
      doc.description,
      doc.content,
    ].join(' ').toLowerCase();

    const terms = this.tokenize(text);
    const count = terms.filter(t => t === term).length;

    return count / terms.length;
  }

  /**
   * Get fields that match the search term
   */
  private getMatchingFields(term: string, doc: DocumentIndexEntry): string[] {
    const fields: string[] = [];

    if (this.tokenize(doc.title).includes(term)) {
      fields.push('title');
    }
    if (this.tokenize(doc.description).includes(term)) {
      fields.push('description');
    }
    if (this.tokenize(doc.content).includes(term)) {
      fields.push('content');
    }
    if (this.tokenize(doc.category).includes(term)) {
      fields.push('category');
    }
    if (doc.tags.some(tag => this.tokenize(tag).includes(term))) {
      fields.push('tags');
    }

    return fields;
  }

  /**
   * Generate excerpt highlighting query terms
   */
  private generateExcerpt(doc: DocumentIndexEntry, queryTerms: string[]): string {
    const content = doc.content;
    const lowerContent = content.toLowerCase();

    // Find first occurrence of any query term
    let firstMatch = -1;
    for (const term of queryTerms) {
      const index = lowerContent.indexOf(term);
      if (index !== -1 && (firstMatch === -1 || index < firstMatch)) {
        firstMatch = index;
      }
    }

    if (firstMatch === -1) {
      return extractExcerpt(content, 200);
    }

    // Extract context around match
    const start = Math.max(0, firstMatch - 100);
    const end = Math.min(content.length, firstMatch + 200);
    let excerpt = content.substring(start, end);

    if (start > 0) excerpt = '...' + excerpt;
    if (end < content.length) excerpt = excerpt + '...';

    return excerpt;
  }

  /**
   * Get search suggestions
   */
  getSuggestions(prefix: string, limit: number = 5): string[] {
    const lowerPrefix = prefix.toLowerCase();
    const suggestions = new Set<string>();

    for (const term of this.termIndex.keys()) {
      if (term.startsWith(lowerPrefix)) {
        suggestions.add(term);
        if (suggestions.size >= limit) break;
      }
    }

    return Array.from(suggestions);
  }
}

/**
 * Global search engine instance
 */
let globalSearchEngine: SearchEngine | null = null;

/**
 * Get or create global search engine
 */
export function getSearchEngine(): SearchEngine {
  if (!globalSearchEngine) {
    globalSearchEngine = new SearchEngine();
  }
  return globalSearchEngine;
}

/**
 * Initialize search engine from index
 */
export async function initializeSearch(): Promise<void> {
  const index = getIndex();
  const documents = index.getAllEntries();
  const searchEngine = getSearchEngine();
  searchEngine.initialize(documents);
}

/**
 * Search documentation
 */
export async function search(query: string, options: Partial<SearchOptions> = {}): Promise<SearchResult[]> {
  const searchEngine = getSearchEngine();
  return searchEngine.search({ query, ...options });
}
