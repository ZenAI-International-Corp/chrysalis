//! Build system configuration (platform-agnostic).

use crate::Result;
use serde::{Deserialize, Serialize};

/// Build configuration shared across all platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BuildConfig {
    /// Whether to clean build directory before building.
    pub clean_before_build: bool,

    /// Whether to enable verbose output.
    pub verbose: bool,

    /// Number of parallel jobs (0 = number of CPUs).
    pub parallel_jobs: usize,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            clean_before_build: true,
            verbose: false,
            parallel_jobs: 0,
        }
    }
}

impl BuildConfig {
    /// Validate build configuration.
    pub fn validate(&self) -> Result<()> {
        // No validation needed currently
        Ok(())
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
        assert!(config.clean_before_build);
        assert_eq!(config.parallel_jobs, 0);
    }

    #[test]
    fn test_parallel_jobs() {
        let config = BuildConfig::default();
        assert!(config.parallel_jobs_or_cpus() > 0);

        let config = BuildConfig {
            parallel_jobs: 4,
            ..Default::default()
        };
        assert_eq!(config.parallel_jobs_or_cpus(), 4);
    }

    #[test]
    fn test_validation() {
        let config = BuildConfig::default();
        assert!(config.validate().is_ok());
    }
}
