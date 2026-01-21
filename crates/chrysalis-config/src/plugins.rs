//! Plugin configuration.

use crate::Result;
use serde::{Deserialize, Serialize};

/// Plugins configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    /// Minification plugin configuration.
    pub minify: MinifyConfig,

    /// Hashing plugin configuration.
    pub hash: HashConfig,

    /// Chunking plugin configuration.
    pub chunk: ChunkConfig,

    /// Injection plugin configuration.
    pub inject: InjectConfig,
}

/// Minification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MinifyConfig {
    /// Whether minification is enabled.
    pub enabled: bool,

    /// Whether to minify JavaScript files.
    pub minify_js: bool,

    /// Whether to minify CSS files.
    pub minify_css: bool,

    /// Whether to minify HTML files.
    pub minify_html: bool,

    /// Whether to minify JSON files.
    pub minify_json: bool,
}

/// Hashing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HashConfig {
    /// Whether hashing is enabled.
    pub enabled: bool,

    /// Files to include in hashing (glob patterns).
    pub include: Vec<String>,

    /// Files to exclude from hashing (glob patterns).
    pub exclude: Vec<String>,
}

/// Chunking configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChunkConfig {
    /// Whether chunking is enabled.
    pub enabled: bool,

    /// Files to include in chunking (glob patterns).
    pub include: Vec<String>,

    /// Files to exclude from chunking (glob patterns).
    pub exclude: Vec<String>,
}

/// Injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InjectConfig {
    /// Whether injection is enabled.
    pub enabled: bool,

    /// Whether to inline the chunk manifest.
    pub inline_manifest: bool,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            minify: MinifyConfig::default(),
            hash: HashConfig::default(),
            chunk: ChunkConfig::default(),
            inject: InjectConfig::default(),
        }
    }
}

impl Default for MinifyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            minify_js: true,
            minify_css: true,
            minify_html: true,
            minify_json: true,
        }
    }
}

impl Default for HashConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include: vec!["*.js".to_string(), "*.css".to_string()],
            exclude: vec!["*.map".to_string()],
        }
    }
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include: vec!["*.js".to_string()],
            exclude: vec!["flutter_service_worker.js".to_string()],
        }
    }
}

impl Default for InjectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            inline_manifest: true,
        }
    }
}

impl PluginsConfig {
    /// Validate plugins configuration.
    pub fn validate(&self) -> Result<()> {
        // No specific validation needed for now
        Ok(())
    }
}
