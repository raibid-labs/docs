/**
 * Unit tests for configuration schema
 */

import { describe, it, expect } from '@jest/globals';
import {
  ConfigSchema,
  LogLevelSchema,
  LogFormatSchema,
  NodeEnvSchema,
  TransportSchema,
  ServerConfigSchema,
  LoggingConfigSchema,
  MCPConfigSchema,
  HardwareConfigSchema,
  SparkConfigSchema,
  PerformanceConfigSchema,
  SecurityConfigSchema,
} from './schema.js';

describe('Configuration Schema', () => {
  describe('LogLevelSchema', () => {
    it('should accept valid log levels', () => {
      expect(() => LogLevelSchema.parse('debug')).not.toThrow();
      expect(() => LogLevelSchema.parse('info')).not.toThrow();
      expect(() => LogLevelSchema.parse('warn')).not.toThrow();
      expect(() => LogLevelSchema.parse('error')).not.toThrow();
    });

    it('should reject invalid log levels', () => {
      expect(() => LogLevelSchema.parse('invalid')).toThrow();
      expect(() => LogLevelSchema.parse('trace')).toThrow();
    });
  });

  describe('LogFormatSchema', () => {
    it('should accept valid log formats', () => {
      expect(() => LogFormatSchema.parse('json')).not.toThrow();
      expect(() => LogFormatSchema.parse('simple')).not.toThrow();
      expect(() => LogFormatSchema.parse('pretty')).not.toThrow();
    });

    it('should reject invalid log formats', () => {
      expect(() => LogFormatSchema.parse('xml')).toThrow();
    });
  });

  describe('NodeEnvSchema', () => {
    it('should accept valid node environments', () => {
      expect(() => NodeEnvSchema.parse('development')).not.toThrow();
      expect(() => NodeEnvSchema.parse('production')).not.toThrow();
      expect(() => NodeEnvSchema.parse('test')).not.toThrow();
    });

    it('should reject invalid environments', () => {
      expect(() => NodeEnvSchema.parse('staging')).toThrow();
    });
  });

  describe('TransportSchema', () => {
    it('should accept valid transports', () => {
      expect(() => TransportSchema.parse('stdio')).not.toThrow();
      expect(() => TransportSchema.parse('http')).not.toThrow();
      expect(() => TransportSchema.parse('sse')).not.toThrow();
    });

    it('should reject invalid transports', () => {
      expect(() => TransportSchema.parse('websocket')).toThrow();
    });
  });

  describe('ServerConfigSchema', () => {
    it('should accept valid server config', () => {
      const config = {
        port: 3000,
        host: 'localhost',
        nodeEnv: 'development' as const,
      };
      expect(() => ServerConfigSchema.parse(config)).not.toThrow();
    });

    it('should apply defaults', () => {
      const result = ServerConfigSchema.parse({});
      expect(result.port).toBe(3000);
      expect(result.host).toBe('localhost');
      expect(result.nodeEnv).toBe('development');
    });

    it('should reject invalid port numbers', () => {
      expect(() => ServerConfigSchema.parse({ port: 0 })).toThrow();
      expect(() => ServerConfigSchema.parse({ port: 70000 })).toThrow();
      expect(() => ServerConfigSchema.parse({ port: -1 })).toThrow();
    });

    it('should accept valid port numbers', () => {
      expect(() => ServerConfigSchema.parse({ port: 1 })).not.toThrow();
      expect(() => ServerConfigSchema.parse({ port: 8080 })).not.toThrow();
      expect(() => ServerConfigSchema.parse({ port: 65535 })).not.toThrow();
    });
  });

  describe('LoggingConfigSchema', () => {
    it('should accept valid logging config', () => {
      const config = {
        level: 'info' as const,
        format: 'json' as const,
        dir: './logs',
        maxFiles: 10,
        maxSize: '10m',
      };
      expect(() => LoggingConfigSchema.parse(config)).not.toThrow();
    });

    it('should apply defaults', () => {
      const result = LoggingConfigSchema.parse({});
      expect(result.level).toBe('info');
      expect(result.format).toBe('json');
      expect(result.dir).toBe('./logs');
      expect(result.maxFiles).toBe(10);
      expect(result.maxSize).toBe('10m');
    });

    it('should reject negative maxFiles', () => {
      expect(() => LoggingConfigSchema.parse({ maxFiles: 0 })).toThrow();
      expect(() => LoggingConfigSchema.parse({ maxFiles: -1 })).toThrow();
    });
  });

  describe('MCPConfigSchema', () => {
    it('should accept valid MCP config', () => {
      const config = {
        serverName: 'dgx-spark-mcp',
        serverVersion: '0.1.0',
        transport: 'stdio' as const,
      };
      expect(() => MCPConfigSchema.parse(config)).not.toThrow();
    });

    it('should apply defaults', () => {
      const result = MCPConfigSchema.parse({});
      expect(result.serverName).toBe('dgx-spark-mcp');
      expect(result.serverVersion).toBe('0.1.0');
      expect(result.transport).toBe('stdio');
    });
  });

  describe('HardwareConfigSchema', () => {
    it('should accept valid hardware config', () => {
      const config = {
        nvidiaSmiPath: '/usr/bin/nvidia-smi',
        cacheTTL: 30000,
        enableGpuMonitoring: true,
      };
      expect(() => HardwareConfigSchema.parse(config)).not.toThrow();
    });

    it('should apply defaults', () => {
      const result = HardwareConfigSchema.parse({});
      expect(result.nvidiaSmiPath).toBe('/usr/bin/nvidia-smi');
      expect(result.cacheTTL).toBe(30000);
      expect(result.enableGpuMonitoring).toBe(true);
    });

    it('should accept zero cache TTL', () => {
      expect(() => HardwareConfigSchema.parse({ cacheTTL: 0 })).not.toThrow();
    });

    it('should reject negative cache TTL', () => {
      expect(() => HardwareConfigSchema.parse({ cacheTTL: -1 })).toThrow();
    });
  });

  describe('SparkConfigSchema', () => {
    it('should accept valid Spark config', () => {
      const config = {
        sparkHome: '/opt/spark',
        sparkConfDir: '/opt/spark/conf',
      };
      expect(() => SparkConfigSchema.parse(config)).not.toThrow();
    });

    it('should accept empty config', () => {
      expect(() => SparkConfigSchema.parse({})).not.toThrow();
    });

    it('should allow optional fields', () => {
      const result = SparkConfigSchema.parse({});
      expect(result.sparkHome).toBeUndefined();
      expect(result.sparkConfDir).toBeUndefined();
    });
  });

  describe('PerformanceConfigSchema', () => {
    it('should accept valid performance config', () => {
      const config = {
        enableMetrics: true,
        metricsInterval: 60000,
        healthCheckInterval: 30000,
      };
      expect(() => PerformanceConfigSchema.parse(config)).not.toThrow();
    });

    it('should apply defaults', () => {
      const result = PerformanceConfigSchema.parse({});
      expect(result.enableMetrics).toBe(true);
      expect(result.metricsInterval).toBe(60000);
      expect(result.healthCheckInterval).toBe(30000);
    });

    it('should reject intervals less than 1000ms', () => {
      expect(() => PerformanceConfigSchema.parse({ metricsInterval: 500 })).toThrow();
      expect(() => PerformanceConfigSchema.parse({ healthCheckInterval: 999 })).toThrow();
    });

    it('should accept intervals >= 1000ms', () => {
      expect(() => PerformanceConfigSchema.parse({ metricsInterval: 1000 })).not.toThrow();
      expect(() => PerformanceConfigSchema.parse({ healthCheckInterval: 5000 })).not.toThrow();
    });
  });

  describe('SecurityConfigSchema', () => {
    it('should accept valid security config', () => {
      const config = {
        enableAuth: true,
        apiKey: 'test-api-key',
      };
      expect(() => SecurityConfigSchema.parse(config)).not.toThrow();
    });

    it('should apply defaults', () => {
      const result = SecurityConfigSchema.parse({});
      expect(result.enableAuth).toBe(false);
      expect(result.apiKey).toBeUndefined();
    });
  });

  describe('ConfigSchema (full)', () => {
    it('should accept complete valid config', () => {
      const config = {
        server: {
          port: 3000,
          host: 'localhost',
          nodeEnv: 'development' as const,
        },
        logging: {
          level: 'info' as const,
          format: 'json' as const,
          dir: './logs',
          maxFiles: 10,
          maxSize: '10m',
        },
        mcp: {
          serverName: 'dgx-spark-mcp',
          serverVersion: '0.1.0',
          transport: 'stdio' as const,
        },
        hardware: {
          nvidiaSmiPath: '/usr/bin/nvidia-smi',
          cacheTTL: 30000,
          enableGpuMonitoring: true,
        },
        spark: {
          sparkHome: '/opt/spark',
          sparkConfDir: '/opt/spark/conf',
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
      expect(() => ConfigSchema.parse(config)).not.toThrow();
    });

    it('should apply all defaults when empty', () => {
      const result = ConfigSchema.parse({
        server: {},
        logging: {},
        mcp: {},
        hardware: {},
        spark: {},
        performance: {},
        security: {},
      });

      expect(result.server.port).toBe(3000);
      expect(result.logging.level).toBe('info');
      expect(result.mcp.transport).toBe('stdio');
      expect(result.hardware.cacheTTL).toBe(30000);
      expect(result.performance.enableMetrics).toBe(true);
      expect(result.security.enableAuth).toBe(false);
    });
  });
});
