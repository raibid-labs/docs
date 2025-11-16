/**
 * MCP (Model Context Protocol) type definitions
 * Extends the official @modelcontextprotocol/sdk types with DGX-specific interfaces
 */

import type { Server } from '@modelcontextprotocol/sdk/server/index.js';

/**
 * Server capabilities for DGX Spark MCP
 */
export interface DGXServerCapabilities {
  resources: {
    listChanged: boolean;
  };
  tools: {
    listChanged?: boolean;
  };
}

/**
 * DGX-specific MCP server interface
 */
export interface DGXMCPServer extends Server {
  name: string;
  version: string;
}

/**
 * Re-export MCP SDK types for convenience
 */
export type {
  ListResourcesRequest,
  ReadResourceRequest,
  ListToolsRequest,
  CallToolRequest,
} from '@modelcontextprotocol/sdk/types.js';
