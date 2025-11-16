/**
 * Test utilities and helper functions
 */

import type { GPU } from '../types/gpu.js';
import type { CPU } from '../types/cpu.js';
import type { MemoryInfo } from '../types/memory.js';
import type { Storage } from '../types/storage.js';
import type { Network } from '../types/network.js';

/**
 * Create a mock GPU for testing
 */
export function createMockGPU(overrides?: Partial<GPU>): GPU {
  return {
    index: 0,
    uuid: 'GPU-12345678-1234-1234-1234-123456789012',
    name: 'NVIDIA A100-SXM4-80GB',
    architecture: 'Ampere',
    cudaCores: 6912,
    computeCapability: '8.0',
    memory: {
      total: 85899345920, // 80GB
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
    clockSpeeds: {
      graphics: 1410,
      sm: 1410,
      memory: 1215,
      video: 1275,
    },
    pciInfo: {
      bus: '0000:07:00.0',
      deviceId: '20B010DE',
      generation: 4,
      linkWidth: 16,
    },
    nvlinkInfo: {
      enabled: true,
      links: 12,
      bandwidthPerLink: 600,
      totalBandwidth: 7200,
    },
    mig: {
      enabled: false,
      instances: [],
    },
    ...overrides,
  };
}

/**
 * Create a mock CPU for testing
 */
export function createMockCPU(overrides?: Partial<CPU>): CPU {
  return {
    model: 'AMD EPYC 7742 64-Core Processor',
    vendor: 'AMD',
    architecture: 'x86_64',
    cores: {
      physical: 64,
      logical: 128,
    },
    frequency: {
      current: 2250,
      min: 1500,
      max: 3400,
    },
    cacheSize: {
      l1d: 2097152,
      l1i: 2097152,
      l2: 33554432,
      l3: 268435456,
    },
    features: [
      'fpu', 'vme', 'de', 'pse', 'tsc', 'msr', 'pae', 'mce',
      'cx8', 'apic', 'sep', 'mtrr', 'pge', 'mca', 'cmov',
      'pat', 'pse36', 'clflush', 'mmx', 'fxsr', 'sse', 'sse2',
      'ht', 'syscall', 'nx', 'mmxext', 'pdpe1gb', 'rdtscp',
      'lm', 'avx', 'avx2',
    ],
    numa: {
      nodes: 2,
      cpusPerNode: 64,
    },
    ...overrides,
  };
}

/**
 * Create mock memory info for testing
 */
export function createMockMemory(overrides?: Partial<MemoryInfo>): MemoryInfo {
  return {
    total: 1099511627776, // 1TB
    available: 1030792151040,
    used: 68719476736,
    free: 1030792151040,
    shared: 0,
    cached: 0,
    buffers: 0,
    swapTotal: 0,
    swapFree: 0,
    swapUsed: 0,
    ...overrides,
  };
}

/**
 * Create mock storage info for testing
 */
export function createMockStorage(overrides?: Partial<Storage>): Storage {
  return {
    devices: [
      {
        name: '/dev/nvme0n1',
        type: 'nvme',
        size: 3840755982336, // 3.5TB
        model: 'Samsung SSD 980 PRO 4TB',
        serial: 'S5GXNX0T123456',
        mountPoint: '/',
        filesystem: 'ext4',
        used: 1073741824000,
        available: 2767014158336,
      },
    ],
    mounts: [
      {
        mountPoint: '/',
        device: '/dev/nvme0n1p2',
        filesystem: 'ext4',
        size: 3840755982336,
        used: 1073741824000,
        available: 2767014158336,
      },
    ],
    ...overrides,
  };
}

/**
 * Create mock network info for testing
 */
export function createMockNetwork(overrides?: Partial<Network>): Network {
  return {
    interfaces: [
      {
        name: 'enp1s0f0',
        type: 'ethernet',
        speed: 100000, // 100Gbps
        mtu: 9000,
        state: 'up',
        macAddress: '00:1a:2b:3c:4d:5e',
        ipv4Address: '10.0.0.100',
        ipv6Address: 'fe80::21a:2bff:fe3c:4d5e',
      },
      {
        name: 'ib0',
        type: 'infiniband',
        speed: 200000, // 200Gbps
        mtu: 4092,
        state: 'up',
        macAddress: '00:02:c9:01:23:45',
        ipv4Address: '192.168.1.100',
        ipv6Address: 'fe80::202:c9ff:fe01:2345',
      },
    ],
    rdma: {
      enabled: true,
      devices: [
        {
          name: 'mlx5_0',
          type: 'ConnectX-7',
          portState: 'active',
          linkLayer: 'InfiniBand',
          maxMtu: 4096,
          activeSpeed: 200,
          activeWidth: 4,
        },
      ],
    },
    ...overrides,
  };
}

/**
 * Create a complete mock hardware topology
 */
export function createMockHardwareTopology() {
  return {
    gpus: [
      createMockGPU({ index: 0 }),
      createMockGPU({ index: 1, uuid: 'GPU-12345678-1234-1234-1234-123456789013' }),
      createMockGPU({ index: 2, uuid: 'GPU-12345678-1234-1234-1234-123456789014' }),
      createMockGPU({ index: 3, uuid: 'GPU-12345678-1234-1234-1234-123456789015' }),
    ],
    cpu: createMockCPU(),
    memory: createMockMemory(),
    storage: createMockStorage(),
    network: createMockNetwork(),
    timestamp: new Date().toISOString(),
  };
}

/**
 * Wait for a condition to be true
 */
export async function waitFor(
  condition: () => boolean | Promise<boolean>,
  timeout = 5000,
  interval = 100
): Promise<void> {
  const startTime = Date.now();
  while (Date.now() - startTime < timeout) {
    if (await condition()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, interval));
  }
  throw new Error(`Condition not met within ${timeout}ms`);
}

/**
 * Create a spy that tracks calls
 */
export function createCallTracker<T extends (...args: any[]) => any>() {
  const calls: Array<{ args: Parameters<T>; result?: ReturnType<T>; error?: Error }> = [];

  return {
    fn: ((...args: Parameters<T>) => {
      const call: any = { args };
      calls.push(call);
      return call.result;
    }) as T,
    calls,
    callCount: () => calls.length,
    lastCall: () => calls[calls.length - 1],
    reset: () => calls.splice(0, calls.length),
  };
}

/**
 * Mock environment variables for a test
 */
export function withEnv<T>(
  envVars: Record<string, string | undefined>,
  fn: () => T
): T {
  const originalEnv = { ...process.env };
  Object.assign(process.env, envVars);
  try {
    return fn();
  } finally {
    process.env = originalEnv;
  }
}

/**
 * Mock environment variables for an async test
 */
export async function withEnvAsync<T>(
  envVars: Record<string, string | undefined>,
  fn: () => Promise<T>
): Promise<T> {
  const originalEnv = { ...process.env };
  Object.assign(process.env, envVars);
  try {
    return await fn();
  } finally {
    process.env = originalEnv;
  }
}
