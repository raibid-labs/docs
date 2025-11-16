#!/usr/bin/env node

/**
 * Configuration Validation Script
 * Validates the configuration without starting the server
 */

import { getConfig } from './index.js';

try {
  console.log('Validating configuration...');
  const config = getConfig();
  console.log('Configuration is valid!');
  console.log('\nConfiguration:');
  console.log(JSON.stringify(config, null, 2));
  process.exit(0);
} catch (error) {
  console.error('Configuration validation failed:');
  console.error((error as Error).message);
  process.exit(1);
}
