//! Multi-platform configuration.

use crate::{Result, WebConfig};
use serde::{Deserialize, Serialize};

/// Multi-platform configuration container.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PlatformsConfig {
    /// Web platform configuration.
    pub web: WebConfig,
    // Future platform configurations (currently disabled by default)
    // pub windows: Option<WindowsConfig>,
    // pub macos: Option<MacOSConfig>,
    // pub linux: Option<LinuxConfig>,
    // pub android: Option<AndroidConfig>,
    // pub ios: Option<IOSConfig>,
}

impl PlatformsConfig {
    /// Validate all platform configurations.
    pub fn validate(&self) -> Result<()> {
        self.web.validate()?;
        // Future: validate other platforms
        Ok(())
    }

    /// Check if any platform is enabled.
    pub fn has_enabled_platform(&self) -> bool {
        self.web.enabled
        // Future: || self.windows.is_some() || self.macos.is_some() ...
    }

    /// Get list of enabled platform names.
    pub fn enabled_platforms(&self) -> Vec<&'static str> {
        let mut platforms = Vec::new();
        if self.web.enabled {
            platforms.push("web");
        }
        // Future: check other platforms
        platforms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_platforms_config() {
        let config = PlatformsConfig::default();
        assert!(config.web.enabled);
        assert!(config.has_enabled_platform());
    }

    #[test]
    fn test_enabled_platforms() {
        let config = PlatformsConfig::default();
        let enabled = config.enabled_platforms();
        assert_eq!(enabled, vec!["web"]);
    }
}
