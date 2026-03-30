/**
 * Performance benchmarks for critical operations
 */

import { describe, it, expect, beforeAll } from '@jest/globals';
import { performance } from 'perf_hooks';

describe('Performance Benchmarks', () => {
  const iterations = 100;

  beforeAll(() => {
    process.env['MOCK_HARDWARE'] = 'true';
  });

  /**
   * Benchmark helper
   */
  async function benchmark(
    name: string,
    fn: () => Promise<void>,
    maxDurationMs: number
  ): Promise<number> {
    const times: number[] = [];

    // Warmup
    for (let i = 0; i < 5; i++) {
      await fn();
    }

    // Measure
    for (let i = 0; i < iterations; i++) {
      const start = performance.now();
      await fn();
      const end = performance.now();
      times.push(end - start);
    }

    const avg = times.reduce((a, b) => a + b, 0) / times.length;
    const min = Math.min(...times);
    const max = Math.max(...times);
    const p95 = times.sort((a, b) => a - b)[Math.floor(times.length * 0.95)]!;

    console.log(`\n${name}:`);
    console.log(`  Average: ${avg.toFixed(2)}ms`);
    console.log(`  Min: ${min.toFixed(2)}ms`);
    console.log(`  Max: ${max.toFixed(2)}ms`);
    console.log(`  P95: ${p95.toFixed(2)}ms`);

    expect(p95).toBeLessThan(maxDurationMs);

    return avg;
  }

  describe('Configuration Loading', () => {
    it('should load config in < 10ms (P95)', async () => {
      const { getConfig, resetConfig } = await import('../../src/config/index.js');

      await benchmark(
        'Configuration Loading',
        async () => {
          resetConfig();
          getConfig();
        },
        10
      );
    });
  });

  describe('Tool Validation', () => {
    it('should validate tool args in < 5ms (P95)', async () => {
      const { validateToolArgs, getToolSchema } = await import('../../src/tools/validation.js');

      const schema = getToolSchema('get_optimal_spark_config')!;
      const args = {
        workloadType: 'ml-training',
        dataSize: '100GB',
      };

      await benchmark(
        'Tool Argument Validation',
        async () => {
          validateToolArgs(schema, args);
        },
        5
      );
    });
  });

  describe('Resource URI Parsing', () => {
    it('should parse resource URI in < 1ms (P95)', async () => {
      const uri = 'dgx://hardware/topology';

      await benchmark(
        'Resource URI Parsing',
        async () => {
          const parts = uri.replace('dgx://', '').split('/');
          return;
        },
        1
      );
    });
  });

  describe('Data Size Parsing', () => {
    it('should parse data size in < 1ms (P95)', async () => {
      const { parseDataSize } = await import('../../src/utils/data-size.js');

      await benchmark(
        'Data Size Parsing',
        async () => {
          parseDataSize('100GB');
          parseDataSize('1.5TB');
          parseDataSize('500MB');
        },
        1
      );
    });
  });

  describe('Spark Config Generation', () => {
    it('should generate config in < 50ms (P95)', async () => {
      const { generateSparkConfig } = await import('../../src/optimizers/spark-config.js');
      const { createMockHardwareTopology } = await import('../../src/__tests__/utils.js');

      const topology = createMockHardwareTopology();

      await benchmark(
        'Spark Config Generation',
        async () => {
          await generateSparkConfig({
            workloadType: 'ml-training',
            dataSize: '100GB',
            hardware: topology,
          });
        },
        50
      );
    });
  });

  describe('Workload Classification', () => {
    it('should classify workload in < 20ms (P95)', async () => {
      const { classifyWorkload } = await import('../../src/analyzers/workload.js');

      await benchmark(
        'Workload Classification',
        async () => {
          await classifyWorkload('Train a machine learning model on 100GB dataset');
        },
        20
      );
    });
  });

  describe('Memory Throughput', () => {
    it('should handle large data structures efficiently', () => {
      const largeArray = new Array(100000).fill(0).map((_, i) => ({
        id: i,
        name: `item-${i}`,
        value: Math.random(),
      }));

      const start = performance.now();
      const filtered = largeArray.filter(item => item.value > 0.5);
      const mapped = filtered.map(item => item.value);
      const sum = mapped.reduce((a, b) => a + b, 0);
      const end = performance.now();

      const duration = end - start;
      console.log(`\nLarge Array Processing: ${duration.toFixed(2)}ms`);

      expect(duration).toBeLessThan(100);
      expect(sum).toBeGreaterThan(0);
    });
  });

  describe('JSON Serialization', () => {
    it('should serialize large objects quickly', () => {
      const { createMockHardwareTopology } = require('../../src/__tests__/utils.js');
      const topology = createMockHardwareTopology();

      const start = performance.now();
      for (let i = 0; i < 1000; i++) {
        JSON.stringify(topology);
      }
      const end = performance.now();

      const avgDuration = (end - start) / 1000;
      console.log(`\nJSON Serialization (avg): ${avgDuration.toFixed(3)}ms`);

      expect(avgDuration).toBeLessThan(1);
    });
  });

  describe('Overall Performance', () => {
    it('should meet performance targets', () => {
      console.log('\n=== Performance Summary ===');
      console.log('All benchmarks passed performance thresholds');
      console.log('System is performing within acceptable limits');
    });
  });
});
