/**
 * GPU detection module
 */

import { GPU, GPUDetectionResult, GPUTopology } from '../types/gpu.js';
import {
  isNvidiaSmiAvailable,
  queryGPUs,
  buildGPUTopology,
} from './nvidia-smi.js';

/**
 * Detect all NVIDIA GPUs in the system
 */
export async function detectGPUs(includeTopology: boolean = false): Promise<GPUDetectionResult> {
  const startTime = Date.now();

  // Check if nvidia-smi is available
  const nvidiaSmiAvailable = await isNvidiaSmiAvailable();
  if (!nvidiaSmiAvailable) {
    throw new Error('nvidia-smi not found. Ensure NVIDIA drivers are installed and nvidia-smi is in PATH.');
  }

  // Query GPU information
  const gpus = await queryGPUs();

  if (gpus.length === 0) {
    throw new Error('No NVIDIA GPUs detected');
  }

  // Build topology if requested
  let topology: GPUTopology | undefined;
  if (includeTopology) {
    topology = await buildGPUTopology(gpus);
  }

  const detectionTime = Date.now() - startTime;

  return {
    gpus: topology ? topology.gpus : gpus,
    topology,
    timestamp: Date.now(),
    detectionTime,
  };
}

/**
 * Get GPU by ID
 */
export async function getGPU(gpuId: number): Promise<GPU | null> {
  const result = await detectGPUs(false);
  return result.gpus.find(gpu => gpu.id === gpuId) || null;
}

/**
 * Get GPU count
 */
export async function getGPUCount(): Promise<number> {
  try {
    const result = await detectGPUs(false);
    return result.gpus.length;
  } catch (error) {
    return 0;
  }
}

/**
 * Check if NVIDIA GPUs are available
 */
export async function hasNVIDIAGPUs(): Promise<boolean> {
  const count = await getGPUCount();
  return count > 0;
}

/**
 * Get total GPU memory across all GPUs
 */
export async function getTotalGPUMemory(): Promise<number> {
  const result = await detectGPUs(false);
  return result.gpus.reduce((total, gpu) => total + gpu.memory.total, 0);
}

/**
 * Get available GPU memory across all GPUs
 */
export async function getAvailableGPUMemory(): Promise<number> {
  const result = await detectGPUs(false);
  return result.gpus.reduce((total, gpu) => total + gpu.memory.free, 0);
}

/**
 * Get average GPU utilization
 */
export async function getAverageGPUUtilization(): Promise<{ gpu: number; memory: number }> {
  const result = await detectGPUs(false);

  if (result.gpus.length === 0) {
    return { gpu: 0, memory: 0 };
  }

  const totalGpu = result.gpus.reduce((sum, gpu) => sum + gpu.utilization.gpu, 0);
  const totalMemory = result.gpus.reduce((sum, gpu) => sum + gpu.utilization.memory, 0);

  return {
    gpu: totalGpu / result.gpus.length,
    memory: totalMemory / result.gpus.length,
  };
}
