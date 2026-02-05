//! Platform types and configuration.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported build platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Web platform
    Web,
    /// Windows platform
    Windows,
    /// macOS platform
    #[serde(rename = "macos")]
    MacOS,
    /// Linux platform
    Linux,
    /// Android platform
    Android,
    /// iOS platform
    #[serde(rename = "ios")]
    IOS,
}

impl Platform {
    /// Get all supported platforms.
    pub fn all() -> Vec<Platform> {
        vec![
            Platform::Web,
            Platform::Windows,
            Platform::MacOS,
            Platform::Linux,
            Platform::Android,
            Platform::IOS,
        ]
    }

    /// Get the platform as a string (lowercase).
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Web => "web",
            Platform::Windows => "windows",
            Platform::MacOS => "macos",
            Platform::Linux => "linux",
            Platform::Android => "android",
            Platform::IOS => "ios",
        }
    }

    /// Get the Flutter build target name.
    pub fn flutter_target(&self) -> &'static str {
        match self {
            Platform::Web => "web",
            Platform::Windows => "windows",
            Platform::MacOS => "macos",
            Platform::Linux => "linux",
            Platform::Android => "apk",
            Platform::IOS => "ios",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "web" => Ok(Platform::Web),
            "windows" => Ok(Platform::Windows),
            "macos" => Ok(Platform::MacOS),
            "linux" => Ok(Platform::Linux),
            "android" => Ok(Platform::Android),
            "ios" => Ok(Platform::IOS),
            _ => Err(format!("Unknown platform: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_from_str() {
        assert_eq!("web".parse::<Platform>().unwrap(), Platform::Web);
        assert_eq!("windows".parse::<Platform>().unwrap(), Platform::Windows);
        assert_eq!("macos".parse::<Platform>().unwrap(), Platform::MacOS);
        assert_eq!("linux".parse::<Platform>().unwrap(), Platform::Linux);
        assert!("unknown".parse::<Platform>().is_err());
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Web.to_string(), "web");
        assert_eq!(Platform::MacOS.to_string(), "macos");
    }

    #[test]
    fn test_flutter_target() {
        assert_eq!(Platform::Web.flutter_target(), "web");
        assert_eq!(Platform::Android.flutter_target(), "apk");
    }
}
