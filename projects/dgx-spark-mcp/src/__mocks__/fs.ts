/**
 * Mock fs module for testing
 */

export const mockFileSystem: Map<string, string> = new Map([
  ['/proc/meminfo', `MemTotal:       1048576000 kB
MemFree:        1006632296 kB
MemAvailable:   1006632296 kB
Buffers:              0 kB
Cached:               0 kB
SwapCached:           0 kB
SwapTotal:            0 kB
SwapFree:             0 kB`],
  ['/proc/cpuinfo', `processor	: 0
vendor_id	: AuthenticAMD
cpu family	: 23
model		: 49
model name	: AMD EPYC 7742 64-Core Processor
stepping	: 0
microcode	: 0x8301055
cpu MHz		: 2250.000
cache size	: 512 KB`],
]);

export const promises = {
  async readFile(path: string, _encoding: string): Promise<string> {
    const content = mockFileSystem.get(path);
    if (!content) {
      throw new Error(`ENOENT: no such file or directory, open '${path}'`);
    }
    return content;
  },

  async readdir(path: string): Promise<string[]> {
    if (path === '/sys/class/net') {
      return ['lo', 'enp1s0f0', 'ib0'];
    }
    return [];
  },

  async stat(path: string): Promise<any> {
    if (mockFileSystem.has(path)) {
      return {
        isFile: () => true,
        isDirectory: () => false,
        size: mockFileSystem.get(path)!.length,
      };
    }
    throw new Error(`ENOENT: no such file or directory, stat '${path}'`);
  },

  async access(path: string): Promise<void> {
    if (!mockFileSystem.has(path)) {
      throw new Error(`ENOENT: no such file or directory, access '${path}'`);
    }
  },
};

export function readFileSync(path: string, _encoding: string): string {
  const content = mockFileSystem.get(path);
  if (!content) {
    throw new Error(`ENOENT: no such file or directory, open '${path}'`);
  }
  return content;
}

export function existsSync(path: string): boolean {
  return mockFileSystem.has(path);
}

export default {
  promises,
  readFileSync,
  existsSync,
};
