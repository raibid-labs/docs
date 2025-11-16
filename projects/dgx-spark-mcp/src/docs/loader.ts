/**
 * Documentation Loader
 * MCP Resource integration for documentation system
 * DGX-Spark MCP Server - Workstream 4
 */

import {
  initialize,
  getDocumentContent,
  listAllDocs,
  searchDocumentation,
} from './api.js';

/**
 * MCP Resource URI patterns:
 * - dgx://docs/list - List all documents
 * - dgx://docs/search?q=query - Search documentation
 * - dgx://docs/{id} - Get specific document
 * - dgx://docs/{category}/{id} - Get document by category
 */

/**
 * Load documentation resource by URI
 */
export async function loadDocumentationResource(uri: string): Promise<{
  content: string;
  mimeType: string;
}> {
  const url = new URL(uri);

  // Handle list request
  if (url.pathname === '/list' || url.pathname === '/docs/list') {
    const response = await listAllDocs();
    return {
      content: JSON.stringify(response, null, 2),
      mimeType: 'application/json',
    };
  }

  // Handle search request
  if (url.pathname === '/search' || url.pathname === '/docs/search') {
    const query = url.searchParams.get('q') || '';
    const limit = parseInt(url.searchParams.get('limit') || '10');
    const category = url.searchParams.get('category');

    const options: any = { limit };
    if (category) {
      options.categories = [category];
    }

    const response = await searchDocumentation(query, options);
    return {
      content: JSON.stringify(response, null, 2),
      mimeType: 'application/json',
    };
  }

  // Handle document request
  const pathParts = url.pathname.split('/').filter(p => p);
  let docId: string;

  if (pathParts.length === 1 && pathParts[0]) {
    // dgx://docs/{id}
    docId = pathParts[0];
  } else if (pathParts.length === 2 && pathParts[0] === 'docs' && pathParts[1]) {
    // dgx://docs/{id}
    docId = pathParts[1];
  } else if (pathParts.length >= 2) {
    // dgx://docs/{category}/{id} or longer paths
    docId = pathParts.slice(1).join('/');
  } else {
    throw new Error(`Invalid documentation URI: ${uri}`);
  }

  const response = await getDocumentContent(docId);

  if (!response.success) {
    throw new Error(response.error || 'Failed to load document');
  }

  // Return as markdown
  return {
    content: formatDocumentAsMarkdown(response.data),
    mimeType: 'text/markdown',
  };
}

/**
 * Format document data as markdown
 */
function formatDocumentAsMarkdown(doc: any): string {
  const lines: string[] = [];

  // Add frontmatter
  lines.push('---');
  lines.push(`title: ${doc.title}`);
  if (doc.metadata?.description) {
    lines.push(`description: ${doc.metadata.description}`);
  }
  if (doc.metadata?.category) {
    lines.push(`category: ${doc.metadata.category}`);
  }
  if (doc.metadata?.tags?.length > 0) {
    lines.push(`tags: [${doc.metadata.tags.join(', ')}]`);
  }
  lines.push('---');
  lines.push('');

  // Add content
  lines.push(doc.content);

  return lines.join('\n');
}

/**
 * Initialize documentation loader
 */
export async function initializeLoader(docsDir: string = 'docs'): Promise<void> {
  await initialize(docsDir);
}

/**
 * Get documentation metadata for MCP resources/list
 */
export async function getDocumentationResourceList(): Promise<Array<{
  uri: string;
  name: string;
  description: string;
  mimeType: string;
}>> {
  const response = await listAllDocs();

  if (!response.success) {
    return [];
  }

  const resources = response.data.map((doc: any) => ({
    uri: doc.url,
    name: doc.title,
    description: doc.description || doc.title,
    mimeType: 'text/markdown',
  }));

  // Add special resources
  resources.unshift({
    uri: 'dgx://docs/list',
    name: 'Documentation Index',
    description: 'List of all available documentation',
    mimeType: 'application/json',
  });

  resources.unshift({
    uri: 'dgx://docs/search',
    name: 'Documentation Search',
    description: 'Search documentation (use ?q=query parameter)',
    mimeType: 'application/json',
  });

  return resources;
}

/**
 * Handle documentation search tool call
 */
export async function handleSearchTool(query: string, options: any = {}): Promise<any> {
  const response = await searchDocumentation(query, options);

  if (!response.success) {
    return {
      error: response.error,
      results: [],
    };
  }

  return {
    query,
    results: response.data,
    total: response.metadata?.totalResults || 0,
  };
}
