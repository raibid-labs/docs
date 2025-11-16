/**
 * MCP-specific type definitions
 */

/**
 * Tool argument schemas for OBI MCP tools
 */

export interface GetStatusArgs {
  verbose?: boolean;
}

export interface GetLogsArgs {
  lines?: number;
  level?: 'info' | 'warn' | 'error' | 'debug' | 'all';
}

export interface UpdateConfigArgs {
  config: Record<string, unknown>;
  merge?: boolean; // if true, merge with existing config; if false, replace
  restart?: boolean; // if true, restart OBI after config update
}

export interface DeployLocalArgs {
  config?: Record<string, unknown>;
  configPath?: string;
  binaryPath?: string;
}

export interface GetMetricsSummaryArgs {
  timeRangeMinutes?: number;
  aggregateBy?: 'protocol' | 'source' | 'destination';
}

/**
 * Resource URI patterns
 */
export const OBI_RESOURCE_URIS = {
  CONFIG_CURRENT: 'obi://config/current',
  STATUS_HEALTH: 'obi://status/health',
  LOGS_RECENT: 'obi://logs/recent',
  METRICS_SUMMARY: 'obi://metrics/summary',
  DOCS_QUICKSTART: 'obi://docs/quickstart',
} as const;

/**
 * Prompt templates
 */
export const OBI_PROMPTS = {
  SETUP_LOCAL: 'setup-obi-local',
  DIAGNOSE_ISSUES: 'diagnose-obi-issues',
  ANALYZE_NETWORK: 'analyze-network-flows',
} as const;
