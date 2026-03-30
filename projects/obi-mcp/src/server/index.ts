/**
 * OBI MCP Server
 * Main server implementation using Model Context Protocol SDK
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  Tool,
} from '@modelcontextprotocol/sdk/types.js';
import logger from '../utils/logger.js';
import { getStatusTool, handleGetStatus } from '../tools/index.js';

/**
 * OBI MCP Server class
 */
export class ObiMcpServer {
  private server: Server;
  private tools: Map<string, Tool>;
  private toolHandlers: Map<string, (args: unknown) => Promise<unknown>>;

  constructor() {
    this.server = new Server(
      {
        name: 'obi-mcp-server',
        version: '0.1.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.tools = new Map();
    this.toolHandlers = new Map();

    this.setupTools();
    this.setupHandlers();
  }

  /**
   * Register all available tools
   */
  private setupTools(): void {
    // Register obi_get_status tool (PoC)
    this.tools.set(getStatusTool.name, getStatusTool);
    this.toolHandlers.set(getStatusTool.name, handleGetStatus);

    logger.info(`Registered ${this.tools.size} tools`);
  }

  /**
   * Setup MCP protocol handlers
   */
  private setupHandlers(): void {
    // Handle tool listing
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      logger.debug('Handling ListTools request');
      return {
        tools: Array.from(this.tools.values()),
      };
    });

    // Handle tool execution
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      logger.info(`Handling CallTool request: ${name}`);

      const handler = this.toolHandlers.get(name);
      if (!handler) {
        logger.warn(`Tool not found: ${name}`);
        throw new Error(`Unknown tool: ${name}`);
      }

      try {
        const result = await handler(args);
        return result;
      } catch (error) {
        logger.error(`Error executing tool ${name}:`, error);
        throw error;
      }
    });

    logger.info('MCP request handlers configured');
  }

  /**
   * Start the MCP server
   */
  async start(): Promise<void> {
    logger.info('Starting OBI MCP Server...');

    const transport = new StdioServerTransport();
    await this.server.connect(transport);

    logger.info('OBI MCP Server started successfully');
    logger.info(`Available tools: ${Array.from(this.tools.keys()).join(', ')}`);
  }

  /**
   * Stop the server gracefully
   */
  async stop(): Promise<void> {
    logger.info('Stopping OBI MCP Server...');
    await this.server.close();
    logger.info('OBI MCP Server stopped');
  }
}

/**
 * Create and export server instance
 */
export function createServer(): ObiMcpServer {
  return new ObiMcpServer();
}
