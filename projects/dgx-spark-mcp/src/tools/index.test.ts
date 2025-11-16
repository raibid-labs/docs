/**
 * Unit tests for tools registry
 */

import { describe, it, expect } from '@jest/globals';
import { listAllTools, callTool, isValidToolName } from './index.js';

describe('Tools Registry', () => {
  describe('listAllTools', () => {
    it('should return all tool descriptors', () => {
      const tools = listAllTools();

      expect(tools).toHaveLength(5);
      expect(tools.map(t => t.name)).toEqual([
        'check_gpu_availability',
        'get_optimal_spark_config',
        'search_documentation',
        'estimate_resources',
        'get_system_health',
      ]);
    });

    it('should include required properties for each tool', () => {
      const tools = listAllTools();

      tools.forEach(tool => {
        expect(tool).toHaveProperty('name');
        expect(tool).toHaveProperty('description');
        expect(tool).toHaveProperty('inputSchema');
        expect(typeof tool.name).toBe('string');
        expect(typeof tool.description).toBe('string');
        expect(typeof tool.inputSchema).toBe('object');
      });
    });

    it('should have valid input schemas', () => {
      const tools = listAllTools();

      tools.forEach(tool => {
        expect(tool.inputSchema).toHaveProperty('type');
        expect(tool.inputSchema.type).toBe('object');
        expect(tool.inputSchema).toHaveProperty('properties');
      });
    });
  });

  describe('isValidToolName', () => {
    it('should return true for valid tool names', () => {
      expect(isValidToolName('check_gpu_availability')).toBe(true);
      expect(isValidToolName('get_optimal_spark_config')).toBe(true);
      expect(isValidToolName('search_documentation')).toBe(true);
      expect(isValidToolName('estimate_resources')).toBe(true);
      expect(isValidToolName('get_system_health')).toBe(true);
    });

    it('should return false for invalid tool names', () => {
      expect(isValidToolName('invalid_tool')).toBe(false);
      expect(isValidToolName('')).toBe(false);
      expect(isValidToolName('random_name')).toBe(false);
    });
  });

  describe('callTool', () => {
    it('should return error for unknown tool', async () => {
      const response = await callTool('unknown_tool', {});

      expect(response.isError).toBe(true);
      expect(response.content[0]!.type).toBe('text');
      const text = JSON.parse(response.content[0]!.text);
      expect(text.error).toContain('Unknown tool');
      expect(text.availableTools).toBeDefined();
    });

    it('should return error for invalid arguments', async () => {
      // get_optimal_spark_config requires workloadType and dataSize
      const response = await callTool('get_optimal_spark_config', {});

      expect(response.isError).toBe(true);
      const text = JSON.parse(response.content[0]!.text);
      expect(text.error).toBe('Invalid tool arguments');
    });

    it('should validate required fields', async () => {
      const response = await callTool('search_documentation', {
        // Missing required 'query' field
        limit: 10,
      });

      expect(response.isError).toBe(true);
      const text = JSON.parse(response.content[0]!.text);
      expect(text.error).toBe('Invalid tool arguments');
    });

    it('should validate enum values', async () => {
      const response = await callTool('get_optimal_spark_config', {
        workloadType: 'invalid-type',
        dataSize: '100GB',
      });

      expect(response.isError).toBe(true);
    });

    it('should validate numeric constraints', async () => {
      const response = await callTool('search_documentation', {
        query: 'test',
        limit: 100, // max is 50
      });

      expect(response.isError).toBe(true);
    });
  });

  describe('Tool Schemas', () => {
    it('check_gpu_availability should have correct schema', () => {
      const tool = listAllTools().find(t => t.name === 'check_gpu_availability')!;

      expect(tool.inputSchema.properties).toHaveProperty('minMemoryGB');
      expect(tool.inputSchema.properties).toHaveProperty('minUtilization');
      expect(tool.inputSchema.properties!.minUtilization).toHaveProperty('minimum', 0);
      expect(tool.inputSchema.properties!.minUtilization).toHaveProperty('maximum', 100);
    });

    it('get_optimal_spark_config should have correct schema', () => {
      const tool = listAllTools().find(t => t.name === 'get_optimal_spark_config')!;

      expect(tool.inputSchema.required).toEqual(['workloadType', 'dataSize']);
      expect(tool.inputSchema.properties!.workloadType).toHaveProperty('enum');
      expect((tool.inputSchema.properties!.workloadType as any).enum).toContain('ml-training');
      expect((tool.inputSchema.properties!.workloadType as any).enum).toContain('analytics');
    });

    it('search_documentation should have correct schema', () => {
      const tool = listAllTools().find(t => t.name === 'search_documentation')!;

      expect(tool.inputSchema.required).toEqual(['query']);
      expect(tool.inputSchema.properties!.limit).toHaveProperty('minimum', 1);
      expect(tool.inputSchema.properties!.limit).toHaveProperty('maximum', 50);
    });

    it('estimate_resources should have correct schema', () => {
      const tool = listAllTools().find(t => t.name === 'estimate_resources')!;

      expect(tool.inputSchema.required).toEqual(['description']);
      expect(tool.inputSchema.properties!.computeType).toHaveProperty('enum');
      expect((tool.inputSchema.properties!.computeType as any).enum).toEqual(['cpu', 'gpu', 'mixed']);
    });

    it('get_system_health should have correct schema', () => {
      const tool = listAllTools().find(t => t.name === 'get_system_health')!;

      expect(tool.inputSchema.properties).toHaveProperty('verbose');
      expect(tool.inputSchema.properties!.verbose).toHaveProperty('type', 'boolean');
    });
  });
});
