/**
 * Documentation System Entry Point
 * DGX-Spark MCP Server - Workstream 4
 */

// Core modules
export * from './api.js';
export * from './cache.js';
export * from './converter.js';
export * from './fetcher.js';
export * from './frontmatter.js';
export * from './indexer.js';
export * from './loader.js';
export * from './parser.js';
export * from './scanner.js';
export * from './search.js';

// Re-export types
export type {
  DocumentMetadata,
  ParsedDocument,
  DocumentHeading,
  CodeBlock,
  DocumentLink,
  SearchResult,
  SearchOptions,
  DocumentIndexEntry,
  IndexStats,
  CachedDocument,
  ExternalDocSource,
  FetchOptions,
  DocsApiResponse,
  ScanResult,
  ScanError,
} from '../types/docs.js';
