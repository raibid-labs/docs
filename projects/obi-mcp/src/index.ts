#!/usr/bin/env node

/**
 * OBI MCP Server Entry Point
 */

import { createServer } from './server/index.js';
import logger from './utils/logger.js';

async function main() {
  try {
    logger.info('Initializing OBI MCP Server...');

    const server = createServer();

    // Handle graceful shutdown
    process.on('SIGINT', async () => {
      logger.info('Received SIGINT, shutting down gracefully...');
      await server.stop();
      process.exit(0);
    });

    process.on('SIGTERM', async () => {
      logger.info('Received SIGTERM, shutting down gracefully...');
      await server.stop();
      process.exit(0);
    });

    // Start server
    await server.start();
  } catch (error) {
    logger.error('Fatal error:', error);
    process.exit(1);
  }
}

main();
