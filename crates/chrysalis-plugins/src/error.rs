//! Error types for plugins.

use std::path::PathBuf;
use thiserror::Error;

/// Plugin error type.
#[derive(Error, Debug)]
pub enum PluginError {
    /// Minification failed.
    #[error("Minification failed for {file}: {reason}")]
    MinificationFailed { file: PathBuf, reason: String },

    /// Hashing failed.
    #[error("Hashing failed for {file}: {reason}")]
    HashingFailed { file: PathBuf, reason: String },

    /// Chunking failed.
    #[error("Chunking failed for {file}: {reason}")]
    ChunkingFailed { file: PathBuf, reason: String },

    /// Injection failed.
    #[error("Injection failed: {0}")]
    InjectionFailed(String),

    /// Template error.
    #[error("Template error: {0}")]
    TemplateError(String),

    /// Build error.
    #[error(transparent)]
    BuildError(#[from] chrysalis_core::BuildError),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenient result type with PluginError.
pub type Result<T> = std::result::Result<T, PluginError>;
