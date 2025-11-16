/**
 * Unit tests for Spark configuration optimizer
 */

import { describe, it, expect } from '@jest/globals';
import { generateConfig } from './spark.js';
import type { SparkConfigRequest } from '../types/spark-config.js';

describe('Spark Configuration Optimizer', () => {
  describe('generateConfig', () => {
    it('should generate config for ML training workload', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'ml-training',
        dataSize: '100GB',
        totalMemory: 512,
        totalCores: 96,
        gpuCount: 4,
      };

      const result = await generateConfig(request);

      expect(result).toHaveProperty('config');
      expect(result).toHaveProperty('rationale');
      expect(result.config.executor).toBeDefined();
      expect(result.config.executor.cores).toBeGreaterThan(0);
      expect(result.config.executor.instances).toBeGreaterThan(0);
    });

    it('should generate config for ETL workload', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'etl',
        dataSize: '1TB',
        totalMemory: 512,
        totalCores: 96,
      };

      const result = await generateConfig(request);

      expect(result.config).toBeDefined();
      expect(result.rationale).toBeInstanceOf(Array);
      expect(result.rationale.length).toBeGreaterThan(0);
    });

    it('should generate config for analytics workload', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'analytics',
        dataSize: '500GB',
      };

      const result = await generateConfig(request);

      expect(result.config.executor).toBeDefined();
      expect(result.config.driver).toBeDefined();
    });

    it('should include GPU config when GPUs available', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'ml-training',
        dataSize: '100GB',
        gpuCount: 4,
      };

      const result = await generateConfig(request);

      expect(result.config.gpu).toBeDefined();
    });

    it('should not include GPU config when no GPUs', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'etl',
        dataSize: '100GB',
        gpuCount: 0,
      };

      const result = await generateConfig(request);

      expect(result.config.gpu).toBeUndefined();
    });

    it('should scale resources with data size', async () => {
      const smallRequest: SparkConfigRequest = {
        workloadType: 'analytics',
        dataSize: '10GB',
      };

      const largeRequest: SparkConfigRequest = {
        workloadType: 'analytics',
        dataSize: '1TB',
      };

      const small = await generateConfig(smallRequest);
      const large = await generateConfig(largeRequest);

      expect(large.config.executor.instances).toBeGreaterThanOrEqual(small.config.executor.instances);
    });

    it('should provide rationale for decisions', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'ml-training',
        dataSize: '100GB',
      };

      const result = await generateConfig(request);

      expect(result.rationale).toBeInstanceOf(Array);
      expect(result.rationale.length).toBeGreaterThan(0);
      expect(result.rationale.some(r => r.includes('workload'))).toBe(true);
    });

    it('should respect constraints', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'analytics',
        dataSize: '100GB',
        constraints: {
          maxExecutors: 4,
          maxCoresPerExecutor: 4,
        },
      };

      const result = await generateConfig(request);

      expect(result.config.executor.instances).toBeLessThanOrEqual(4);
      expect(result.config.executor.cores).toBeLessThanOrEqual(4);
    });
  });

  describe('Edge Cases', () => {
    it('should handle very small data sizes', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'analytics',
        dataSize: '1MB',
      };

      const result = await generateConfig(request);

      expect(result.config).toBeDefined();
      expect(result.config.executor.instances).toBeGreaterThan(0);
    });

    it('should handle very large data sizes', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'analytics',
        dataSize: '100TB',
      };

      const result = await generateConfig(request);

      expect(result.config).toBeDefined();
      expect(result.config.executor.instances).toBeGreaterThan(0);
    });

    it('should handle missing optional parameters', async () => {
      const request: SparkConfigRequest = {
        workloadType: 'analytics',
        dataSize: '100GB',
      };

      const result = await generateConfig(request);

      expect(result.config).toBeDefined();
    });
  });
});
