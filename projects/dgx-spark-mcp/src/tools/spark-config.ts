/**
 * Spark configuration generator tool
 * Generates optimal Spark configurations based on workload
 */

import { generateConfig, configToSparkSubmitArgs } from '../optimizers/spark.js';
import { getHardwareSnapshot } from '../hardware/topology.js';
import type { GetOptimalSparkConfigArgs, ToolCallResponse } from '../types/tools.js';
import type { SparkConfigRequest, WorkloadType } from '../types/spark-config.js';

/**
 * Get optimal Spark configuration
 */
export async function getOptimalSparkConfig(args: GetOptimalSparkConfigArgs): Promise<ToolCallResponse> {
  try {
    // Get hardware snapshot
    const snapshot = await getHardwareSnapshot({ useCache: true });
    const { topology } = snapshot;

    // Calculate total memory and cores
    const totalMemoryGB = Math.round(topology.memory.info.total / (1024 * 1024 * 1024));
    const totalCores = topology.cpu.cores.physical;
    const gpuCount = topology.gpus?.length || 0;

    // Map workload type from tool args to Spark config type
    const workloadTypeMap: Record<string, WorkloadType> = {
      'etl': 'etl',
      'ml-training': 'ml-training',
      'ml-inference': 'ml-inference',
      'analytics': 'analytics',
      'streaming': 'streaming',
    };

    const workloadType = workloadTypeMap[args.workloadType] || 'etl';

    // Build Spark config request
    const request: SparkConfigRequest = {
      workloadType,
      dataSize: args.dataSize,
      totalMemory: totalMemoryGB,
      totalCores,
      gpuCount: args.useGPU ? gpuCount : 0,
      constraints: {
        maxExecutors: args.numExecutors,
        maxExecutorMemory: args.executorMemory ? parseMemoryString(args.executorMemory) : undefined,
        enableGPU: args.useGPU,
      },
    };

    // Generate optimal configuration
    const result = await generateConfig(request);

    // Convert to spark-submit arguments
    const sparkSubmitArgs = configToSparkSubmitArgs(result.config);

    // Build response
    const response = {
      configuration: result.config,
      sparkSubmitCommand: buildSparkSubmitCommand(sparkSubmitArgs),
      estimatedPerformance: result.estimatedPerformance,
      rationale: result.rationale,
      systemContext: {
        totalMemoryGB,
        totalCores,
        gpuCount,
        gpuEnabled: result.config.gpu?.enabled || false,
      },
      alternatives: result.alternatives,
      howToUse: {
        description: 'Use the spark-submit command below to run your job with these optimized settings',
        example: `spark-submit ${sparkSubmitArgs.slice(0, 6).join(' \\\n  ')} ... your-app.jar`,
        configFile: 'You can also save these settings to spark-defaults.conf',
      },
    };

    return {
      content: [{
        type: 'text',
        text: JSON.stringify(response, null, 2),
      }],
    };
  } catch (error) {
    return {
      content: [{
        type: 'text',
        text: JSON.stringify({
          error: 'Failed to generate Spark configuration',
          message: error instanceof Error ? error.message : 'Unknown error',
        }, null, 2),
      }],
      isError: true,
    };
  }
}

/**
 * Build spark-submit command string
 */
function buildSparkSubmitCommand(args: string[]): string {
  return `spark-submit \\\n  ${args.join(' \\\n  ')}`;
}

/**
 * Parse memory string to number (GB)
 */
function parseMemoryString(memory: string): number {
  const match = memory.match(/^(\d+)([gmk])$/i);
  if (!match || !match[1] || !match[2]) {
    throw new Error(`Invalid memory format: ${memory}`);
  }

  const value = parseInt(match[1], 10);
  const unit = match[2].toLowerCase();

  switch (unit) {
    case 'g': return value;
    case 'm': return value / 1024;
    case 'k': return value / (1024 * 1024);
    default: return value;
  }
}
