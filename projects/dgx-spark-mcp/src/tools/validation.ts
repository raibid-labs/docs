/**
 * Tool argument validation
 * Validates tool arguments using Zod schemas
 */

import { ZodSchema, ZodError } from 'zod';
import {
  CheckGPUAvailabilityArgsSchema,
  GetOptimalSparkConfigArgsSchema,
  SearchDocumentationArgsSchema,
  EstimateResourcesArgsSchema,
  GetSystemHealthArgsSchema,
} from '../types/tools.js';

export interface ValidationResult<T> {
  success: boolean;
  data?: T;
  errors?: ValidationError[];
}

export interface ValidationError {
  path: string;
  message: string;
}

/**
 * Validate tool arguments against schema
 */
export function validateToolArgs<T>(
  schema: ZodSchema<T>,
  args: unknown
): ValidationResult<T> {
  try {
    const data = schema.parse(args);
    return {
      success: true,
      data,
    };
  } catch (error) {
    if (error instanceof ZodError) {
      const errors: ValidationError[] = error.errors.map(err => ({
        path: err.path.join('.'),
        message: err.message,
      }));

      return {
        success: false,
        errors,
      };
    }

    return {
      success: false,
      errors: [{
        path: 'unknown',
        message: 'Validation failed with unknown error',
      }],
    };
  }
}

/**
 * Get schema for tool by name
 */
export function getToolSchema(toolName: string): ZodSchema<any> | null {
  const schemas: Record<string, ZodSchema<any>> = {
    'check_gpu_availability': CheckGPUAvailabilityArgsSchema,
    'get_optimal_spark_config': GetOptimalSparkConfigArgsSchema,
    'search_documentation': SearchDocumentationArgsSchema,
    'estimate_resources': EstimateResourcesArgsSchema,
    'get_system_health': GetSystemHealthArgsSchema,
  };

  return schemas[toolName] || null;
}

/**
 * Format validation errors for user
 */
export function formatValidationErrors(errors: ValidationError[]): string {
  if (errors.length === 0) {
    return 'No validation errors';
  }

  const formatted = errors.map(err =>
    err.path ? `${err.path}: ${err.message}` : err.message
  );

  return `Validation errors:\n${formatted.join('\n')}`;
}
