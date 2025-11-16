/**
 * System health checker tool
 * Provides comprehensive system health status
 */

import { getHardwareSnapshot } from '../hardware/topology.js';
import type { GetSystemHealthArgs, ToolCallResponse } from '../types/tools.js';

export interface SystemHealthResult {
  status: 'healthy' | 'degraded' | 'critical';
  timestamp: number;
  components: {
    cpu: HealthComponent;
    memory: HealthComponent;
    gpu?: HealthComponent;
    storage: HealthComponent;
    network: HealthComponent;
  };
  alerts: HealthAlert[];
  summary: string;
}

interface HealthComponent {
  status: 'healthy' | 'warning' | 'critical';
  message: string;
  metrics?: Record<string, any>;
}

interface HealthAlert {
  severity: 'info' | 'warning' | 'critical';
  component: string;
  message: string;
  recommendation?: string;
}

/**
 * Get system health
 */
export async function getSystemHealth(args?: GetSystemHealthArgs): Promise<ToolCallResponse> {
  try {
    const verbose = args?.verbose || false;

    // Get hardware snapshot
    const snapshot = await getHardwareSnapshot({ useCache: false }); // Always get fresh data for health
    const { topology } = snapshot;

    const alerts: HealthAlert[] = [];
    const components: SystemHealthResult['components'] = {
      cpu: { status: 'healthy', message: 'CPU operating normally' },
      memory: { status: 'healthy', message: 'Memory available' },
      storage: { status: 'healthy', message: 'Storage available' },
      network: { status: 'healthy', message: 'Network interfaces active' },
    };

    // Check CPU health
    const cpuHealth = checkCPUHealth(topology, verbose);
    components.cpu = cpuHealth.component;
    alerts.push(...cpuHealth.alerts);

    // Check Memory health
    const memoryHealth = checkMemoryHealth(topology, verbose);
    components.memory = memoryHealth.component;
    alerts.push(...memoryHealth.alerts);

    // Check GPU health (if available)
    if (topology.gpus && topology.gpus.length > 0) {
      const gpuHealth = checkGPUHealth(topology.gpus, verbose);
      components.gpu = gpuHealth.component;
      alerts.push(...gpuHealth.alerts);
    }

    // Check Storage health
    const storageHealth = checkStorageHealth(topology, verbose);
    components.storage = storageHealth.component;
    alerts.push(...storageHealth.alerts);

    // Check Network health
    const networkHealth = checkNetworkHealth(topology, verbose);
    components.network = networkHealth.component;
    alerts.push(...networkHealth.alerts);

    // Determine overall status
    const overallStatus = determineOverallStatus(components);

    // Generate summary
    const summary = generateHealthSummary(overallStatus, components, alerts);

    const result: SystemHealthResult = {
      status: overallStatus,
      timestamp: Date.now(),
      components,
      alerts,
      summary,
    };

    return {
      content: [{
        type: 'text',
        text: JSON.stringify(result, null, 2),
      }],
    };
  } catch (error) {
    return {
      content: [{
        type: 'text',
        text: JSON.stringify({
          error: 'Failed to check system health',
          message: error instanceof Error ? error.message : 'Unknown error',
        }, null, 2),
      }],
      isError: true,
    };
  }
}

/**
 * Check CPU health
 */
function checkCPUHealth(topology: any, verbose: boolean) {
  const alerts: HealthAlert[] = [];
  const metrics = verbose ? {
    cores: topology.cpu.cores,
    modelName: topology.cpu.modelName,
    frequency: topology.cpu.frequency,
  } : undefined;

  // CPU is generally healthy if detected properly
  const component: HealthComponent = {
    status: 'healthy',
    message: `${topology.cpu.cores.physical} physical cores, ${topology.cpu.cores.logical} logical cores`,
    metrics,
  };

  return { component, alerts };
}

/**
 * Check memory health
 */
function checkMemoryHealth(topology: any, verbose: boolean) {
  const alerts: HealthAlert[] = [];
  const totalGB = Math.round(topology.memory.info.total / (1024 * 1024 * 1024));
  const availableGB = Math.round(topology.memory.info.available / (1024 * 1024 * 1024));
  const usedPercent = ((totalGB - availableGB) / totalGB) * 100;

  const metrics = verbose ? {
    totalGB,
    availableGB,
    usedPercent: Math.round(usedPercent),
  } : undefined;

  let status: HealthComponent['status'] = 'healthy';
  let message = `${availableGB}GB available of ${totalGB}GB total (${Math.round(100 - usedPercent)}% free)`;

  if (usedPercent > 90) {
    status = 'critical';
    message = `Critical: Only ${availableGB}GB available of ${totalGB}GB total`;
    alerts.push({
      severity: 'critical',
      component: 'memory',
      message: 'Memory usage above 90%',
      recommendation: 'Free up memory or reduce workload size',
    });
  } else if (usedPercent > 80) {
    status = 'warning';
    message = `Warning: ${availableGB}GB available of ${totalGB}GB total`;
    alerts.push({
      severity: 'warning',
      component: 'memory',
      message: 'Memory usage above 80%',
      recommendation: 'Monitor memory usage closely',
    });
  }

  const component: HealthComponent = { status, message, metrics };
  return { component, alerts };
}

/**
 * Check GPU health
 */
function checkGPUHealth(gpus: any[], verbose: boolean) {
  const alerts: HealthAlert[] = [];
  let worstStatus: HealthComponent['status'] = 'healthy';

  const gpuMetrics = gpus.map(gpu => {

    // Check temperature
    if (gpu.temperature.current > gpu.temperature.slowdown - 10) {
      worstStatus = 'warning';
      alerts.push({
        severity: 'warning',
        component: 'gpu',
        message: `GPU ${gpu.id} temperature high (${gpu.temperature.current}°C)`,
        recommendation: 'Check cooling system',
      });
    }

    // Check memory usage
    const memoryUsedPercent = (gpu.memory.used / gpu.memory.total) * 100;
    if (memoryUsedPercent > 95) {
      worstStatus = 'critical';
      alerts.push({
        severity: 'critical',
        component: 'gpu',
        message: `GPU ${gpu.id} memory usage critical (${Math.round(memoryUsedPercent)}%)`,
        recommendation: 'Reduce GPU workload',
      });
    }

    return verbose ? {
      id: gpu.id,
      name: gpu.name,
      temperature: gpu.temperature.current,
      utilization: gpu.utilization.gpu,
      memoryUsedPercent: Math.round(memoryUsedPercent),
      power: gpu.power.current,
    } : null;
  }).filter(m => m !== null);

  const message = `${gpus.length} GPU(s) detected, ${alerts.length} alert(s)`;

  const component: HealthComponent = {
    status: worstStatus,
    message,
    metrics: verbose ? { gpus: gpuMetrics } : undefined,
  };

  return { component, alerts };
}

/**
 * Check storage health
 */
function checkStorageHealth(topology: any, verbose: boolean) {
  const alerts: HealthAlert[] = [];
  const totalGB = Math.round(topology.storage.totalCapacity / (1024 * 1024 * 1024));

  const metrics = verbose ? {
    totalCapacityGB: totalGB,
    devices: topology.storage.devices.length,
    hasNVMe: topology.capabilities.hasNVMe,
  } : undefined;

  const component: HealthComponent = {
    status: 'healthy',
    message: `${topology.storage.devices.length} storage device(s), ${totalGB}GB total capacity`,
    metrics,
  };

  return { component, alerts };
}

/**
 * Check network health
 */
function checkNetworkHealth(topology: any, verbose: boolean) {
  const alerts: HealthAlert[] = [];

  const metrics = verbose ? {
    totalInterfaces: topology.network.totalInterfaces,
    hasInfiniBand: topology.capabilities.hasInfiniBand,
  } : undefined;

  const component: HealthComponent = {
    status: 'healthy',
    message: `${topology.network.totalInterfaces} network interface(s) active`,
    metrics,
  };

  return { component, alerts };
}

/**
 * Determine overall system status
 */
function determineOverallStatus(components: SystemHealthResult['components']): SystemHealthResult['status'] {
  const statuses = Object.values(components).map(c => c.status);

  if (statuses.includes('critical')) {
    return 'critical';
  }
  if (statuses.includes('warning')) {
    return 'degraded';
  }
  return 'healthy';
}

/**
 * Generate health summary
 */
function generateHealthSummary(
  status: SystemHealthResult['status'],
  _components: SystemHealthResult['components'],
  alerts: HealthAlert[]
): string {
  const parts: string[] = [];

  if (status === 'healthy') {
    parts.push('System is healthy and ready for workloads.');
  } else if (status === 'degraded') {
    parts.push('System is operational but has some warnings.');
  } else {
    parts.push('System has critical issues that need immediate attention.');
  }

  const criticalAlerts = alerts.filter(a => a.severity === 'critical').length;
  const warningAlerts = alerts.filter(a => a.severity === 'warning').length;

  if (criticalAlerts > 0) {
    parts.push(`${criticalAlerts} critical alert(s).`);
  }
  if (warningAlerts > 0) {
    parts.push(`${warningAlerts} warning(s).`);
  }

  if (alerts.length === 0) {
    parts.push('All components operating normally.');
  }

  return parts.join(' ');
}
