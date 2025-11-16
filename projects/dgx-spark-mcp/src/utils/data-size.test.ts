/**
 * Unit tests for data size parsing utilities
 */

import { describe, it, expect } from '@jest/globals';
import { parseDataSize, formatBytes, bytesToHuman } from './data-size.js';

describe('Data Size Utilities', () => {
  describe('parseDataSize', () => {
    it('should parse bytes', () => {
      expect(parseDataSize('100')).toBe(100);
      expect(parseDataSize('1024')).toBe(1024);
    });

    it('should parse KB', () => {
      expect(parseDataSize('1KB')).toBe(1024);
      expect(parseDataSize('10KB')).toBe(10240);
      expect(parseDataSize('1kb')).toBe(1024);
      expect(parseDataSize('1 KB')).toBe(1024);
    });

    it('should parse MB', () => {
      expect(parseDataSize('1MB')).toBe(1024 * 1024);
      expect(parseDataSize('100MB')).toBe(100 * 1024 * 1024);
      expect(parseDataSize('1.5MB')).toBe(1.5 * 1024 * 1024);
    });

    it('should parse GB', () => {
      expect(parseDataSize('1GB')).toBe(1024 * 1024 * 1024);
      expect(parseDataSize('10GB')).toBe(10 * 1024 * 1024 * 1024);
      expect(parseDataSize('2.5GB')).toBe(2.5 * 1024 * 1024 * 1024);
    });

    it('should parse TB', () => {
      expect(parseDataSize('1TB')).toBe(1024 * 1024 * 1024 * 1024);
      expect(parseDataSize('5TB')).toBe(5 * 1024 * 1024 * 1024 * 1024);
      expect(parseDataSize('1.5TB')).toBe(1.5 * 1024 * 1024 * 1024 * 1024);
    });

    it('should parse PB', () => {
      expect(parseDataSize('1PB')).toBe(1024 * 1024 * 1024 * 1024 * 1024);
    });

    it('should handle decimal values', () => {
      expect(parseDataSize('1.5GB')).toBe(1.5 * 1024 * 1024 * 1024);
      expect(parseDataSize('0.5TB')).toBe(0.5 * 1024 * 1024 * 1024 * 1024);
      expect(parseDataSize('2.25MB')).toBe(2.25 * 1024 * 1024);
    });

    it('should handle spaces', () => {
      expect(parseDataSize('100 GB')).toBe(100 * 1024 * 1024 * 1024);
      expect(parseDataSize('50  MB')).toBe(50 * 1024 * 1024);
    });

    it('should be case insensitive', () => {
      expect(parseDataSize('1gb')).toBe(parseDataSize('1GB'));
      expect(parseDataSize('1Gb')).toBe(parseDataSize('1GB'));
      expect(parseDataSize('1gB')).toBe(parseDataSize('1GB'));
    });

    it('should throw on invalid input', () => {
      expect(() => parseDataSize('invalid')).toThrow();
      expect(() => parseDataSize('ABC')).toThrow();
      expect(() => parseDataSize('')).toThrow();
    });

    it('should throw on negative values', () => {
      expect(() => parseDataSize('-100GB')).toThrow();
      expect(() => parseDataSize('-1MB')).toThrow();
    });
  });

  describe('formatBytes', () => {
    it('should format bytes', () => {
      expect(formatBytes(0)).toBe('0 B');
      expect(formatBytes(100)).toBe('100 B');
      expect(formatBytes(1000)).toBe('1000 B');
    });

    it('should format KB', () => {
      expect(formatBytes(1024)).toBe('1.00 KB');
      expect(formatBytes(10240)).toBe('10.00 KB');
    });

    it('should format MB', () => {
      expect(formatBytes(1024 * 1024)).toBe('1.00 MB');
      expect(formatBytes(100 * 1024 * 1024)).toBe('100.00 MB');
    });

    it('should format GB', () => {
      expect(formatBytes(1024 * 1024 * 1024)).toBe('1.00 GB');
      expect(formatBytes(50 * 1024 * 1024 * 1024)).toBe('50.00 GB');
    });

    it('should format TB', () => {
      expect(formatBytes(1024 * 1024 * 1024 * 1024)).toBe('1.00 TB');
      expect(formatBytes(5 * 1024 * 1024 * 1024 * 1024)).toBe('5.00 TB');
    });

    it('should format PB', () => {
      expect(formatBytes(1024 * 1024 * 1024 * 1024 * 1024)).toBe('1.00 PB');
    });

    it('should handle decimals', () => {
      expect(formatBytes(1536 * 1024 * 1024)).toBe('1.50 GB');
      expect(formatBytes(2.5 * 1024 * 1024 * 1024)).toBe('2.50 GB');
    });

    it('should allow custom decimal places', () => {
      expect(formatBytes(1536 * 1024 * 1024, 1)).toBe('1.5 GB');
      expect(formatBytes(1536 * 1024 * 1024, 3)).toBe('1.500 GB');
    });
  });

  describe('bytesToHuman', () => {
    it('should convert bytes to human readable format', () => {
      expect(bytesToHuman(0)).toBe('0 B');
      expect(bytesToHuman(1024)).toContain('KB');
      expect(bytesToHuman(1024 * 1024)).toContain('MB');
      expect(bytesToHuman(1024 * 1024 * 1024)).toContain('GB');
    });

    it('should round appropriately', () => {
      const result = bytesToHuman(1536 * 1024 * 1024);
      expect(result).toMatch(/1\.5.*GB/);
    });
  });

  describe('Edge Cases', () => {
    it('should handle very large numbers', () => {
      const exabyte = 1024 * 1024 * 1024 * 1024 * 1024 * 1024;
      expect(parseDataSize('1EB')).toBe(exabyte);
    });

    it('should handle very small decimals', () => {
      expect(parseDataSize('0.001GB')).toBe(0.001 * 1024 * 1024 * 1024);
    });

    it('should handle scientific notation', () => {
      const result = parseDataSize('1e9');
      expect(result).toBe(1000000000);
    });
  });

  describe('Round Trip', () => {
    it('should parse and format consistently', () => {
      const sizes = ['1GB', '10MB', '500KB', '2TB'];

      sizes.forEach(size => {
        const bytes = parseDataSize(size);
        const formatted = formatBytes(bytes);
        expect(formatted).toBeTruthy();
      });
    });
  });
});
