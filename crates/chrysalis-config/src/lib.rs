//! Configuration system for Chrysalis build tool.
//!
//! This crate provides configuration management with support for:
//! - TOML configuration files
//! - Environment variables
//! - CLI overrides
//! - Sensible defaults

mod build;
mod config;
mod error;
mod flutter;
mod plugins;

pub use build::BuildConfig;
pub use config::{Config, ConfigBuilder};
pub use error::{ConfigError, Result};
pub use flutter::FlutterConfig;
pub use plugins::{ChunkConfig, HashConfig, InjectConfig, MinifyConfig, PluginsConfig};

#[cfg(test)]
mod tests;
