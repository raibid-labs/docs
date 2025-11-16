/**
 * Network detection module (simplified for TypeScript strict mode)
 */

import { Network, NetworkDetectionResult, NetworkInterface, InfiniBandDevice, InfiniBandPort, NetworkBandwidth } from '../types/network.js';
import { executeCommand } from '../utils/exec.js';
import { readSysFile, readNumericSysFile } from '../utils/proc-parser.js';

export async function detectNetwork(includeInfiniBand: boolean = true): Promise<NetworkDetectionResult> {
  const startTime = Date.now();

  const interfaces = await detectNetworkInterfaces();
  const infinibandDevices = includeInfiniBand ? await detectInfiniBandDevices() : undefined;
  const bandwidth = await detectBandwidth(interfaces);

  const totalInterfaces = interfaces.length;
  const activeInterfaces = interfaces.filter(iface => iface.state.toLowerCase() === 'up').length;

  const network: Network = {
    interfaces,
    infinibandDevices,
    bandwidth,
    totalInterfaces,
    activeInterfaces,
  };

  return {
    network,
    timestamp: Date.now(),
    detectionTime: Date.now() - startTime,
  };
}

async function detectNetworkInterfaces(): Promise<NetworkInterface[]> {
  const result = await executeCommand('ip -j addr show');

  if (result.exitCode !== 0) {
    return detectNetworkInterfacesFallback();
  }

  try {
    const data = JSON.parse(result.stdout);
    const interfaces: NetworkInterface[] = [];

    for (const iface of data) {
      const ipv4: string[] = [];
      const ipv6: string[] = [];

      if (iface.addr_info && Array.isArray(iface.addr_info)) {
        for (const addr of iface.addr_info) {
          if (addr.family === 'inet') {
            ipv4.push(addr.local);
          } else if (addr.family === 'inet6') {
            ipv6.push(addr.local);
          }
        }
      }

      const stats = await getInterfaceStats(iface.ifname);

      interfaces.push({
        name: iface.ifname,
        type: iface.link_type || 'unknown',
        state: iface.operstate || 'unknown',
        mac: iface.address || '',
        mtu: iface.mtu || 0,
        speed: await getInterfaceSpeed(iface.ifname),
        duplex: await getInterfaceDuplex(iface.ifname),
        ipv4: ipv4.length > 0 ? ipv4 : undefined,
        ipv6: ipv6.length > 0 ? ipv6 : undefined,
        ...stats,
      });
    }

    return interfaces;
  } catch (error) {
    return detectNetworkInterfacesFallback();
  }
}

async function detectNetworkInterfacesFallback(): Promise<NetworkInterface[]> {
  const result = await executeCommand('ip addr show');

  if (result.exitCode !== 0) {
    return [];
  }

  const interfaces: NetworkInterface[] = [];
  const lines = result.stdout.split('\n');

  let currentInterface: Partial<NetworkInterface> | null = null;

  for (const line of lines) {
    const ifaceMatch = line.match(/^\d+:\s+(\S+):\s+<([^>]+)>\s+mtu\s+(\d+)/);
    if (ifaceMatch && ifaceMatch[1] && ifaceMatch[2] && ifaceMatch[3]) {
      if (currentInterface && currentInterface.name) {
        interfaces.push(currentInterface as NetworkInterface);
      }

      const name = ifaceMatch[1].replace(/@.*$/, '');
      const flags = ifaceMatch[2].split(',');
      const mtu = parseInt(ifaceMatch[3], 10);

      currentInterface = {
        name,
        type: 'unknown',
        state: flags.includes('UP') ? 'up' : 'down',
        mac: '',
        mtu,
        ipv4: [],
        ipv6: [],
      };
      continue;
    }

    if (!currentInterface) continue;

    const macMatch = line.match(/link\/\w+\s+([0-9a-f:]+)/i);
    if (macMatch && macMatch[1]) {
      currentInterface.mac = macMatch[1];
    }

    const ipv4Match = line.match(/inet\s+([0-9.]+)/);
    if (ipv4Match && ipv4Match[1]) {
      if (!currentInterface.ipv4) currentInterface.ipv4 = [];
      (currentInterface.ipv4 as string[]).push(ipv4Match[1]);
    }

    const ipv6Match = line.match(/inet6\s+([0-9a-f:]+)/i);
    if (ipv6Match && ipv6Match[1]) {
      if (!currentInterface.ipv6) currentInterface.ipv6 = [];
      (currentInterface.ipv6 as string[]).push(ipv6Match[1]);
    }
  }

  if (currentInterface && currentInterface.name) {
    interfaces.push(currentInterface as NetworkInterface);
  }

  return interfaces;
}

async function getInterfaceSpeed(ifname: string): Promise<number | undefined> {
  const speed = await readNumericSysFile(`/sys/class/net/${ifname}/speed`);
  return speed !== null && speed > 0 ? speed : undefined;
}

async function getInterfaceDuplex(ifname: string): Promise<string | undefined> {
  const duplex = await readSysFile(`/sys/class/net/${ifname}/duplex`);
  return duplex || undefined;
}

async function getInterfaceStats(ifname: string): Promise<Partial<NetworkInterface>> {
  const rxBytes = await readNumericSysFile(`/sys/class/net/${ifname}/statistics/rx_bytes`);
  const txBytes = await readNumericSysFile(`/sys/class/net/${ifname}/statistics/tx_bytes`);
  const rxPackets = await readNumericSysFile(`/sys/class/net/${ifname}/statistics/rx_packets`);
  const txPackets = await readNumericSysFile(`/sys/class/net/${ifname}/statistics/tx_packets`);
  const rxErrors = await readNumericSysFile(`/sys/class/net/${ifname}/statistics/rx_errors`);
  const txErrors = await readNumericSysFile(`/sys/class/net/${ifname}/statistics/tx_errors`);

  return {
    rxBytes: rxBytes !== null ? rxBytes : undefined,
    txBytes: txBytes !== null ? txBytes : undefined,
    rxPackets: rxPackets !== null ? rxPackets : undefined,
    txPackets: txPackets !== null ? txPackets : undefined,
    rxErrors: rxErrors !== null ? rxErrors : undefined,
    txErrors: txErrors !== null ? txErrors : undefined,
  };
}

async function detectInfiniBandDevices(): Promise<InfiniBandDevice[] | undefined> {
  const result = await executeCommand('ibstat 2>/dev/null');

  if (result.exitCode !== 0) {
    return undefined;
  }

  const devices: InfiniBandDevice[] = [];
  const lines = result.stdout.split('\n');

  let currentDevice: Partial<InfiniBandDevice> | null = null;
  let currentPort: Partial<InfiniBandPort> | null = null;

  for (const line of lines) {
    const trimmed = line.trim();

    const deviceMatch = trimmed.match(/^CA '(.+)'$/);
    if (deviceMatch && deviceMatch[1]) {
      if (currentDevice && currentDevice.name) {
        devices.push(currentDevice as InfiniBandDevice);
      }

      currentDevice = {
        name: deviceMatch[1],
        type: 'InfiniBand',
        ports: [],
      };
      currentPort = null;
      continue;
    }

    if (!currentDevice) continue;

    if (trimmed.startsWith('CA type:')) {
      currentDevice.type = trimmed.split(':')[1]?.trim() || 'InfiniBand';
    }

    if (trimmed.startsWith('Firmware version:')) {
      currentDevice.firmwareVersion = trimmed.split(':')[1]?.trim();
    }

    if (trimmed.startsWith('Hardware version:')) {
      currentDevice.hardwareVersion = trimmed.split(':')[1]?.trim();
    }

    const portMatch = trimmed.match(/^Port (\d+):$/);
    if (portMatch && portMatch[1]) {
      if (currentPort && currentPort.port !== undefined) {
        (currentDevice.ports as InfiniBandPort[]).push(currentPort as InfiniBandPort);
      }

      currentPort = {
        caName: currentDevice.name!,
        caType: currentDevice.type!,
        port: parseInt(portMatch[1], 10),
        state: '',
        physicalState: '',
        rate: 0,
        linkLayer: '',
        portGuid: '',
      };
      continue;
    }

    if (!currentPort) continue;

    if (trimmed.startsWith('State:')) {
      currentPort.state = trimmed.split(':')[1]?.trim() || '';
    }

    if (trimmed.startsWith('Physical state:')) {
      currentPort.physicalState = trimmed.split(':')[1]?.trim() || '';
    }

    if (trimmed.startsWith('Rate:')) {
      const rateStr = trimmed.split(':')[1]?.trim() || '';
      const rateMatch = rateStr.match(/(\d+)/);
      if (rateMatch && rateMatch[1]) {
        currentPort.rate = parseInt(rateMatch[1], 10);
      }
    }

    if (trimmed.startsWith('Link layer:')) {
      currentPort.linkLayer = trimmed.split(':')[1]?.trim() || '';
    }

    if (trimmed.startsWith('Port GUID:')) {
      currentPort.portGuid = trimmed.split(':')[1]?.trim() || '';
    }
  }

  if (currentPort && currentPort.port !== undefined && currentDevice) {
    (currentDevice.ports as InfiniBandPort[]).push(currentPort as InfiniBandPort);
  }

  if (currentDevice && currentDevice.name) {
    devices.push(currentDevice as InfiniBandDevice);
  }

  return devices.length > 0 ? devices : undefined;
}

async function detectBandwidth(interfaces: NetworkInterface[]): Promise<NetworkBandwidth[] | undefined> {
  const bandwidth: NetworkBandwidth[] = [];

  for (const iface of interfaces) {
    if (!iface.speed || iface.state.toLowerCase() !== 'up') {
      continue;
    }

    bandwidth.push({
      interface: iface.name,
      maxBandwidth: iface.speed,
      currentUtilization: 0,
    });
  }

  return bandwidth.length > 0 ? bandwidth : undefined;
}

export async function hasInfiniBand(): Promise<boolean> {
  const result = await detectNetwork(true);
  return result.network.infinibandDevices !== undefined && result.network.infinibandDevices.length > 0;
}

export async function getActiveInterfaces(): Promise<NetworkInterface[]> {
  const result = await detectNetwork(false);
  return result.network.interfaces.filter(iface => iface.state.toLowerCase() === 'up');
}

export async function getInterface(name: string): Promise<NetworkInterface | null> {
  const result = await detectNetwork(false);
  return result.network.interfaces.find(iface => iface.name === name) || null;
}
