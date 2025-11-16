#!/bin/bash

# Fix storage.ts
cat > /home/beengud/raibid-labs/dgx-spark-mcp/src/hardware/storage.ts << 'EOF'
/**
 * Storage detection module
 */

import { Storage, StorageDetectionResult, BlockDevice, MountPoint, NVMeDevice, RAIDArray } from '../types/storage.js';
import { executeCommand } from '../utils/exec.js';

export async function detectStorage(includeNVMe: boolean = true, includeRAID: boolean = true): Promise<StorageDetectionResult> {
  const startTime = Date.now();

  const blockDevices = await detectBlockDevices();
  const mountPoints = await detectMountPoints();
  const nvmeDevices = includeNVMe ? await detectNVMeDevices() : undefined;
  const raidArrays = includeRAID ? await detectRAIDArrays() : undefined;

  const { totalCapacity, totalUsed, totalAvailable } = calculateTotals(mountPoints);

  const storage: Storage = {
    blockDevices,
    mountPoints,
    nvmeDevices,
    raidArrays,
    totalCapacity,
    totalUsed,
    totalAvailable,
  };

  return {
    storage,
    timestamp: Date.now(),
    detectionTime: Date.now() - startTime,
  };
}

async function detectBlockDevices(): Promise<BlockDevice[]> {
  const result = await executeCommand('lsblk -b -J -o NAME,TYPE,SIZE,MOUNTPOINT,FSTYPE,MODEL,SERIAL,UUID');
  if (result.exitCode !== 0) throw new Error(`Failed to detect block devices`);

  try {
    const data = JSON.parse(result.stdout);
    return (data.blockdevices || []).map((d: any) => parseBlockDevice(d));
  } catch (error) {
    throw new Error(`Failed to parse lsblk output`);
  }
}

function parseBlockDevice(device: any): BlockDevice {
  const bd: BlockDevice = {
    name: device.name || '',
    type: device.type || '',
    size: parseInt(device.size, 10) || 0,
    mountpoint: device.mountpoint || undefined,
    fstype: device.fstype || undefined,
    model: device.model || undefined,
    serial: device.serial || undefined,
    uuid: device.uuid || undefined,
  };

  if (device.children && Array.isArray(device.children)) {
    bd.children = device.children.map((c: any) => parseBlockDevice(c));
  }

  return bd;
}

async function detectMountPoints(): Promise<MountPoint[]> {
  const result = await executeCommand('df -B1 -T');
  if (result.exitCode !== 0) return [];

  const mountPoints: MountPoint[] = [];
  const lines = result.stdout.split('\n');

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i];
    if (!line || line.trim() === '') continue;

    const parts = line.trim().split(/\s+/);
    if (parts.length < 7) continue;

    const fstype = parts[1] || '';
    if (fstype.startsWith('tmpfs') || fstype === 'devtmpfs' || fstype === 'squashfs') continue;

    mountPoints.push({
      device: parts[0] || '',
      mountpoint: parts[6] || '',
      fstype,
      size: parseInt(parts[2] || '0', 10),
      used: parseInt(parts[3] || '0', 10),
      available: parseInt(parts[4] || '0', 10),
      usedPercent: parseInt((parts[5] || '0').replace('%', ''), 10),
    });
  }

  return mountPoints;
}

async function detectNVMeDevices(): Promise<NVMeDevice[] | undefined> {
  const result = await executeCommand('nvme list -o json 2>/dev/null');
  if (result.exitCode !== 0) return undefined;

  try {
    const data = JSON.parse(result.stdout);
    if (!data.Devices || !Array.isArray(data.Devices)) return undefined;

    const devices: NVMeDevice[] = data.Devices.map((device: any) => ({
      device: device.DevicePath || '',
      model: device.ModelNumber || '',
      serial: device.SerialNumber || '',
      firmware: device.Firmware || '',
      totalCapacity: parseInt(device.PhysicalSize, 10) || 0,
      usedCapacity: parseInt(device.UsedBytes, 10) || 0,
      temperatureCelsius: device.Temperature ? parseInt(device.Temperature, 10) : undefined,
      criticalWarning: device.CriticalWarning ? parseInt(device.CriticalWarning, 10) : undefined,
    }));

    return devices.length > 0 ? devices : undefined;
  } catch (error) {
    return undefined;
  }
}

async function detectRAIDArrays(): Promise<RAIDArray[] | undefined> {
  const result = await executeCommand('cat /proc/mdstat 2>/dev/null');
  if (result.exitCode !== 0 || result.stdout.trim() === '') return undefined;

  const arrays: RAIDArray[] = [];
  const lines = result.stdout.split('\n');
  let currentArray: Partial<RAIDArray> | null = null;

  for (const line of lines) {
    const trimmed = line.trim();

    const arrayMatch = trimmed.match(/^(md\d+)\s*:\s*(\w+)\s+(raid\d+)\s+(.+)$/);
    if (arrayMatch && arrayMatch[1] && arrayMatch[2] && arrayMatch[3] && arrayMatch[4]) {
      if (currentArray) arrays.push(currentArray as RAIDArray);

      const devices = arrayMatch[4]
        .split(/\s+/)
        .map(d => d.replace(/\[\d+\]/g, ''))
        .filter(d => d.length > 0);

      currentArray = {
        name: arrayMatch[1],
        level: arrayMatch[3],
        devices,
        state: arrayMatch[2],
        size: 0,
        usedSize: 0,
      };
    }

    const sizeMatch = trimmed.match(/^(\d+)\s+blocks/);
    if (sizeMatch && sizeMatch[1] && currentArray) {
      const blocks = parseInt(sizeMatch[1], 10);
      currentArray.size = blocks * 1024;
      currentArray.usedSize = currentArray.size;
    }
  }

  if (currentArray) arrays.push(currentArray as RAIDArray);
  return arrays.length > 0 ? arrays : undefined;
}

function calculateTotals(mountPoints: MountPoint[]): { totalCapacity: number; totalUsed: number; totalAvailable: number } {
  let totalCapacity = 0;
  let totalUsed = 0;
  let totalAvailable = 0;

  for (const mp of mountPoints) {
    totalCapacity += mp.size;
    totalUsed += mp.used;
    totalAvailable += mp.available;
  }

  return { totalCapacity, totalUsed, totalAvailable };
}

export async function getTotalStorageCapacity(): Promise<number> {
  const result = await detectStorage(false, false);
  return result.storage.totalCapacity;
}

export async function getAvailableStorage(): Promise<number> {
  const result = await detectStorage(false, false);
  return result.storage.totalAvailable;
}

export async function getStorageUtilization(): Promise<number> {
  const result = await detectStorage(false, false);
  const { totalCapacity, totalUsed } = result.storage;
  if (totalCapacity === 0) return 0;
  return (totalUsed / totalCapacity) * 100;
}

export async function hasNVMe(): Promise<boolean> {
  const result = await detectStorage(true, false);
  return result.storage.nvmeDevices !== undefined && result.storage.nvmeDevices.length > 0;
}

export async function hasRAID(): Promise<boolean> {
  const result = await detectStorage(false, true);
  return result.storage.raidArrays !== undefined && result.storage.raidArrays.length > 0;
}
EOF

echo "storage.ts fixed"
