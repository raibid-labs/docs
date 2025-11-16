/**
 * Unit tests for default configuration
 */

import { describe, it, expect } from '@jest/globals';
import { defaultConfig } from './defaults.js';

describe('Default Configuration', () => {
  it('should have valid server defaults', () => {
    expect(defaultConfig.server.port).toBe(3000);
    expect(defaultConfig.server.host).toBe('localhost');
    expect(defaultConfig.server.nodeEnv).toBe('development');
  });

  it('should have valid logging defaults', () => {
    expect(defaultConfig.logging.level).toBe('info');
    expect(defaultConfig.logging.format).toBe('json');
    expect(defaultConfig.logging.dir).toBe('./logs');
    expect(defaultConfig.logging.maxFiles).toBe(10);
    expect(defaultConfig.logging.maxSize).toBe('10m');
  });

  it('should have valid MCP defaults', () => {
    expect(defaultConfig.mcp.serverName).toBe('dgx-spark-mcp');
    expect(defaultConfig.mcp.serverVersion).toBe('0.1.0');
    expect(defaultConfig.mcp.transport).toBe('stdio');
  });

  it('should have valid hardware defaults', () => {
    expect(defaultConfig.hardware.nvidiaSmiPath).toBe('/usr/bin/nvidia-smi');
    expect(defaultConfig.hardware.cacheTTL).toBe(30000);
    expect(defaultConfig.hardware.enableGpuMonitoring).toBe(true);
  });

  it('should have valid performance defaults', () => {
    expect(defaultConfig.performance.enableMetrics).toBe(true);
    expect(defaultConfig.performance.metricsInterval).toBe(60000);
    expect(defaultConfig.performance.healthCheckInterval).toBe(30000);
  });

  it('should have valid security defaults', () => {
    expect(defaultConfig.security.enableAuth).toBe(false);
    expect(defaultConfig.security.apiKey).toBeUndefined();
  });

  it('should have all required top-level keys', () => {
    expect(defaultConfig).toHaveProperty('server');
    expect(defaultConfig).toHaveProperty('logging');
    expect(defaultConfig).toHaveProperty('mcp');
    expect(defaultConfig).toHaveProperty('hardware');
    expect(defaultConfig).toHaveProperty('spark');
    expect(defaultConfig).toHaveProperty('performance');
    expect(defaultConfig).toHaveProperty('security');
  });

  it('should have sensible cache TTL (30 seconds)', () => {
    expect(defaultConfig.hardware.cacheTTL).toBe(30000); // 30 seconds
  });

  it('should have sensible metrics interval (1 minute)', () => {
    expect(defaultConfig.performance.metricsInterval).toBe(60000); // 60 seconds
  });

  it('should have sensible health check interval (30 seconds)', () => {
    expect(defaultConfig.performance.healthCheckInterval).toBe(30000); // 30 seconds
  });

  it('should have GPU monitoring enabled by default', () => {
    expect(defaultConfig.hardware.enableGpuMonitoring).toBe(true);
  });

  it('should have metrics enabled by default', () => {
    expect(defaultConfig.performance.enableMetrics).toBe(true);
  });

  it('should have auth disabled by default', () => {
    expect(defaultConfig.security.enableAuth).toBe(false);
  });

  it('should use stdio transport by default', () => {
    expect(defaultConfig.mcp.transport).toBe('stdio');
  });
});
