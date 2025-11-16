/**
 * Unit tests for GPU detection
 */

import { describe, it, expect, jest, beforeEach } from '@jest/globals';
import {
  detectGPUs,
  getGPU,
  getGPUCount,
  hasNVIDIAGPUs,
  getTotalGPUMemory,
  getAvailableGPUMemory,
  getAverageGPUUtilization,
} from './gpu.js';

// Mock nvidia-smi module
jest.mock('./nvidia-smi.js');

import {
  isNvidiaSmiAvailable,
  queryGPUs,
  buildGPUTopology,
} from './nvidia-smi.js';

const mockIsNvidiaSmiAvailable = isNvidiaSmiAvailable as jest.MockedFunction<typeof isNvidiaSmiAvailable>;
const mockQueryGPUs = queryGPUs as jest.MockedFunction<typeof queryGPUs>;
const mockBuildGPUTopology = buildGPUTopology as jest.MockedFunction<typeof buildGPUTopology>;

describe('GPU Detection', () => {
  const mockGPU = {
    id: 0,
    uuid: 'GPU-12345678-1234-1234-1234-123456789012',
    name: 'NVIDIA A100-SXM4-80GB',
    architecture: 'Ampere',
    memory: {
      total: 85899345920,
      free: 85899345920,
      used: 0,
    },
    utilization: {
      gpu: 0,
      memory: 0,
      encoder: 0,
      decoder: 0,
    },
    temperature: 35,
    powerDraw: 50,
    powerLimit: 400,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('detectGPUs', () => {
    it('should detect GPUs when nvidia-smi is available', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);

      const result = await detectGPUs(false);

      expect(result.gpus).toHaveLength(1);
      expect(result.gpus[0]).toEqual(mockGPU);
      expect(result.timestamp).toBeDefined();
      expect(result.detectionTime).toBeGreaterThanOrEqual(0);
      expect(mockIsNvidiaSmiAvailable).toHaveBeenCalled();
      expect(mockQueryGPUs).toHaveBeenCalled();
    });

    it('should throw error when nvidia-smi is not available', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(false);

      await expect(detectGPUs(false)).rejects.toThrow('nvidia-smi not found');
    });

    it('should throw error when no GPUs are detected', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([]);

      await expect(detectGPUs(false)).rejects.toThrow('No NVIDIA GPUs detected');
    });

    it('should include topology when requested', async () => {
      const mockTopology = {
        gpus: [mockGPU],
        nvlinks: [],
        pciTopology: {},
      };

      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);
      mockBuildGPUTopology.mockResolvedValue(mockTopology);

      const result = await detectGPUs(true);

      expect(result.topology).toEqual(mockTopology);
      expect(mockBuildGPUTopology).toHaveBeenCalledWith([mockGPU]);
    });

    it('should not include topology when not requested', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);

      const result = await detectGPUs(false);

      expect(result.topology).toBeUndefined();
      expect(mockBuildGPUTopology).not.toHaveBeenCalled();
    });

    it('should detect multiple GPUs', async () => {
      const gpu1 = { ...mockGPU, id: 0 };
      const gpu2 = { ...mockGPU, id: 1, uuid: 'GPU-87654321-4321-4321-4321-210987654321' };

      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([gpu1, gpu2]);

      const result = await detectGPUs(false);

      expect(result.gpus).toHaveLength(2);
      expect(result.gpus[0]!.id).toBe(0);
      expect(result.gpus[1]!.id).toBe(1);
    });
  });

  describe('getGPU', () => {
    it('should return GPU by ID', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);

      const gpu = await getGPU(0);

      expect(gpu).toEqual(mockGPU);
    });

    it('should return null for non-existent GPU ID', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);

      const gpu = await getGPU(999);

      expect(gpu).toBeNull();
    });
  });

  describe('getGPUCount', () => {
    it('should return GPU count', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);

      const count = await getGPUCount();

      expect(count).toBe(1);
    });

    it('should return 0 when detection fails', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(false);

      const count = await getGPUCount();

      expect(count).toBe(0);
    });
  });

  describe('hasNVIDIAGPUs', () => {
    it('should return true when GPUs exist', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);

      const hasGPUs = await hasNVIDIAGPUs();

      expect(hasGPUs).toBe(true);
    });

    it('should return false when no GPUs exist', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(false);

      const hasGPUs = await hasNVIDIAGPUs();

      expect(hasGPUs).toBe(false);
    });
  });

  describe('getTotalGPUMemory', () => {
    it('should calculate total GPU memory', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);

      const totalMemory = await getTotalGPUMemory();

      expect(totalMemory).toBe(85899345920);
    });

    it('should sum memory across multiple GPUs', async () => {
      const gpu1 = { ...mockGPU, id: 0 };
      const gpu2 = { ...mockGPU, id: 1 };

      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([gpu1, gpu2]);

      const totalMemory = await getTotalGPUMemory();

      expect(totalMemory).toBe(85899345920 * 2);
    });
  });

  describe('getAvailableGPUMemory', () => {
    it('should calculate available GPU memory', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([mockGPU]);

      const availableMemory = await getAvailableGPUMemory();

      expect(availableMemory).toBe(85899345920);
    });

    it('should sum available memory across multiple GPUs', async () => {
      const gpu1 = { ...mockGPU, id: 0, memory: { total: 80 * 1024 ** 3, free: 40 * 1024 ** 3, used: 40 * 1024 ** 3 } };
      const gpu2 = { ...mockGPU, id: 1, memory: { total: 80 * 1024 ** 3, free: 60 * 1024 ** 3, used: 20 * 1024 ** 3 } };

      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([gpu1, gpu2]);

      const availableMemory = await getAvailableGPUMemory();

      expect(availableMemory).toBe(100 * 1024 ** 3);
    });
  });

  describe('getAverageGPUUtilization', () => {
    it('should calculate average GPU utilization', async () => {
      const gpu1 = { ...mockGPU, id: 0, utilization: { gpu: 50, memory: 60, encoder: 0, decoder: 0 } };
      const gpu2 = { ...mockGPU, id: 1, utilization: { gpu: 30, memory: 40, encoder: 0, decoder: 0 } };

      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([gpu1, gpu2]);

      const avgUtil = await getAverageGPUUtilization();

      expect(avgUtil.gpu).toBe(40);
      expect(avgUtil.memory).toBe(50);
    });

    it('should return 0 when no GPUs exist', async () => {
      mockIsNvidiaSmiAvailable.mockResolvedValue(true);
      mockQueryGPUs.mockResolvedValue([]);

      await expect(detectGPUs(false)).rejects.toThrow();
    });
  });
});
