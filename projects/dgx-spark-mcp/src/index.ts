#!/usr/bin/env node

/**
 * DGX Spark MCP Server - Entry Point
 * Model Context Protocol server for DGX Spark optimization and hardware introspection
 */

import { getConfig } from './config/index.js';
import { getLogger } from './logger/index.js';
import { DGXSparkMCPServer } from './server.js';
import { ConfigurationError } from './errors/index.js';

async function main(): Promise<void> {
  try {
    // Load configuration
    const config = getConfig();

    // Initialize logger
    const logger = getLogger(config.logging);

    logger.info('Starting DGX Spark MCP Server', {
      version: config.mcp.serverVersion,
      nodeEnv: config.server.nodeEnv,
    });

    // Create and start server
    const server = new DGXSparkMCPServer(config, logger);
    await server.start();
  } catch (error) {
    if (error instanceof ConfigurationError) {
      console.error('Configuration error:', error.message);
      process.exit(1);
    }

    console.error('Fatal error:', error);
    process.exit(1);
  }
}

// Start the server
void main();
