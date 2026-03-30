/**
 * MCP Tool type definitions
 */

import { z } from 'zod';
import type { CallToolResult } from '@modelcontextprotocol/sdk/types.js';

/**
 * Tool descriptor interface
 */
export interface ToolDescriptor {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: Record<string, unknown>;
    required?: string[];
  };
}

/**
 * Tool call response interface - uses MCP SDK type
 */
export type ToolCallResponse = CallToolResult;

/**
 * GPU availability check arguments schema
 */
export const CheckGPUAvailabilityArgsSchema = z.object({
  minMemoryGB: z.number().optional(),
  minUtilization: z.number().min(0).max(100).optional(),
});

export type CheckGPUAvailabilityArgs = z.infer<typeof CheckGPUAvailabilityArgsSchema>;

/**
 * Optimal Spark config arguments schema
 */
export const GetOptimalSparkConfigArgsSchema = z.object({
  workloadType: z.enum(['etl', 'ml-training', 'ml-inference', 'analytics', 'streaming']),
  dataSize: z.string(), // e.g., "100GB", "1TB"
  numExecutors: z.number().optional(),
  executorMemory: z.string().optional(),
  useGPU: z.boolean().optional(),
});

export type GetOptimalSparkConfigArgs = z.infer<typeof GetOptimalSparkConfigArgsSchema>;

/**
 * Search documentation arguments schema
 */
export const SearchDocumentationArgsSchema = z.object({
  query: z.string().min(1),
  limit: z.number().min(1).max(50).optional().default(10),
  topics: z.array(z.string()).optional(),
});

export type SearchDocumentationArgs = z.infer<typeof SearchDocumentationArgsSchema>;

/**
 * Estimate resources arguments schema
 */
export const EstimateResourcesArgsSchema = z.object({
  description: z.string().min(1),
  dataSize: z.string().optional(),
  computeType: z.enum(['cpu', 'gpu', 'mixed']).optional().default('cpu'),
});

export type EstimateResourcesArgs = z.infer<typeof EstimateResourcesArgsSchema>;

/**
 * Get system health arguments schema (no arguments required)
 */
export const GetSystemHealthArgsSchema = z.object({
  verbose: z.boolean().optional().default(false),
});

export type GetSystemHealthArgs = z.infer<typeof GetSystemHealthArgsSchema>;

/**
 * Tool names enum
 */
export enum ToolName {
  CHECK_GPU_AVAILABILITY = 'check_gpu_availability',
  GET_OPTIMAL_SPARK_CONFIG = 'get_optimal_spark_config',
  SEARCH_DOCUMENTATION = 'search_documentation',
  ESTIMATE_RESOURCES = 'estimate_resources',
  GET_SYSTEM_HEALTH = 'get_system_health',
}
