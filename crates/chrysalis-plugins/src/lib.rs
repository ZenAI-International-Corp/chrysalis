//! Plugin system for Chrysalis.
//!
//! This crate provides all build plugins:
//! - Minify: JS/CSS/HTML/JSON minification
//! - Hash: Content-based hashing
//! - Chunk: Large file chunking
//! - Inject: Chunk loader injection

mod error;
mod plugin;

pub mod chunk;
pub mod hash;
pub mod inject;
pub mod minify;

pub use error::{PluginError, Result};
pub use plugin::{Plugin, PluginContext};

pub use chunk::ChunkPlugin;
pub use hash::HashPlugin;
pub use inject::InjectPlugin;
/// Re-export all plugins.
pub use minify::MinifyPlugin;
