/**
 * Executor resource calculation for Spark
 */

export interface ExecutorResourceRequest {
  totalMemory: number; // GB
  totalCores: number;
  gpuCount?: number;
  workloadType?: string;
  constraints?: {
    maxExecutorMemory?: number;
    maxExecutorCores?: number;
    minExecutors?: number;
    maxExecutors?: number;
  };
}

export interface ExecutorResourceResult {
  executorMemoryGB: number;
  executorCores: number;
  executorCount: number;
  executorsPerNode?: number;
  gpuPerExecutor?: number;
  totalExecutorMemoryGB: number;
  totalExecutorCores: number;
  parallelism: number;
  rationale: string[];
}

/**
 * Calculate optimal executor resources
 */
export async function calculateExecutorResources(
  request: ExecutorResourceRequest
): Promise<ExecutorResourceResult> {
  const rationale: string[] = [];

  // Determine optimal cores per executor
  // Best practice: 4-6 cores per executor for non-GPU workloads
  // For GPU workloads: match cores to GPU count
  let executorCores: number;
  let gpuPerExecutor: number | undefined;

  if (request.gpuCount && request.gpuCount > 0) {
    // GPU workload
    if (request.workloadType === 'ml-training' || request.workloadType === 'ml-inference') {
      // Typically 1 GPU per executor with 4-8 cores
      gpuPerExecutor = 1;
      executorCores = Math.min(8, Math.floor(request.totalCores / request.gpuCount));
      rationale.push(
        `GPU workload: Allocating ${executorCores} cores per GPU for optimal data feeding.`
      );
    } else {
      gpuPerExecutor = 1;
      executorCores = 5;
      rationale.push('GPU-accelerated workload with standard core allocation.');
    }
  } else {
    // CPU-only workload
    executorCores = request.workloadType === 'streaming' ? 4 : 5;
    rationale.push(
      `CPU workload: Using ${executorCores} cores per executor for balanced performance.`
    );
  }

  // Apply constraints
  if (request.constraints?.maxExecutorCores) {
    executorCores = Math.min(executorCores, request.constraints.maxExecutorCores);
    rationale.push(`Limited to ${executorCores} cores per constraint.`);
  }

  // Calculate number of executors
  let executorCount: number;

  if (gpuPerExecutor && request.gpuCount) {
    // GPU-based calculation
    executorCount = Math.floor(request.gpuCount / gpuPerExecutor);
    rationale.push(`${executorCount} executors based on ${request.gpuCount} GPUs available.`);
  } else {
    // CPU-based calculation
    // Reserve 1 core for driver
    const availableCores = Math.max(1, request.totalCores - 1);
    executorCount = Math.floor(availableCores / executorCores);
    rationale.push(
      `${executorCount} executors calculated from ${availableCores} available cores.`
    );
  }

  // Apply executor count constraints
  if (request.constraints?.minExecutors) {
    executorCount = Math.max(executorCount, request.constraints.minExecutors);
  }
  if (request.constraints?.maxExecutors) {
    executorCount = Math.min(executorCount, request.constraints.maxExecutors);
  }

  // Ensure at least 1 executor
  executorCount = Math.max(1, executorCount);

  // Calculate memory per executor
  // Reserve ~1GB for driver, distribute rest among executors
  const driverMemoryGB = Math.min(8, request.totalMemory * 0.1);
  const availableMemoryGB = request.totalMemory - driverMemoryGB;
  let executorMemoryGB = Math.floor(availableMemoryGB / executorCount);

  // Apply memory constraints
  if (request.constraints?.maxExecutorMemory) {
    executorMemoryGB = Math.min(executorMemoryGB, request.constraints.maxExecutorMemory);
  }

  // Best practice: Executor memory should be 4GB - 64GB
  if (executorMemoryGB > 64) {
    rationale.push(
      'Warning: Executor memory >64GB may cause GC issues. Consider increasing executor count.'
    );
  } else if (executorMemoryGB < 4) {
    rationale.push(
      'Warning: Executor memory <4GB may be insufficient. Consider reducing executor count.'
    );
  }

  // Memory per executor should account for overhead
  // Actual usable memory is ~90% of allocated
  const memoryOverheadFactor = 0.9;
  executorMemoryGB = Math.floor(executorMemoryGB * memoryOverheadFactor);

  // Calculate total resources
  const totalExecutorMemoryGB = executorMemoryGB * executorCount;
  const totalExecutorCores = executorCores * executorCount;

  // Parallelism recommendation: 2-3x number of cores
  const parallelism = totalExecutorCores * 2;

  // Calculate executors per node (if applicable)
  // This is a rough estimate - actual deployment may vary
  const executorsPerNode = executorCount >= 4 ? Math.ceil(executorCount / 4) : executorCount;

  rationale.push(
    `Total resources: ${totalExecutorMemoryGB}GB memory, ${totalExecutorCores} cores across ${executorCount} executors.`
  );
  rationale.push(`Recommended parallelism: ${parallelism} (2x cores).`);

  return {
    executorMemoryGB,
    executorCores,
    executorCount,
    executorsPerNode,
    gpuPerExecutor,
    totalExecutorMemoryGB,
    totalExecutorCores,
    parallelism,
    rationale,
  };
}

/**
 * Calculate dynamic allocation parameters
 */
export async function calculateDynamicAllocation(
  baseExecutorCount: number,
  workloadType?: string
): Promise<{
  initialExecutors: number;
  minExecutors: number;
  maxExecutors: number;
  recommendations: string[];
}> {
  const recommendations: string[] = [];

  // Start with fewer executors, scale up as needed
  const initialExecutors = Math.max(1, Math.floor(baseExecutorCount * 0.3));
  const minExecutors = Math.max(1, Math.floor(baseExecutorCount * 0.2));
  let maxExecutors = Math.ceil(baseExecutorCount * 1.5);

  if (workloadType === 'streaming') {
    // Streaming jobs need more stable allocation
    recommendations.push('Streaming workload: Using narrower dynamic allocation range.');
    return {
      initialExecutors: baseExecutorCount,
      minExecutors: Math.floor(baseExecutorCount * 0.8),
      maxExecutors: Math.ceil(baseExecutorCount * 1.2),
      recommendations,
    };
  }

  recommendations.push(
    `Dynamic allocation: Starting with ${initialExecutors} executors, scaling between ${minExecutors}-${maxExecutors}.`
  );

  return {
    initialExecutors,
    minExecutors,
    maxExecutors,
    recommendations,
  };
}

/**
 * Calculate optimal partition count
 */
export function calculatePartitionCount(
  dataSizeGB: number,
  executorCount: number,
  executorCores: number
): number {
  // Target partition size: 128-256MB
  const targetPartitionSizeMB = 128;
  const dataSizeMB = dataSizeGB * 1024;

  const partitionsFromSize = Math.ceil(dataSizeMB / targetPartitionSizeMB);

  // Also consider parallelism (2-3x total cores)
  const totalCores = executorCount * executorCores;
  const partitionsFromCores = totalCores * 3;

  // Use the larger of the two
  return Math.max(partitionsFromSize, partitionsFromCores);
}
