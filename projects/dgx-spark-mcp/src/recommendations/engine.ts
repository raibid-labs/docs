/**
 * Recommendation engine for Spark optimization
 */

import { SparkConfig, WorkloadType } from '../types/spark-config.js';
import { HardwareContext } from '../types/estimation.js';
import { checkAntiPatterns } from '../validators/best-practices.js';
import { detectBottlenecks } from '../models/bottleneck.js';
// import { predictPerformance } from '../models/performance.js';

export interface Recommendation {
  id: string;
  category: 'performance' | 'cost' | 'reliability' | 'best-practice';
  priority: 'critical' | 'high' | 'medium' | 'low';
  title: string;
  description: string;
  impact: {
    performanceGain?: string;
    costSavings?: string;
    reliabilityImprovement?: string;
  };
  implementation: {
    difficulty: 'easy' | 'medium' | 'hard';
    steps: string[];
    configChanges?: Partial<SparkConfig>;
  };
  rationale: string;
}

export interface RecommendationSet {
  recommendations: Recommendation[];
  summary: {
    total: number;
    byPriority: Record<string, number>;
    byCategory: Record<string, number>;
    estimatedImpact: string;
  };
  appliedConfig?: SparkConfig;
}

/**
 * Generate recommendations for Spark configuration
 */
export async function generateRecommendations(request: {
  config: SparkConfig;
  hardware: HardwareContext;
  workload: {
    type: WorkloadType;
    dataSize?: number;
  };
}): Promise<RecommendationSet> {
  const { config, hardware, workload } = request;
  const recommendations: Recommendation[] = [];

  // Check for anti-patterns
  const bestPractices = await checkAntiPatterns(config);
  for (const violation of bestPractices.violations) {
    recommendations.push({
      id: `bp-${violation.pattern.toLowerCase().replace(/\s+/g, '-')}`,
      category: violation.category === 'security' ? 'best-practice' : violation.category,
      priority: violation.severity === 'critical' ? 'critical' :
                violation.severity === 'high' ? 'high' :
                violation.severity === 'medium' ? 'medium' : 'low',
      title: `Fix: ${violation.pattern}`,
      description: violation.description,
      impact: {
        performanceGain: violation.category === 'performance' ? 'Moderate to High' : undefined,
        costSavings: violation.category === 'cost' ? 'Moderate' : undefined,
        reliabilityImprovement: violation.category === 'reliability' ? 'High' : undefined,
      },
      implementation: {
        difficulty: violation.autoFix ? 'easy' : 'medium',
        steps: [violation.recommendation],
        configChanges: violation.autoFix,
      },
      rationale: violation.recommendation,
    });
  }

  // Detect bottlenecks
  const bottleneckAnalysis = await detectBottlenecks({
    config,
    hardware,
    workloadType: workload.type,
  });

  for (const bottleneck of bottleneckAnalysis.bottlenecks) {
    if (bottleneck.severity === 'critical' || bottleneck.severity === 'high') {
      recommendations.push({
        id: `bottleneck-${bottleneck.type}`,
        category: 'performance',
        priority: bottleneck.severity as any,
        title: `Address ${bottleneck.type.toUpperCase()} Bottleneck`,
        description: bottleneck.description,
        impact: {
          performanceGain: bottleneck.severity === 'critical' ? 'Very High' : 'High',
        },
        implementation: {
          difficulty: 'medium',
          steps: [bottleneck.suggestedFix],
        },
        rationale: bottleneck.impact,
      });
    }
  }

  // Workload-specific recommendations
  const workloadRecs = generateWorkloadRecommendations(config, workload.type);
  recommendations.push(...workloadRecs);

  // Hardware optimization recommendations
  const hardwareRecs = generateHardwareRecommendations(config, hardware);
  recommendations.push(...hardwareRecs);

  // Calculate summary
  const byPriority: Record<string, number> = {
    critical: 0,
    high: 0,
    medium: 0,
    low: 0,
  };

  const byCategory: Record<string, number> = {
    performance: 0,
    cost: 0,
    reliability: 0,
    'best-practice': 0,
  };

  recommendations.forEach(rec => {
    byPriority[rec.priority] = (byPriority[rec.priority] || 0) + 1;
    byCategory[rec.category] = (byCategory[rec.category] || 0) + 1;
  });

  // Estimate overall impact
  const estimatedImpact =
    (byPriority["critical"] || 0) > 0 ? '50-100% performance improvement possible' :
    (byPriority["high"] || 0) > 0 ? '20-50% performance improvement possible' :
    (byPriority["medium"] || 0) > 0 ? '10-20% performance improvement possible' :
    '5-10% performance improvement possible';

  return {
    recommendations,
    summary: {
      total: recommendations.length,
      byPriority,
      byCategory,
      estimatedImpact,
    },
  };
}

/**
 * Generate workload-specific recommendations
 */
function generateWorkloadRecommendations(
  config: SparkConfig,
  workloadType: WorkloadType
): Recommendation[] {
  const recommendations: Recommendation[] = [];

  switch (workloadType) {
    case 'ml-training':
      if (!config.gpu?.enabled) {
        recommendations.push({
          id: 'ml-gpu',
          category: 'performance',
          priority: 'high',
          title: 'Enable GPU Acceleration for ML Training',
          description: 'ML training workloads benefit significantly from GPU acceleration',
          impact: {
            performanceGain: '3-10x faster training',
          },
          implementation: {
            difficulty: 'medium',
            steps: [
              'Enable GPU in Spark configuration',
              'Configure RAPIDS for GPU-accelerated DataFrames',
              'Ensure training library supports GPUs',
            ],
          },
          rationale: 'ML training is compute-intensive and GPUs provide massive parallelism',
        });
      }

      if (!config.memory.offHeap.enabled) {
        recommendations.push({
          id: 'ml-offheap',
          category: 'performance',
          priority: 'medium',
          title: 'Enable Off-Heap Memory for ML Workloads',
          description: 'Reduce GC pressure during iterative ML algorithms',
          impact: {
            performanceGain: '10-20% faster iterations',
          },
          implementation: {
            difficulty: 'easy',
            steps: ['Enable off-heap memory with 20% of executor memory'],
          },
          rationale: 'Iterative algorithms create memory pressure that benefits from off-heap storage',
        });
      }
      break;

    case 'analytics':
      if (!config.optimization.adaptiveExecution?.skewJoin) {
        recommendations.push({
          id: 'analytics-skew',
          category: 'performance',
          priority: 'high',
          title: 'Enable Skew Join Optimization',
          description: 'Analytical queries often have data skew in joins',
          impact: {
            performanceGain: '20-50% faster for skewed joins',
          },
          implementation: {
            difficulty: 'easy',
            steps: ['Enable AQE skew join optimization'],
          },
          rationale: 'Analytics workloads frequently join on skewed dimensions',
        });
      }
      break;

    case 'streaming':
      if (config.dynamicAllocation.enabled) {
        recommendations.push({
          id: 'streaming-static',
          category: 'reliability',
          priority: 'medium',
          title: 'Use Static Allocation for Streaming',
          description: 'Streaming jobs benefit from predictable resources',
          impact: {
            reliabilityImprovement: 'More stable streaming performance',
          },
          implementation: {
            difficulty: 'easy',
            steps: ['Disable dynamic allocation for streaming jobs'],
          },
          rationale: 'Dynamic allocation can cause latency spikes in streaming',
        });
      }
      break;

    case 'etl':
      if (parseMemory(config.executor.memory) > 32) {
        recommendations.push({
          id: 'etl-smaller-executors',
          category: 'cost',
          priority: 'medium',
          title: 'Use Smaller Executors for ETL',
          description: 'ETL workloads work well with many smaller executors',
          impact: {
            costSavings: '10-20% better resource utilization',
          },
          implementation: {
            difficulty: 'medium',
            steps: [
              'Reduce executor memory to 16-24GB',
              'Increase executor count proportionally',
            ],
          },
          rationale: 'ETL is I/O bound and benefits from more parallelism',
        });
      }
      break;
  }

  return recommendations;
}

/**
 * Generate hardware optimization recommendations
 */
function generateHardwareRecommendations(
  config: SparkConfig,
  hardware: HardwareContext
): Recommendation[] {
  const recommendations: Recommendation[] = [];

  const executorMemGB = parseMemory(config.executor.memory);
  const totalExecutorMem = executorMemGB * (config.executor.instances ?? 1);
  const memUtilization = totalExecutorMem / hardware.totalMemory;

  // Underutilized hardware
  if (memUtilization < 0.5) {
    recommendations.push({
      id: 'hw-underutilized',
      category: 'cost',
      priority: 'medium',
      title: 'Increase Resource Utilization',
      description: `Only using ${(memUtilization * 100).toFixed(0)}% of available memory`,
      impact: {
        performanceGain: '30-50% faster with better resource usage',
        costSavings: 'Better ROI on hardware investment',
      },
      implementation: {
        difficulty: 'easy',
        steps: [
          'Increase executor memory or executor count',
          'Target 70-80% memory utilization',
        ],
      },
      rationale: 'Leaving resources idle reduces throughput',
    });
  }

  // GPU available but not used
  if (hardware.gpuCount && hardware.gpuCount > 0 && !config.gpu?.enabled) {
    recommendations.push({
      id: 'hw-unused-gpu',
      category: 'cost',
      priority: 'low',
      title: 'Consider GPU Acceleration',
      description: `${hardware.gpuCount} GPUs available but not configured`,
      impact: {
        performanceGain: 'Potentially 3-10x for suitable workloads',
      },
      implementation: {
        difficulty: 'medium',
        steps: [
          'Evaluate if workload can benefit from GPU',
          'Enable GPU and RAPIDS if applicable',
        ],
      },
      rationale: 'GPUs provide massive parallel processing for certain operations',
    });
  }

  return recommendations;
}

/**
 * Helper to parse memory
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
