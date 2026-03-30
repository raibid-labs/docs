/**
 * Documentation resource handlers
 * Provides MCP resources for Spark documentation
 */

import { getDocumentationResourceList, loadDocumentationResource } from '../docs/loader.js';
import type { ResourceDescriptor, ResourceContent } from '../types/resources.js';

/**
 * Get all documentation resource descriptors
 */
export async function getDocsResourceDescriptors(): Promise<ResourceDescriptor[]> {
  const allDocs = await getDocumentationResourceList();

  const descriptors: ResourceDescriptor[] = allDocs.map(doc => ({
    uri: doc.uri,
    name: doc.name,
    description: doc.description,
    mimeType: doc.mimeType || 'text/markdown',
  }));

  return descriptors;
}

/**
 * Read documentation resource by URI
 */
export async function readDocsResource(uri: string): Promise<ResourceContent[]> {
  try {
    const doc = await loadDocumentationResource(uri);

    return [{
      uri,
      mimeType: doc.mimeType || 'text/markdown',
      text: doc.content,
    }];
  } catch (error) {
    throw new Error(`Documentation not found for URI: ${uri}`);
  }
}

/**
 * Check if URI is a documentation resource
 */
export function isDocsResourceURI(uri: string): boolean {
  return uri.startsWith('dgx://docs/spark/');
}

/**
 * Get available documentation topics
 */
export async function getDocumentationTopics(): Promise<string[]> {
  const allDocs = await getDocumentationResourceList();
  // Extract topics from URIs (dgx://docs/spark/{topic})
  return allDocs
    .map(doc => {
      const match = doc.uri.match(/^dgx:\/\/docs\/spark\/(.+)$/);
      return match ? match[1] : '';
    })
    .filter((t): t is string => t !== '');
}
