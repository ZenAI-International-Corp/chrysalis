//! Main configuration structure.

use crate::{BuildConfig, ConfigError, FlutterConfig, PluginsConfig, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration for Chrysalis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Flutter-specific configuration.
    pub flutter: FlutterConfig,

    /// Build configuration.
    pub build: BuildConfig,

    /// Plugins configuration.
    pub plugins: PluginsConfig,
}

impl Config {
    /// Load configuration from a file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use chrysalis_config::Config;
    ///
    /// let config = Config::from_file("chrysalis.toml")?;
    /// # Ok::<(), chrysalis_config::ConfigError>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound(path.to_path_buf()))?;

        let config: Self = toml::from_str(&content).map_err(|source| ConfigError::InvalidToml {
            file: path.to_path_buf(),
            source,
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from a file, or use default if file doesn't exist.
    pub fn from_file_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::from_file(path).unwrap_or_default()
    }

    /// Save configuration to a file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        self.flutter.validate()?;
        self.build.validate()?;
        self.plugins.validate()?;
        Ok(())
    }

    /// Create a new builder.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flutter: FlutterConfig::default(),
            build: BuildConfig::default(),
            plugins: PluginsConfig::default(),
        }
    }
}

/// Builder for Config.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    flutter: Option<FlutterConfig>,
    build: Option<BuildConfig>,
    plugins: Option<PluginsConfig>,
}

impl ConfigBuilder {
    /// Set Flutter configuration.
    pub fn flutter(mut self, flutter: FlutterConfig) -> Self {
        self.flutter = Some(flutter);
        self
    }

    /// Set build configuration.
    pub fn with_build(mut self, build: BuildConfig) -> Self {
        self.build = Some(build);
        self
    }

    /// Set plugins configuration.
    pub fn plugins(mut self, plugins: PluginsConfig) -> Self {
        self.plugins = Some(plugins);
        self
    }

    /// Build the configuration.
    pub fn build(self) -> Config {
        Config {
            flutter: self.flutter.unwrap_or_default(),
            build: self.build.unwrap_or_default(),
            plugins: self.plugins.unwrap_or_default(),
        }
    }
}
