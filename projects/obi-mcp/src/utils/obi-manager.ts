/**
 * OBI Process Manager
 * Handles lifecycle management of OBI processes
 */

import { spawn, ChildProcess } from 'child_process';
import { promises as fs } from 'fs';
import { join } from 'path';
import YAML from 'yaml';
import logger from './logger.js';
import { findProcessByName, getProcessInfo, isProcessRunning, killProcess } from './process.js';
import {
  ObiConfig,
  ObiConfigSchema,
  ObiControlResult,
  ObiDeploymentOptions,
  ObiHealthCheck,
  ObiStatus,
} from '../types/obi.js';

/**
 * Singleton class to manage OBI process
 */
export class ObiManager {
  private static instance: ObiManager;
  private process?: ChildProcess;
  private pid?: number;
  private configPath?: string;
  private logPath?: string;
  private startTime?: Date;

  private constructor() {}

  static getInstance(): ObiManager {
    if (!ObiManager.instance) {
      ObiManager.instance = new ObiManager();
    }
    return ObiManager.instance;
  }

  /**
   * Deploy OBI in standalone mode
   */
  async deployLocal(options: ObiDeploymentOptions): Promise<ObiControlResult> {
    try {
      // Check if already running
      const currentStatus = await this.getStatus();
      if (currentStatus.status === ObiStatus.RUNNING) {
        return {
          success: false,
          message: 'OBI is already running',
          data: currentStatus,
          error: 'Process already exists',
        };
      }

      // Prepare config
      let config: ObiConfig;
      if (options.config) {
        config = ObiConfigSchema.parse(options.config);
      } else if (options.configPath) {
        const configContent = await fs.readFile(options.configPath, 'utf-8');
        config = ObiConfigSchema.parse(YAML.parse(configContent));
      } else {
        // Use default config
        config = { network: { enable: true } };
      }

      // Write config to temporary file
      this.configPath = options.configPath || join(process.cwd(), '.obi', 'obi-config.yml');
      await fs.mkdir(join(process.cwd(), '.obi'), { recursive: true });
      await fs.writeFile(this.configPath, YAML.stringify(config));

      // Set log path
      this.logPath = options.logPath || join(process.cwd(), '.obi', 'obi.log');

      // Determine OBI binary path
      const binaryPath = options.binaryPath || 'obi'; // assume in PATH

      logger.info(`Starting OBI with config: ${this.configPath}`);

      // Spawn OBI process
      const obiProcess = spawn(binaryPath, [], {
        env: {
          ...process.env,
          OBI_CONFIG_PATH: this.configPath,
          OBI_NETWORK_METRICS: 'true',
        },
        detached: false,
      });

      // Handle process output
      const logStream = await fs.open(this.logPath, 'a');
      obiProcess.stdout?.on('data', async (data) => {
        const logData = `[stdout] ${data}`;
        logger.debug(logData);
        await logStream.write(logData);
      });

      obiProcess.stderr?.on('data', async (data) => {
        const logData = `[stderr] ${data}`;
        logger.warn(logData);
        await logStream.write(logData);
      });

      obiProcess.on('error', (error) => {
        logger.error(`OBI process error: ${error.message}`);
      });

      obiProcess.on('exit', (code, signal) => {
        logger.info(`OBI process exited with code ${code} and signal ${signal}`);
        this.process = undefined;
        this.pid = undefined;
      });

      // Store process info
      this.process = obiProcess;
      this.pid = obiProcess.pid;
      this.startTime = new Date();

      // Wait a bit to ensure it started successfully
      await new Promise((resolve) => setTimeout(resolve, 2000));

      // Verify it's still running
      if (this.pid && (await isProcessRunning(this.pid))) {
        logger.info(`OBI started successfully with PID ${this.pid}`);
        return {
          success: true,
          message: `OBI deployed successfully with PID ${this.pid}`,
          data: { pid: this.pid, configPath: this.configPath, logPath: this.logPath },
        };
      } else {
        return {
          success: false,
          message: 'OBI process failed to start',
          error: 'Process terminated shortly after start',
        };
      }
    } catch (error) {
      logger.error(`Failed to deploy OBI: ${error}`);
      return {
        success: false,
        message: 'Failed to deploy OBI',
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Get OBI process status
   */
  async getStatus(verbose = false): Promise<ObiHealthCheck> {
    try {
      // First check our tracked PID
      if (this.pid && (await isProcessRunning(this.pid))) {
        const processInfo = await getProcessInfo(this.pid);

        return {
          status: ObiStatus.RUNNING,
          pid: this.pid,
          uptime: processInfo?.uptime,
          cpuUsage: processInfo?.cpuUsage,
          memoryUsage: processInfo?.memoryUsage,
          configPath: this.configPath,
        };
      }

      // If not tracked, try to find by name
      const pids = await findProcessByName('obi');
      if (pids.length > 0) {
        const pid = pids[0]; // Take the first one
        const processInfo = await getProcessInfo(pid);

        // Update tracked info
        this.pid = pid;

        return {
          status: ObiStatus.RUNNING,
          pid,
          uptime: processInfo?.uptime,
          cpuUsage: processInfo?.cpuUsage,
          memoryUsage: processInfo?.memoryUsage,
        };
      }

      // Not running
      return {
        status: ObiStatus.STOPPED,
      };
    } catch (error) {
      logger.error(`Failed to get OBI status: ${error}`);
      return {
        status: ObiStatus.ERROR,
        lastError: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Stop OBI process
   */
  async stop(): Promise<ObiControlResult> {
    try {
      const status = await this.getStatus();

      if (status.status !== ObiStatus.RUNNING || !status.pid) {
        return {
          success: false,
          message: 'OBI is not running',
        };
      }

      logger.info(`Stopping OBI process ${status.pid}`);

      // Send SIGTERM
      await killProcess(status.pid, 'SIGTERM');

      // Wait for graceful shutdown
      await new Promise((resolve) => setTimeout(resolve, 2000));

      // Check if still running
      if (await isProcessRunning(status.pid)) {
        logger.warn('OBI did not stop gracefully, sending SIGKILL');
        await killProcess(status.pid, 'SIGKILL');
      }

      // Clear tracked info
      this.process = undefined;
      this.pid = undefined;

      return {
        success: true,
        message: 'OBI stopped successfully',
      };
    } catch (error) {
      logger.error(`Failed to stop OBI: ${error}`);
      return {
        success: false,
        message: 'Failed to stop OBI',
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Get current config
   */
  async getConfig(): Promise<ObiConfig | null> {
    try {
      if (!this.configPath) {
        return null;
      }

      const configContent = await fs.readFile(this.configPath, 'utf-8');
      return ObiConfigSchema.parse(YAML.parse(configContent));
    } catch (error) {
      logger.error(`Failed to read OBI config: ${error}`);
      return null;
    }
  }

  /**
   * Update config
   */
  async updateConfig(
    newConfig: ObiConfig,
    merge = true,
    restart = false
  ): Promise<ObiControlResult> {
    try {
      if (!this.configPath) {
        return {
          success: false,
          message: 'No config path set',
          error: 'OBI has not been deployed yet',
        };
      }

      let finalConfig: ObiConfig;

      if (merge) {
        const currentConfig = await this.getConfig();
        finalConfig = { ...currentConfig, ...newConfig };
      } else {
        finalConfig = newConfig;
      }

      // Validate
      const validatedConfig = ObiConfigSchema.parse(finalConfig);

      // Write to file
      await fs.writeFile(this.configPath, YAML.stringify(validatedConfig));

      logger.info('OBI config updated successfully');

      // Restart if requested
      if (restart) {
        const status = await this.getStatus();
        if (status.status === ObiStatus.RUNNING) {
          await this.stop();
          await this.deployLocal({ mode: 'standalone' as const, configPath: this.configPath });
        }
      }

      return {
        success: true,
        message: restart
          ? 'Config updated and OBI restarted'
          : 'Config updated (restart required for changes to take effect)',
        data: validatedConfig,
      };
    } catch (error) {
      logger.error(`Failed to update OBI config: ${error}`);
      return {
        success: false,
        message: 'Failed to update config',
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Get recent logs
   */
  async getLogs(lines = 100): Promise<string[]> {
    try {
      if (!this.logPath) {
        return ['No log path configured'];
      }

      const content = await fs.readFile(this.logPath, 'utf-8');
      const allLines = content.split('\n').filter((line) => line.length > 0);
      return allLines.slice(-lines);
    } catch (error) {
      logger.error(`Failed to read logs: ${error}`);
      return [`Error reading logs: ${error}`];
    }
  }
}

export default ObiManager.getInstance();
