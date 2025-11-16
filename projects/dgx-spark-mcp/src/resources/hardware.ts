/**
 * Hardware resource handlers
 * Provides MCP resources for hardware specifications and topology
 */

import { getHardwareSnapshot } from '../hardware/topology.js';
import type { ResourceDescriptor, ResourceContent } from '../types/resources.js';
import { HardwareResourceURIs } from '../types/resources.js';

/**
 * Get all hardware resource descriptors
 */
export function getHardwareResourceDescriptors(): ResourceDescriptor[] {
  return [
    {
      uri: HardwareResourceURIs.SPECS,
      name: 'Hardware Specifications',
      description: 'Complete hardware specifications including CPU, memory, GPU, storage, and network',
      mimeType: 'application/json',
    },
    {
      uri: HardwareResourceURIs.TOPOLOGY,
      name: 'System Topology',
      description: 'Detailed system topology including NUMA nodes, PCIe topology, and interconnects',
      mimeType: 'application/json',
    },
    {
      uri: HardwareResourceURIs.GPUS,
      name: 'GPU Information',
      description: 'Detailed information about all GPUs including utilization, memory, and NVLink topology',
      mimeType: 'application/json',
    },
    {
      uri: HardwareResourceURIs.CPU,
      name: 'CPU Information',
      description: 'CPU specifications including cores, frequency, cache, and architecture',
      mimeType: 'application/json',
    },
    {
      uri: HardwareResourceURIs.MEMORY,
      name: 'Memory Information',
      description: 'System memory information including total, available, and NUMA layout',
      mimeType: 'application/json',
    },
    {
      uri: HardwareResourceURIs.STORAGE,
      name: 'Storage Information',
      description: 'Storage devices information including capacity, type, and performance characteristics',
      mimeType: 'application/json',
    },
    {
      uri: HardwareResourceURIs.NETWORK,
      name: 'Network Information',
      description: 'Network interfaces including InfiniBand, Ethernet, and bandwidth',
      mimeType: 'application/json',
    },
  ];
}

/**
 * Read hardware resource by URI
 */
export async function readHardwareResource(uri: string): Promise<ResourceContent[]> {
  // Get hardware snapshot with caching
  const snapshot = await getHardwareSnapshot({ useCache: true });
  const { topology } = snapshot;

  switch (uri) {
    case HardwareResourceURIs.SPECS:
      return [{
        uri,
        mimeType: 'application/json',
        text: JSON.stringify({
          system: {
            hostname: topology.hostname,
            os: topology.os,
            kernel: topology.kernel,
            uptime: topology.uptime,
          },
          cpu: {
            model: topology.cpu.modelName,
            vendor: topology.cpu.vendor,
            architecture: topology.cpu.architecture,
            cores: topology.cpu.cores,
            threads: topology.cpu.cores.logical,
            frequency: topology.cpu.frequency,
            cache: topology.cpu.cache,
          },
          memory: {
            total: topology.memory.info.total,
            totalGB: Math.round(topology.memory.info.total / (1024 * 1024 * 1024)),
            available: topology.memory.info.available,
            availableGB: Math.round(topology.memory.info.available / (1024 * 1024 * 1024)),
            numaNodes: topology.cpu.numaNodes,
          },
          gpu: topology.gpus ? {
            count: topology.gpus.length,
            gpus: topology.gpus.map(gpu => ({
              id: gpu.id,
              name: gpu.name,
              uuid: gpu.uuid,
              memory: {
                total: gpu.memory.total,
                totalGB: Math.round(gpu.memory.total / (1024 * 1024 * 1024)),
                used: gpu.memory.used,
                free: gpu.memory.free,
              },
              utilization: gpu.utilization,
              temperature: gpu.temperature,
              power: gpu.power,
            })),
          } : null,
          storage: {
            totalCapacity: topology.storage.totalCapacity,
            totalCapacityGB: Math.round(topology.storage.totalCapacity / (1024 * 1024 * 1024)),
            devices: topology.storage.blockDevices.length,
          },
          network: {
            totalInterfaces: topology.network.totalInterfaces,
            hasInfiniBand: topology.capabilities.hasInfiniBand,
          },
          capabilities: topology.capabilities,
          metadata: {
            timestamp: snapshot.timestamp,
            cached: snapshot.cached,
            detectionTime: snapshot.detectionTime,
          },
        }, null, 2),
      }];

    case HardwareResourceURIs.TOPOLOGY:
      return [{
        uri,
        mimeType: 'application/json',
        text: JSON.stringify({
          topology,
          metadata: {
            timestamp: snapshot.timestamp,
            cached: snapshot.cached,
            detectionTime: snapshot.detectionTime,
          },
        }, null, 2),
      }];

    case HardwareResourceURIs.GPUS:
      if (!topology.gpus || topology.gpus.length === 0) {
        return [{
          uri,
          mimeType: 'application/json',
          text: JSON.stringify({
            error: 'No GPUs detected',
            gpus: [],
          }, null, 2),
        }];
      }

      return [{
        uri,
        mimeType: 'application/json',
        text: JSON.stringify({
          count: topology.gpus.length,
          gpus: topology.gpus,
          topology: topology.gpuTopology,
          metadata: {
            timestamp: snapshot.timestamp,
            cached: snapshot.cached,
          },
        }, null, 2),
      }];

    case HardwareResourceURIs.CPU:
      return [{
        uri,
        mimeType: 'application/json',
        text: JSON.stringify({
          cpu: topology.cpu,
          metadata: {
            timestamp: snapshot.timestamp,
            cached: snapshot.cached,
          },
        }, null, 2),
      }];

    case HardwareResourceURIs.MEMORY:
      return [{
        uri,
        mimeType: 'application/json',
        text: JSON.stringify({
          memory: topology.memory,
          metadata: {
            timestamp: snapshot.timestamp,
            cached: snapshot.cached,
          },
        }, null, 2),
      }];

    case HardwareResourceURIs.STORAGE:
      return [{
        uri,
        mimeType: 'application/json',
        text: JSON.stringify({
          storage: topology.storage,
          metadata: {
            timestamp: snapshot.timestamp,
            cached: snapshot.cached,
          },
        }, null, 2),
      }];

    case HardwareResourceURIs.NETWORK:
      return [{
        uri,
        mimeType: 'application/json',
        text: JSON.stringify({
          network: topology.network,
          metadata: {
            timestamp: snapshot.timestamp,
            cached: snapshot.cached,
          },
        }, null, 2),
      }];

    default:
      throw new Error(`Unknown hardware resource URI: ${uri}`);
  }
}

/**
 * Check if URI is a hardware resource
 */
export function isHardwareResourceURI(uri: string): boolean {
  return Object.values(HardwareResourceURIs).includes(uri as any);
}
