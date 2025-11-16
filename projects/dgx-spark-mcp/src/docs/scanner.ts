/**
 * Document Scanner
 * Scans filesystem for markdown documentation files
 * DGX-Spark MCP Server - Workstream 4
 */

import * as fs from 'fs/promises';
import * as path from 'path';
import { ScanResult, ScanError } from '../types/docs.js';

/**
 * Scan directory for markdown files
 */
export async function scanDirectory(dirPath: string, recursive: boolean = true): Promise<ScanResult> {
  const startTime = Date.now();
  const files: string[] = [];
  const errors: ScanError[] = [];
  let totalSize = 0;

  try {
    await scanDirectoryRecursive(dirPath, recursive, files, errors);

    // Get file sizes
    for (const file of files) {
      try {
        const stats = await fs.stat(file);
        totalSize += stats.size;
      } catch (error) {
        errors.push({
          filePath: file,
          error: `Failed to stat file: ${error}`,
        });
      }
    }
  } catch (error) {
    errors.push({
      filePath: dirPath,
      error: `Failed to scan directory: ${error}`,
    });
  }

  const scanDuration = Date.now() - startTime;

  return {
    files,
    totalSize,
    scanDuration,
    errors,
  };
}

/**
 * Recursive directory scanning helper
 */
async function scanDirectoryRecursive(
  dirPath: string,
  recursive: boolean,
  files: string[],
  errors: ScanError[]
): Promise<void> {
  try {
    const entries = await fs.readdir(dirPath, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dirPath, entry.name);

      // Skip hidden files and directories
      if (entry.name.startsWith('.')) {
        continue;
      }

      // Skip node_modules and other common excludes
      if (entry.name === 'node_modules' || entry.name === 'dist' || entry.name === 'build') {
        continue;
      }

      if (entry.isDirectory()) {
        if (recursive) {
          await scanDirectoryRecursive(fullPath, recursive, files, errors);
        }
      } else if (entry.isFile() && isMarkdownFile(entry.name)) {
        files.push(fullPath);
      }
    }
  } catch (error) {
    errors.push({
      filePath: dirPath,
      error: `Failed to read directory: ${error}`,
    });
  }
}

/**
 * Check if file is a markdown file
 */
function isMarkdownFile(filename: string): boolean {
  const ext = path.extname(filename).toLowerCase();
  return ext === '.md' || ext === '.markdown';
}

/**
 * Watch directory for changes
 */
export async function watchDirectory(
  dirPath: string,
  onChange: (filePath: string, eventType: 'add' | 'change' | 'delete') => void
): Promise<void> {
  try {
    const watcher = fs.watch(dirPath, { recursive: true });

    for await (const event of watcher) {
      if (event.filename && isMarkdownFile(event.filename)) {
        const fullPath = path.join(dirPath, event.filename);

        try {
          await fs.access(fullPath);
          onChange(fullPath, event.eventType === 'rename' ? 'add' : 'change');
        } catch {
          onChange(fullPath, 'delete');
        }
      }
    }
  } catch (error) {
    console.error(`Failed to watch directory ${dirPath}:`, error);
  }
}

/**
 * Filter files by pattern
 */
export function filterFiles(files: string[], patterns: string[]): string[] {
  return files.filter(file => {
    const normalized = file.replace(/\\/g, '/');
    return patterns.some(pattern => {
      const regex = new RegExp(pattern.replace(/\*/g, '.*'));
      return regex.test(normalized);
    });
  });
}
