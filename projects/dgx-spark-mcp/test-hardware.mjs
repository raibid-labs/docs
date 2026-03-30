#!/usr/bin/env node
/**
 * Simple hardware detection test script
 */

import { getHardwareSummary, detectCPU, detectMemory, detectStorage, detectNetwork } from './dist/hardware/index.js';

async function main() {
  console.log('=== Hardware Detection Test ===\n');

  try {
    console.log('1. Testing CPU Detection...');
    const cpuResult = await detectCPU();
    console.log(`   CPU: ${cpuResult.cpu.modelName}`);
    console.log(`   Cores: ${cpuResult.cpu.cores.physical} physical, ${cpuResult.cpu.cores.logical} logical`);
    console.log(`   ✓ CPU detection successful\n`);

    console.log('2. Testing Memory Detection...');
    const memoryResult = await detectMemory(false);
    const totalGB = Math.round(memoryResult.memory.info.total / (1024 * 1024 * 1024));
    const availableGB = Math.round(memoryResult.memory.info.available / (1024 * 1024 * 1024));
    console.log(`   Total RAM: ${totalGB} GB`);
    console.log(`   Available RAM: ${availableGB} GB`);
    console.log(`   ✓ Memory detection successful\n`);

    console.log('3. Testing Storage Detection...');
    const storageResult = await detectStorage(false, false);
    const totalStorageGB = Math.round(storageResult.storage.totalCapacity / (1024 * 1024 * 1024));
    const availableStorageGB = Math.round(storageResult.storage.totalAvailable / (1024 * 1024 * 1024));
    console.log(`   Total Storage: ${totalStorageGB} GB`);
    console.log(`   Available Storage: ${availableStorageGB} GB`);
    console.log(`   Mount Points: ${storageResult.storage.mountPoints.length}`);
    console.log(`   ✓ Storage detection successful\n`);

    console.log('4. Testing Network Detection...');
    const networkResult = await detectNetwork(false);
    console.log(`   Total Interfaces: ${networkResult.network.totalInterfaces}`);
    console.log(`   Active Interfaces: ${networkResult.network.activeInterfaces}`);
    console.log(`   ✓ Network detection successful\n`);

    console.log('5. Testing Complete Hardware Summary...');
    const summary = await getHardwareSummary();
    console.log('   System Summary:');
    console.log(`   - Hostname: ${summary.hostname}`);
    console.log(`   - CPU: ${summary.cpu}`);
    console.log(`   - Memory: ${summary.memoryGB} GB`);
    console.log(`   - Storage: ${summary.storageGB} GB`);
    console.log(`   - GPUs: ${summary.gpuCount}`);
    console.log(`   - Network Interfaces: ${summary.networkInterfaces}`);
    console.log(`   ✓ Summary generation successful\n`);

    console.log('=== All Hardware Detection Tests Passed ===');
  } catch (error) {
    console.error('Error during hardware detection:', error);
    process.exit(1);
  }
}

main();
