/**
 * Telemetry Module
 * Collects performance metrics and system telemetry data
 */

import type { Logger } from '../logger/index.js';
import { DGXMetrics } from './metrics.js';

export interface PerformanceMetrics {
  requestCount: number;
  errorCount: number;
  averageResponseTime: number;
  peakMemoryUsage: number;
  uptime: number;
}

export interface SystemMetrics {
  cpuUsage: number;
  memoryUsage: number;
  memoryTotal: number;
  processUptime: number;
  nodeVersion: string;
}

/**
 * Telemetry Collector
 * Collects and tracks system and application telemetry
 */
export class TelemetryCollector {
  private logger: Logger;
  private metrics: DGXMetrics;
  private startTime: number;
  private requestTimes: number[] = [];
  private maxRequestTimesStored = 1000;

  constructor(logger: Logger) {
    this.logger = logger;
    this.metrics = new DGXMetrics();
    this.startTime = Date.now();
  }

  /**
   * Record a request start
   */
  startRequest(method: string): () => void {
    const startTime = Date.now();

    return () => {
      const duration = Date.now() - startTime;
      this.recordRequest(method, duration, 'success');
    };
  }

  /**
   * Record a completed request
   */
  recordRequest(method: string, durationMs: number, status: 'success' | 'error' = 'success'): void {
    // Record in metrics
    this.metrics.recordRequest(method, status);
    this.metrics.recordRequestDuration(method, durationMs);

    // Store for statistics
    this.requestTimes.push(durationMs);
    if (this.requestTimes.length > this.maxRequestTimesStored) {
      this.requestTimes.shift();
    }

    // Log slow requests
    if (durationMs > 1000) {
      this.logger.warn(`Slow request detected: ${method} took ${durationMs}ms`, {
        method,
        duration: durationMs,
      });
    }
  }

  /**
   * Record a tool execution
   */
  recordToolExecution(toolName: string, durationMs: number, status: 'success' | 'error' = 'success'): void {
    this.metrics.recordToolExecution(toolName, status, durationMs);

    this.logger.debug(`Tool execution: ${toolName}`, {
      tool: toolName,
      duration: durationMs,
      status,
    });
  }

  /**
   * Record a resource read
   */
  recordResourceRead(resourceType: string, status: 'success' | 'error' = 'success'): void {
    this.metrics.recordResourceRead(resourceType, status);
  }

  /**
   * Record an error
   */
  recordError(type: string, severity: string, error: Error): void {
    this.metrics.recordError(type, severity);

    this.logger.error(`Error recorded: ${type}`, error, {
      type,
      severity,
    });
  }

  /**
   * Get performance metrics
   */
  getPerformanceMetrics(): PerformanceMetrics {
    const requestCount = this.requestTimes.length;
    const averageResponseTime =
      requestCount > 0 ? this.requestTimes.reduce((a, b) => a + b, 0) / requestCount : 0;

    return {
      requestCount,
      errorCount: 0, // Would need to track errors separately
      averageResponseTime,
      peakMemoryUsage: process.memoryUsage().heapUsed,
      uptime: Date.now() - this.startTime,
    };
  }

  /**
   * Get system metrics
   */
  getSystemMetrics(): SystemMetrics {
    const memUsage = process.memoryUsage();
    const uptime = process.uptime();

    return {
      cpuUsage: process.cpuUsage().user / 1000000, // Convert to seconds
      memoryUsage: memUsage.heapUsed,
      memoryTotal: memUsage.heapTotal,
      processUptime: uptime,
      nodeVersion: process.version,
    };
  }

  /**
   * Collect and update GPU metrics
   */
  async collectGPUMetrics(gpuData: {
    index: number;
    temperature?: number;
    utilization?: number;
    memoryUsed?: number;
    memoryTotal?: number;
    powerUsage?: number;
  }): Promise<void> {
    this.metrics.setGPUMetrics(gpuData.index, {
      temperature: gpuData.temperature,
      utilization: gpuData.utilization,
      memoryUsed: gpuData.memoryUsed,
      memoryTotal: gpuData.memoryTotal,
      powerUsage: gpuData.powerUsage,
    });
  }

  /**
   * Get metrics in Prometheus format
   */
  getPrometheusMetrics(): string {
    return this.metrics.export();
  }

  /**
   * Generate telemetry report
   */
  generateReport(): string {
    const perfMetrics = this.getPerformanceMetrics();
    const sysMetrics = this.getSystemMetrics();

    return JSON.stringify(
      {
        timestamp: new Date().toISOString(),
        performance: {
          requestCount: perfMetrics.requestCount,
          errorCount: perfMetrics.errorCount,
          averageResponseTime: perfMetrics.averageResponseTime.toFixed(2) + 'ms',
          peakMemoryUsage: (perfMetrics.peakMemoryUsage / 1024 / 1024).toFixed(2) + 'MB',
          uptime: (perfMetrics.uptime / 1000).toFixed(2) + 's',
        },
        system: {
          cpuUsage: sysMetrics.cpuUsage.toFixed(2) + 's',
          memoryUsage: (sysMetrics.memoryUsage / 1024 / 1024).toFixed(2) + 'MB',
          memoryTotal: (sysMetrics.memoryTotal / 1024 / 1024).toFixed(2) + 'MB',
          processUptime: sysMetrics.processUptime.toFixed(2) + 's',
          nodeVersion: sysMetrics.nodeVersion,
        },
      },
      null,
      2
    );
  }

  /**
   * Start periodic metric collection
   */
  startPeriodicCollection(intervalMs = 60000): NodeJS.Timeout {
    this.logger.info(`Starting periodic telemetry collection (interval: ${intervalMs}ms)`);

    return setInterval(() => {
      const report = this.generateReport();
      this.logger.debug('Telemetry report', { report });
    }, intervalMs);
  }
}

/**
 * Request Timer
 * Helper class for timing requests
 */
export class RequestTimer {
  private startTime: number;
  private method: string;
  private telemetry: TelemetryCollector;

  constructor(method: string, telemetry: TelemetryCollector) {
    this.method = method;
    this.telemetry = telemetry;
    this.startTime = Date.now();
  }

  /**
   * End the timer and record the request
   */
  end(status: 'success' | 'error' = 'success'): void {
    const duration = Date.now() - this.startTime;
    this.telemetry.recordRequest(this.method, duration, status);
  }

  /**
   * Get elapsed time
   */
  elapsed(): number {
    return Date.now() - this.startTime;
  }
}

/**
 * Create a timer for measuring operation duration
 */
export function createTimer(): { elapsed: () => number } {
  const start = Date.now();

  return {
    elapsed: () => Date.now() - start,
  };
}

/**
 * Measure async function execution time
 */
export async function measureAsync<T>(
  fn: () => Promise<T>,
  onComplete?: (duration: number) => void
): Promise<T> {
  const start = Date.now();

  try {
    const result = await fn();
    const duration = Date.now() - start;

    if (onComplete) {
      onComplete(duration);
    }

    return result;
  } catch (error) {
    const duration = Date.now() - start;

    if (onComplete) {
      onComplete(duration);
    }

    throw error;
  }
}

/**
 * Measure sync function execution time
 */
export function measureSync<T>(fn: () => T, onComplete?: (duration: number) => void): T {
  const start = Date.now();

  try {
    const result = fn();
    const duration = Date.now() - start;

    if (onComplete) {
      onComplete(duration);
    }

    return result;
  } catch (error) {
    const duration = Date.now() - start;

    if (onComplete) {
      onComplete(duration);
    }

    throw error;
  }
}
