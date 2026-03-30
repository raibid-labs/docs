import type { Logger } from '../logger/index.js';

/**
 * Health check system
 * Monitors server health and provides health check endpoints
 */

export enum HealthStatus {
  HEALTHY = 'healthy',
  DEGRADED = 'degraded',
  UNHEALTHY = 'unhealthy',
}

export interface HealthCheck {
  name: string;
  checker: () => Promise<HealthCheckResult>;
  critical: boolean; // If true, failure marks entire system as unhealthy
}

export interface HealthCheckResult {
  status: HealthStatus;
  message?: string;
  metadata?: Record<string, unknown>;
}

export interface SystemHealth {
  status: HealthStatus;
  timestamp: number;
  uptime: number;
  checks: Record<string, HealthCheckResult>;
}

export class HealthManager {
  private logger: Logger;
  private checks: Map<string, HealthCheck> = new Map();
  private startTime: number;
  private checkInterval?: NodeJS.Timeout;

  constructor(logger: Logger) {
    this.logger = logger;
    this.startTime = Date.now();
  }

  /**
   * Register a health check
   */
  registerCheck(name: string, checker: () => Promise<HealthCheckResult>, critical = false): void {
    this.checks.set(name, { name, checker, critical });
    this.logger.debug(`Registered health check: ${name} (critical: ${critical})`);
  }

  /**
   * Execute all health checks
   */
  async check(): Promise<SystemHealth> {
    const results: Record<string, HealthCheckResult> = {};
    let overallStatus = HealthStatus.HEALTHY;

    for (const [name, check] of this.checks) {
      try {
        const result = await check.checker();
        results[name] = result;

        // Update overall status based on check result
        if (check.critical && result.status === HealthStatus.UNHEALTHY) {
          overallStatus = HealthStatus.UNHEALTHY;
        } else if (result.status === HealthStatus.DEGRADED && overallStatus === HealthStatus.HEALTHY) {
          overallStatus = HealthStatus.DEGRADED;
        }
      } catch (error) {
        const errorResult: HealthCheckResult = {
          status: HealthStatus.UNHEALTHY,
          message: (error as Error).message,
        };
        results[name] = errorResult;

        if (check.critical) {
          overallStatus = HealthStatus.UNHEALTHY;
        }

        this.logger.error(`Health check failed: ${name}`, error as Error);
      }
    }

    return {
      status: overallStatus,
      timestamp: Date.now(),
      uptime: Date.now() - this.startTime,
      checks: results,
    };
  }

  /**
   * Start periodic health checks
   */
  startPeriodicChecks(interval: number): void {
    if (this.checkInterval) {
      this.logger.warn('Periodic health checks already running');
      return;
    }

    this.logger.info(`Starting periodic health checks (interval: ${interval}ms)`);

    this.checkInterval = setInterval(() => {
      void this.check().then((health) => {
        if (health.status !== HealthStatus.HEALTHY) {
          this.logger.warn('System health degraded', { health });
        }
      });
    }, interval);
  }

  /**
   * Stop periodic health checks
   */
  stopPeriodicChecks(): void {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = undefined;
      this.logger.info('Stopped periodic health checks');
    }
  }

  /**
   * Get simple ready status (for k8s readiness probes)
   */
  async isReady(): Promise<boolean> {
    const health = await this.check();
    return health.status !== HealthStatus.UNHEALTHY;
  }

  /**
   * Get simple alive status (for k8s liveness probes)
   */
  isAlive(): boolean {
    return true; // If we can execute this, we're alive
  }
}
