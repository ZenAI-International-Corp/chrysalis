//! Error types for Flutter integration.

use std::path::PathBuf;
use thiserror::Error;

/// Flutter error type.
#[derive(Error, Debug)]
pub enum FlutterError {
    /// Flutter SDK not found.
    #[error("Flutter SDK not found. Please install Flutter or add it to PATH.")]
    SdkNotFound,

    /// Flutter command failed.
    #[error("Flutter command failed: {command}\nExit code: {exit_code}\nStderr: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    /// Invalid Flutter version.
    #[error("Invalid Flutter version: {0}")]
    InvalidVersion(String),

    /// Project not found.
    #[error("Flutter project not found at: {0}")]
    ProjectNotFound(PathBuf),

    /// Missing pubspec.yaml.
    #[error("pubspec.yaml not found in project directory: {0}")]
    MissingPubspec(PathBuf),

    /// Build output directory not found.
    #[error("Build output directory not found: {0}")]
    BuildOutputNotFound(PathBuf),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Other errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenient result type with FlutterError.
pub type Result<T> = std::result::Result<T, FlutterError>;
