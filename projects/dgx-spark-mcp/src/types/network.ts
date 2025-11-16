/**
 * Network hardware type definitions
 */

export interface NetworkInterface {
  name: string;
  type: string;
  state: string;
  mac: string;
  mtu: number;
  speed?: number; // Mbps
  duplex?: string;
  ipv4?: string[];
  ipv6?: string[];
  rxBytes?: number;
  txBytes?: number;
  rxPackets?: number;
  txPackets?: number;
  rxErrors?: number;
  txErrors?: number;
}

export interface InfiniBandPort {
  caName: string;
  caType: string;
  port: number;
  state: string;
  physicalState: string;
  rate: number; // Gbps
  linkLayer: string;
  portGuid: string;
}

export interface InfiniBandDevice {
  name: string;
  type: string;
  ports: InfiniBandPort[];
  firmwareVersion?: string;
  hardwareVersion?: string;
}

export interface NetworkBandwidth {
  interface: string;
  maxBandwidth: number; // Mbps
  currentUtilization: number; // percentage
}

export interface Network {
  interfaces: NetworkInterface[];
  infinibandDevices?: InfiniBandDevice[];
  bandwidth?: NetworkBandwidth[];
  totalInterfaces: number;
  activeInterfaces: number;
}

export interface NetworkDetectionResult {
  network: Network;
  timestamp: number;
  detectionTime: number; // milliseconds
}
