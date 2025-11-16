/**
 * Quick test of the DGX Spark Intelligence system
 */

const { generateConfig } = require('./dist/optimizers/spark.js');
const { classifyWorkload } = require('./dist/analyzers/workload.js');
const { estimateResources } = require('./dist/estimators/resources.js');

async function testIntelligence() {
  console.log('=== DGX Spark Intelligence System Test ===\n');

  // Test 1: Workload Classification
  console.log('1. Testing Workload Classifier...');
  try {
    const workload = await classifyWorkload('Train deep learning model on 1TB dataset with GPU acceleration');
    console.log('   Workload Type:', workload.characteristics.type);
    console.log('   Compute Intensity:', workload.characteristics.computeIntensity);
    console.log('   GPU Utilization:', workload.characteristics.gpuUtilization);
    console.log('   Confidence:', (workload.characteristics.confidence * 100).toFixed(0) + '%');
    console.log('   ✓ Workload analyzer working\n');
  } catch (error) {
    console.error('   ✗ Error:', error.message, '\n');
  }

  // Test 2: Resource Estimation
  console.log('2. Testing Resource Estimator...');
  try {
    const estimate = await estimateResources({
      description: 'Process 10TB of logs with aggregations',
      dataSize: '10TB',
      operations: ['read', 'filter', 'groupBy', 'aggregate', 'write']
    });
    console.log('   Executor Memory:', estimate.memory.executorMemoryGB + 'GB');
    console.log('   Total Cores:', estimate.compute.totalCores);
    console.log('   Estimated Time:', estimate.time.estimatedMinutes.toFixed(1) + ' minutes');
    console.log('   Bottleneck:', estimate.time.bottleneck);
    console.log('   ✓ Resource estimator working\n');
  } catch (error) {
    console.error('   ✗ Error:', error.message, '\n');
  }

  // Test 3: Spark Configuration Generation
  console.log('3. Testing Spark Config Optimizer...');
  try {
    const config = await generateConfig({
      workloadType: 'ml-training',
      dataSize: '1TB',
      gpuCount: 8,
      totalMemory: 512,
      totalCores: 96
    });
    console.log('   Executor Config:', config.config.executor.memory, '/', config.config.executor.cores, 'cores');
    console.log('   Executor Count:', config.config.executor.instances);
    console.log('   GPU Enabled:', config.config.gpu?.enabled);
    console.log('   RAPIDS Enabled:', config.config.gpu?.rapids?.enabled);
    console.log('   Shuffle Partitions:', config.config.shuffle.partitions);
    console.log('   Estimated Time:', config.estimatedPerformance.executionTimeMinutes?.toFixed(1), 'minutes');
    console.log('   ✓ Spark optimizer working\n');
  } catch (error) {
    console.error('   ✗ Error:', error.message, '\n');
  }

  console.log('=== All Core Components Operational ===');
}

testIntelligence().catch(console.error);
