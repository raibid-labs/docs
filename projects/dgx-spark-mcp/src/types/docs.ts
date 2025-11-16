/**
 * Type definitions for the Documentation System
 * DGX-Spark MCP Server - Workstream 4
 */

/**
 * Document metadata extracted from frontmatter
 */
export interface DocumentMetadata {
  title: string;
  description?: string;
  tags?: string[];
  category?: string;
  author?: string;
  dateCreated?: string;
  dateModified?: string;
  version?: string;
  relatedDocs?: string[];
}

/**
 * Parsed document with content and metadata
 */
export interface ParsedDocument {
  id: string;
  filePath: string;
  metadata: DocumentMetadata;
  content: string;
  rawContent: string;
  headings: DocumentHeading[];
  codeBlocks: CodeBlock[];
  links: DocumentLink[];
}

/**
 * Document heading with hierarchy
 */
export interface DocumentHeading {
  level: number;
  text: string;
  id: string;
  children?: DocumentHeading[];
}

/**
 * Code block extracted from document
 */
export interface CodeBlock {
  language: string;
  code: string;
  lineStart: number;
  lineEnd: number;
}

/**
 * Document link (internal or external)
 */
export interface DocumentLink {
  text: string;
  url: string;
  isExternal: boolean;
}

/**
 * Search result with relevance scoring
 */
export interface SearchResult {
  id: string;
  title: string;
  description: string;
  category: string;
  score: number;
  matchedFields: string[];
  excerpt: string;
  filePath: string;
  url: string;
}

/**
 * Search query options
 */
export interface SearchOptions {
  query: string;
  limit?: number;
  offset?: number;
  categories?: string[];
  tags?: string[];
  minScore?: number;
}

/**
 * Document index entry
 */
export interface DocumentIndexEntry {
  id: string;
  title: string;
  description: string;
  category: string;
  tags: string[];
  filePath: string;
  content: string;
  lastModified: number;
  size: number;
}

/**
 * Document index statistics
 */
export interface IndexStats {
  totalDocuments: number;
  totalSize: number;
  categories: Record<string, number>;
  tags: Record<string, number>;
  lastBuilt: number;
  buildDuration: number;
}

/**
 * Document cache entry
 */
export interface CachedDocument {
  url: string;
  content: string;
  fetchedAt: number;
  ttl: number;
  etag?: string;
}

/**
 * External documentation source
 */
export interface ExternalDocSource {
  name: string;
  baseUrl: string;
  cacheTTL: number;
  enabled: boolean;
}

/**
 * Document fetcher options
 */
export interface FetchOptions {
  useCache?: boolean;
  forceRefresh?: boolean;
  timeout?: number;
  retries?: number;
}

/**
 * Documentation API response
 */
export interface DocsApiResponse {
  success: boolean;
  data?: any;
  error?: string;
  metadata?: {
    totalResults?: number;
    page?: number;
    pageSize?: number;
  };
}

/**
 * Document scanner result
 */
export interface ScanResult {
  files: string[];
  totalSize: number;
  scanDuration: number;
  errors: ScanError[];
}

/**
 * Scanner error
 */
export interface ScanError {
  filePath: string;
  error: string;
}
