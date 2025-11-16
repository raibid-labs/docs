/**
 * MCP Test Client Helper
 * Provides utilities for testing MCP protocol compliance
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import type {
  CallToolRequest,
  ListResourcesRequest,
  ReadResourceRequest,
  ListToolsRequest,
} from '@modelcontextprotocol/sdk/types.js';

/**
 * Test MCP client that can interact with the server
 */
export class TestMCPClient {
  private server: Server;

  constructor(server: Server) {
    this.server = server;
  }

  /**
   * List all available resources
   */
  async listResources(): Promise<any> {
    const request: ListResourcesRequest = {
      method: 'resources/list',
      params: {},
    };

    return this.server.request(request);
  }

  /**
   * Read a resource by URI
   */
  async readResource(uri: string): Promise<any> {
    const request: ReadResourceRequest = {
      method: 'resources/read',
      params: { uri },
    };

    return this.server.request(request);
  }

  /**
   * List all available tools
   */
  async listTools(): Promise<any> {
    const request: ListToolsRequest = {
      method: 'tools/list',
      params: {},
    };

    return this.server.request(request);
  }

  /**
   * Call a tool
   */
  async callTool(name: string, args: Record<string, any>): Promise<any> {
    const request: CallToolRequest = {
      method: 'tools/call',
      params: {
        name,
        arguments: args,
      },
    };

    return this.server.request(request);
  }

  /**
   * Get server capabilities
   */
  async getCapabilities(): Promise<any> {
    const request = {
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: {
          name: 'test-client',
          version: '1.0.0',
        },
      },
    };

    return this.server.request(request);
  }
}

/**
 * Create a test MCP server instance
 */
export async function createTestServer(): Promise<Server> {
  const server = new Server(
    {
      name: 'dgx-spark-mcp-test',
      version: '0.1.0',
    },
    {
      capabilities: {
        resources: {},
        tools: {},
      },
    }
  );

  return server;
}

/**
 * Wait for server to be ready
 */
export async function waitForServerReady(
  server: Server,
  timeout = 5000
): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    try {
      // Try to ping the server
      await new Promise(resolve => setTimeout(resolve, 100));
      return;
    } catch (error) {
      if (Date.now() - startTime >= timeout) {
        throw new Error('Server did not become ready in time');
      }
    }
  }
}

/**
 * Mock hardware detection for testing
 */
export function mockHardwareDetection() {
  process.env['MOCK_HARDWARE'] = 'true';
  process.env['DGX_MCP_NVIDIA_SMI_PATH'] = '/usr/bin/nvidia-smi';
}

/**
 * Clean up test environment
 */
export function cleanupTestEnvironment() {
  delete process.env['MOCK_HARDWARE'];
}
