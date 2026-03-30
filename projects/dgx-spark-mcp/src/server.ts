import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListResourcesRequestSchema,
  ListToolsRequestSchema,
  ReadResourceRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import type { Logger } from './logger/index.js';
import type { Config } from './config/schema.js';
import { LifecycleManager } from './lifecycle/index.js';
import { HealthManager, HealthStatus } from './health/index.js';
import type { DGXMCPServer } from './types/mcp.js';
import { listAllResources, readResource } from './resources/index.js';
import { listAllTools, callTool } from './tools/index.js';

/**
 * DGX Spark MCP Server
 * Main server class that coordinates all MCP protocol operations
 */
export class DGXSparkMCPServer {
  private server: Server;
  private logger: Logger;
  private config: Config;
  private lifecycle: LifecycleManager;
  private health: HealthManager;
  private transport?: StdioServerTransport;

  constructor(config: Config, logger: Logger) {
    this.config = config;
    this.logger = logger;

    // Initialize MCP server
    this.server = new Server(
      {
        name: config.mcp.serverName,
        version: config.mcp.serverVersion,
      },
      {
        capabilities: {
          resources: {
            listChanged: true,
          },
          tools: {
            listChanged: true,
          },
        },
      }
    ) as DGXMCPServer;

    // Initialize lifecycle manager
    this.lifecycle = new LifecycleManager(logger);

    // Initialize health manager
    this.health = new HealthManager(logger);

    // Setup lifecycle hooks
    this.setupLifecycleHooks();

    // Setup health checks
    this.setupHealthChecks();

    // Setup MCP handlers
    this.setupMCPHandlers();
  }

  /**
   * Setup lifecycle hooks
   */
  private setupLifecycleHooks(): void {
    // Startup hooks
    this.lifecycle.onStartup('initialize-server', async () => {
      this.logger.info('Initializing MCP server', {
        name: this.config.mcp.serverName,
        version: this.config.mcp.serverVersion,
      });
    });

    this.lifecycle.onStartup('start-health-checks', async () => {
      if (this.config.performance.enableMetrics) {
        this.health.startPeriodicChecks(this.config.performance.healthCheckInterval);
      }
    });

    // Shutdown hooks
    this.lifecycle.onShutdown('stop-health-checks', async () => {
      this.health.stopPeriodicChecks();
    });

    this.lifecycle.onShutdown('close-transport', async () => {
      if (this.transport) {
        this.logger.info('Closing MCP transport');
        await this.transport.close();
      }
    });
  }

  /**
   * Setup health checks
   */
  private setupHealthChecks(): void {
    // Server alive check
    this.health.registerCheck(
      'server-alive',
      async () => ({
        status: HealthStatus.HEALTHY,
        message: 'Server is alive',
      }),
      true
    );

    // Configuration check
    this.health.registerCheck(
      'configuration',
      async () => ({
        status: HealthStatus.HEALTHY,
        message: 'Configuration loaded',
        metadata: {
          serverName: this.config.mcp.serverName,
          serverVersion: this.config.mcp.serverVersion,
        },
      }),
      true
    );
  }

  /**
   * Setup MCP protocol handlers
   */
  private setupMCPHandlers(): void {
    // List resources handler
    this.server.setRequestHandler(ListResourcesRequestSchema, async () => {
      this.logger.debug('Received list_resources request');

      try {
        const resources = await listAllResources();
        return { resources };
      } catch (error) {
        this.logger.error('Failed to list resources', error as Error);
        throw error;
      }
    });

    // Read resource handler
    this.server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
      const uri = request.params.uri;
      this.logger.debug('Received read_resource request', { uri });

      try {
        const contents = await readResource(uri);
        return { contents };
      } catch (error) {
        this.logger.error('Failed to read resource', error as Error, { uri });
        throw new Error(`Failed to read resource ${uri}: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    });

    // List tools handler
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      this.logger.debug('Received list_tools request');

      try {
        const tools = listAllTools();
        return { tools };
      } catch (error) {
        this.logger.error('Failed to list tools', error as Error);
        throw error;
      }
    });

    // Call tool handler
    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const toolName = request.params.name;
      const args = request.params.arguments;

      this.logger.debug('Received call_tool request', { tool: toolName, args });

      try {
        const result = await callTool(toolName, args);
        return result;
      } catch (error) {
        this.logger.error('Failed to call tool', error as Error, { tool: toolName });
        return {
          content: [{
            type: 'text' as const,
            text: JSON.stringify({
              error: `Failed to execute tool ${toolName}`,
              message: error instanceof Error ? error.message : 'Unknown error',
            }, null, 2),
          }],
          isError: true,
        };
      }
    });
  }

  /**
   * Start the MCP server
   */
  async start(): Promise<void> {
    try {
      // Execute startup hooks
      await this.lifecycle.startup();

      // Create and connect transport based on configuration
      if (this.config.mcp.transport === 'stdio') {
        this.logger.info('Starting MCP server with stdio transport');
        this.transport = new StdioServerTransport();
        await this.server.connect(this.transport);
        this.logger.info('MCP server started successfully');
      } else {
        throw new Error(`Unsupported transport: ${this.config.mcp.transport}`);
      }
    } catch (error) {
      this.logger.error('Failed to start MCP server', error as Error);
      throw error;
    }
  }

  /**
   * Get server instance
   */
  getServer(): Server {
    return this.server;
  }

  /**
   * Get health manager
   */
  getHealth(): HealthManager {
    return this.health;
  }

  /**
   * Get lifecycle manager
   */
  getLifecycle(): LifecycleManager {
    return this.lifecycle;
  }
}
