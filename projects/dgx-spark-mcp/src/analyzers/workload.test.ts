/**
 * Unit tests for workload analyzer
 */

import { describe, it, expect } from '@jest/globals';
import { classifyWorkload, analyzeWorkloadRequirements } from './workload.js';

describe('Workload Analyzer', () => {
  describe('classifyWorkload', () => {
    it('should classify ML training workloads', async () => {
      const result = await classifyWorkload('Train a deep learning model');

      expect(result.workloadType).toBe('ml-training');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('should classify ETL workloads', async () => {
      const result = await classifyWorkload('Extract and transform customer data');

      expect(result.workloadType).toBe('etl');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('should classify analytics workloads', async () => {
      const result = await classifyWorkload('Analyze sales trends and generate reports');

      expect(result.workloadType).toBe('analytics');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('should classify ML inference workloads', async () => {
      const result = await classifyWorkload('Run batch predictions on new data');

      expect(result.workloadType).toBe('ml-inference');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('should classify streaming workloads', async () => {
      const result = await classifyWorkload('Process real-time event stream');

      expect(result.workloadType).toBe('streaming');
      expect(result.confidence).toBeGreaterThan(0.5);
    });

    it('should include detected characteristics', async () => {
      const result = await classifyWorkload('Train a transformer model on 1TB dataset');

      expect(result.characteristics).toBeDefined();
      expect(result.characteristics.gpuRequired).toBeDefined();
      expect(result.characteristics.memoryIntensive).toBeDefined();
    });

    it('should detect GPU requirements', async () => {
      const result = await classifyWorkload('Train neural network with GPU acceleration');

      expect(result.characteristics.gpuRequired).toBe(true);
    });

    it('should detect distributed requirements', async () => {
      const result = await classifyWorkload('Process petabyte-scale dataset across cluster');

      expect(result.characteristics.distributed).toBe(true);
    });

    it('should handle ambiguous descriptions', async () => {
      const result = await classifyWorkload('Process some data');

      expect(result.workloadType).toBeDefined();
      expect(result.confidence).toBeGreaterThan(0);
      expect(result.confidence).toBeLessThan(1);
    });
  });

  describe('analyzeWorkloadRequirements', () => {
    it('should analyze resource requirements', async () => {
      const result = await analyzeWorkloadRequirements({
        workloadType: 'ml-training',
        dataSize: '100GB',
      });

      expect(result).toHaveProperty('estimatedCores');
      expect(result).toHaveProperty('estimatedMemoryGB');
      expect(result).toHaveProperty('estimatedExecutors');
      expect(result.estimatedCores).toBeGreaterThan(0);
      expect(result.estimatedMemoryGB).toBeGreaterThan(0);
    });

    it('should recommend GPU for ML workloads', async () => {
      const result = await analyzeWorkloadRequirements({
        workloadType: 'ml-training',
        dataSize: '1TB',
      });

      expect(result.recommendGPU).toBe(true);
    });

    it('should not recommend GPU for ETL workloads', async () => {
      const result = await analyzeWorkloadRequirements({
        workloadType: 'etl',
        dataSize: '100GB',
      });

      expect(result.recommendGPU).toBe(false);
    });

    it('should scale resources with data size', async () => {
      const small = await analyzeWorkloadRequirements({
        workloadType: 'analytics',
        dataSize: '10GB',
      });

      const large = await analyzeWorkloadRequirements({
        workloadType: 'analytics',
        dataSize: '1TB',
      });

      expect(large.estimatedCores).toBeGreaterThan(small.estimatedCores);
      expect(large.estimatedMemoryGB).toBeGreaterThan(small.estimatedMemoryGB);
    });

    it('should provide execution time estimates', async () => {
      const result = await analyzeWorkloadRequirements({
        workloadType: 'ml-training',
        dataSize: '100GB',
      });

      expect(result.estimatedDurationMinutes).toBeDefined();
      expect(result.estimatedDurationMinutes).toBeGreaterThan(0);
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty descriptions', async () => {
      const result = await classifyWorkload('');

      expect(result.workloadType).toBeDefined();
      expect(result.confidence).toBeLessThan(0.5);
    });

    it('should handle very long descriptions', async () => {
      const longDesc = 'Train '.repeat(100) + 'a model';
      const result = await classifyWorkload(longDesc);

      expect(result.workloadType).toBe('ml-training');
    });

    it('should handle special characters', async () => {
      const result = await classifyWorkload('ML/DL training @ 100% GPU!');

      expect(result.workloadType).toBe('ml-training');
    });

    it('should handle very small data sizes', async () => {
      const result = await analyzeWorkloadRequirements({
        workloadType: 'analytics',
        dataSize: '1MB',
      });

      expect(result.estimatedCores).toBeGreaterThan(0);
      expect(result.estimatedMemoryGB).toBeGreaterThan(0);
    });

    it('should handle very large data sizes', async () => {
      const result = await analyzeWorkloadRequirements({
        workloadType: 'analytics',
        dataSize: '100PB',
      });

      expect(result.estimatedCores).toBeGreaterThan(0);
      expect(result.estimatedMemoryGB).toBeGreaterThan(0);
    });
  });
});
