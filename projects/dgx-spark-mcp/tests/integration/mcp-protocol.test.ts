/**
 * Integration tests for MCP protocol compliance
 */

import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';
import { createTestServer, TestMCPClient, mockHardwareDetection, cleanupTestEnvironment } from '../helpers/mcp-client.js';

describe('MCP Protocol Compliance', () => {
  let client: TestMCPClient;

  beforeAll(async () => {
    mockHardwareDetection();
    const server = await createTestServer();
    client = new TestMCPClient(server);
  });

  afterAll(() => {
    cleanupTestEnvironment();
  });

  describe('Server Initialization', () => {
    it('should respond to initialization request', async () => {
      const response = await client.getCapabilities();

      expect(response).toBeDefined();
      expect(response).toHaveProperty('protocolVersion');
      expect(response).toHaveProperty('capabilities');
      expect(response).toHaveProperty('serverInfo');
    });

    it('should expose correct capabilities', async () => {
      const response = await client.getCapabilities();

      expect(response.capabilities).toHaveProperty('resources');
      expect(response.capabilities).toHaveProperty('tools');
    });

    it('should have correct server info', async () => {
      const response = await client.getCapabilities();

      expect(response.serverInfo.name).toBe('dgx-spark-mcp-test');
      expect(response.serverInfo.version).toBeDefined();
    });
  });

  describe('Resources Protocol', () => {
    it('should list all resources', async () => {
      const response = await client.listResources();

      expect(response).toHaveProperty('resources');
      expect(Array.isArray(response.resources)).toBe(true);
      expect(response.resources.length).toBeGreaterThan(0);
    });

    it('should include required resource fields', async () => {
      const response = await client.listResources();

      response.resources.forEach((resource: any) => {
        expect(resource).toHaveProperty('uri');
        expect(resource).toHaveProperty('name');
        expect(resource).toHaveProperty('description');
        expect(resource.uri).toMatch(/^dgx:\/\//);
      });
    });

    it('should read a resource by URI', async () => {
      const listResponse = await client.listResources();
      const firstResource = listResponse.resources[0];

      const readResponse = await client.readResource(firstResource.uri);

      expect(readResponse).toHaveProperty('contents');
      expect(Array.isArray(readResponse.contents)).toBe(true);
    });

    it('should return error for invalid resource URI', async () => {
      await expect(
        client.readResource('dgx://invalid/resource')
      ).rejects.toThrow();
    });
  });

  describe('Tools Protocol', () => {
    it('should list all tools', async () => {
      const response = await client.listTools();

      expect(response).toHaveProperty('tools');
      expect(Array.isArray(response.tools)).toBe(true);
      expect(response.tools.length).toBe(5);
    });

    it('should include required tool fields', async () => {
      const response = await client.listTools();

      response.tools.forEach((tool: any) => {
        expect(tool).toHaveProperty('name');
        expect(tool).toHaveProperty('description');
        expect(tool).toHaveProperty('inputSchema');
      });
    });

    it('should call a tool with valid arguments', async () => {
      const response = await client.callTool('get_system_health', {
        verbose: false,
      });

      expect(response).toHaveProperty('content');
      expect(Array.isArray(response.content)).toBe(true);
    });

    it('should return error for unknown tool', async () => {
      const response = await client.callTool('unknown_tool', {});

      expect(response.isError).toBe(true);
    });

    it('should validate tool arguments', async () => {
      const response = await client.callTool('search_documentation', {
        // Missing required 'query' field
        limit: 10,
      });

      expect(response.isError).toBe(true);
    });
  });

  describe('Error Handling', () => {
    it('should handle malformed requests gracefully', async () => {
      await expect(
        client.readResource('not-a-valid-uri')
      ).rejects.toThrow();
    });

    it('should provide meaningful error messages', async () => {
      const response = await client.callTool('unknown_tool', {});

      expect(response.isError).toBe(true);
      const text = JSON.parse(response.content[0].text);
      expect(text).toHaveProperty('error');
      expect(text.error).toContain('Unknown tool');
    });
  });
});
