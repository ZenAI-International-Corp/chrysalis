//! Error types for configuration system.

use std::path::PathBuf;
use thiserror::Error;

/// Configuration error type.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file not found.
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),

    /// Invalid TOML syntax.
    #[error("Invalid TOML syntax in {file}: {source}")]
    InvalidToml {
        file: PathBuf,
        source: toml::de::Error,
    },

    /// Invalid JSON syntax.
    #[error("Invalid JSON syntax in {file}: {source}")]
    InvalidJson {
        file: PathBuf,
        source: serde_json::Error,
    },

    /// Missing required field.
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid value for field.
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue { field: String, reason: String },

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenient result type with ConfigError.
pub type Result<T> = std::result::Result<T, ConfigError>;
