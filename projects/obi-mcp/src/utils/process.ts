/**
 * Process management utilities
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

/**
 * Check if a process is running by PID
 */
export async function isProcessRunning(pid: number): Promise<boolean> {
  try {
    // On Unix, kill with signal 0 checks if process exists without killing it
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return false;
  }
}

/**
 * Get process info by PID
 */
export async function getProcessInfo(pid: number): Promise<{
  pid: number;
  cpuUsage: number;
  memoryUsage: number;
  uptime: number;
} | null> {
  try {
    // Use ps command to get process info
    const { stdout } = await execAsync(
      `ps -p ${pid} -o pid,%cpu,%mem,etimes --no-headers`
    );
    const parts = stdout.trim().split(/\s+/);

    if (parts.length >= 4) {
      return {
        pid: parseInt(parts[0], 10),
        cpuUsage: parseFloat(parts[1]),
        memoryUsage: parseFloat(parts[2]),
        uptime: parseInt(parts[3], 10),
      };
    }
    return null;
  } catch (error) {
    return null;
  }
}

/**
 * Find process by name
 */
export async function findProcessByName(name: string): Promise<number[]> {
  try {
    const { stdout } = await execAsync(`pgrep -f "${name}"`);
    return stdout
      .trim()
      .split('\n')
      .filter((line) => line.length > 0)
      .map((pid) => parseInt(pid, 10));
  } catch (error) {
    return [];
  }
}

/**
 * Kill process by PID
 */
export async function killProcess(pid: number, signal: NodeJS.Signals = 'SIGTERM'): Promise<void> {
  process.kill(pid, signal);
}
