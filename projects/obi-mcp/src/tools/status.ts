/**
 * OBI Status Tool - PoC Implementation
 * Demonstrates MCP tool integration with OBI
 */

import { Tool } from '@modelcontextprotocol/sdk/types.js';
import { z } from 'zod';
import obiManager from '../utils/obi-manager.js';
import logger from '../utils/logger.js';
import { GetStatusArgs } from '../types/mcp.js';

/**
 * Tool definition for MCP
 */
export const getStatusTool: Tool = {
  name: 'obi_get_status',
  description:
    'Get the current status of the OpenTelemetry eBPF Instrumentation (OBI) process. ' +
    'Returns information about whether OBI is running, its PID, resource usage, and health.',
  inputSchema: {
    type: 'object',
    properties: {
      verbose: {
        type: 'boolean',
        description: 'Include detailed process information (CPU, memory, uptime)',
        default: false,
      },
    },
  },
};

/**
 * Argument validation schema
 */
const GetStatusArgsSchema = z.object({
  verbose: z.boolean().optional().default(false),
});

/**
 * Tool handler implementation
 */
export async function handleGetStatus(args: unknown) {
  try {
    // Validate arguments
    const validatedArgs = GetStatusArgsSchema.parse(args) as GetStatusArgs;

    logger.info('Getting OBI status', { verbose: validatedArgs.verbose });

    // Get status from OBI manager
    const status = await obiManager.getStatus(validatedArgs.verbose);

    // Format response
    const response = {
      status: status.status,
      ...(status.pid && { pid: status.pid }),
      ...(status.uptime && { uptime: `${status.uptime}s` }),
      ...(validatedArgs.verbose && {
        details: {
          cpuUsage: status.cpuUsage ? `${status.cpuUsage.toFixed(2)}%` : 'N/A',
          memoryUsage: status.memoryUsage ? `${status.memoryUsage.toFixed(2)} MB` : 'N/A',
          configPath: status.configPath || 'Not configured',
        },
      }),
      ...(status.lastError && { lastError: status.lastError }),
    };

    // Return formatted text content
    return {
      content: [
        {
          type: 'text',
          text: formatStatusResponse(response, validatedArgs.verbose),
        },
      ],
    };
  } catch (error) {
    logger.error('Error getting OBI status', { error });
    return {
      content: [
        {
          type: 'text',
          text: `Error: ${error instanceof Error ? error.message : String(error)}`,
        },
      ],
      isError: true,
    };
  }
}

/**
 * Format status response as human-readable text
 */
function formatStatusResponse(response: Record<string, unknown>, verbose: boolean): string {
  const lines: string[] = [];

  lines.push('=== OBI Status ===\n');
  lines.push(`Status: ${response.status}`);

  if (response.pid) {
    lines.push(`PID: ${response.pid}`);
  }

  if (response.uptime) {
    lines.push(`Uptime: ${response.uptime}`);
  }

  if (verbose && response.details) {
    const details = response.details as Record<string, string>;
    lines.push('\n--- Details ---');
    lines.push(`CPU Usage: ${details.cpuUsage}`);
    lines.push(`Memory Usage: ${details.memoryUsage}`);
    lines.push(`Config Path: ${details.configPath}`);
  }

  if (response.lastError) {
    lines.push(`\n⚠️  Last Error: ${response.lastError}`);
  }

  return lines.join('\n');
}
