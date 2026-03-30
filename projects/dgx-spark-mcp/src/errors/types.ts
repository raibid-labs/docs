/**
 * Error types and error codes for DGX Spark MCP Server
 * Provides structured error handling with proper categorization
 */

export enum ErrorCode {
  // General errors (1000-1999)
  UNKNOWN_ERROR = 1000,
  VALIDATION_ERROR = 1001,
  CONFIGURATION_ERROR = 1002,
  INITIALIZATION_ERROR = 1003,

  // MCP Protocol errors (2000-2999)
  MCP_PROTOCOL_ERROR = 2000,
  MCP_INVALID_REQUEST = 2001,
  MCP_METHOD_NOT_FOUND = 2002,
  MCP_INVALID_PARAMS = 2003,
  MCP_INTERNAL_ERROR = 2004,

  // Hardware detection errors (3000-3999)
  HARDWARE_DETECTION_ERROR = 3000,
  GPU_NOT_FOUND = 3001,
  NVIDIA_SMI_ERROR = 3002,
  TOPOLOGY_ERROR = 3003,

  // Spark errors (4000-4999)
  SPARK_CONFIG_ERROR = 4000,
  SPARK_OPTIMIZER_ERROR = 4001,
  SPARK_ESTIMATION_ERROR = 4002,

  // Resource errors (5000-5999)
  RESOURCE_NOT_FOUND = 5000,
  RESOURCE_ACCESS_DENIED = 5001,
  RESOURCE_LIMIT_EXCEEDED = 5002,

  // Tool errors (6000-6999)
  TOOL_EXECUTION_ERROR = 6000,
  TOOL_INVALID_INPUT = 6001,
  TOOL_TIMEOUT = 6002,
}

export enum ErrorSeverity {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical',
}

export interface ErrorContext {
  [key: string]: unknown;
}

export interface ErrorDetails {
  code: ErrorCode;
  message: string;
  severity: ErrorSeverity;
  context?: ErrorContext;
  originalError?: Error;
  timestamp: number;
  stack?: string;
}
