//! Build system configuration.

use crate::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BuildConfig {
    /// Build directory (relative to project root).
    pub build_dir: PathBuf,

    /// Chunk size in kilobytes.
    pub chunk_size_kb: usize,

    /// Minimum file size for chunking in kilobytes.
    pub min_chunk_size_kb: usize,

    /// Hash length for content-based hashing.
    pub hash_length: usize,

    /// Whether to clean build directory before building.
    pub clean_before_build: bool,

    /// File patterns to exclude from processing.
    pub exclude_patterns: Vec<String>,

    /// Whether to enable verbose output.
    pub verbose: bool,

    /// Number of parallel jobs (0 = number of CPUs).
    pub parallel_jobs: usize,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            build_dir: PathBuf::from("build/web"),
            chunk_size_kb: 400,
            min_chunk_size_kb: 400,
            hash_length: 8,
            clean_before_build: true,
            exclude_patterns: vec!["*.map".to_string(), "*.txt".to_string()],
            verbose: false,
            parallel_jobs: 0,
        }
    }
}

impl BuildConfig {
    /// Validate build configuration.
    pub fn validate(&self) -> Result<()> {
        // Validate chunk size
        if self.chunk_size_kb == 0 {
            return Err(ConfigError::InvalidValue {
                field: "build.chunk_size_kb".to_string(),
                reason: "chunk size must be greater than 0".to_string(),
            });
        }

        // Validate min chunk size
        if self.min_chunk_size_kb == 0 {
            return Err(ConfigError::InvalidValue {
                field: "build.min_chunk_size_kb".to_string(),
                reason: "min chunk size must be greater than 0".to_string(),
            });
        }

        // Validate hash length
        if self.hash_length == 0 || self.hash_length > 32 {
            return Err(ConfigError::InvalidValue {
                field: "build.hash_length".to_string(),
                reason: "hash length must be between 1 and 32".to_string(),
            });
        }

        // Validate build directory
        if self.build_dir.as_os_str().is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "build.build_dir".to_string(),
                reason: "build directory cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Get chunk size in bytes.
    pub fn chunk_size_bytes(&self) -> usize {
        self.chunk_size_kb * 1024
    }

    /// Get minimum chunk size in bytes.
    pub fn min_chunk_size_bytes(&self) -> usize {
        self.min_chunk_size_kb * 1024
    }

    /// Get number of parallel jobs (or CPU count if 0).
    pub fn parallel_jobs_or_cpus(&self) -> usize {
        if self.parallel_jobs == 0 {
            num_cpus::get()
        } else {
            self.parallel_jobs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_build_config() {
        let config = BuildConfig::default();
        assert_eq!(config.chunk_size_kb, 400);
        assert_eq!(config.hash_length, 8);
        assert!(config.clean_before_build);
    }

    #[test]
    fn test_chunk_size_bytes() {
        let config = BuildConfig::default();
        assert_eq!(config.chunk_size_bytes(), 400 * 1024);
    }

    #[test]
    fn test_validation() {
        let mut config = BuildConfig::default();
        assert!(config.validate().is_ok());

        config.chunk_size_kb = 0;
        assert!(config.validate().is_err());

        config.chunk_size_kb = 400;
        config.hash_length = 0;
        assert!(config.validate().is_err());

        config.hash_length = 33;
        assert!(config.validate().is_err());
    }
}
