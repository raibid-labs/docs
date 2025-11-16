/**
 * Jest setup file - runs before all tests
 */

// Set test environment variables
process.env['NODE_ENV'] = 'test';
process.env['LOG_LEVEL'] = 'error'; // Reduce noise in test output

// Mock hardware detection by default in tests
process.env['MOCK_HARDWARE'] = 'true';

// Extend Jest matchers if needed
expect.extend({
  toBeValidSparkConfig(received: any) {
    const hasRequiredFields =
      received &&
      typeof received === 'object' &&
      'spark.executor.memory' in received;

    return {
      pass: hasRequiredFields,
      message: () =>
        hasRequiredFields
          ? `Expected ${JSON.stringify(received)} not to be a valid Spark config`
          : `Expected ${JSON.stringify(received)} to be a valid Spark config`,
    };
  },
});

declare global {
  namespace jest {
    interface Matchers<R> {
      toBeValidSparkConfig(): R;
    }
  }
}
