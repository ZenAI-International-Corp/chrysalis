//! Configuration system for Chrysalis build tool.
//!
//! This crate provides configuration management with support for:
//! - TOML configuration files
//! - Environment variables
//! - CLI overrides
//! - Sensible defaults

mod config;
mod error;
mod flutter;
mod build;
mod plugins;

pub use config::{Config, ConfigBuilder};
pub use error::{ConfigError, Result};
pub use flutter::FlutterConfig;
pub use build::BuildConfig;
pub use plugins::{PluginsConfig, MinifyConfig, HashConfig, ChunkConfig, InjectConfig};

#[cfg(test)]
mod tests;
