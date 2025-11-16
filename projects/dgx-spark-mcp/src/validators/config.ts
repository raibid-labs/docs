/**
 * Spark configuration validator
 */

import {
  SparkConfig,
  ConfigValidationResult,
  ConfigValidationError,
  ConfigValidationWarning,
} from '../types/spark-config.js';

/**
 * Validate Spark configuration
 */
export async function validate(config: SparkConfig): Promise<ConfigValidationResult> {
  const errors: ConfigValidationError[] = [];
  const warnings: ConfigValidationWarning[] = [];
  const suggestions: string[] = [];

  // Validate executor configuration
  validateExecutorConfig(config, errors, warnings);

  // Validate driver configuration
  validateDriverConfig(config, errors, warnings);

  // Validate memory configuration
  validateMemoryConfig(config, errors, warnings);

  // Validate shuffle configuration
  validateShuffleConfig(config, errors, warnings);

  // Validate GPU configuration
  if (config.gpu?.enabled) {
    validateGPUConfig(config, errors, warnings);
  }

  // Validate dynamic allocation
  validateDynamicAllocation(config, errors, warnings);

  // Generate suggestions
  generateSuggestions(config, suggestions);

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    suggestions,
  };
}

/**
 * Validate executor configuration
 */
function validateExecutorConfig(
  config: SparkConfig,
  errors: ConfigValidationError[],
  warnings: ConfigValidationWarning[]
): void {
  const executorMemGB = parseMemory(config.executor.memory);

  // Validate memory
  if (executorMemGB < 1) {
    errors.push({
      field: 'executor.memory',
      message: `Executor memory (${config.executor.memory}) is too low`,
      severity: 'error',
      fix: 'Set executor memory to at least 1g',
    });
  } else if (executorMemGB < 4) {
    warnings.push({
      field: 'executor.memory',
      message: `Executor memory (${config.executor.memory}) is quite low`,
      severity: 'warning',
      suggestion: 'Consider at least 4g for production workloads',
    });
  }

  if (executorMemGB > 64) {
    warnings.push({
      field: 'executor.memory',
      message: `Executor memory (${config.executor.memory}) is very high`,
      severity: 'warning',
      suggestion: 'Large executor memory (>64g) can cause GC issues. Use more executors with less memory.',
    });
  }

  // Validate cores
  if (config.executor.cores < 1) {
    errors.push({
      field: 'executor.cores',
      message: 'Executor cores must be at least 1',
      severity: 'error',
      fix: 'Set executor.cores to at least 1',
    });
  } else if (config.executor.cores > 8) {
    warnings.push({
      field: 'executor.cores',
      message: `Executor cores (${config.executor.cores}) exceeds recommended maximum`,
      severity: 'warning',
      suggestion: 'Best practice is 4-6 cores per executor',
    });
  }

  // Validate instances
  if (config.executor.instances !== undefined && config.executor.instances < 1) {
    errors.push({
      field: 'executor.instances',
      message: 'Executor instances must be at least 1',
      severity: 'error',
      fix: 'Set executor.instances to at least 1',
    });
  }
}

/**
 * Validate driver configuration
 */
function validateDriverConfig(
  config: SparkConfig,
  errors: ConfigValidationError[],
  warnings: ConfigValidationWarning[]
): void {
  const driverMemGB = parseMemory(config.driver.memory);
  const executorMemGB = parseMemory(config.executor.memory);

  // Validate memory
  if (driverMemGB < 1) {
    errors.push({
      field: 'driver.memory',
      message: 'Driver memory is too low',
      severity: 'error',
      fix: 'Set driver memory to at least 1g',
    });
  }

  if (driverMemGB > executorMemGB * 4) {
    warnings.push({
      field: 'driver.memory',
      message: 'Driver memory is much larger than executor memory',
      severity: 'warning',
      suggestion: 'Driver memory should typically be 1-2x executor memory',
    });
  }

  // Validate cores
  if (config.driver.cores < 1) {
    errors.push({
      field: 'driver.cores',
      message: 'Driver cores must be at least 1',
      severity: 'error',
      fix: 'Set driver.cores to at least 1',
    });
  } else if (config.driver.cores > 8) {
    warnings.push({
      field: 'driver.cores',
      message: 'Driver typically does not need more than 4-8 cores',
      severity: 'warning',
      suggestion: 'Consider reducing driver cores',
    });
  }
}

/**
 * Validate memory configuration
 */
function validateMemoryConfig(
  config: SparkConfig,
  errors: ConfigValidationError[],
  warnings: ConfigValidationWarning[]
): void {
  // Validate fractions
  if (config.memory.fraction < 0 || config.memory.fraction > 1) {
    errors.push({
      field: 'memory.fraction',
      message: `Memory fraction (${config.memory.fraction}) must be between 0 and 1`,
      severity: 'error',
      fix: 'Set memory.fraction to a value between 0 and 1',
    });
  }

  if (config.memory.storageFraction < 0 || config.memory.storageFraction > 1) {
    errors.push({
      field: 'memory.storageFraction',
      message: `Storage fraction (${config.memory.storageFraction}) must be between 0 and 1`,
      severity: 'error',
      fix: 'Set memory.storageFraction to a value between 0 and 1',
    });
  }

  // Check if fractions are reasonable
  if (config.memory.fraction < 0.4) {
    warnings.push({
      field: 'memory.fraction',
      message: 'Memory fraction is quite low',
      severity: 'warning',
      suggestion: 'Default is 0.6, consider using at least 0.4',
    });
  }

  if (config.memory.fraction + config.memory.storageFraction > 1) {
    warnings.push({
      field: 'memory',
      message: 'Combined memory fractions may lead to spilling',
      severity: 'warning',
      suggestion: 'Ensure memory.fraction * memory.storageFraction leaves enough room',
    });
  }
}

/**
 * Validate shuffle configuration
 */
function validateShuffleConfig(
  config: SparkConfig,
  errors: ConfigValidationError[],
  warnings: ConfigValidationWarning[]
): void {
  const totalCores = config.executor.cores * (config.executor.instances ?? 1);

  if (config.shuffle.partitions < 1) {
    errors.push({
      field: 'shuffle.partitions',
      message: 'Shuffle partitions must be at least 1',
      severity: 'error',
      fix: 'Set shuffle.partitions to at least 1',
    });
  } else if (config.shuffle.partitions < totalCores) {
    warnings.push({
      field: 'shuffle.partitions',
      message: `Shuffle partitions (${config.shuffle.partitions}) less than total cores (${totalCores})`,
      severity: 'warning',
      suggestion: `Increase to at least ${totalCores * 2} for better parallelism`,
    });
  }
}

/**
 * Validate GPU configuration
 */
function validateGPUConfig(
  config: SparkConfig,
  errors: ConfigValidationError[],
  _warnings: ConfigValidationWarning[]
): void {
  if (!config.gpu) return;

  if (config.gpu.amount !== undefined && config.gpu.amount < 0) {
    errors.push({
      field: 'gpu.amount',
      message: 'GPU amount cannot be negative',
      severity: 'error',
      fix: 'Set gpu.amount to at least 1',
    });
  }

  if (config.gpu.rapids?.enabled && !config.gpu.enabled) {
    errors.push({
      field: 'gpu.rapids',
      message: 'RAPIDS enabled but GPU is not enabled',
      severity: 'error',
      fix: 'Enable gpu.enabled when using RAPIDS',
    });
  }

  if (config.gpu.rapids?.memoryFraction !== undefined) {
    if (config.gpu.rapids.memoryFraction < 0 || config.gpu.rapids.memoryFraction > 1) {
      errors.push({
        field: 'gpu.rapids.memoryFraction',
        message: 'RAPIDS memory fraction must be between 0 and 1',
        severity: 'error',
        fix: 'Set rapids.memoryFraction to a value between 0 and 1',
      });
    }
  }
}

/**
 * Validate dynamic allocation
 */
function validateDynamicAllocation(
  config: SparkConfig,
  errors: ConfigValidationError[],
  warnings: ConfigValidationWarning[]
): void {
  if (!config.dynamicAllocation.enabled) return;

  const { minExecutors, maxExecutors, initialExecutors } = config.dynamicAllocation;

  if (minExecutors !== undefined && maxExecutors !== undefined) {
    if (minExecutors > maxExecutors) {
      errors.push({
        field: 'dynamicAllocation',
        message: 'Min executors cannot be greater than max executors',
        severity: 'error',
        fix: `Set minExecutors <= maxExecutors`,
      });
    }
  }

  if (initialExecutors !== undefined && minExecutors !== undefined && maxExecutors !== undefined) {
    if (initialExecutors < minExecutors || initialExecutors > maxExecutors) {
      warnings.push({
        field: 'dynamicAllocation.initialExecutors',
        message: 'Initial executors should be between min and max',
        severity: 'warning',
        suggestion: `Set initialExecutors between ${minExecutors} and ${maxExecutors}`,
      });
    }
  }
}

/**
 * Generate suggestions
 */
function generateSuggestions(config: SparkConfig, suggestions: string[]): void {
  // Serialization suggestion
  if (!config.serializer?.includes('Kryo')) {
    suggestions.push('Use Kryo serialization for better performance');
  }

  // Off-heap memory suggestion
  if (!config.memory.offHeap.enabled) {
    suggestions.push('Enable off-heap memory to reduce GC pressure');
  }

  // Adaptive execution suggestion
  if (!config.optimization.adaptiveExecution?.enabled) {
    suggestions.push('Enable Adaptive Query Execution (AQE) for runtime optimizations');
  }

  // Speculation suggestion
  if (!config.speculation?.enabled) {
    suggestions.push('Enable speculation to handle slow tasks');
  }

  // Compression suggestion
  if (!config.shuffle.compress) {
    suggestions.push('Enable shuffle compression to reduce I/O');
  }
}

/**
 * Parse memory string to GB
 */
function parseMemory(memory: string): number {
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
