/**
 * Type definitions for OpenTelemetry eBPF Instrumentation (OBI)
 */

import { z } from 'zod';

/**
 * OBI process status
 */
export enum ObiStatus {
  RUNNING = 'running',
  STOPPED = 'stopped',
  ERROR = 'error',
  UNKNOWN = 'unknown',
}

/**
 * OBI deployment mode
 */
export enum ObiDeploymentMode {
  STANDALONE = 'standalone',
  DOCKER = 'docker',
  KUBERNETES = 'kubernetes',
}

/**
 * OBI health check result
 */
export interface ObiHealthCheck {
  status: ObiStatus;
  pid?: number;
  uptime?: number; // seconds
  cpuUsage?: number; // percentage
  memoryUsage?: number; // MB
  lastError?: string;
  configPath?: string;
}

/**
 * OBI configuration schema (simplified for MVP)
 */
export const ObiConfigSchema = z.object({
  network: z
    .object({
      enable: z.boolean().default(true),
      allowed_attributes: z.array(z.string()).optional(),
      cidrs: z
        .array(
          z.object({
            cidr: z.string(),
            name: z.string(),
          })
        )
        .optional(),
    })
    .optional(),
  attributes: z
    .object({
      kubernetes: z
        .object({
          enable: z.boolean().default(false),
        })
        .optional(),
    })
    .optional(),
  export: z
    .object({
      otlp: z
        .object({
          endpoint: z.string(),
          protocol: z.enum(['grpc', 'http/protobuf']).default('grpc'),
        })
        .optional(),
    })
    .optional(),
});

export type ObiConfig = z.infer<typeof ObiConfigSchema>;

/**
 * OBI metrics summary (parsed from logs)
 */
export interface ObiMetricsSummary {
  totalFlows: number;
  uniqueSourceIPs: number;
  uniqueDestinationIPs: number;
  protocols: Record<string, number>; // e.g., { "HTTP": 150, "gRPC": 50 }
  topSources: Array<{ ip: string; count: number }>;
  topDestinations: Array<{ ip: string; count: number }>;
  timeRange: {
    start: Date;
    end: Date;
  };
}

/**
 * OBI log entry (parsed)
 */
export interface ObiLogEntry {
  timestamp: Date;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
  fields?: Record<string, unknown>;
}

/**
 * OBI deployment options
 */
export interface ObiDeploymentOptions {
  mode: ObiDeploymentMode;
  configPath?: string;
  config?: ObiConfig;
  binaryPath?: string; // path to OBI binary (if not in PATH)
  logPath?: string; // where to store logs
  autoRestart?: boolean;
}

/**
 * OBI control result
 */
export interface ObiControlResult {
  success: boolean;
  message: string;
  data?: unknown;
  error?: string;
}
