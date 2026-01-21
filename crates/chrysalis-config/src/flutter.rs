//! Flutter-specific configuration.

use crate::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Flutter configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FlutterConfig {
    /// Path to Flutter SDK (defaults to PATH).
    pub flutter_path: Option<PathBuf>,

    /// Whether to run `flutter pub get` before build.
    pub run_pub_get: bool,

    /// Whether to run in release mode.
    pub release: bool,

    /// Target directory for Flutter output.
    pub target_dir: PathBuf,

    /// Additional Flutter build arguments.
    pub extra_args: Vec<String>,

    /// Web renderer type: "auto", "canvaskit", or "html".
    pub web_renderer: WebRenderer,

    /// Base href for the Flutter web app.
    pub base_href: Option<String>,

    /// Whether to enable source maps.
    pub source_maps: bool,

    /// Whether to enable tree shaking of icons.
    pub tree_shake_icons: bool,
}

/// Web renderer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebRenderer {
    /// Automatically choose the best renderer.
    Auto,
    /// Use CanvasKit renderer.
    Canvaskit,
    /// Use HTML renderer.
    Html,
}

impl Default for FlutterConfig {
    fn default() -> Self {
        Self {
            flutter_path: None,
            run_pub_get: true,
            release: true,
            target_dir: PathBuf::from("build/web"),
            extra_args: Vec::new(),
            web_renderer: WebRenderer::Auto,
            base_href: None,
            source_maps: false,
            tree_shake_icons: true,
        }
    }
}

impl FlutterConfig {
    /// Validate Flutter configuration.
    pub fn validate(&self) -> Result<()> {
        // Validate target directory is not empty
        if self.target_dir.as_os_str().is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "flutter.target_dir".to_string(),
                reason: "target directory cannot be empty".to_string(),
            });
        }

        // Validate base_href format if provided
        if let Some(ref base_href) = self.base_href {
            if !base_href.starts_with('/') || !base_href.ends_with('/') {
                return Err(ConfigError::InvalidValue {
                    field: "flutter.base_href".to_string(),
                    reason: "base_href must start and end with '/'".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get Flutter build arguments.
    ///
    /// Note: `--no-web-resources-cdn` is always enforced to ensure all resources
    /// are bundled locally for proper hash processing and offline support.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["build".to_string(), "web".to_string()];

        // Release or profile mode
        if self.release {
            args.push("--release".to_string());
        } else {
            args.push("--profile".to_string());
        }

        // IMPORTANT: Always disable web resources CDN
        // This ensures CanvasKit and other resources are bundled locally,
        // allowing Chrysalis to properly hash and optimize them.
        args.push("--no-web-resources-cdn".to_string());

        // Base href
        if let Some(ref base_href) = self.base_href {
            args.push(format!("--base-href={}", base_href));
        }

        // Source maps
        if self.source_maps {
            args.push("--source-maps".to_string());
        }

        // Tree shake icons
        if !self.tree_shake_icons {
            args.push("--no-tree-shake-icons".to_string());
        }

        // Extra args
        args.extend(self.extra_args.clone());

        args
    }
}

impl WebRenderer {
    /// Get the string representation for Flutter CLI.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Canvaskit => "canvaskit",
            Self::Html => "html",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_flutter_config() {
        let config = FlutterConfig::default();
        assert!(config.run_pub_get);
        assert!(config.release);
        assert_eq!(config.target_dir, PathBuf::from("build/web"));
    }

    #[test]
    fn test_build_args_always_disables_cdn() {
        let config = FlutterConfig::default();
        let args = config.build_args();
        
        // Should always contain these
        assert!(args.contains(&"build".to_string()));
        assert!(args.contains(&"web".to_string()));
        assert!(args.contains(&"--release".to_string()));
        
        // CRITICAL: Must always disable CDN for proper hash processing
        assert!(args.contains(&"--no-web-resources-cdn".to_string()));
    }

    #[test]
    fn test_build_args_profile_mode() {
        let mut config = FlutterConfig::default();
        config.release = false;
        let args = config.build_args();
        
        assert!(args.contains(&"--profile".to_string()));
        assert!(!args.contains(&"--release".to_string()));
        
        // Still must disable CDN
        assert!(args.contains(&"--no-web-resources-cdn".to_string()));
    }

    #[test]
    fn test_base_href_validation() {
        let mut config = FlutterConfig::default();
        config.base_href = Some("/app/".to_string());
        assert!(config.validate().is_ok());

        config.base_href = Some("app/".to_string());
        assert!(config.validate().is_err());

        config.base_href = Some("/app".to_string());
        assert!(config.validate().is_err());
    }
}
