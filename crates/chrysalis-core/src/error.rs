//! Error types for core build system.

use std::path::PathBuf;
use thiserror::Error;

/// Build error type.
#[derive(Error, Debug)]
pub enum BuildError {
    /// File not found in context.
    #[error("File not found in build context: {0}")]
    FileNotFound(PathBuf),

    /// Directory not found.
    #[error("Directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    /// File already exists.
    #[error("File already exists: {0}")]
    FileAlreadyExists(PathBuf),

    /// Invalid file path.
    #[error("Invalid file path: {0}")]
    InvalidPath(PathBuf),

    /// I/O error.
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// UTF-8 conversion error.
    #[error("UTF-8 conversion error for file {path}: {source}")]
    Utf8Error {
        path: PathBuf,
        source: std::string::FromUtf8Error,
    },

    /// Glob pattern error.
    #[error("Invalid glob pattern: {0}")]
    GlobPattern(String),

    /// Other errors.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Convenient result type with BuildError.
pub type Result<T> = std::result::Result<T, BuildError>;
