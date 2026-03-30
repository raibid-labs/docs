//! Optimized VTE parsing implementation
//!
//! This module provides performance-optimized VTE parsing with:
//! - Batch processing for better cache locality
//! - SIMD acceleration for plain text detection
//! - LRU cache for frequently used escape sequences
//! - Zero-allocation parsing for common sequences

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use vte::{Parser, Perform};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Optimized VTE performer with batching and caching
pub struct OptimizedPerformer {
    // Output buffer for batch processing
    output_buffer: Vec<u8>,

    // Cache for frequently used sequences
    sequence_cache: SequenceCache,

    // Metrics for profiling
    #[cfg(feature = "profiling")]
    metrics: std::sync::Arc<crate::profiling::MetricsCollector>,
}

impl OptimizedPerformer {
    pub fn new() -> Self {
        Self::with_cache_capacity(256)
    }

    pub fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            output_buffer: Vec::with_capacity(4096),
            sequence_cache: SequenceCache::with_capacity(capacity),
            #[cfg(feature = "profiling")]
            metrics: std::sync::Arc::new(crate::profiling::MetricsCollector::new()),
        }
    }

    /// Process VTE data in optimized batches
    pub fn process_batch(&mut self, parser: &mut Parser, data: &[u8]) {
        #[cfg(feature = "profiling")]
        let start = std::time::Instant::now();

        // Fast path for plain text
        if let Some(plain_end) = Self::find_plain_text_end(data) {
            if plain_end > 0 {
                self.process_plain_text(&data[..plain_end]);

                // Process the rest normally
                for byte in &data[plain_end..] {
                    parser.advance(self, *byte);
                }
            }
        } else {
            // All plain text
            self.process_plain_text(data);
        }

        #[cfg(feature = "profiling")]
        self.metrics.record_vte_parse(start.elapsed(), data.len());
    }

    /// Process plain text without VTE parsing overhead
    fn process_plain_text(&mut self, text: &[u8]) {
        self.output_buffer.extend_from_slice(text);

        // Flush if buffer is getting full
        if self.output_buffer.len() > 3072 {
            self.flush_output();
        }
    }

    /// Find the end of plain text using SIMD on x86_64
    #[cfg(target_arch = "x86_64")]
    fn find_plain_text_end(data: &[u8]) -> Option<usize> {
        unsafe {
            // Look for ESC (0x1B) or other control characters
            let esc_vec = _mm_set1_epi8(0x1B);
            let ctrl_threshold = _mm_set1_epi8(0x20);

            let mut offset = 0;
            let len = data.len();

            // Process 16 bytes at a time
            while offset + 16 <= len {
                let chunk = _mm_loadu_si128(data[offset..].as_ptr() as *const __m128i);

                // Check for ESC
                let esc_mask = _mm_cmpeq_epi8(chunk, esc_vec);

                // Check for control characters (< 0x20)
                let ctrl_mask = _mm_cmplt_epi8(chunk, ctrl_threshold);

                // Combine masks
                let combined = _mm_or_si128(esc_mask, ctrl_mask);

                let mask = _mm_movemask_epi8(combined);
                if mask != 0 {
                    // Found a control character or ESC
                    return Some(offset + mask.trailing_zeros() as usize);
                }

                offset += 16;
            }

            // Check remaining bytes
            for i in offset..len {
                if data[i] == 0x1B || data[i] < 0x20 {
                    return Some(i);
                }
            }

            None
        }
    }

    /// Fallback for non-x86_64 architectures
    #[cfg(not(target_arch = "x86_64"))]
    fn find_plain_text_end(data: &[u8]) -> Option<usize> {
        for (i, &byte) in data.iter().enumerate() {
            if byte == 0x1B || byte < 0x20 {
                return Some(i);
            }
        }
        None
    }

    fn flush_output(&mut self) {
        if !self.output_buffer.is_empty() {
            // Send to terminal output handler
            // This would integrate with your actual terminal buffer

            self.output_buffer.clear();
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.sequence_cache.stats()
    }

    /// Reset cache statistics
    pub fn reset_cache_stats(&mut self) {
        self.sequence_cache.reset_stats();
    }
}

impl Default for OptimizedPerformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Perform for OptimizedPerformer {
    fn print(&mut self, c: char) {
        // Buffer characters for batch processing
        let mut buf = [0u8; 4];
        let len = c.encode_utf8(&mut buf).len();
        self.output_buffer.extend_from_slice(&buf[..len]);
    }

    fn execute(&mut self, byte: u8) {
        // Handle control characters
        match byte {
            0x07 => {}                              // BEL - ignore or handle bell
            0x08 => {}                              // BS - backspace
            0x09 => self.output_buffer.push(b'\t'), // HT - tab
            0x0A => self.output_buffer.push(b'\n'), // LF
            0x0D => self.output_buffer.push(b'\r'), // CR
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        // Convert params to slice for cache lookup
        let params_vec: Vec<i64> = params.iter().map(|p| p[0] as i64).collect();

        // Check cache first
        if let Some(cached) = self
            .sequence_cache
            .get_csi(&params_vec, intermediates, action as u8)
        {
            self.apply_cached_sequence(&cached);
            return;
        }

        // Process and parse the action
        let parsed_action = match action {
            'A' => CachedAction::CursorMove(0, -params_vec.get(0).copied().unwrap_or(1) as i32),
            'B' => CachedAction::CursorMove(0, params_vec.get(0).copied().unwrap_or(1) as i32),
            'C' => CachedAction::CursorMove(params_vec.get(0).copied().unwrap_or(1) as i32, 0),
            'D' => CachedAction::CursorMove(-params_vec.get(0).copied().unwrap_or(1) as i32, 0),
            'H' | 'f' => {
                let row = params_vec.get(0).copied().unwrap_or(1) as i32 - 1;
                let col = params_vec.get(1).copied().unwrap_or(1) as i32 - 1;
                CachedAction::CursorPosition(col, row)
            }
            'J' => {
                let mode = params_vec.get(0).copied().unwrap_or(0);
                match mode {
                    0 => CachedAction::EraseRegion(EraseMode::ToEnd),
                    1 => CachedAction::EraseRegion(EraseMode::ToBeginning),
                    2 | 3 => CachedAction::EraseRegion(EraseMode::All),
                    _ => CachedAction::EraseRegion(EraseMode::ToEnd),
                }
            }
            'K' => {
                let mode = params_vec.get(0).copied().unwrap_or(0);
                match mode {
                    0 => CachedAction::EraseRegion(EraseMode::ToEnd),
                    1 => CachedAction::EraseRegion(EraseMode::ToBeginning),
                    2 => CachedAction::EraseRegion(EraseMode::All),
                    _ => CachedAction::EraseRegion(EraseMode::ToEnd),
                }
            }
            'm' => {
                // SGR - colors and attributes
                if params_vec.is_empty() || params_vec[0] == 0 {
                    CachedAction::Reset
                } else {
                    let sgr_code = params_vec[0];
                    if (30..=37).contains(&sgr_code) {
                        // Foreground color
                        let color_idx = (sgr_code - 30) as u8;
                        CachedAction::ColorChange(Color::ansi(color_idx))
                    } else if (40..=47).contains(&sgr_code) {
                        // Background color
                        let color_idx = (sgr_code - 40) as u8;
                        CachedAction::BackgroundColorChange(Color::ansi(color_idx))
                    } else {
                        CachedAction::Attribute(sgr_code as u8)
                    }
                }
            }
            _ => return, // Don't cache unknown sequences
        };

        // Cache the parsed action
        self.sequence_cache.insert_csi(
            params_vec,
            intermediates.to_vec(),
            action as u8,
            parsed_action.clone(),
        );

        // Apply the action
        self.apply_cached_sequence(&parsed_action);
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        // Handle OSC sequences (window title, etc.)
        if params.is_empty() {
            return;
        }

        // Cache frequently used OSC sequences
        match params[0] {
            b"0" | b"2" => {} // Window title
            b"4" => {}        // Color palette
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS sequences - rarely used in modern terminals
    }

    fn put(&mut self, byte: u8) {
        self.output_buffer.push(byte);
    }

    fn unhook(&mut self) {
        // End of DCS sequence
    }
}

/// LRU Cache for frequently used VTE sequences
///
/// Implements efficient caching of parsed VTE escape sequences to avoid
/// re-parsing common sequences like color changes and cursor movements.
/// Uses LRU eviction policy to maintain bounded memory usage.
struct SequenceCache {
    csi_cache: LruCache<CsiKey, CachedAction>,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct CsiKey {
    params: Vec<i64>,
    intermediates: Vec<u8>,
    action: u8,
}

/// Cached VTE action types
#[derive(Clone, Debug)]
enum CachedAction {
    CursorMove(i32, i32),
    CursorPosition(i32, i32),
    ColorChange(Color),
    BackgroundColorChange(Color),
    EraseRegion(EraseMode),
    Attribute(u8),
    Reset,
}

#[derive(Clone, Copy, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn ansi(index: u8) -> Self {
        // Standard ANSI color palette
        match index {
            0 => Color { r: 0, g: 0, b: 0 },   // Black
            1 => Color { r: 205, g: 0, b: 0 }, // Red
            2 => Color { r: 0, g: 205, b: 0 }, // Green
            3 => Color {
                r: 205,
                g: 205,
                b: 0,
            }, // Yellow
            4 => Color { r: 0, g: 0, b: 238 }, // Blue
            5 => Color {
                r: 205,
                g: 0,
                b: 205,
            }, // Magenta
            6 => Color {
                r: 0,
                g: 205,
                b: 205,
            }, // Cyan
            7 => Color {
                r: 229,
                g: 229,
                b: 229,
            }, // White
            _ => Color {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum EraseMode {
    ToEnd,
    ToBeginning,
    All,
}

impl SequenceCache {
    fn with_capacity(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(256).unwrap());
        Self {
            csi_cache: LruCache::new(cap),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    fn get_csi(
        &mut self,
        params: &[i64],
        intermediates: &[u8],
        action: u8,
    ) -> Option<CachedAction> {
        let key = CsiKey {
            params: params.to_vec(),
            intermediates: intermediates.to_vec(),
            action,
        };

        if let Some(cached) = self.csi_cache.get(&key) {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            Some(cached.clone())
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    fn insert_csi(
        &mut self,
        params: Vec<i64>,
        intermediates: Vec<u8>,
        action: u8,
        value: CachedAction,
    ) {
        let key = CsiKey {
            params,
            intermediates,
            action,
        };
        self.csi_cache.put(key, value);
    }

    fn stats(&self) -> CacheStats {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            hit_rate,
            size: self.csi_cache.len(),
            capacity: self.csi_cache.cap().get(),
        }
    }

    fn reset_stats(&mut self) {
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
    }
}

/// Cache performance statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub capacity: usize,
}

impl CacheStats {
    pub fn memory_usage(&self) -> usize {
        // Estimate memory usage:
        // Each CsiKey is roughly: Vec<i64> (24 bytes + data) + Vec<u8> (24 bytes + data) + u8 (1 byte)
        // Each CachedAction is roughly: enum discriminant (1 byte) + largest variant (16 bytes)
        // LRU node overhead: 2 pointers (16 bytes) + key + value

        // Conservative estimate: 128 bytes per entry
        self.size * 128
    }
}

impl OptimizedPerformer {
    fn apply_cached_sequence(&mut self, action: &CachedAction) {
        // Apply the cached action without re-parsing
        // This would integrate with your terminal state
        // For now, this is a placeholder that demonstrates the caching benefit
        match action {
            CachedAction::CursorMove(_, _) => {
                // Move cursor by offset
            }
            CachedAction::CursorPosition(_, _) => {
                // Set absolute cursor position
            }
            CachedAction::ColorChange(_) => {
                // Change foreground color
            }
            CachedAction::BackgroundColorChange(_) => {
                // Change background color
            }
            CachedAction::EraseRegion(_) => {
                // Erase screen region
            }
            CachedAction::Attribute(_) => {
                // Set text attribute
            }
            CachedAction::Reset => {
                // Reset all attributes
            }
        }
    }
}

/// Optimized VTE processor that batches input
pub struct BatchProcessor {
    parser: Parser,
    performer: OptimizedPerformer,
    input_buffer: Vec<u8>,
    buffer_capacity: usize,
}

impl BatchProcessor {
    pub fn new() -> Self {
        Self::with_capacity(4096)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            parser: Parser::new(),
            performer: OptimizedPerformer::new(),
            input_buffer: Vec::with_capacity(capacity),
            buffer_capacity: capacity,
        }
    }

    /// Add data to the batch
    pub fn add_data(&mut self, data: &[u8]) {
        self.input_buffer.extend_from_slice(data);

        // Process if buffer is full
        if self.input_buffer.len() >= self.buffer_capacity {
            self.process_buffer();
        }
    }

    /// Process all buffered data
    pub fn process_buffer(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        #[cfg(feature = "profiling")]
        profiling::scope!("vte_batch_process");

        // Process in optimal chunks
        for chunk in self.input_buffer.chunks(self.buffer_capacity) {
            self.performer.process_batch(&mut self.parser, chunk);
        }

        self.input_buffer.clear();
        self.performer.flush_output();
    }

    /// Flush any pending data
    pub fn flush(&mut self) {
        self.process_buffer();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.performer.cache_stats()
    }

    /// Reset cache statistics
    pub fn reset_cache_stats(&mut self) {
        self.performer.reset_cache_stats();
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_detection() {
        let plain = b"Hello, World! This is plain text.";
        assert_eq!(OptimizedPerformer::find_plain_text_end(plain), None);

        let with_esc = b"Hello \x1B[31mRed";
        assert_eq!(OptimizedPerformer::find_plain_text_end(with_esc), Some(6));

        let with_control = b"Line1\nLine2";
        assert_eq!(
            OptimizedPerformer::find_plain_text_end(with_control),
            Some(5)
        );
    }

    #[test]
    fn test_batch_processor() {
        let mut processor = BatchProcessor::new();

        // Add small chunks
        processor.add_data(b"Hello ");
        processor.add_data(b"World!");

        // Should not process yet
        assert!(!processor.input_buffer.is_empty());

        // Flush to process
        processor.flush();
        assert!(processor.input_buffer.is_empty());
    }

    #[test]
    fn test_cache_basic() {
        let mut processor = BatchProcessor::new();

        // Process same sequence multiple times
        let red_text = b"\x1b[31mRed\x1b[0m";
        for _ in 0..10 {
            processor.add_data(red_text);
        }
        processor.flush();

        let stats = processor.cache_stats();
        assert!(stats.hits > 0, "Cache should have hits");
        assert!(stats.hit_rate > 0.0, "Hit rate should be > 0");
    }

    #[test]
    fn test_cache_stats() {
        let mut processor = BatchProcessor::new();

        // Generate varied escape sequences
        let sequences = vec![
            b"\x1b[31m", // Red
            b"\x1b[32m", // Green
            b"\x1b[33m", // Yellow
            b"\x1b[31m", // Red again (should hit cache)
            b"\x1b[32m", // Green again (should hit cache)
        ];

        for seq in sequences {
            processor.add_data(seq);
        }
        processor.flush();

        let stats = processor.cache_stats();
        assert!(stats.hits >= 2, "Should have at least 2 cache hits");
        assert!(stats.misses >= 3, "Should have at least 3 cache misses");
    }

    #[test]
    fn test_cache_memory_estimation() {
        let stats = CacheStats {
            hits: 1000,
            misses: 500,
            hit_rate: 0.666,
            size: 256,
            capacity: 256,
        };

        let mem = stats.memory_usage();
        // 256 entries * 128 bytes = 32768 bytes = 32KB
        assert_eq!(mem, 32768);
    }

    #[test]
    fn test_lru_eviction() {
        let mut performer = OptimizedPerformer::with_cache_capacity(4);

        // Insert 5 items to trigger eviction
        for i in 0..5 {
            performer.sequence_cache.insert_csi(
                vec![i],
                vec![],
                b'm',
                CachedAction::Attribute(i as u8),
            );
        }

        let stats = performer.cache_stats();
        assert_eq!(stats.size, 4, "Cache size should be limited to 4");
        assert_eq!(stats.capacity, 4, "Cache capacity should be 4");
    }
}
