//! Metrics state management
//!
//! Manages system metrics data for dashboard display

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// System metrics state
#[derive(Debug, Clone)]
pub struct MetricsState {
    /// CPU usage history (0.0 to 1.0)
    pub cpu_history: VecDeque<f64>,
    /// Memory usage history (0.0 to 1.0)
    pub memory_history: VecDeque<f64>,
    /// Disk I/O usage history (0.0 to 1.0)
    pub disk_history: VecDeque<f64>,
    /// Temperature history (Celsius)
    pub temp_history: VecDeque<f64>,
    /// Network upload history (MB/s)
    pub network_upload: VecDeque<u64>,
    /// Network download history (MB/s)
    pub network_download: VecDeque<u64>,
    /// Active processes
    pub processes: Vec<ProcessInfo>,
    /// Last update time
    pub last_update: Instant,
    /// Maximum data points to keep
    max_points: usize,
}

/// Process information
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub threads: u32,
    pub status: ProcessStatus,
}

/// Process status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
}

impl ProcessStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Running => "● Running",
            Self::Sleeping => "◐ Sleeping",
            Self::Stopped => "○ Stopped",
        }
    }
}

impl MetricsState {
    /// Create new metrics state
    pub fn new(max_points: usize) -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(max_points),
            memory_history: VecDeque::with_capacity(max_points),
            disk_history: VecDeque::with_capacity(max_points),
            temp_history: VecDeque::with_capacity(max_points),
            network_upload: VecDeque::with_capacity(max_points),
            network_download: VecDeque::with_capacity(max_points),
            processes: Vec::new(),
            last_update: Instant::now(),
            max_points,
        }
    }

    /// Get current CPU usage (most recent value)
    pub fn current_cpu(&self) -> f64 {
        self.cpu_history.back().copied().unwrap_or(0.0)
    }

    /// Get current memory usage (most recent value)
    pub fn current_memory(&self) -> f64 {
        self.memory_history.back().copied().unwrap_or(0.0)
    }

    /// Get current disk I/O (most recent value)
    pub fn current_disk(&self) -> f64 {
        self.disk_history.back().copied().unwrap_or(0.0)
    }

    /// Get current temperature (most recent value)
    pub fn current_temp(&self) -> f64 {
        self.temp_history.back().copied().unwrap_or(0.0)
    }

    /// Add a CPU sample
    pub fn add_cpu_sample(&mut self, value: f64) {
        if self.cpu_history.len() >= self.max_points {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(value.clamp(0.0, 1.0));
        self.last_update = Instant::now();
    }

    /// Add a memory sample
    pub fn add_memory_sample(&mut self, value: f64) {
        if self.memory_history.len() >= self.max_points {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(value.clamp(0.0, 1.0));
        self.last_update = Instant::now();
    }

    /// Add a disk I/O sample
    pub fn add_disk_sample(&mut self, value: f64) {
        if self.disk_history.len() >= self.max_points {
            self.disk_history.pop_front();
        }
        self.disk_history.push_back(value.clamp(0.0, 1.0));
        self.last_update = Instant::now();
    }

    /// Add a temperature sample
    pub fn add_temp_sample(&mut self, value: f64) {
        if self.temp_history.len() >= self.max_points {
            self.temp_history.pop_front();
        }
        self.temp_history.push_back(value.max(0.0));
        self.last_update = Instant::now();
    }

    /// Add network samples
    pub fn add_network_sample(&mut self, upload: u64, download: u64) {
        if self.network_upload.len() >= self.max_points {
            self.network_upload.pop_front();
        }
        if self.network_download.len() >= self.max_points {
            self.network_download.pop_front();
        }

        self.network_upload.push_back(upload);
        self.network_download.push_back(download);
        self.last_update = Instant::now();
    }

    /// Update processes list
    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.processes = processes;
        self.last_update = Instant::now();
    }

    /// Get time since last update
    pub fn time_since_update(&self) -> Duration {
        Instant::now().duration_since(self.last_update)
    }

    /// Simulate metric updates (for demo purposes)
    pub fn simulate_update(&mut self) {
        // Generate simulated data with some patterns
        let time = self.cpu_history.len() as f64;

        // CPU: sine wave with noise
        let cpu = (0.5 + 0.3 * (time * 0.1).sin() + 0.1 * (time * 0.5).sin()).clamp(0.0, 1.0);
        self.add_cpu_sample(cpu);

        // Memory: slowly increasing
        let memory = (0.4 + 0.2 * (time * 0.05).sin() + time * 0.001).clamp(0.0, 1.0);
        self.add_memory_sample(memory);

        // Disk: random spikes
        let disk = if time as u64 % 10 == 0 {
            0.7 + 0.2 * (time * 0.3).sin()
        } else {
            0.2 + 0.1 * (time * 0.2).sin()
        }.clamp(0.0, 1.0);
        self.add_disk_sample(disk);

        // Temperature: stable with small variations
        let temp = 35.0 + 10.0 * (time * 0.08).sin() + 5.0 * (time * 0.3).cos();
        self.add_temp_sample(temp);

        // Network: variable traffic
        let upload = (50.0 + 30.0 * (time * 0.15).sin()).max(0.0) as u64;
        let download = (150.0 + 100.0 * (time * 0.12).cos()).max(0.0) as u64;
        self.add_network_sample(upload, download);
    }

    /// Generate simulated process data
    pub fn simulate_processes(&mut self) {
        self.processes = vec![
            ProcessInfo {
                pid: 1234,
                name: "rust-analyzer".to_string(),
                cpu_percent: 12.3,
                memory_mb: 256,
                threads: 4,
                status: ProcessStatus::Running,
            },
            ProcessInfo {
                pid: 5678,
                name: "firefox".to_string(),
                cpu_percent: 23.1,
                memory_mb: 1200,
                threads: 12,
                status: ProcessStatus::Running,
            },
            ProcessInfo {
                pid: 9012,
                name: "cargo-watch".to_string(),
                cpu_percent: 8.5,
                memory_mb: 128,
                threads: 2,
                status: ProcessStatus::Running,
            },
            ProcessInfo {
                pid: 3456,
                name: "node".to_string(),
                cpu_percent: 15.2,
                memory_mb: 512,
                threads: 8,
                status: ProcessStatus::Running,
            },
            ProcessInfo {
                pid: 7890,
                name: "vscode".to_string(),
                cpu_percent: 18.9,
                memory_mb: 768,
                threads: 16,
                status: ProcessStatus::Running,
            },
            ProcessInfo {
                pid: 2345,
                name: "docker".to_string(),
                cpu_percent: 10.3,
                memory_mb: 384,
                threads: 6,
                status: ProcessStatus::Running,
            },
        ];
    }
}

impl Default for MetricsState {
    fn default() -> Self {
        Self::new(60)
    }
}
