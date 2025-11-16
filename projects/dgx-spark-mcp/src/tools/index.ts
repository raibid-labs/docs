/**
 * Tools registry and handlers
 * Central module for all MCP tools
 */

import type { ToolDescriptor, ToolCallResponse } from '../types/tools.js';
import { checkGPUAvailability } from './gpu-availability.js';
import { getOptimalSparkConfig } from './spark-config.js';
import { searchDocumentation } from './search-docs.js';
import { estimateResources } from './estimate-resources.js';
import { getSystemHealth } from './system-health.js';
import { validateToolArgs, getToolSchema, formatValidationErrors } from './validation.js';

/**
 * Get all available MCP tools
 */
export function listAllTools(): ToolDescriptor[] {
  return [
    {
      name: 'check_gpu_availability',
      description: 'Check current GPU availability and get recommendations for job placement. Returns available GPUs with utilization and memory status.',
      inputSchema: {
        type: 'object',
        properties: {
          minMemoryGB: {
            type: 'number',
            description: 'Minimum free GPU memory in GB required',
          },
          minUtilization: {
            type: 'number',
            description: 'Maximum GPU utilization percentage to consider available (default: 20)',
            minimum: 0,
            maximum: 100,
          },
        },
      },
    },
    {
      name: 'get_optimal_spark_config',
      description: 'Generate optimal Spark configuration based on workload type and data size. Returns recommended executor settings, memory configuration, and spark-submit command.',
      inputSchema: {
        type: 'object',
        properties: {
          workloadType: {
            type: 'string',
            enum: ['etl', 'ml-training', 'ml-inference', 'analytics', 'streaming'],
            description: 'Type of Spark workload',
          },
          dataSize: {
            type: 'string',
            description: 'Size of data to process (e.g., "100GB", "1TB")',
          },
          numExecutors: {
            type: 'number',
            description: 'Optional: specific number of executors to use',
          },
          executorMemory: {
            type: 'string',
            description: 'Optional: specific executor memory (e.g., "8g")',
          },
          useGPU: {
            type: 'boolean',
            description: 'Optional: enable GPU acceleration (default: false)',
          },
        },
        required: ['workloadType', 'dataSize'],
      },
    },
    {
      name: 'search_documentation',
      description: 'Search Spark documentation by query. Returns relevant documentation sections with excerpts and relevance scores.',
      inputSchema: {
        type: 'object',
        properties: {
          query: {
            type: 'string',
            description: 'Search query',
          },
          limit: {
            type: 'number',
            description: 'Maximum number of results to return (default: 10, max: 50)',
            minimum: 1,
            maximum: 50,
          },
          topics: {
            type: 'array',
            items: { type: 'string' },
            description: 'Optional: filter by specific topics',
          },
        },
        required: ['query'],
      },
    },
    {
      name: 'estimate_resources',
      description: 'Estimate resource requirements for a workload based on description and optional data size. Returns feasibility analysis and recommendations.',
      inputSchema: {
        type: 'object',
        properties: {
          description: {
            type: 'string',
            description: 'Description of the workload (e.g., "Train 1B parameter model", "Process daily logs")',
          },
          dataSize: {
            type: 'string',
            description: 'Optional: size of data (e.g., "500GB")',
          },
          computeType: {
            type: 'string',
            enum: ['cpu', 'gpu', 'mixed'],
            description: 'Type of compute required (default: cpu)',
          },
        },
        required: ['description'],
      },
    },
    {
      name: 'get_system_health',
      description: 'Check overall system health including CPU, memory, GPU, storage, and network. Returns status and alerts.',
      inputSchema: {
        type: 'object',
        properties: {
          verbose: {
            type: 'boolean',
            description: 'Include detailed metrics (default: false)',
          },
        },
      },
    },
  ];
}

/**
 * Call a tool by name
 */
export async function callTool(name: string, args: unknown): Promise<ToolCallResponse> {
  // Validate arguments
  const schema = getToolSchema(name);
  if (schema) {
    const validation = validateToolArgs(schema, args || {});
    if (!validation.success) {
      return {
        content: [{
          type: 'text',
          text: JSON.stringify({
            error: 'Invalid tool arguments',
            details: formatValidationErrors(validation.errors || []),
          }, null, 2),
        }],
        isError: true,
      };
    }
    args = validation.data;
  }

  // Call the appropriate tool
  switch (name) {
    case 'check_gpu_availability':
      return checkGPUAvailability(args as any);

    case 'get_optimal_spark_config':
      return getOptimalSparkConfig(args as any);

    case 'search_documentation':
      return searchDocumentation(args as any);

    case 'estimate_resources':
      return estimateResources(args as any);

    case 'get_system_health':
      return getSystemHealth(args as any);

    default:
      return {
        content: [{
          type: 'text',
          text: JSON.stringify({
            error: `Unknown tool: ${name}`,
            availableTools: listAllTools().map(t => t.name),
          }, null, 2),
        }],
        isError: true,
      };
  }
}

/**
 * Check if a tool name is valid
 */
export function isValidToolName(name: string): boolean {
  const validNames = listAllTools().map(t => t.name);
  return validNames.includes(name);
}

// Re-export tool functions
export { checkGPUAvailability } from './gpu-availability.js';
export { getOptimalSparkConfig } from './spark-config.js';
export { searchDocumentation } from './search-docs.js';
export { estimateResources } from './estimate-resources.js';
export { getSystemHealth } from './system-health.js';
