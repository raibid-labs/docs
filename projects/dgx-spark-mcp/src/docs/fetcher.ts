/**
 * External Documentation Fetcher
 * Fetches documentation from external sources with caching
 * DGX-Spark MCP Server - Workstream 4
 */

import { FetchOptions, ExternalDocSource } from '../types/docs.js';
import { getCache } from './cache.js';
import { htmlToMarkdown } from './converter.js';

/**
 * External documentation sources
 */
const EXTERNAL_SOURCES: Record<string, ExternalDocSource> = {
  'nvidia-spark': {
    name: 'NVIDIA Spark Documentation',
    baseUrl: 'https://docs.nvidia.com/spark-rapids',
    cacheTTL: 86400000, // 24 hours
    enabled: true,
  },
  'nvidia-dgx': {
    name: 'NVIDIA DGX Documentation',
    baseUrl: 'https://docs.nvidia.com/dgx',
    cacheTTL: 86400000, // 24 hours
    enabled: true,
  },
  'apache-spark': {
    name: 'Apache Spark Documentation',
    baseUrl: 'https://spark.apache.org/docs/latest',
    cacheTTL: 604800000, // 7 days
    enabled: true,
  },
};

/**
 * Fetch external documentation
 */
export async function fetchExternalDoc(
  source: string,
  path: string,
  options: FetchOptions = {}
): Promise<{ content: string; fromCache: boolean }> {
  const docSource = EXTERNAL_SOURCES[source];
  if (!docSource) {
    throw new Error(`Unknown documentation source: ${source}`);
  }

  if (!docSource.enabled) {
    throw new Error(`Documentation source is disabled: ${source}`);
  }

  const url = `${docSource.baseUrl}/${path}`;
  const cache = getCache();

  // Check cache unless force refresh
  if (options.useCache !== false && !options.forceRefresh) {
    const cached = await cache.get(url);
    if (cached) {
      return { content: cached.content, fromCache: true };
    }
  }

  // Fetch from external source
  const content = await fetchWithRetry(url, options);

  // Convert HTML to Markdown if needed
  const markdown = isHtmlContent(content) ? htmlToMarkdown(content) : content;

  // Cache the result
  await cache.set(url, markdown, docSource.cacheTTL);

  return { content: markdown, fromCache: false };
}

/**
 * Fetch NVIDIA Spark documentation
 */
export async function fetchNvidiaDoc(topic: string, options: FetchOptions = {}): Promise<{ content: string; fromCache: boolean }> {
  return fetchExternalDoc('nvidia-spark', topic, options);
}

/**
 * Fetch with retry logic
 */
async function fetchWithRetry(
  url: string,
  options: FetchOptions = {}
): Promise<string> {
  const { timeout = 30000, retries = 3 } = options;
  let lastError: Error | null = null;

  for (let attempt = 0; attempt < retries; attempt++) {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);

      const response = await fetch(url, {
        signal: controller.signal,
        headers: {
          'User-Agent': 'DGX-Spark-MCP-Server/1.0',
        },
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const content = await response.text();
      return content;
    } catch (error) {
      lastError = error as Error;

      if (attempt < retries - 1) {
        // Wait before retrying (exponential backoff)
        const delay = Math.min(1000 * Math.pow(2, attempt), 10000);
        await sleep(delay);
      }
    }
  }

  throw new Error(`Failed to fetch ${url} after ${retries} attempts: ${lastError?.message}`);
}

/**
 * Check if content is HTML
 */
function isHtmlContent(content: string): boolean {
  const htmlPattern = /<\s*html[^>]*>/i;
  const doctypePattern = /<!DOCTYPE\s+html/i;
  const bodyPattern = /<\s*body[^>]*>/i;

  return htmlPattern.test(content) || doctypePattern.test(content) || bodyPattern.test(content);
}

/**
 * Sleep utility
 */
function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Get list of available external sources
 */
export function getExternalSources(): ExternalDocSource[] {
  return Object.values(EXTERNAL_SOURCES).filter(source => source.enabled);
}

/**
 * Check if external documentation is available
 */
export async function checkExternalAvailability(source: string): Promise<boolean> {
  const docSource = EXTERNAL_SOURCES[source];
  if (!docSource) {
    return false;
  }

  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000);

    const response = await fetch(docSource.baseUrl, {
      method: 'HEAD',
      signal: controller.signal,
    });

    clearTimeout(timeoutId);
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Prefetch common documentation
 */
export async function prefetchCommonDocs(): Promise<void> {
  const commonDocs = [
    { source: 'nvidia-spark', path: 'getting-started' },
    { source: 'nvidia-spark', path: 'tuning-guide' },
    { source: 'nvidia-dgx', path: 'overview' },
  ];

  const promises = commonDocs.map(doc =>
    fetchExternalDoc(doc.source, doc.path, { useCache: true }).catch(err => {
      console.warn(`Failed to prefetch ${doc.source}/${doc.path}:`, err.message);
    })
  );

  await Promise.all(promises);
}
