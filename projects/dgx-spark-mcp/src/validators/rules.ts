/**
 * Validation rules for Spark configurations
 */

export interface ValidationRule {
  id: string;
  name: string;
  category: 'executor' | 'driver' | 'memory' | 'shuffle' | 'gpu' | 'optimization';
  severity: 'error' | 'warning' | 'info';
  description: string;
  check: (config: any) => boolean;
  message: (config: any) => string;
  fix?: (config: any) => any;
}

/**
 * Get all validation rules
 */
export async function listRules(): Promise<ValidationRule[]> {
  return ALL_RULES;
}

/**
 * Get rules by category
 */
export async function getRulesByCategory(category: string): Promise<ValidationRule[]> {
  return ALL_RULES.filter(rule => rule.category === category);
}

/**
 * Get rules by severity
 */
export async function getRulesBySeverity(severity: string): Promise<ValidationRule[]> {
  return ALL_RULES.filter(rule => rule.severity === severity);
}

/**
 * All validation rules
 */
const ALL_RULES: ValidationRule[] = [
  // Executor Rules
  {
    id: 'exec-001',
    name: 'Minimum Executor Memory',
    category: 'executor',
    severity: 'error',
    description: 'Executor memory must be at least 1GB',
    check: (config) => parseMemory(config.executor?.memory) >= 1,
    message: (config) => `Executor memory ${config.executor?.memory} is below minimum of 1GB`,
  },
  {
    id: 'exec-002',
    name: 'Maximum Executor Memory',
    category: 'executor',
    severity: 'warning',
    description: 'Executor memory should not exceed 64GB to avoid GC issues',
    check: (config) => parseMemory(config.executor?.memory) <= 64,
    message: (config) => `Executor memory ${config.executor?.memory} exceeds recommended 64GB`,
  },
  {
    id: 'exec-003',
    name: 'Executor Cores Range',
    category: 'executor',
    severity: 'warning',
    description: 'Executor cores should be between 4-6 for optimal performance',
    check: (config) => config.executor?.cores >= 4 && config.executor?.cores <= 6,
    message: (config) => `Executor cores ${config.executor?.cores} outside optimal range of 4-6`,
  },
  {
    id: 'exec-004',
    name: 'Minimum Executor Cores',
    category: 'executor',
    severity: 'error',
    description: 'Executor must have at least 1 core',
    check: (config) => config.executor?.cores >= 1,
    message: () => 'Executor cores must be at least 1',
  },

  // Driver Rules
  {
    id: 'drv-001',
    name: 'Minimum Driver Memory',
    category: 'driver',
    severity: 'error',
    description: 'Driver memory must be at least 1GB',
    check: (config) => parseMemory(config.driver?.memory) >= 1,
    message: (config) => `Driver memory ${config.driver?.memory} is below minimum of 1GB`,
  },
  {
    id: 'drv-002',
    name: 'Driver Memory Proportion',
    category: 'driver',
    severity: 'warning',
    description: 'Driver memory should be 1-2x executor memory',
    check: (config) => {
      const driverMem = parseMemory(config.driver?.memory);
      const executorMem = parseMemory(config.executor?.memory);
      return driverMem >= executorMem && driverMem <= executorMem * 2;
    },
    message: (config) => {
      const driverMem = parseMemory(config.driver?.memory);
      const executorMem = parseMemory(config.executor?.memory);
      return `Driver memory (${driverMem}GB) should be 1-2x executor memory (${executorMem}GB)`;
    },
  },

  // Memory Rules
  {
    id: 'mem-001',
    name: 'Memory Fraction Range',
    category: 'memory',
    severity: 'error',
    description: 'Memory fraction must be between 0 and 1',
    check: (config) => config.memory?.fraction >= 0 && config.memory?.fraction <= 1,
    message: (config) => `Memory fraction ${config.memory?.fraction} must be between 0 and 1`,
  },
  {
    id: 'mem-002',
    name: 'Storage Fraction Range',
    category: 'memory',
    severity: 'error',
    description: 'Storage fraction must be between 0 and 1',
    check: (config) => config.memory?.storageFraction >= 0 && config.memory?.storageFraction <= 1,
    message: (config) => `Storage fraction ${config.memory?.storageFraction} must be between 0 and 1`,
  },
  {
    id: 'mem-003',
    name: 'Recommended Memory Fraction',
    category: 'memory',
    severity: 'info',
    description: 'Memory fraction should be at least 0.4',
    check: (config) => config.memory?.fraction >= 0.4,
    message: (config) => `Memory fraction ${config.memory?.fraction} is below recommended 0.4`,
  },

  // Shuffle Rules
  {
    id: 'shuf-001',
    name: 'Minimum Shuffle Partitions',
    category: 'shuffle',
    severity: 'error',
    description: 'Shuffle partitions must be at least 1',
    check: (config) => config.shuffle?.partitions >= 1,
    message: () => 'Shuffle partitions must be at least 1',
  },
  {
    id: 'shuf-002',
    name: 'Shuffle Partitions vs Cores',
    category: 'shuffle',
    severity: 'warning',
    description: 'Shuffle partitions should be 2-3x total cores',
    check: (config) => {
      const totalCores = config.executor?.cores * (config.executor?.instances ?? 1);
      return config.shuffle?.partitions >= totalCores * 2;
    },
    message: (config) => {
      const totalCores = config.executor?.cores * (config.executor?.instances ?? 1);
      return `Shuffle partitions ${config.shuffle?.partitions} should be at least ${totalCores * 2} (2x cores)`;
    },
  },
  {
    id: 'shuf-003',
    name: 'Shuffle Compression',
    category: 'shuffle',
    severity: 'info',
    description: 'Shuffle compression should be enabled',
    check: (config) => config.shuffle?.compress === true,
    message: () => 'Enable shuffle compression to reduce I/O',
  },

  // GPU Rules
  {
    id: 'gpu-001',
    name: 'GPU Amount Positive',
    category: 'gpu',
    severity: 'error',
    description: 'GPU amount must be positive',
    check: (config) => !config.gpu?.enabled || (config.gpu?.amount ?? 1) > 0,
    message: (config) => `GPU amount ${config.gpu?.amount} must be positive`,
  },
  {
    id: 'gpu-002',
    name: 'RAPIDS Memory Fraction',
    category: 'gpu',
    severity: 'error',
    description: 'RAPIDS memory fraction must be between 0 and 1',
    check: (config) => {
      if (!config.gpu?.rapids?.enabled) return true;
      const frac = config.gpu.rapids.memoryFraction;
      return frac === undefined || (frac >= 0 && frac <= 1);
    },
    message: (config) => `RAPIDS memory fraction ${config.gpu?.rapids?.memoryFraction} must be between 0 and 1`,
  },
  {
    id: 'gpu-003',
    name: 'RAPIDS Enabled with GPU',
    category: 'gpu',
    severity: 'info',
    description: 'Enable RAPIDS when using GPUs',
    check: (config) => !config.gpu?.enabled || config.gpu?.rapids?.enabled,
    message: () => 'Enable RAPIDS for GPU-accelerated operations',
  },

  // Optimization Rules
  {
    id: 'opt-001',
    name: 'Kryo Serialization',
    category: 'optimization',
    severity: 'warning',
    description: 'Use Kryo serialization for better performance',
    check: (config) => config.serializer?.includes('Kryo'),
    message: () => 'Use Kryo serialization instead of Java serialization',
  },
  {
    id: 'opt-002',
    name: 'Adaptive Execution',
    category: 'optimization',
    severity: 'info',
    description: 'Enable Adaptive Query Execution',
    check: (config) => config.optimization?.adaptiveExecution?.enabled === true,
    message: () => 'Enable Adaptive Query Execution for runtime optimizations',
  },
  {
    id: 'opt-003',
    name: 'Off-Heap Memory',
    category: 'optimization',
    severity: 'info',
    description: 'Enable off-heap memory for large executors',
    check: (config) => {
      const execMem = parseMemory(config.executor?.memory);
      return execMem < 16 || config.memory?.offHeap?.enabled === true;
    },
    message: () => 'Enable off-heap memory for executors with >16GB memory',
  },
  {
    id: 'opt-004',
    name: 'Speculation',
    category: 'optimization',
    severity: 'info',
    description: 'Enable speculation to handle slow tasks',
    check: (config) => config.speculation?.enabled === true,
    message: () => 'Enable speculation to mitigate stragglers',
  },
];

/**
 * Helper to parse memory
 */
function parseMemory(memory?: string): number {
  if (!memory) return 0;

  const match = memory.match(/^(\d+)([gmk])$/i);
  if (!match || !match[1] || !match[2]) return 0;

  const value = parseInt(match[1], 10);
  const unit = match[2].toLowerCase();

  switch (unit) {
    case 'g': return value;
    case 'm': return value / 1024;
    case 'k': return value / (1024 * 1024);
    default: return value;
  }
}
