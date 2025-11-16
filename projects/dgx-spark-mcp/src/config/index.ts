import { config as loadEnv } from 'dotenv';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';
import { ConfigSchema, type Config } from './schema.js';
import { defaultConfig } from './defaults.js';

/**
 * Load configuration from environment variables and config files
 * Priority: Environment Variables > Config File > Defaults
 */
export class ConfigLoader {
  private config: Config;

  constructor() {
    // Load .env file if it exists
    loadEnv();

    // Start with defaults
    this.config = this.deepClone(defaultConfig);

    // Load from config file if exists
    this.loadFromFile();

    // Override with environment variables
    this.loadFromEnv();

    // Validate final configuration
    this.validate();
  }

  /**
   * Deep clone configuration
   */
  private deepClone(obj: Config): Config {
    return JSON.parse(JSON.stringify(obj)) as Config;
  }

  /**
   * Load configuration from JSON file
   */
  private loadFromFile(): void {
    const configPath = join(process.cwd(), 'config', 'default.json');
    if (existsSync(configPath)) {
      try {
        const fileConfig = JSON.parse(readFileSync(configPath, 'utf-8')) as Partial<Config>;
        this.mergeConfig(this.config, fileConfig);
      } catch (error) {
        console.warn(`Failed to load config file: ${(error as Error).message}`);
      }
    }
  }

  /**
   * Load configuration from environment variables
   */
  private loadFromEnv(): void {
    const env = process.env;

    // Server configuration
    if (env['DGX_MCP_PORT']) {
      this.config.server.port = parseInt(env['DGX_MCP_PORT'], 10);
    }
    if (env['DGX_MCP_HOST']) {
      this.config.server.host = env['DGX_MCP_HOST'];
    }
    if (env['NODE_ENV']) {
      this.config.server.nodeEnv = env['NODE_ENV'] as Config['server']['nodeEnv'];
    }

    // Logging configuration
    if (env['DGX_MCP_LOG_LEVEL']) {
      this.config.logging.level = env['DGX_MCP_LOG_LEVEL'] as Config['logging']['level'];
    }
    if (env['DGX_MCP_LOG_FORMAT']) {
      this.config.logging.format = env['DGX_MCP_LOG_FORMAT'] as Config['logging']['format'];
    }
    if (env['DGX_MCP_LOG_DIR']) {
      this.config.logging.dir = env['DGX_MCP_LOG_DIR'];
    }
    if (env['DGX_MCP_LOG_MAX_FILES']) {
      this.config.logging.maxFiles = parseInt(env['DGX_MCP_LOG_MAX_FILES'], 10);
    }
    if (env['DGX_MCP_LOG_MAX_SIZE']) {
      this.config.logging.maxSize = env['DGX_MCP_LOG_MAX_SIZE'];
    }

    // MCP configuration
    if (env['DGX_MCP_SERVER_NAME']) {
      this.config.mcp.serverName = env['DGX_MCP_SERVER_NAME'];
    }
    if (env['DGX_MCP_SERVER_VERSION']) {
      this.config.mcp.serverVersion = env['DGX_MCP_SERVER_VERSION'];
    }
    if (env['DGX_MCP_TRANSPORT']) {
      this.config.mcp.transport = env['DGX_MCP_TRANSPORT'] as Config['mcp']['transport'];
    }

    // Hardware configuration
    if (env['DGX_MCP_NVIDIA_SMI_PATH']) {
      this.config.hardware.nvidiaSmiPath = env['DGX_MCP_NVIDIA_SMI_PATH'];
    }
    if (env['DGX_MCP_HARDWARE_CACHE_TTL']) {
      this.config.hardware.cacheTTL = parseInt(env['DGX_MCP_HARDWARE_CACHE_TTL'], 10);
    }
    if (env['DGX_MCP_ENABLE_GPU_MONITORING']) {
      this.config.hardware.enableGpuMonitoring = env['DGX_MCP_ENABLE_GPU_MONITORING'] === 'true';
    }

    // Spark configuration
    if (env['DGX_MCP_SPARK_HOME']) {
      this.config.spark.sparkHome = env['DGX_MCP_SPARK_HOME'];
    }
    if (env['DGX_MCP_SPARK_CONF_DIR']) {
      this.config.spark.sparkConfDir = env['DGX_MCP_SPARK_CONF_DIR'];
    }

    // Performance configuration
    if (env['DGX_MCP_ENABLE_METRICS']) {
      this.config.performance.enableMetrics = env['DGX_MCP_ENABLE_METRICS'] === 'true';
    }
    if (env['DGX_MCP_METRICS_INTERVAL']) {
      this.config.performance.metricsInterval = parseInt(env['DGX_MCP_METRICS_INTERVAL'], 10);
    }
    if (env['DGX_MCP_HEALTH_CHECK_INTERVAL']) {
      this.config.performance.healthCheckInterval = parseInt(
        env['DGX_MCP_HEALTH_CHECK_INTERVAL'],
        10
      );
    }

    // Security configuration
    if (env['DGX_MCP_ENABLE_AUTH']) {
      this.config.security.enableAuth = env['DGX_MCP_ENABLE_AUTH'] === 'true';
    }
    if (env['DGX_MCP_API_KEY']) {
      this.config.security.apiKey = env['DGX_MCP_API_KEY'];
    }
  }

  /**
   * Validate configuration using Zod schema
   */
  private validate(): void {
    try {
      this.config = ConfigSchema.parse(this.config);
    } catch (error) {
      throw new Error(`Configuration validation failed: ${(error as Error).message}`);
    }
  }

  /**
   * Deep merge two config objects (mutates target)
   */
  private mergeConfig(target: Config, source: Partial<Config>): void {
    for (const key in source) {
      if (Object.prototype.hasOwnProperty.call(source, key)) {
        const k = key as keyof Config;
        const sourceValue = source[k];
        if (sourceValue && typeof sourceValue === 'object' && !Array.isArray(sourceValue)) {
          Object.assign(target[k], sourceValue);
        } else if (sourceValue !== undefined) {
          (target[k] as typeof sourceValue) = sourceValue;
        }
      }
    }
  }

  /**
   * Get the current configuration
   */
  public getConfig(): Config {
    return this.config;
  }
}

// Singleton instance
let configInstance: ConfigLoader | null = null;

/**
 * Get configuration singleton
 */
export function getConfig(): Config {
  if (!configInstance) {
    configInstance = new ConfigLoader();
  }
  return configInstance.getConfig();
}

/**
 * Reset configuration (useful for testing)
 */
export function resetConfig(): void {
  configInstance = null;
}

// Export types
export type { Config } from './schema.js';
