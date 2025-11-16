/**
 * MCP Resource type definitions
 */

export interface ResourceDescriptor {
  uri: string;
  name: string;
  description: string;
  mimeType: string;
  metadata?: Record<string, unknown>;
}

export interface ResourceContent {
  uri: string;
  mimeType: string;
  text?: string;
  blob?: Uint8Array;
}

export interface ResourceListResponse {
  resources: ResourceDescriptor[];
}

export interface ResourceReadResponse {
  contents: ResourceContent[];
}

/**
 * Hardware resource URIs
 */
export const HardwareResourceURIs = {
  SPECS: 'dgx://hardware/specs',
  TOPOLOGY: 'dgx://hardware/topology',
  GPUS: 'dgx://hardware/gpus',
  CPU: 'dgx://hardware/cpu',
  MEMORY: 'dgx://hardware/memory',
  STORAGE: 'dgx://hardware/storage',
  NETWORK: 'dgx://hardware/network',
} as const;

/**
 * System resource URIs
 */
export const SystemResourceURIs = {
  CAPABILITIES: 'dgx://system/capabilities',
  HEALTH: 'dgx://system/health',
  INFO: 'dgx://server/info',
} as const;

/**
 * Documentation resource URI pattern
 */
export const DocsResourceURIs = {
  BASE: 'dgx://docs/spark',
  INSTALLATION: 'dgx://docs/spark/installation',
  CONFIGURATION: 'dgx://docs/spark/configuration',
  TUNING: 'dgx://docs/spark/tuning',
  GPU_ACCELERATION: 'dgx://docs/spark/gpu-acceleration',
  TROUBLESHOOTING: 'dgx://docs/spark/troubleshooting',
} as const;

export type HardwareResourceURI = typeof HardwareResourceURIs[keyof typeof HardwareResourceURIs];
export type SystemResourceURI = typeof SystemResourceURIs[keyof typeof SystemResourceURIs];
export type DocsResourceURI = typeof DocsResourceURIs[keyof typeof DocsResourceURIs];
export type ResourceURI = HardwareResourceURI | SystemResourceURI | DocsResourceURI | string;
