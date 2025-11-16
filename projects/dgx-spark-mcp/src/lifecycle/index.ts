import type { Logger } from '../logger/index.js';

/**
 * Server lifecycle manager
 * Handles graceful startup and shutdown
 */

export interface LifecycleHook {
  name: string;
  handler: () => Promise<void>;
}

export class LifecycleManager {
  private logger: Logger;
  private startupHooks: LifecycleHook[] = [];
  private shutdownHooks: LifecycleHook[] = [];
  private isShuttingDown = false;
  private shutdownTimeout: number;

  constructor(logger: Logger, shutdownTimeout = 30000) {
    this.logger = logger;
    this.shutdownTimeout = shutdownTimeout;
    this.setupSignalHandlers();
  }

  /**
   * Register a startup hook
   */
  onStartup(name: string, handler: () => Promise<void>): void {
    this.startupHooks.push({ name, handler });
  }

  /**
   * Register a shutdown hook
   */
  onShutdown(name: string, handler: () => Promise<void>): void {
    this.shutdownHooks.push({ name, handler });
  }

  /**
   * Execute all startup hooks
   */
  async startup(): Promise<void> {
    this.logger.info('Starting server lifecycle...');

    for (const hook of this.startupHooks) {
      try {
        this.logger.debug(`Executing startup hook: ${hook.name}`);
        await hook.handler();
        this.logger.debug(`Startup hook completed: ${hook.name}`);
      } catch (error) {
        this.logger.error(`Startup hook failed: ${hook.name}`, error as Error);
        throw error;
      }
    }

    this.logger.info('Server startup complete');
  }

  /**
   * Execute all shutdown hooks
   */
  async shutdown(signal?: string): Promise<void> {
    if (this.isShuttingDown) {
      this.logger.warn('Shutdown already in progress');
      return;
    }

    this.isShuttingDown = true;
    this.logger.info(`Initiating graceful shutdown${signal ? ` (signal: ${signal})` : ''}`);

    // Create a timeout promise
    const timeoutPromise = new Promise<void>((_, reject) => {
      setTimeout(() => {
        reject(new Error(`Shutdown timeout after ${this.shutdownTimeout}ms`));
      }, this.shutdownTimeout);
    });

    try {
      // Execute all shutdown hooks with timeout
      await Promise.race([this.executeShutdownHooks(), timeoutPromise]);

      this.logger.info('Graceful shutdown complete');
      process.exit(0);
    } catch (error) {
      this.logger.error('Error during shutdown', error as Error);
      process.exit(1);
    }
  }

  /**
   * Execute all shutdown hooks in reverse order
   */
  private async executeShutdownHooks(): Promise<void> {
    // Execute in reverse order (LIFO)
    const hooks = [...this.shutdownHooks].reverse();

    for (const hook of hooks) {
      try {
        this.logger.debug(`Executing shutdown hook: ${hook.name}`);
        await hook.handler();
        this.logger.debug(`Shutdown hook completed: ${hook.name}`);
      } catch (error) {
        this.logger.error(`Shutdown hook failed: ${hook.name}`, error as Error);
        // Continue with other hooks even if one fails
      }
    }
  }

  /**
   * Setup signal handlers for graceful shutdown
   */
  private setupSignalHandlers(): void {
    const signals: NodeJS.Signals[] = ['SIGTERM', 'SIGINT'];

    signals.forEach((signal) => {
      process.on(signal, () => {
        void this.shutdown(signal);
      });
    });

    // Handle uncaught exceptions
    process.on('uncaughtException', (error: Error) => {
      this.logger.error('Uncaught exception', error);
      void this.shutdown('uncaughtException');
    });

    // Handle unhandled promise rejections
    process.on('unhandledRejection', (reason: unknown) => {
      this.logger.error('Unhandled rejection', reason as Error);
      void this.shutdown('unhandledRejection');
    });
  }

  /**
   * Check if shutdown is in progress
   */
  isShuttingDownStatus(): boolean {
    return this.isShuttingDown;
  }
}
