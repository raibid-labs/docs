/**
 * System capabilities resource handler
 * Provides MCP resource for system capabilities analysis
 */

import { analyzeCapabilities } from '../analyzers/capabilities.js';
import type { ResourceDescriptor, ResourceContent } from '../types/resources.js';
import { SystemResourceURIs } from '../types/resources.js';

/**
 * Get capabilities resource descriptor
 */
export function getCapabilitiesResourceDescriptor(): ResourceDescriptor {
  return {
    uri: SystemResourceURIs.CAPABILITIES,
    name: 'System Capabilities',
    description: 'Analyzed system capabilities including Spark recommendations, GPU support, and performance estimates',
    mimeType: 'application/json',
  };
}

/**
 * Read capabilities resource
 */
export async function readCapabilitiesResource(): Promise<ResourceContent[]> {
  const capabilities = await analyzeCapabilities();

  return [{
    uri: SystemResourceURIs.CAPABILITIES,
    mimeType: 'application/json',
    text: JSON.stringify(capabilities, null, 2),
  }];
}

/**
 * Check if URI is the capabilities resource
 */
export function isCapabilitiesResourceURI(uri: string): boolean {
  return uri === SystemResourceURIs.CAPABILITIES;
}
