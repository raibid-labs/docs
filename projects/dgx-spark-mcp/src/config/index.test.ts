/**
 * Unit tests for configuration loader
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { ConfigLoader, getConfig, resetConfig } from './index.js';

describe('ConfigLoader', () => {
  beforeEach(() => {
    // Reset config singleton before each test
    resetConfig();
    // Clear environment variables
    delete process.env['DGX_MCP_PORT'];
    delete process.env['DGX_MCP_HOST'];
    delete process.env['NODE_ENV'];
    delete process.env['DGX_MCP_LOG_LEVEL'];
    delete process.env['DGX_MCP_ENABLE_GPU_MONITORING'];
  });

  afterEach(() => {
    resetConfig();
  });

  describe('constructor', () => {
    it('should create config with defaults', () => {
      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.server.port).toBe(3000);
      expect(config.server.host).toBe('localhost');
      expect(config.logging.level).toBe('info');
      expect(config.mcp.transport).toBe('stdio');
    });

    it('should load configuration from environment variables', () => {
      process.env['DGX_MCP_PORT'] = '8080';
      process.env['DGX_MCP_HOST'] = '0.0.0.0';
      process.env['DGX_MCP_LOG_LEVEL'] = 'debug';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.server.port).toBe(8080);
      expect(config.server.host).toBe('0.0.0.0');
      expect(config.logging.level).toBe('debug');
    });

    it('should parse boolean environment variables correctly', () => {
      process.env['DGX_MCP_ENABLE_GPU_MONITORING'] = 'true';
      process.env['DGX_MCP_ENABLE_METRICS'] = 'false';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.hardware.enableGpuMonitoring).toBe(true);
      expect(config.performance.enableMetrics).toBe(false);
    });

    it('should parse numeric environment variables correctly', () => {
      process.env['DGX_MCP_PORT'] = '9000';
      process.env['DGX_MCP_HARDWARE_CACHE_TTL'] = '60000';
      process.env['DGX_MCP_LOG_MAX_FILES'] = '20';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.server.port).toBe(9000);
      expect(config.hardware.cacheTTL).toBe(60000);
      expect(config.logging.maxFiles).toBe(20);
    });

    it('should load NODE_ENV correctly', () => {
      process.env['NODE_ENV'] = 'production';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.server.nodeEnv).toBe('production');
    });

    it('should load all server config from env', () => {
      process.env['DGX_MCP_PORT'] = '4000';
      process.env['DGX_MCP_HOST'] = '127.0.0.1';
      process.env['NODE_ENV'] = 'test';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.server.port).toBe(4000);
      expect(config.server.host).toBe('127.0.0.1');
      expect(config.server.nodeEnv).toBe('test');
    });

    it('should load all logging config from env', () => {
      process.env['DGX_MCP_LOG_LEVEL'] = 'warn';
      process.env['DGX_MCP_LOG_FORMAT'] = 'pretty';
      process.env['DGX_MCP_LOG_DIR'] = '/var/log/dgx-mcp';
      process.env['DGX_MCP_LOG_MAX_FILES'] = '5';
      process.env['DGX_MCP_LOG_MAX_SIZE'] = '20m';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.logging.level).toBe('warn');
      expect(config.logging.format).toBe('pretty');
      expect(config.logging.dir).toBe('/var/log/dgx-mcp');
      expect(config.logging.maxFiles).toBe(5);
      expect(config.logging.maxSize).toBe('20m');
    });

    it('should load all MCP config from env', () => {
      process.env['DGX_MCP_SERVER_NAME'] = 'test-server';
      process.env['DGX_MCP_SERVER_VERSION'] = '1.0.0';
      process.env['DGX_MCP_TRANSPORT'] = 'http';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.mcp.serverName).toBe('test-server');
      expect(config.mcp.serverVersion).toBe('1.0.0');
      expect(config.mcp.transport).toBe('http');
    });

    it('should load all hardware config from env', () => {
      process.env['DGX_MCP_NVIDIA_SMI_PATH'] = '/usr/local/bin/nvidia-smi';
      process.env['DGX_MCP_HARDWARE_CACHE_TTL'] = '45000';
      process.env['DGX_MCP_ENABLE_GPU_MONITORING'] = 'false';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.hardware.nvidiaSmiPath).toBe('/usr/local/bin/nvidia-smi');
      expect(config.hardware.cacheTTL).toBe(45000);
      expect(config.hardware.enableGpuMonitoring).toBe(false);
    });

    it('should load all Spark config from env', () => {
      process.env['DGX_MCP_SPARK_HOME'] = '/opt/apache-spark';
      process.env['DGX_MCP_SPARK_CONF_DIR'] = '/etc/spark/conf';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.spark.sparkHome).toBe('/opt/apache-spark');
      expect(config.spark.sparkConfDir).toBe('/etc/spark/conf');
    });

    it('should load all performance config from env', () => {
      process.env['DGX_MCP_ENABLE_METRICS'] = 'true';
      process.env['DGX_MCP_METRICS_INTERVAL'] = '120000';
      process.env['DGX_MCP_HEALTH_CHECK_INTERVAL'] = '15000';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.performance.enableMetrics).toBe(true);
      expect(config.performance.metricsInterval).toBe(120000);
      expect(config.performance.healthCheckInterval).toBe(15000);
    });

    it('should load all security config from env', () => {
      process.env['DGX_MCP_ENABLE_AUTH'] = 'true';
      process.env['DGX_MCP_API_KEY'] = 'secret-key-123';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.security.enableAuth).toBe(true);
      expect(config.security.apiKey).toBe('secret-key-123');
    });

    it('should throw on invalid configuration', () => {
      process.env['DGX_MCP_PORT'] = 'invalid';

      expect(() => new ConfigLoader()).toThrow('Configuration validation failed');
    });

    it('should throw on out of range port', () => {
      process.env['DGX_MCP_PORT'] = '70000';

      expect(() => new ConfigLoader()).toThrow();
    });

    it('should throw on invalid log level', () => {
      process.env['DGX_MCP_LOG_LEVEL'] = 'invalid';

      expect(() => new ConfigLoader()).toThrow();
    });
  });

  describe('getConfig (singleton)', () => {
    it('should return same instance on multiple calls', () => {
      const config1 = getConfig();
      const config2 = getConfig();

      expect(config1).toBe(config2);
    });

    it('should create new instance after reset', () => {
      const config1 = getConfig();
      resetConfig();
      const config2 = getConfig();

      expect(config1).not.toBe(config2);
    });

    it('should reflect environment changes after reset', () => {
      const config1 = getConfig();
      expect(config1.server.port).toBe(3000);

      process.env['DGX_MCP_PORT'] = '9000';
      resetConfig();

      const config2 = getConfig();
      expect(config2.server.port).toBe(9000);
    });
  });

  describe('resetConfig', () => {
    it('should reset configuration singleton', () => {
      getConfig();
      resetConfig();

      // Next call should create new instance
      const config = getConfig();
      expect(config).toBeDefined();
    });
  });

  describe('edge cases', () => {
    it('should handle empty string environment variables', () => {
      process.env['DGX_MCP_SERVER_NAME'] = '';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.mcp.serverName).toBe('');
    });

    it('should handle whitespace in environment variables', () => {
      process.env['DGX_MCP_LOG_DIR'] = '  /var/log  ';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.logging.dir).toBe('  /var/log  ');
    });

    it('should not override defaults with undefined env vars', () => {
      delete process.env['DGX_MCP_PORT'];

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.server.port).toBe(3000);
    });

    it('should handle boolean string variants', () => {
      process.env['DGX_MCP_ENABLE_GPU_MONITORING'] = 'false';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.hardware.enableGpuMonitoring).toBe(false);
    });

    it('should handle zero values correctly', () => {
      process.env['DGX_MCP_HARDWARE_CACHE_TTL'] = '0';

      const loader = new ConfigLoader();
      const config = loader.getConfig();

      expect(config.hardware.cacheTTL).toBe(0);
    });
  });
});
