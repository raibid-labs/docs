import winston from 'winston';
import { existsSync, mkdirSync } from 'fs';
import { join } from 'path';
import type { Config } from '../config/schema.js';
import { DGXError } from '../errors/index.js';

/**
 * Logger module using Winston
 * Provides structured logging with different formats and transports
 */

export class Logger {
  private logger: winston.Logger;
  private config: Config['logging'];

  constructor(config: Config['logging']) {
    this.config = config;
    this.ensureLogDirectory();
    this.logger = this.createLogger();
  }

  /**
   * Ensure log directory exists
   */
  private ensureLogDirectory(): void {
    if (!existsSync(this.config.dir)) {
      mkdirSync(this.config.dir, { recursive: true });
    }
  }

  /**
   * Create Winston logger instance
   */
  private createLogger(): winston.Logger {
    const formats = this.getFormats();
    const transports = this.getTransports();

    return winston.createLogger({
      level: this.config.level,
      format: formats,
      transports,
      exitOnError: false,
    });
  }

  /**
   * Get log formats based on configuration
   */
  private getFormats(): winston.Logform.Format {
    const timestamp = winston.format.timestamp({
      format: 'YYYY-MM-DD HH:mm:ss.SSS',
    });

    const errors = winston.format.errors({ stack: true });

    switch (this.config.format) {
      case 'json':
        return winston.format.combine(timestamp, errors, winston.format.json());

      case 'pretty':
        return winston.format.combine(
          timestamp,
          errors,
          winston.format.colorize(),
          winston.format.printf(({ timestamp, level, message, ...metadata }) => {
            let msg = `${timestamp} [${level}]: ${message}`;
            if (Object.keys(metadata).length > 0) {
              msg += ` ${JSON.stringify(metadata, null, 2)}`;
            }
            return msg;
          })
        );

      case 'simple':
      default:
        return winston.format.combine(
          timestamp,
          errors,
          winston.format.printf(({ timestamp, level, message }) => {
            return `${timestamp} [${level}]: ${message}`;
          })
        );
    }
  }

  /**
   * Get transports based on configuration
   */
  private getTransports(): winston.transport[] {
    const transports: winston.transport[] = [];

    // Console transport
    transports.push(
      new winston.transports.Console({
        format:
          this.config.format === 'json'
            ? winston.format.json()
            : winston.format.combine(winston.format.colorize(), this.getFormats()),
      })
    );

    // File transport for all logs
    transports.push(
      new winston.transports.File({
        filename: join(this.config.dir, 'dgx-mcp-combined.log'),
        maxsize: this.parseSize(this.config.maxSize),
        maxFiles: this.config.maxFiles,
      })
    );

    // File transport for errors only
    transports.push(
      new winston.transports.File({
        filename: join(this.config.dir, 'dgx-mcp-error.log'),
        level: 'error',
        maxsize: this.parseSize(this.config.maxSize),
        maxFiles: this.config.maxFiles,
      })
    );

    return transports;
  }

  /**
   * Parse size string to bytes
   */
  private parseSize(size: string): number {
    const units: Record<string, number> = {
      b: 1,
      k: 1024,
      m: 1024 * 1024,
      g: 1024 * 1024 * 1024,
    };

    const match = size.toLowerCase().match(/^(\d+)([bkmg])$/);
    if (!match) {
      return 10 * 1024 * 1024; // Default 10MB
    }

    const [, num, unit] = match;
    if (!num || !unit) {
      return 10 * 1024 * 1024;
    }

    return parseInt(num, 10) * (units[unit] || 1);
  }

  /**
   * Log debug message
   */
  debug(message: string, metadata?: object): void {
    this.logger.debug(message, metadata);
  }

  /**
   * Log info message
   */
  info(message: string, metadata?: object): void {
    this.logger.info(message, metadata);
  }

  /**
   * Log warning message
   */
  warn(message: string, metadata?: object): void {
    this.logger.warn(message, metadata);
  }

  /**
   * Log error message
   */
  error(message: string, error?: Error | DGXError, metadata?: object): void {
    const errorMetadata = error
      ? {
          error: {
            message: error.message,
            stack: error.stack,
            ...(error instanceof DGXError ? { code: error.code, severity: error.severity } : {}),
          },
          ...metadata,
        }
      : metadata;

    this.logger.error(message, errorMetadata);
  }

  /**
   * Create child logger with additional metadata
   */
  child(metadata: object): Logger {
    const childLogger = new Logger(this.config);
    childLogger.logger = this.logger.child(metadata);
    return childLogger;
  }
}

// Singleton instance
let loggerInstance: Logger | null = null;

/**
 * Get logger singleton
 */
export function getLogger(config?: Config['logging']): Logger {
  if (!loggerInstance && config) {
    loggerInstance = new Logger(config);
  } else if (!loggerInstance) {
    throw new Error('Logger not initialized. Call getLogger with config first.');
  }
  return loggerInstance;
}

/**
 * Reset logger (useful for testing)
 */
export function resetLogger(): void {
  loggerInstance = null;
}
