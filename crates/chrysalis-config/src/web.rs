//! Web platform configuration.

use crate::{FlutterConfig, PluginsConfig, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Web platform configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebConfig {
    /// Whether web platform is enabled.
    pub enabled: bool,

    /// Build directory (relative to project root).
    pub build_dir: PathBuf,

    /// File patterns to exclude from processing.
    pub exclude_patterns: Vec<String>,

    /// Flutter-specific configuration for web.
    pub flutter: FlutterConfig,

    /// Plugins configuration for web.
    pub plugins: PluginsConfig,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            build_dir: PathBuf::from("build/web"),
            exclude_patterns: vec!["*.map".to_string(), "*.txt".to_string()],
            flutter: FlutterConfig::default(),
            plugins: PluginsConfig::default(),
        }
    }
}

impl WebConfig {
    /// Validate web configuration.
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Validate build directory
        if self.build_dir.as_os_str().is_empty() {
            return Err(crate::ConfigError::InvalidValue {
                field: "platforms.web.build_dir".to_string(),
                reason: "build directory cannot be empty".to_string(),
            });
        }

        self.flutter.validate()?;
        self.plugins.validate()?;
        Ok(())
    }

    /// Get the build output directory.
    pub fn build_output_dir(&self) -> &PathBuf {
        &self.flutter.target_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_web_config() {
        let config = WebConfig::default();
        assert!(config.enabled);
        assert_eq!(config.flutter.target_dir, PathBuf::from("build/web"));
        assert_eq!(config.plugins.chunk.chunk_size_kb, 400);
        assert_eq!(config.plugins.hash.hash_length, 8);
    }

    #[test]
    fn test_disabled_web_config_validation() {
        let config = WebConfig {
            enabled: false,
            ..Default::default()
        };
        // Disabled config should skip validation
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_build_dir_validation() {
        let mut config = WebConfig::default();
        assert!(config.validate().is_ok());

        config.build_dir = PathBuf::from("");
        assert!(config.validate().is_err());
    }
}
