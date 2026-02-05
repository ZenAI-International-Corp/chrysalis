//! Configuration system for Chrysalis build tool.
//!
//! This crate provides configuration management with support for:
//! - YAML configuration files
//! - Environment variables
//! - CLI overrides
//! - Sensible defaults

mod build;
mod config;
mod env;
mod env_loader;
mod error;
mod flutter;
mod platform;
mod platforms;
mod plugins;
mod web;

pub use build::BuildConfig;
pub use config::{Config, ConfigBuilder};
pub use env::EnvConfig;
pub use env_loader::EnvLoader;
pub use error::{ConfigError, Result};
pub use flutter::FlutterConfig;
pub use platform::Platform;
pub use platforms::PlatformsConfig;
pub use plugins::{ChunkConfig, HashConfig, InjectConfig, MinifyConfig, PluginsConfig};
pub use web::WebConfig;

#[cfg(test)]
mod tests;
