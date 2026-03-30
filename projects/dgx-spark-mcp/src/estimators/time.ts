/**
 * Execution time prediction for Spark jobs
 */

export interface TimeEstimationRequest {
  dataSize: string | number;
  operations: number | string[];
  hardware: {
    cpuCores: number;
    totalMemory?: number;
    gpuCount?: number;
  };
  workloadType?: 'etl' | 'analytics' | 'ml-training' | 'ml-inference' | 'streaming';
  historicalData?: HistoricalExecution[];
}

export interface HistoricalExecution {
  dataSize: number;
  executionTimeMs: number;
  cpuCores: number;
  gpuCount?: number;
  workloadType?: string;
}

export interface TimeEstimationResult {
  estimatedMinutes: number;
  estimatedSeconds: number;
  range: {
    minMinutes: number;
    maxMinutes: number;
  };
  breakdown: {
    dataLoadingMinutes: number;
    computationMinutes: number;
    shuffleMinutes: number;
    outputMinutes: number;
  };
  scalingPrediction: {
    with2xCores: number;
    with4xCores: number;
    with2xData: number;
  };
  confidence: number;
}

/**
 * Predict execution time for Spark job
 */
export async function predictExecutionTime(
  request: TimeEstimationRequest
): Promise<TimeEstimationResult> {
  // Parse data size
  const dataSizeBytes = typeof request.dataSize === 'number'
    ? request.dataSize
    : parseDataSize(request.dataSize);

  const dataSizeGB = dataSizeBytes / (1024 ** 3);

  // Use historical data if available
  if (request.historicalData && request.historicalData.length > 0) {
    return predictFromHistorical(
      dataSizeGB,
      request.hardware.cpuCores,
      request.historicalData,
      request.workloadType
    );
  }

  // Use model-based prediction
  return predictFromModel(
    dataSizeGB,
    request.operations,
    request.hardware,
    request.workloadType
  );
}

/**
 * Predict from historical data
 */
function predictFromHistorical(
  dataSizeGB: number,
  cpuCores: number,
  historicalData: HistoricalExecution[],
  workloadType?: string
): TimeEstimationResult {
  // Filter for similar workload type
  const relevantData = workloadType
    ? historicalData.filter(h => h.workloadType === workloadType)
    : historicalData;

  if (relevantData.length === 0) {
    // Fall back to model-based prediction
    return predictFromModel(dataSizeGB, 10, { cpuCores }, workloadType);
  }

  // Find closest data size match
  const closest = relevantData.reduce((prev, curr) => {
    const prevDiff = Math.abs((prev.dataSize / (1024 ** 3)) - dataSizeGB);
    const currDiff = Math.abs((curr.dataSize / (1024 ** 3)) - dataSizeGB);
    return currDiff < prevDiff ? curr : prev;
  });

  const closestDataSizeGB = closest.dataSize / (1024 ** 3);
  const baseTimeMinutes = closest.executionTimeMs / 60000;

  // Scale based on data size (linear assumption)
  const dataSizeRatio = dataSizeGB / closestDataSizeGB;
  const scaledTimeMinutes = baseTimeMinutes * dataSizeRatio;

  // Adjust for CPU cores (sub-linear scaling)
  const coreRatio = closest.cpuCores / cpuCores;
  const coreScalingFactor = Math.pow(coreRatio, 0.7); // Amdahl's law approximation

  const estimatedMinutes = scaledTimeMinutes * coreScalingFactor;

  // Estimate breakdown (rough approximation)
  const breakdown = {
    dataLoadingMinutes: estimatedMinutes * 0.2,
    computationMinutes: estimatedMinutes * 0.5,
    shuffleMinutes: estimatedMinutes * 0.2,
    outputMinutes: estimatedMinutes * 0.1,
  };

  // Scaling predictions
  const scalingPrediction = {
    with2xCores: estimatedMinutes * Math.pow(0.5, 0.7),
    with4xCores: estimatedMinutes * Math.pow(0.25, 0.7),
    with2xData: estimatedMinutes * 2,
  };

  return {
    estimatedMinutes,
    estimatedSeconds: estimatedMinutes * 60,
    range: {
      minMinutes: estimatedMinutes * 0.8,
      maxMinutes: estimatedMinutes * 1.2,
    },
    breakdown,
    scalingPrediction,
    confidence: 0.85, // Higher confidence with historical data
  };
}

/**
 * Predict from model
 */
function predictFromModel(
  dataSizeGB: number,
  operations: number | string[],
  hardware: { cpuCores: number; totalMemory?: number; gpuCount?: number },
  workloadType?: string
): TimeEstimationResult {
  const operationCount = Array.isArray(operations) ? operations.length : operations;

  // Base throughput per core in GB/min
  let throughputPerCore = 2; // Conservative default

  // Adjust for workload type
  switch (workloadType) {
    case 'ml-training':
      throughputPerCore = 0.5; // Compute-intensive
      break;
    case 'ml-inference':
      throughputPerCore = 1.5;
      break;
    case 'analytics':
      throughputPerCore = 1.0; // Lots of shuffling
      break;
    case 'etl':
      throughputPerCore = 3.0; // I/O optimized
      break;
    case 'streaming':
      throughputPerCore = 2.5;
      break;
  }

  // GPU acceleration
  if (hardware.gpuCount && hardware.gpuCount > 0) {
    if (workloadType === 'ml-training' || workloadType === 'ml-inference') {
      throughputPerCore *= 4; // Significant speedup for ML workloads
    } else if (workloadType === 'analytics') {
      throughputPerCore *= 2; // RAPIDS acceleration
    }
  }

  // Adjust for operation complexity
  const complexityFactor = 1 + (operationCount * 0.1);
  const adjustedThroughput = throughputPerCore / complexityFactor;

  const totalThroughput = adjustedThroughput * hardware.cpuCores;

  // Base computation time
  const computationMinutes = dataSizeGB / totalThroughput;

  // Breakdown components
  const dataLoadingMinutes = (dataSizeGB / (hardware.cpuCores * 5)) * 0.5; // I/O bound
  const shuffleMinutes = operationCount > 5 ? computationMinutes * 0.3 : computationMinutes * 0.1;
  const outputMinutes = (dataSizeGB / (hardware.cpuCores * 8)) * 0.3; // Write is faster than compute

  const estimatedMinutes = dataLoadingMinutes + computationMinutes + shuffleMinutes + outputMinutes;

  // Scaling predictions
  const scalingPrediction = {
    with2xCores: estimatedMinutes * Math.pow(0.5, 0.75), // Sub-linear scaling
    with4xCores: estimatedMinutes * Math.pow(0.25, 0.7),
    with2xData: estimatedMinutes * 2,
  };

  return {
    estimatedMinutes,
    estimatedSeconds: estimatedMinutes * 60,
    range: {
      minMinutes: estimatedMinutes * 0.7, // Wider range without historical data
      maxMinutes: estimatedMinutes * 1.5,
    },
    breakdown: {
      dataLoadingMinutes,
      computationMinutes,
      shuffleMinutes,
      outputMinutes,
    },
    scalingPrediction,
    confidence: 0.6, // Lower confidence without historical data
  };
}

/**
 * Parse data size string
 */
function parseDataSize(size: string): number {
  const units: Record<string, number> = {
    b: 1,
    kb: 1024,
    mb: 1024 ** 2,
    gb: 1024 ** 3,
    tb: 1024 ** 4,
  };

  const match = size.toLowerCase().match(/^(\d+(?:\.\d+)?)\s*([kmgt]?b)$/);
  if (!match || !match[1] || !match[2]) {
    throw new Error(`Invalid size format: ${size}`);
  }

  const value = match[1];
  const unit = match[2];
  return parseFloat(value) * (units[unit] || 1);
}
