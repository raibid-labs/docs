//! Profiling infrastructure for performance analysis
//!
//! This module provides profiling utilities using Tracy and puffin
//! to help identify performance bottlenecks and optimize critical paths.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Re-export profiling macros for convenience
#[cfg(feature = "profiling")]
pub use profiling::{function, scope};

// If profiling is disabled, provide no-op macros
#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! function {
    () => {};
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! scope {
    ($name:expr) => {};
    ($name:expr, $data:expr) => {};
}

/// Performance metrics collector
pub struct MetricsCollector {
    // Frame timing metrics
    frame_times: Arc<AtomicU64>,
    frame_count: Arc<AtomicU64>,

    // VTE parsing metrics
    vte_parse_time_ns: Arc<AtomicU64>,
    vte_parse_count: Arc<AtomicU64>,
    vte_bytes_processed: Arc<AtomicU64>,

    // Rendering metrics
    render_time_ns: Arc<AtomicU64>,
    render_count: Arc<AtomicU64>,
    draw_calls: Arc<AtomicU64>,

    // Memory metrics
    allocated_bytes: Arc<AtomicU64>,
    gpu_memory_bytes: Arc<AtomicU64>,

    // IPC metrics
    ipc_messages_sent: Arc<AtomicU64>,
    ipc_messages_received: Arc<AtomicU64>,
    ipc_bytes_transferred: Arc<AtomicU64>,

    // Shared memory metrics
    shmem_sync_time_ns: Arc<AtomicU64>,
    shmem_sync_count: Arc<AtomicU64>,

    // Plugin metrics
    plugin_load_time_ns: Arc<AtomicU64>,
    plugin_load_count: Arc<AtomicU64>,
    plugin_output_time_ns: Arc<AtomicU64>,
    plugin_output_count: Arc<AtomicU64>,
    plugin_input_time_ns: Arc<AtomicU64>,
    plugin_input_count: Arc<AtomicU64>,
    plugin_resize_time_ns: Arc<AtomicU64>,
    plugin_resize_count: Arc<AtomicU64>,
    plugin_vm_exec_time_ns: Arc<AtomicU64>,
    plugin_vm_exec_count: Arc<AtomicU64>,

    collection_enabled: Arc<AtomicBool>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            frame_times: Arc::new(AtomicU64::new(0)),
            frame_count: Arc::new(AtomicU64::new(0)),
            vte_parse_time_ns: Arc::new(AtomicU64::new(0)),
            vte_parse_count: Arc::new(AtomicU64::new(0)),
            vte_bytes_processed: Arc::new(AtomicU64::new(0)),
            render_time_ns: Arc::new(AtomicU64::new(0)),
            render_count: Arc::new(AtomicU64::new(0)),
            draw_calls: Arc::new(AtomicU64::new(0)),
            allocated_bytes: Arc::new(AtomicU64::new(0)),
            gpu_memory_bytes: Arc::new(AtomicU64::new(0)),
            ipc_messages_sent: Arc::new(AtomicU64::new(0)),
            ipc_messages_received: Arc::new(AtomicU64::new(0)),
            ipc_bytes_transferred: Arc::new(AtomicU64::new(0)),
            shmem_sync_time_ns: Arc::new(AtomicU64::new(0)),
            shmem_sync_count: Arc::new(AtomicU64::new(0)),
            plugin_load_time_ns: Arc::new(AtomicU64::new(0)),
            plugin_load_count: Arc::new(AtomicU64::new(0)),
            plugin_output_time_ns: Arc::new(AtomicU64::new(0)),
            plugin_output_count: Arc::new(AtomicU64::new(0)),
            plugin_input_time_ns: Arc::new(AtomicU64::new(0)),
            plugin_input_count: Arc::new(AtomicU64::new(0)),
            plugin_resize_time_ns: Arc::new(AtomicU64::new(0)),
            plugin_resize_count: Arc::new(AtomicU64::new(0)),
            plugin_vm_exec_time_ns: Arc::new(AtomicU64::new(0)),
            plugin_vm_exec_count: Arc::new(AtomicU64::new(0)),
            collection_enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn enable(&self) {
        self.collection_enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.collection_enabled.store(false, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.collection_enabled.load(Ordering::Relaxed)
    }

    // Frame timing
    pub fn record_frame_time(&self, duration: Duration) {
        if self.is_enabled() {
            self.frame_times
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.frame_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    // VTE parsing
    pub fn record_vte_parse(&self, duration: Duration, bytes: usize) {
        if self.is_enabled() {
            self.vte_parse_time_ns
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.vte_parse_count.fetch_add(1, Ordering::Relaxed);
            self.vte_bytes_processed
                .fetch_add(bytes as u64, Ordering::Relaxed);
        }
    }

    // Rendering
    pub fn record_render(&self, duration: Duration, draw_calls: u32) {
        if self.is_enabled() {
            self.render_time_ns
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.render_count.fetch_add(1, Ordering::Relaxed);
            self.draw_calls
                .fetch_add(draw_calls as u64, Ordering::Relaxed);
        }
    }

    // Memory
    pub fn set_allocated_bytes(&self, bytes: usize) {
        if self.is_enabled() {
            self.allocated_bytes.store(bytes as u64, Ordering::Relaxed);
        }
    }

    pub fn set_gpu_memory(&self, bytes: usize) {
        if self.is_enabled() {
            self.gpu_memory_bytes.store(bytes as u64, Ordering::Relaxed);
        }
    }

    // IPC
    pub fn record_ipc_send(&self, bytes: usize) {
        if self.is_enabled() {
            self.ipc_messages_sent.fetch_add(1, Ordering::Relaxed);
            self.ipc_bytes_transferred
                .fetch_add(bytes as u64, Ordering::Relaxed);
        }
    }

    pub fn record_ipc_receive(&self, bytes: usize) {
        if self.is_enabled() {
            self.ipc_messages_received.fetch_add(1, Ordering::Relaxed);
            self.ipc_bytes_transferred
                .fetch_add(bytes as u64, Ordering::Relaxed);
        }
    }

    // Shared memory
    pub fn record_shmem_sync(&self, duration: Duration) {
        if self.is_enabled() {
            self.shmem_sync_time_ns
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.shmem_sync_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    // Plugin metrics
    pub fn record_plugin_load(&self, duration: Duration) {
        if self.is_enabled() {
            self.plugin_load_time_ns
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.plugin_load_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_plugin_output(&self, duration: Duration) {
        if self.is_enabled() {
            self.plugin_output_time_ns
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.plugin_output_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_plugin_input(&self, duration: Duration) {
        if self.is_enabled() {
            self.plugin_input_time_ns
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.plugin_input_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_plugin_resize(&self, duration: Duration) {
        if self.is_enabled() {
            self.plugin_resize_time_ns
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.plugin_resize_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_plugin_vm_exec(&self, duration: Duration) {
        if self.is_enabled() {
            self.plugin_vm_exec_time_ns
                .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            self.plugin_vm_exec_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get current performance report
    pub fn report(&self) -> PerformanceReport {
        let frame_count = self.frame_count.load(Ordering::Relaxed);
        let frame_time_total = self.frame_times.load(Ordering::Relaxed);
        let avg_frame_time_ms = if frame_count > 0 {
            (frame_time_total / frame_count) as f64 / 1_000_000.0
        } else {
            0.0
        };

        let vte_count = self.vte_parse_count.load(Ordering::Relaxed);
        let vte_time_total = self.vte_parse_time_ns.load(Ordering::Relaxed);
        let avg_vte_time_us = if vte_count > 0 {
            (vte_time_total / vte_count) as f64 / 1_000.0
        } else {
            0.0
        };

        let render_count = self.render_count.load(Ordering::Relaxed);
        let render_time_total = self.render_time_ns.load(Ordering::Relaxed);
        let avg_render_time_ms = if render_count > 0 {
            (render_time_total / render_count) as f64 / 1_000_000.0
        } else {
            0.0
        };

        let shmem_count = self.shmem_sync_count.load(Ordering::Relaxed);
        let shmem_time_total = self.shmem_sync_time_ns.load(Ordering::Relaxed);
        let avg_shmem_sync_time_us = if shmem_count > 0 {
            (shmem_time_total / shmem_count) as f64 / 1_000.0
        } else {
            0.0
        };

        // Plugin metrics
        let plugin_load_count = self.plugin_load_count.load(Ordering::Relaxed);
        let plugin_load_time_total = self.plugin_load_time_ns.load(Ordering::Relaxed);
        let avg_plugin_load_time_ms = if plugin_load_count > 0 {
            (plugin_load_time_total / plugin_load_count) as f64 / 1_000_000.0
        } else {
            0.0
        };

        let plugin_output_count = self.plugin_output_count.load(Ordering::Relaxed);
        let plugin_output_time_total = self.plugin_output_time_ns.load(Ordering::Relaxed);
        let avg_plugin_output_time_us = if plugin_output_count > 0 {
            (plugin_output_time_total / plugin_output_count) as f64 / 1_000.0
        } else {
            0.0
        };

        let plugin_input_count = self.plugin_input_count.load(Ordering::Relaxed);
        let plugin_input_time_total = self.plugin_input_time_ns.load(Ordering::Relaxed);
        let avg_plugin_input_time_us = if plugin_input_count > 0 {
            (plugin_input_time_total / plugin_input_count) as f64 / 1_000.0
        } else {
            0.0
        };

        let plugin_vm_exec_count = self.plugin_vm_exec_count.load(Ordering::Relaxed);
        let plugin_vm_exec_time_total = self.plugin_vm_exec_time_ns.load(Ordering::Relaxed);
        let avg_plugin_vm_exec_time_us = if plugin_vm_exec_count > 0 {
            (plugin_vm_exec_time_total / plugin_vm_exec_count) as f64 / 1_000.0
        } else {
            0.0
        };

        PerformanceReport {
            avg_frame_time_ms,
            avg_vte_parse_time_us: avg_vte_time_us,
            vte_bytes_per_sec: self.vte_bytes_processed.load(Ordering::Relaxed) as f64,
            avg_render_time_ms,
            draw_calls_per_frame: if render_count > 0 {
                self.draw_calls.load(Ordering::Relaxed) as f64 / render_count as f64
            } else {
                0.0
            },
            allocated_mb: self.allocated_bytes.load(Ordering::Relaxed) as f64 / 1_048_576.0,
            gpu_memory_mb: self.gpu_memory_bytes.load(Ordering::Relaxed) as f64 / 1_048_576.0,
            ipc_messages_per_sec: 0.0, // Will calculate based on time window
            avg_shmem_sync_time_us,
            avg_plugin_load_time_ms,
            avg_plugin_output_time_us,
            avg_plugin_input_time_us,
            avg_plugin_vm_exec_time_us,
            plugin_output_count,
            plugin_input_count,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.frame_times.store(0, Ordering::Relaxed);
        self.frame_count.store(0, Ordering::Relaxed);
        self.vte_parse_time_ns.store(0, Ordering::Relaxed);
        self.vte_parse_count.store(0, Ordering::Relaxed);
        self.vte_bytes_processed.store(0, Ordering::Relaxed);
        self.render_time_ns.store(0, Ordering::Relaxed);
        self.render_count.store(0, Ordering::Relaxed);
        self.draw_calls.store(0, Ordering::Relaxed);
        self.ipc_messages_sent.store(0, Ordering::Relaxed);
        self.ipc_messages_received.store(0, Ordering::Relaxed);
        self.ipc_bytes_transferred.store(0, Ordering::Relaxed);
        self.shmem_sync_time_ns.store(0, Ordering::Relaxed);
        self.shmem_sync_count.store(0, Ordering::Relaxed);
        self.plugin_load_time_ns.store(0, Ordering::Relaxed);
        self.plugin_load_count.store(0, Ordering::Relaxed);
        self.plugin_output_time_ns.store(0, Ordering::Relaxed);
        self.plugin_output_count.store(0, Ordering::Relaxed);
        self.plugin_input_time_ns.store(0, Ordering::Relaxed);
        self.plugin_input_count.store(0, Ordering::Relaxed);
        self.plugin_resize_time_ns.store(0, Ordering::Relaxed);
        self.plugin_resize_count.store(0, Ordering::Relaxed);
        self.plugin_vm_exec_time_ns.store(0, Ordering::Relaxed);
        self.plugin_vm_exec_count.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub avg_frame_time_ms: f64,
    pub avg_vte_parse_time_us: f64,
    pub vte_bytes_per_sec: f64,
    pub avg_render_time_ms: f64,
    pub draw_calls_per_frame: f64,
    pub allocated_mb: f64,
    pub gpu_memory_mb: f64,
    pub ipc_messages_per_sec: f64,
    pub avg_shmem_sync_time_us: f64,
    pub avg_plugin_load_time_ms: f64,
    pub avg_plugin_output_time_us: f64,
    pub avg_plugin_input_time_us: f64,
    pub avg_plugin_vm_exec_time_us: f64,
    pub plugin_output_count: u64,
    pub plugin_input_count: u64,
}

impl PerformanceReport {
    pub fn print_summary(&self) {
        println!("=== Performance Report ===");
        println!("Frame Time:        {:.2} ms", self.avg_frame_time_ms);
        println!("VTE Parse:         {:.2} us", self.avg_vte_parse_time_us);
        println!(
            "VTE Throughput:    {:.2} KB/s",
            self.vte_bytes_per_sec / 1024.0
        );
        println!("Render Time:       {:.2} ms", self.avg_render_time_ms);
        println!("Draw Calls/Frame:  {:.1}", self.draw_calls_per_frame);
        println!("Memory:            {:.2} MB", self.allocated_mb);
        println!("GPU Memory:        {:.2} MB", self.gpu_memory_mb);
        println!("IPC Messages/sec:  {:.0}", self.ipc_messages_per_sec);
        println!("Shmem Sync:        {:.2} us", self.avg_shmem_sync_time_us);
        println!("\n=== Plugin Performance ===");
        println!("Plugin Load:       {:.2} ms", self.avg_plugin_load_time_ms);
        println!("Plugin Output:     {:.2} us (count: {})", self.avg_plugin_output_time_us, self.plugin_output_count);
        println!("Plugin Input:      {:.2} us (count: {})", self.avg_plugin_input_time_us, self.plugin_input_count);
        println!("Plugin VM Exec:    {:.2} us", self.avg_plugin_vm_exec_time_us);
    }

    pub fn check_targets(&self) -> bool {
        // Check if we meet performance targets
        let mut success = true;

        if self.avg_frame_time_ms > 50.0 {
            println!(
                "FAIL Frame time {:.2}ms exceeds target of 50ms",
                self.avg_frame_time_ms
            );
            success = false;
        } else {
            println!("PASS Frame time {:.2}ms meets target", self.avg_frame_time_ms);
        }

        // Plugin-specific targets
        if self.avg_plugin_load_time_ms > 100.0 {
            println!(
                "FAIL Plugin load time {:.2}ms exceeds target of 100ms",
                self.avg_plugin_load_time_ms
            );
            success = false;
        } else {
            println!("PASS Plugin load time {:.2}ms meets target", self.avg_plugin_load_time_ms);
        }

        if self.avg_plugin_output_time_us > 50.0 {
            println!(
                "FAIL Plugin output hook {:.2}us exceeds target of 50us",
                self.avg_plugin_output_time_us
            );
            success = false;
        } else {
            println!("PASS Plugin output hook {:.2}us meets target", self.avg_plugin_output_time_us);
        }

        if self.avg_plugin_input_time_us > 50.0 {
            println!(
                "FAIL Plugin input hook {:.2}us exceeds target of 50us",
                self.avg_plugin_input_time_us
            );
            success = false;
        } else {
            println!("PASS Plugin input hook {:.2}us meets target", self.avg_plugin_input_time_us);
        }

        if self.avg_plugin_vm_exec_time_us > 20.0 {
            println!(
                "FAIL Plugin VM execution {:.2}us exceeds target of 20us",
                self.avg_plugin_vm_exec_time_us
            );
            success = false;
        } else {
            println!("PASS Plugin VM execution {:.2}us meets target", self.avg_plugin_vm_exec_time_us);
        }

        // Estimate CPU usage from timing (rough approximation)
        // Assuming 60 FPS target, we have 16.67ms per frame
        let frame_budget_ms = 16.67;
        let cpu_usage_vte = (self.avg_vte_parse_time_us / 1000.0) / frame_budget_ms * 100.0;
        let cpu_usage_render = self.avg_render_time_ms / frame_budget_ms * 100.0;
        let cpu_usage_shmem = (self.avg_shmem_sync_time_us / 1000.0) / frame_budget_ms * 100.0;

        if cpu_usage_vte > 2.0 {
            println!(
                "FAIL VTE CPU usage {:.2}% exceeds target of 2%",
                cpu_usage_vte
            );
            success = false;
        } else {
            println!("PASS VTE CPU usage {:.2}% meets target", cpu_usage_vte);
        }

        if cpu_usage_render > 3.0 {
            println!(
                "FAIL Render CPU usage {:.2}% exceeds target of 3%",
                cpu_usage_render
            );
            success = false;
        } else {
            println!("PASS Render CPU usage {:.2}% meets target", cpu_usage_render);
        }

        if cpu_usage_shmem > 0.5 {
            println!(
                "FAIL Shmem CPU usage {:.2}% exceeds target of 0.5%",
                cpu_usage_shmem
            );
            success = false;
        } else {
            println!("PASS Shmem CPU usage {:.2}% meets target", cpu_usage_shmem);
        }

        if self.allocated_mb > 100.0 {
            println!(
                "FAIL Memory usage {:.2}MB exceeds target of 100MB",
                self.allocated_mb
            );
            success = false;
        } else {
            println!("PASS Memory usage {:.2}MB meets target", self.allocated_mb);
        }

        if self.gpu_memory_mb > 150.0 {
            println!(
                "FAIL GPU memory {:.2}MB exceeds target of 150MB",
                self.gpu_memory_mb
            );
            success = false;
        } else {
            println!("PASS GPU memory {:.2}MB meets target", self.gpu_memory_mb);
        }

        success
    }
}

/// Profiling timer for measuring execution time
pub struct Timer {
    start: Instant,
    name: &'static str,
}

impl Timer {
    pub fn new(name: &'static str) -> Self {
        Self {
            start: Instant::now(),
            name,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        log::trace!("{} took {:?}", self.name, elapsed);
    }
}

/// Initialize profiling infrastructure
pub fn init_profiling() {
    #[cfg(feature = "tracy")]
    {
        // Tracy client is automatically initialized on first use
        log::info!("Tracy profiling enabled");
    }

    #[cfg(feature = "puffin-profiling")]
    {
        puffin::set_scopes_on(true);
        log::info!("Puffin profiling enabled");
    }

    #[cfg(not(feature = "profiling"))]
    {
        log::info!("Profiling disabled (compile with --features profiling to enable)");
    }
}

/// Macro for timing a block of code
#[macro_export]
macro_rules! time_block {
    ($name:expr, $block:expr) => {{
        let _timer = $crate::profiling::Timer::new($name);
        let result = $block;
        result
    }};
}
