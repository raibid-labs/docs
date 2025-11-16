/**
 * Prometheus Metrics Module
 * Exports metrics in Prometheus format for monitoring and observability
 */

export interface MetricValue {
  value: number;
  timestamp: number;
  labels?: Record<string, string>;
}

export interface Counter {
  name: string;
  help: string;
  value: number;
  labels: Record<string, string>;
}

export interface Gauge {
  name: string;
  help: string;
  value: number;
  labels: Record<string, string>;
}

export interface Histogram {
  name: string;
  help: string;
  sum: number;
  count: number;
  buckets: Map<number, number>;
  labels: Record<string, string>;
}

/**
 * Metrics Registry
 * Central registry for all application metrics
 */
export class MetricsRegistry {
  private counters: Map<string, Counter> = new Map();
  private gauges: Map<string, Gauge> = new Map();
  private histograms: Map<string, Histogram> = new Map();
  private startTime: number = Date.now();

  /**
   * Increment a counter metric
   */
  incrementCounter(name: string, help: string, value = 1, labels: Record<string, string> = {}): void {
    const key = this.getMetricKey(name, labels);
    const existing = this.counters.get(key);

    if (existing) {
      existing.value += value;
    } else {
      this.counters.set(key, { name, help, value, labels });
    }
  }

  /**
   * Set a gauge metric
   */
  setGauge(name: string, help: string, value: number, labels: Record<string, string> = {}): void {
    const key = this.getMetricKey(name, labels);
    this.gauges.set(key, { name, help, value, labels });
  }

  /**
   * Observe a value in a histogram
   */
  observeHistogram(
    name: string,
    help: string,
    value: number,
    buckets: number[] = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
    labels: Record<string, string> = {}
  ): void {
    const key = this.getMetricKey(name, labels);
    let histogram = this.histograms.get(key);

    if (!histogram) {
      histogram = {
        name,
        help,
        sum: 0,
        count: 0,
        buckets: new Map(buckets.map((b) => [b, 0])),
        labels,
      };
      this.histograms.set(key, histogram);
    }

    histogram.sum += value;
    histogram.count += 1;

    // Update buckets
    for (const [bucket, _] of histogram.buckets) {
      if (value <= bucket) {
        histogram.buckets.set(bucket, (histogram.buckets.get(bucket) || 0) + 1);
      }
    }
  }

  /**
   * Generate metric key from name and labels
   */
  private getMetricKey(name: string, labels: Record<string, string>): string {
    const labelPairs = Object.entries(labels)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([k, v]) => `${k}="${v}"`)
      .join(',');

    return labelPairs ? `${name}{${labelPairs}}` : name;
  }

  /**
   * Format labels for Prometheus
   */
  private formatLabels(labels: Record<string, string>): string {
    if (Object.keys(labels).length === 0) return '';

    const labelPairs = Object.entries(labels)
      .map(([k, v]) => `${k}="${v}"`)
      .join(',');

    return `{${labelPairs}}`;
  }

  /**
   * Export metrics in Prometheus text format
   */
  export(): string {
    const lines: string[] = [];

    // Process info metric
    lines.push('# HELP dgx_mcp_build_info Build information');
    lines.push('# TYPE dgx_mcp_build_info gauge');
    lines.push(`dgx_mcp_build_info{version="0.1.0"} 1`);
    lines.push('');

    // Process uptime
    const uptime = (Date.now() - this.startTime) / 1000;
    lines.push('# HELP dgx_mcp_uptime_seconds Uptime in seconds');
    lines.push('# TYPE dgx_mcp_uptime_seconds counter');
    lines.push(`dgx_mcp_uptime_seconds ${uptime.toFixed(2)}`);
    lines.push('');

    // Export counters
    const counterGroups = new Map<string, Counter[]>();
    for (const counter of this.counters.values()) {
      if (!counterGroups.has(counter.name)) {
        counterGroups.set(counter.name, []);
      }
      counterGroups.get(counter.name)?.push(counter);
    }

    for (const [name, counters] of counterGroups) {
      lines.push(`# HELP ${name} ${counters[0]?.help || ''}`);
      lines.push(`# TYPE ${name} counter`);
      for (const counter of counters) {
        lines.push(`${counter.name}${this.formatLabels(counter.labels)} ${counter.value}`);
      }
      lines.push('');
    }

    // Export gauges
    const gaugeGroups = new Map<string, Gauge[]>();
    for (const gauge of this.gauges.values()) {
      if (!gaugeGroups.has(gauge.name)) {
        gaugeGroups.set(gauge.name, []);
      }
      gaugeGroups.get(gauge.name)?.push(gauge);
    }

    for (const [name, gauges] of gaugeGroups) {
      lines.push(`# HELP ${name} ${gauges[0]?.help || ''}`);
      lines.push(`# TYPE ${name} gauge`);
      for (const gauge of gauges) {
        lines.push(`${gauge.name}${this.formatLabels(gauge.labels)} ${gauge.value}`);
      }
      lines.push('');
    }

    // Export histograms
    const histogramGroups = new Map<string, Histogram[]>();
    for (const histogram of this.histograms.values()) {
      if (!histogramGroups.has(histogram.name)) {
        histogramGroups.set(histogram.name, []);
      }
      histogramGroups.get(histogram.name)?.push(histogram);
    }

    for (const [name, histograms] of histogramGroups) {
      lines.push(`# HELP ${name} ${histograms[0]?.help || ''}`);
      lines.push(`# TYPE ${name} histogram`);

      for (const histogram of histograms) {
        const labelStr = this.formatLabels(histogram.labels);

        // Export buckets
        for (const [bucket, count] of histogram.buckets) {
          const bucketLabels = { ...histogram.labels, le: bucket.toString() };
          lines.push(`${name}_bucket${this.formatLabels(bucketLabels)} ${count}`);
        }

        // Export +Inf bucket
        const infLabels = { ...histogram.labels, le: '+Inf' };
        lines.push(`${name}_bucket${this.formatLabels(infLabels)} ${histogram.count}`);

        // Export sum and count
        lines.push(`${name}_sum${labelStr} ${histogram.sum}`);
        lines.push(`${name}_count${labelStr} ${histogram.count}`);
      }
      lines.push('');
    }

    return lines.join('\n');
  }

  /**
   * Reset all metrics
   */
  reset(): void {
    this.counters.clear();
    this.gauges.clear();
    this.histograms.clear();
  }
}

// Singleton instance
let metricsRegistry: MetricsRegistry | null = null;

/**
 * Get metrics registry singleton
 */
export function getMetricsRegistry(): MetricsRegistry {
  if (!metricsRegistry) {
    metricsRegistry = new MetricsRegistry();
  }
  return metricsRegistry;
}

/**
 * Application-specific metrics helpers
 */
export class DGXMetrics {
  private registry: MetricsRegistry;

  constructor() {
    this.registry = getMetricsRegistry();
  }

  /**
   * Record MCP request
   */
  recordRequest(method: string, status: 'success' | 'error'): void {
    this.registry.incrementCounter(
      'dgx_mcp_requests_total',
      'Total number of MCP requests',
      1,
      { method, status }
    );
  }

  /**
   * Record request duration
   */
  recordRequestDuration(method: string, durationMs: number): void {
    this.registry.observeHistogram(
      'dgx_mcp_request_duration_seconds',
      'MCP request duration in seconds',
      durationMs / 1000,
      undefined,
      { method }
    );
  }

  /**
   * Record tool execution
   */
  recordToolExecution(toolName: string, status: 'success' | 'error', durationMs: number): void {
    this.registry.incrementCounter(
      'dgx_mcp_tool_executions_total',
      'Total number of tool executions',
      1,
      { tool: toolName, status }
    );

    this.registry.observeHistogram(
      'dgx_mcp_tool_duration_seconds',
      'Tool execution duration in seconds',
      durationMs / 1000,
      undefined,
      { tool: toolName }
    );
  }

  /**
   * Record resource read
   */
  recordResourceRead(resourceType: string, status: 'success' | 'error'): void {
    this.registry.incrementCounter(
      'dgx_mcp_resource_reads_total',
      'Total number of resource reads',
      1,
      { type: resourceType, status }
    );
  }

  /**
   * Set GPU metrics
   */
  setGPUMetrics(gpuIndex: number, metrics: {
    temperature?: number;
    utilization?: number;
    memoryUsed?: number;
    memoryTotal?: number;
    powerUsage?: number;
  }): void {
    const labels = { gpu: gpuIndex.toString() };

    if (metrics.temperature !== undefined) {
      this.registry.setGauge(
        'dgx_gpu_temperature_celsius',
        'GPU temperature in Celsius',
        metrics.temperature,
        labels
      );
    }

    if (metrics.utilization !== undefined) {
      this.registry.setGauge(
        'dgx_gpu_utilization_percent',
        'GPU utilization percentage',
        metrics.utilization,
        labels
      );
    }

    if (metrics.memoryUsed !== undefined) {
      this.registry.setGauge(
        'dgx_gpu_memory_used_bytes',
        'GPU memory used in bytes',
        metrics.memoryUsed,
        labels
      );
    }

    if (metrics.memoryTotal !== undefined) {
      this.registry.setGauge(
        'dgx_gpu_memory_total_bytes',
        'GPU memory total in bytes',
        metrics.memoryTotal,
        labels
      );
    }

    if (metrics.powerUsage !== undefined) {
      this.registry.setGauge(
        'dgx_gpu_power_usage_watts',
        'GPU power usage in watts',
        metrics.powerUsage,
        labels
      );
    }
  }

  /**
   * Record error
   */
  recordError(type: string, severity: string): void {
    this.registry.incrementCounter(
      'dgx_mcp_errors_total',
      'Total number of errors',
      1,
      { type, severity }
    );
  }

  /**
   * Export all metrics
   */
  export(): string {
    return this.registry.export();
  }
}
