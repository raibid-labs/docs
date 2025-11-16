import type { Config } from './schema.js';

/**
 * Default configuration values
 * These are used when no environment variables or config files override them
 */
export const defaultConfig: Config = {
  server: {
    port: 3000,
    host: 'localhost',
    nodeEnv: 'development',
  },
  logging: {
    level: 'info',
    format: 'json',
    dir: './logs',
    maxFiles: 10,
    maxSize: '10m',
  },
  mcp: {
    serverName: 'dgx-spark-mcp',
    serverVersion: '0.1.0',
    transport: 'stdio',
  },
  hardware: {
    nvidiaSmiPath: '/usr/bin/nvidia-smi',
    cacheTTL: 30000,
    enableGpuMonitoring: true,
  },
  spark: {
    sparkHome: process.env['SPARK_HOME'],
    sparkConfDir: process.env['SPARK_CONF_DIR'],
  },
  performance: {
    enableMetrics: true,
    metricsInterval: 60000,
    healthCheckInterval: 30000,
  },
  security: {
    enableAuth: false,
    apiKey: undefined,
  },
};
