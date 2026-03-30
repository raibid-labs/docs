/**
 * Mock child_process module for testing
 * Provides mock nvidia-smi and other hardware detection commands
 */

import { EventEmitter } from 'events';

const mockNvidiaSmiOutput = `<?xml version="1.0" ?>
<!DOCTYPE nvidia_smi_log SYSTEM "nvsmi_device_v11.dtd">
<nvidia_smi_log>
  <timestamp>Thu Nov 14 00:00:00 2025</timestamp>
  <driver_version>535.129.03</driver_version>
  <cuda_version>12.2</cuda_version>
  <attached_gpus>4</attached_gpus>
  <gpu id="00000000:07:00.0">
    <product_name>NVIDIA A100-SXM4-80GB</product_name>
    <product_brand>Tesla</product_brand>
    <product_architecture>Ampere</product_architecture>
    <uuid>GPU-12345678-1234-1234-1234-123456789012</uuid>
    <minor_number>0</minor_number>
    <fb_memory_usage>
      <total>81920 MiB</total>
      <used>0 MiB</used>
      <free>81920 MiB</free>
    </fb_memory_usage>
    <utilization>
      <gpu_util>0 %</gpu_util>
      <memory_util>0 %</memory_util>
      <encoder_util>0 %</encoder_util>
      <decoder_util>0 %</decoder_util>
    </utilization>
    <temperature>
      <gpu_temp>35 C</gpu_temp>
    </temperature>
    <power_readings>
      <power_draw>50.00 W</power_draw>
      <power_limit>400.00 W</power_limit>
    </power_readings>
    <clocks>
      <graphics_clock>1410 MHz</graphics_clock>
      <sm_clock>1410 MHz</sm_clock>
      <mem_clock>1215 MHz</mem_clock>
      <video_clock>1275 MHz</video_clock>
    </clocks>
    <max_clocks>
      <graphics_clock>1410 MHz</graphics_clock>
      <sm_clock>1410 MHz</sm_clock>
      <mem_clock>1215 MHz</mem_clock>
      <video_clock>1275 MHz</video_clock>
    </max_clocks>
    <pci>
      <pci_bus>07</pci_bus>
      <pci_device>00</pci_device>
      <pci_domain>0000</pci_domain>
      <pci_device_id>20B010DE</pci_device_id>
      <pci_bus_id>00000000:07:00.0</pci_bus_id>
      <pci_sub_system_id>134D10DE</pci_sub_system_id>
      <pci_gpu_link_info>
        <pcie_gen>
          <max_link_gen>4</max_link_gen>
          <current_link_gen>4</current_link_gen>
        </pcie_gen>
        <link_widths>
          <max_link_width>16x</max_link_width>
          <current_link_width>16x</current_link_width>
        </link_widths>
      </pci_gpu_link_info>
    </pci>
    <compute_mode>Default</compute_mode>
    <cuda_compute_capability>
      <major>8</major>
      <minor>0</minor>
    </cuda_compute_capability>
  </gpu>
</nvidia_smi_log>`;

const mockLscpuOutput = `Architecture:                       x86_64
CPU op-mode(s):                     32-bit, 64-bit
Byte Order:                         Little Endian
Address sizes:                      48 bits physical, 48 bits virtual
CPU(s):                             128
On-line CPU(s) list:                0-127
Thread(s) per core:                 2
Core(s) per socket:                 64
Socket(s):                          1
NUMA node(s):                       2
Vendor ID:                          AuthenticAMD
CPU family:                         23
Model:                              49
Model name:                         AMD EPYC 7742 64-Core Processor
Stepping:                           0
Frequency boost:                    enabled
CPU MHz:                            2250.000
CPU max MHz:                        3400.0000
CPU min MHz:                        1500.0000
BogoMIPS:                           4500.00
L1d cache:                          2 MiB
L1i cache:                          2 MiB
L2 cache:                           32 MiB
L3 cache:                           256 MiB
NUMA node0 CPU(s):                  0-63
NUMA node1 CPU(s):                  64-127
Flags:                              fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext pdpe1gb rdtscp lm avx avx2`;


class MockChildProcess extends EventEmitter {
  stdout = new EventEmitter();
  stderr = new EventEmitter();
  exitCode: number | null = null;

  constructor(
    _command: string,
    _args: string[]
  ) {
    super();
  }

  kill() {
    this.exitCode = -1;
    this.emit('exit', this.exitCode);
  }
}

export function execFile(
  command: string,
  args: string[],
  callback: (error: Error | null, stdout: string, stderr: string) => void
): MockChildProcess {
  const proc = new MockChildProcess(command, args);

  setImmediate(() => {
    if (command.includes('nvidia-smi')) {
      callback(null, mockNvidiaSmiOutput, '');
      proc.exitCode = 0;
      proc.emit('exit', 0);
    } else if (command.includes('lscpu')) {
      callback(null, mockLscpuOutput, '');
      proc.exitCode = 0;
      proc.emit('exit', 0);
    } else {
      callback(new Error(`Command not found: ${command}`), '', '');
      proc.exitCode = 127;
      proc.emit('exit', 127);
    }
  });

  return proc;
}

export function exec(
  command: string,
  callback: (error: Error | null, stdout: string, stderr: string) => void
): MockChildProcess {
  const [cmd, ...args] = command.split(' ');
  return execFile(cmd!, args, callback);
}

export function spawn(command: string, args: string[]): MockChildProcess {
  const proc = new MockChildProcess(command, args);

  setImmediate(() => {
    if (command.includes('nvidia-smi')) {
      proc.stdout.emit('data', Buffer.from(mockNvidiaSmiOutput));
      proc.exitCode = 0;
      proc.emit('exit', 0);
    } else {
      proc.stderr.emit('data', Buffer.from(`Command not found: ${command}`));
      proc.exitCode = 127;
      proc.emit('exit', 127);
    }
  });

  return proc;
}
