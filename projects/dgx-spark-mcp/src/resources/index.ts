/**
 * Resource registry and handlers
 * Central module for all MCP resources
 */

import type { ResourceDescriptor, ResourceContent } from '../types/resources.js';
import { getHardwareResourceDescriptors, readHardwareResource, isHardwareResourceURI } from './hardware.js';
import { getCapabilitiesResourceDescriptor, readCapabilitiesResource, isCapabilitiesResourceURI } from './capabilities.js';
import { getDocsResourceDescriptors, readDocsResource, isDocsResourceURI } from './docs.js';
import { SystemResourceURIs } from '../types/resources.js';

/**
 * Get all available MCP resources
 */
export async function listAllResources(): Promise<ResourceDescriptor[]> {
  const hardwareResources = getHardwareResourceDescriptors();
  const capabilitiesResource = getCapabilitiesResourceDescriptor();
  const docsResources = await getDocsResourceDescriptors();

  // Server info resource
  const serverInfoResource: ResourceDescriptor = {
    uri: SystemResourceURIs.INFO,
    name: 'Server Information',
    description: 'Basic server information and capabilities',
    mimeType: 'application/json',
  };

  return [
    serverInfoResource,
    ...hardwareResources,
    capabilitiesResource,
    ...docsResources,
  ];
}

/**
 * Read a resource by URI
 */
export async function readResource(uri: string): Promise<ResourceContent[]> {
  // Server info resource
  if (uri === SystemResourceURIs.INFO) {
    return [{
      uri,
      mimeType: 'application/json',
      text: JSON.stringify({
        name: 'dgx-spark-mcp',
        version: '0.1.0',
        description: 'Model Context Protocol server for DGX Spark optimization',
        capabilities: {
          resources: true,
          tools: true,
          hardware: true,
          documentation: true,
        },
      }, null, 2),
    }];
  }

  // Hardware resources
  if (isHardwareResourceURI(uri)) {
    return readHardwareResource(uri);
  }

  // Capabilities resource
  if (isCapabilitiesResourceURI(uri)) {
    return readCapabilitiesResource();
  }

  // Documentation resources
  if (isDocsResourceURI(uri)) {
    return readDocsResource(uri);
  }

  throw new Error(`Unknown resource URI: ${uri}`);
}

/**
 * Check if a URI is a valid resource
 */
export function isValidResourceURI(uri: string): boolean {
  return (
    uri === SystemResourceURIs.INFO ||
    isHardwareResourceURI(uri) ||
    isCapabilitiesResourceURI(uri) ||
    isDocsResourceURI(uri)
  );
}

// Re-export for convenience
export { getHardwareResourceDescriptors, readHardwareResource } from './hardware.js';
export { getCapabilitiesResourceDescriptor, readCapabilitiesResource } from './capabilities.js';
export { getDocsResourceDescriptors, readDocsResource, getDocumentationTopics } from './docs.js';
