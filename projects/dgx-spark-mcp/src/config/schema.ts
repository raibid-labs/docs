import { z } from 'zod';

/**
 * Configuration Schema using Zod for runtime validation
 * Ensures type safety and validation of all configuration values
 */

export const LogLevelSchema = z.enum(['debug', 'info', 'warn', 'error']);
export const LogFormatSchema = z.enum(['json', 'simple', 'pretty']);
export const NodeEnvSchema = z.enum(['development', 'production', 'test']);
export const TransportSchema = z.enum(['stdio', 'http', 'sse']);

export const ServerConfigSchema = z.object({
  port: z.number().int().min(1).max(65535).default(3000),
  host: z.string().default('localhost'),
  nodeEnv: NodeEnvSchema.default('development'),
});

export const LoggingConfigSchema = z.object({
  level: LogLevelSchema.default('info'),
  format: LogFormatSchema.default('json'),
  dir: z.string().default('./logs'),
  maxFiles: z.number().int().min(1).default(10),
  maxSize: z.string().default('10m'),
});

export const MCPConfigSchema = z.object({
  serverName: z.string().default('dgx-spark-mcp'),
  serverVersion: z.string().default('0.1.0'),
  transport: TransportSchema.default('stdio'),
});

export const HardwareConfigSchema = z.object({
  nvidiaSmiPath: z.string().default('/usr/bin/nvidia-smi'),
  cacheTTL: z.number().int().min(0).default(30000),
  enableGpuMonitoring: z.boolean().default(true),
});

export const SparkConfigSchema = z.object({
  sparkHome: z.string().optional(),
  sparkConfDir: z.string().optional(),
});

export const PerformanceConfigSchema = z.object({
  enableMetrics: z.boolean().default(true),
  metricsInterval: z.number().int().min(1000).default(60000),
  healthCheckInterval: z.number().int().min(1000).default(30000),
});

export const SecurityConfigSchema = z.object({
  enableAuth: z.boolean().default(false),
  apiKey: z.string().optional(),
});

/**
 * Complete configuration schema
 */
export const ConfigSchema = z.object({
  server: ServerConfigSchema,
  logging: LoggingConfigSchema,
  mcp: MCPConfigSchema,
  hardware: HardwareConfigSchema,
  spark: SparkConfigSchema,
  performance: PerformanceConfigSchema,
  security: SecurityConfigSchema,
});

/**
 * TypeScript type derived from schema
 */
export type Config = z.infer<typeof ConfigSchema>;
export type LogLevel = z.infer<typeof LogLevelSchema>;
export type LogFormat = z.infer<typeof LogFormatSchema>;
export type NodeEnv = z.infer<typeof NodeEnvSchema>;
export type Transport = z.infer<typeof TransportSchema>;
