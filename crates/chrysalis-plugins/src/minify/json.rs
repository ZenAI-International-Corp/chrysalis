//! JSON minification.

use crate::{PluginError, Result};
use std::path::PathBuf;

/// Minify JSON content (remove whitespace).
pub fn minify_json(content: &[u8]) -> Result<Vec<u8>> {
    let value: serde_json::Value =
        serde_json::from_slice(content).map_err(|e| PluginError::MinificationFailed {
            file: PathBuf::from("unknown.json"),
            reason: format!("Parse error: {}", e),
        })?;

    let minified = serde_json::to_vec(&value).map_err(|e| PluginError::MinificationFailed {
        file: PathBuf::from("unknown.json"),
        reason: format!("Serialize error: {}", e),
    })?;

    Ok(minified)
}
