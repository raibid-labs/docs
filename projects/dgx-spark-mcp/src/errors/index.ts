import { ErrorCode, ErrorSeverity, type ErrorContext, type ErrorDetails } from './types.js';

/**
 * Base error class for DGX Spark MCP Server
 * Extends native Error with additional metadata
 */
export class DGXError extends Error {
  public readonly code: ErrorCode;
  public readonly severity: ErrorSeverity;
  public readonly context?: ErrorContext;
  public readonly timestamp: number;
  public readonly originalError?: Error;

  constructor(
    message: string,
    code: ErrorCode = ErrorCode.UNKNOWN_ERROR,
    severity: ErrorSeverity = ErrorSeverity.MEDIUM,
    context?: ErrorContext,
    originalError?: Error
  ) {
    super(message);
    this.name = 'DGXError';
    this.code = code;
    this.severity = severity;
    this.context = context;
    this.timestamp = Date.now();
    this.originalError = originalError;

    // Maintains proper stack trace for where error was thrown
    Error.captureStackTrace(this, this.constructor);
  }

  /**
   * Convert error to JSON format
   */
  toJSON(): ErrorDetails {
    return {
      code: this.code,
      message: this.message,
      severity: this.severity,
      context: this.context,
      timestamp: this.timestamp,
      stack: this.stack,
    };
  }

  /**
   * Convert error to string representation
   */
  override toString(): string {
    return `[${this.code}] ${this.message}`;
  }
}

/**
 * Configuration-related errors
 */
export class ConfigurationError extends DGXError {
  constructor(message: string, context?: ErrorContext, originalError?: Error) {
    super(message, ErrorCode.CONFIGURATION_ERROR, ErrorSeverity.HIGH, context, originalError);
    this.name = 'ConfigurationError';
  }
}

/**
 * MCP Protocol errors
 */
export class MCPError extends DGXError {
  constructor(message: string, code: ErrorCode, context?: ErrorContext, originalError?: Error) {
    super(message, code, ErrorSeverity.MEDIUM, context, originalError);
    this.name = 'MCPError';
  }
}

/**
 * Hardware detection errors
 */
export class HardwareError extends DGXError {
  constructor(message: string, code: ErrorCode, context?: ErrorContext, originalError?: Error) {
    super(message, code, ErrorSeverity.HIGH, context, originalError);
    this.name = 'HardwareError';
  }
}

/**
 * Spark-related errors
 */
export class SparkError extends DGXError {
  constructor(message: string, code: ErrorCode, context?: ErrorContext, originalError?: Error) {
    super(message, code, ErrorSeverity.MEDIUM, context, originalError);
    this.name = 'SparkError';
  }
}

/**
 * Resource errors
 */
export class ResourceError extends DGXError {
  constructor(message: string, code: ErrorCode, context?: ErrorContext, originalError?: Error) {
    super(message, code, ErrorSeverity.MEDIUM, context, originalError);
    this.name = 'ResourceError';
  }
}

/**
 * Tool execution errors
 */
export class ToolError extends DGXError {
  constructor(message: string, code: ErrorCode, context?: ErrorContext, originalError?: Error) {
    super(message, code, ErrorSeverity.MEDIUM, context, originalError);
    this.name = 'ToolError';
  }
}

/**
 * Validation errors
 */
export class ValidationError extends DGXError {
  constructor(message: string, context?: ErrorContext, originalError?: Error) {
    super(message, ErrorCode.VALIDATION_ERROR, ErrorSeverity.LOW, context, originalError);
    this.name = 'ValidationError';
  }
}

// Export types
export { ErrorCode, ErrorSeverity, type ErrorContext, type ErrorDetails } from './types.js';
