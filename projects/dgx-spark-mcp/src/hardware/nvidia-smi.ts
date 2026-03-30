/**
 * nvidia-smi command wrapper for GPU detection
 */

import { executeCommand, commandExists } from '../utils/exec.js';
import { GPU, GPUTopology, NVLinkConnection, PCIeTopology } from '../types/gpu.js';

/**
 * Check if nvidia-smi is available
 */
export async function isNvidiaSmiAvailable(): Promise<boolean> {
  return commandExists('nvidia-smi');
}

/**
 * Get NVIDIA driver version
 */
export async function getDriverVersion(): Promise<string | null> {
  const result = await executeCommand('nvidia-smi --query-gpu=driver_version --format=csv,noheader -i 0');
  if (result.exitCode !== 0) return null;
  const lines = result.stdout.split('\n');
  return lines[0] || null;
}

/**
 * Get CUDA version
 */
export async function getCudaVersion(): Promise<string | null> {
  const result = await executeCommand('nvidia-smi --query-gpu=cuda_version --format=csv,noheader -i 0');
  if (result.exitCode !== 0) return null;
  const lines = result.stdout.split('\n');
  return lines[0] || null;
}

/**
 * Query GPU information using nvidia-smi
 */
export async function queryGPUs(): Promise<GPU[]> {
  const fields = [
    'index',
    'uuid',
    'name',
    'pci.bus_id',
    'memory.total',
    'memory.used',
    'memory.free',
    'utilization.gpu',
    'utilization.memory',
    'temperature.gpu',
    'temperature.memory',
    'power.draw',
    'power.limit',
    'power.default_limit',
    'clocks.current.graphics',
    'clocks.current.sm',
    'clocks.current.memory',
    'clocks.current.video',
    'compute_cap',
  ];

  const query = fields.join(',');
  const result = await executeCommand(
    `nvidia-smi --query-gpu=${query} --format=csv,noheader,nounits`
  );

  if (result.exitCode !== 0) {
    throw new Error(`nvidia-smi failed: ${result.stderr}`);
  }

  const gpus: GPU[] = [];
  const lines = result.stdout.split('\n').filter(line => line.trim() !== '');

  // Get driver and CUDA versions once
  const driverVersion = await getDriverVersion() || 'unknown';
  const cudaVersion = await getCudaVersion() || 'unknown';

  for (const line of lines) {
    const values = line.split(',').map(v => v.trim());
    if (values.length < fields.length) continue;

    const capStr = values[18] || '0.0';
    const [capMajorStr, capMinorStr] = capStr.split('.');
    const capMajor = capMajorStr ? parseInt(capMajorStr, 10) : 0;
    const capMinor = capMinorStr ? parseInt(capMinorStr, 10) : 0;

    const gpu: GPU = {
      id: parseInt(values[0] || '0', 10),
      uuid: values[1] || '',
      name: values[2] || 'Unknown',
      busId: values[3] || '',
      memory: {
        total: parseFloat(values[4] || '0') * 1024 * 1024, // MiB to bytes
        used: parseFloat(values[5] || '0') * 1024 * 1024,
        free: parseFloat(values[6] || '0') * 1024 * 1024,
      },
      utilization: {
        gpu: parseFloat(values[7] || '0'),
        memory: parseFloat(values[8] || '0'),
      },
      temperature: {
        current: parseFloat(values[9] || '0'),
        max: 90,
        slowdown: 85,
        shutdown: 95,
      },
      power: {
        current: parseFloat(values[11] || '0'),
        limit: parseFloat(values[12] || '0'),
        default: parseFloat(values[13] || '0'),
      },
      clocks: {
        graphics: parseFloat(values[14] || '0'),
        sm: parseFloat(values[15] || '0'),
        memory: parseFloat(values[16] || '0'),
        video: parseFloat(values[17] || '0'),
      },
      computeCapability: {
        major: capMajor,
        minor: capMinor,
      },
      driverVersion,
      cudaVersion,
    };

    gpus.push(gpu);
  }

  return gpus;
}

/**
 * Get NVLink topology matrix
 */
export async function getNVLinkTopology(gpuCount: number): Promise<number[][]> {
  const result = await executeCommand('nvidia-smi topo -m');

  if (result.exitCode !== 0) {
    // NVLink not available, return empty matrix
    return Array(gpuCount).fill(null).map(() => Array(gpuCount).fill(0));
  }

  const matrix: number[][] = Array(gpuCount).fill(null).map(() => Array(gpuCount).fill(0));
  const lines = result.stdout.split('\n');

  // Parse nvidia-smi topo output
  let inMatrix = false;
  let gpuIds: number[] = [];

  for (const line of lines) {
    const trimmed = line.trim();

    // Find header row with GPU IDs
    if (trimmed.startsWith('GPU')) {
      const parts = trimmed.split(/\s+/);
      gpuIds = parts.slice(1).map((p, i) => {
        const match = p.match(/GPU(\d+)/);
        return match && match[1] ? parseInt(match[1], 10) : i;
      });
      inMatrix = true;
      continue;
    }

    if (inMatrix && trimmed.startsWith('GPU')) {
      const parts = trimmed.split(/\s+/);
      const sourceGpuMatch = parts[0]?.match(/GPU(\d+)/);
      if (!sourceGpuMatch || !sourceGpuMatch[1]) continue;

      const sourceGpu = parseInt(sourceGpuMatch[1], 10);
      const connections = parts.slice(1);

      connections.forEach((conn, idx) => {
        if (idx >= gpuIds.length) return;
        const targetGpu = gpuIds[idx]!;

        // Parse NVLink connection
        const nvMatch = conn.match(/NV(\d+)/i);
        if (nvMatch && nvMatch[1]) {
          const linkCount = parseInt(nvMatch[1], 10);
          matrix[sourceGpu]![targetGpu] = linkCount * 25;
        }
      });
    }
  }

  return matrix;
}

/**
 * Get PCIe topology information
 */
export async function getPCIeTopology(): Promise<PCIeTopology[]> {
  const result = await executeCommand(
    'nvidia-smi --query-gpu=index,pci.bus_id,pcie.link.gen.current,pcie.link.width.current,pcie.link.width.max --format=csv,noheader,nounits'
  );

  if (result.exitCode !== 0) {
    return [];
  }

  const topology: PCIeTopology[] = [];
  const lines = result.stdout.split('\n').filter(line => line.trim() !== '');

  for (const line of lines) {
    const values = line.split(',').map(v => v.trim());
    if (values.length < 5) continue;

    topology.push({
      gpuId: parseInt(values[0] || '0', 10),
      busId: values[1] || '',
      generation: parseInt(values[2] || '0', 10),
      width: parseInt(values[3] || '0', 10),
      maxWidth: parseInt(values[4] || '0', 10),
    });
  }

  return topology;
}

/**
 * Build complete GPU topology including NVLink
 */
export async function buildGPUTopology(gpus: GPU[]): Promise<GPUTopology> {
  const nvlinkMatrix = await getNVLinkTopology(gpus.length);
  const pcieTopology = await getPCIeTopology();

  // Add NVLink connections to each GPU
  const gpusWithNVLink = gpus.map((gpu, idx) => {
    const nvlinks: NVLinkConnection[] = [];
    const row = nvlinkMatrix[idx];

    if (row) {
      row.forEach((bandwidth, targetIdx) => {
        if (bandwidth > 0 && targetIdx !== idx) {
          nvlinks.push({
            gpu: targetIdx,
            link: nvlinks.length,
            connected: true,
            bandwidth,
          });
        }
      });
    }

    return {
      ...gpu,
      nvlinks: nvlinks.length > 0 ? nvlinks : undefined,
    };
  });

  return {
    gpus: gpusWithNVLink,
    nvlinkMatrix,
    pcieTopology,
  };
}
