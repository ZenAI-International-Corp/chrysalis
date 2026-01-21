//! Build statistics tracking.

use std::time::Instant;

/// Build statistics.
#[derive(Debug, Clone, Default)]
pub struct BuildStats {
    /// Start time of build.
    start_time: Option<Instant>,

    /// Total number of files scanned.
    pub total_files: usize,

    /// Number of files minified.
    pub minified_files: usize,

    /// Number of files hashed.
    pub hashed_files: usize,

    /// Number of files chunked.
    pub chunked_files: usize,

    /// Total number of chunks created.
    pub total_chunks: usize,

    /// Bytes saved by minification.
    pub bytes_saved: u64,

    /// Original total size.
    pub original_size: u64,

    /// Final total size.
    pub final_size: u64,
}

impl BuildStats {
    /// Create new build statistics.
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    /// Get elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }

    /// Get compression ratio as percentage.
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        
        let saved = self.original_size.saturating_sub(self.final_size);
        (saved as f64 / self.original_size as f64) * 100.0
    }

    /// Record minification.
    pub fn record_minification(&mut self, original: u64, minified: u64) {
        self.minified_files += 1;
        self.bytes_saved += original.saturating_sub(minified);
    }

    /// Record hashing.
    pub fn record_hash(&mut self) {
        self.hashed_files += 1;
    }

    /// Record chunking.
    pub fn record_chunk(&mut self, num_chunks: usize) {
        self.chunked_files += 1;
        self.total_chunks += num_chunks;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_stats() {
        let mut stats = BuildStats::new();
        assert_eq!(stats.total_files, 0);
        
        stats.record_minification(1000, 800);
        assert_eq!(stats.minified_files, 1);
        assert_eq!(stats.bytes_saved, 200);
        
        stats.record_hash();
        assert_eq!(stats.hashed_files, 1);
        
        stats.record_chunk(3);
        assert_eq!(stats.chunked_files, 1);
        assert_eq!(stats.total_chunks, 3);
    }

    #[test]
    fn test_compression_ratio() {
        let mut stats = BuildStats::new();
        stats.original_size = 1000;
        stats.final_size = 700;
        
        assert_eq!(stats.compression_ratio(), 30.0);
    }
}
