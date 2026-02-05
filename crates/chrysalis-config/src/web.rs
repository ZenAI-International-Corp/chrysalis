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

    /// Output directory for processed files (relative to project root).
    /// Defaults to "dist/web". Set to None to process files in-place.
    pub output_dir: Option<PathBuf>,

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
            output_dir: Some(PathBuf::from("dist/web")),
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

        self.flutter.validate()?;
        self.plugins.validate()?;
        Ok(())
    }

    /// Get the Flutter build output directory (where Flutter writes its output).
    /// This is always "build/{platform}" by Flutter convention.
    pub fn flutter_build_dir(&self) -> PathBuf {
        PathBuf::from("build/web")
    }

    /// Get the final output directory (where Chrysalis writes processed files).
    /// Returns None if processing in-place.
    pub fn output_dir(&self) -> Option<&PathBuf> {
        self.output_dir.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_web_config() {
        let config = WebConfig::default();
        assert!(config.enabled);
        assert_eq!(config.output_dir, Some(PathBuf::from("dist/web")));
        assert_eq!(config.flutter_build_dir(), PathBuf::from("build/web"));
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
}
