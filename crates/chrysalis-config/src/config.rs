//! Main configuration structure.

use crate::{BuildConfig, ConfigError, EnvConfig, PlatformsConfig, ProjectConfig, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration for Chrysalis.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Project-level configuration.
    pub project: ProjectConfig,

    /// Build configuration (shared across platforms).
    pub build: BuildConfig,

    /// Environment variable configuration.
    pub env: EnvConfig,

    /// Multi-platform configuration.
    pub platforms: PlatformsConfig,
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
        self.project.validate()?;
        self.build.validate()?;
        self.env.validate()?;
        self.platforms.validate()?;

        // Ensure at least one platform is enabled
        if !self.platforms.has_enabled_platform() {
            return Err(ConfigError::InvalidValue {
                field: "platforms".to_string(),
                reason: "at least one platform must be enabled".to_string(),
            });
        }

        Ok(())
    }

    /// Create a new builder.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// Builder for Config.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    project: Option<ProjectConfig>,
    build: Option<BuildConfig>,
    env: Option<EnvConfig>,
    platforms: Option<PlatformsConfig>,
}

impl ConfigBuilder {
    /// Set project configuration.
    pub fn project(mut self, project: ProjectConfig) -> Self {
        self.project = Some(project);
        self
    }

    /// Set build configuration.
    pub fn with_build(mut self, build: BuildConfig) -> Self {
        self.build = Some(build);
        self
    }

    /// Set environment configuration.
    pub fn env(mut self, env: EnvConfig) -> Self {
        self.env = Some(env);
        self
    }

    /// Set platforms configuration.
    pub fn platforms(mut self, platforms: PlatformsConfig) -> Self {
        self.platforms = Some(platforms);
        self
    }

    /// Build the configuration.
    pub fn build(self) -> Config {
        Config {
            project: self.project.unwrap_or_default(),
            build: self.build.unwrap_or_default(),
            env: self.env.unwrap_or_default(),
            platforms: self.platforms.unwrap_or_default(),
        }
    }
}
