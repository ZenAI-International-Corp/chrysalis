//! Plugin system for Chrysalis.
//!
//! This crate provides all build plugins:
//! - Minify: JS/CSS/HTML/JSON minification
//! - Hash: Content-based hashing
//! - Chunk: Large file chunking
//! - Inject: Chunk loader injection

mod error;
mod plugin;

pub mod minify;
pub mod hash;
pub mod chunk;
pub mod inject;

pub use error::{PluginError, Result};
pub use plugin::{Plugin, PluginContext};

/// Re-export all plugins.
pub use minify::MinifyPlugin;
pub use hash::HashPlugin;
pub use chunk::ChunkPlugin;
pub use inject::InjectPlugin;
